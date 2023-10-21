#![allow(dead_code)]

use serde_derive::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashSet;
use std::error::Error;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::automod::{censor::*, clean, spam::*};
use crate::config::Config;
use crate::handlers::Handler;
use crate::util::duration::Duration;
use crate::util::permissions;

use super::MessageTrait;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuildOptions {
    pub minimum_account_age: String,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Censor {
    pub filter_zalgo: Option<bool>,
    pub filter_invites: Option<bool>,
    pub filter_domains: Option<bool>,
    pub filter_strings: Option<bool>,
    pub filter_ips: Option<bool>,
    pub invites_whitelist: Option<Vec<String>>,
    pub invites_blacklist: Option<Vec<String>>,
    pub domain_whitelist: Option<Vec<String>>,
    pub domain_blacklist: Option<Vec<String>>,
    pub blocked_substrings: Option<Vec<String>>,
    pub blocked_strings: Option<Vec<String>>,
    pub regex: Option<String>,

    pub bypass: Vec<String>,
    pub monitor_channels: Vec<String>,
    pub ignore_channels: Vec<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Spam {
    pub interval: Option<i64>,
    pub max_messages: Option<i64>,
    pub max_mentions: Option<i64>,
    pub max_links: Option<i64>,
    pub max_attachments: Option<i64>,
    pub max_emojis: Option<i64>,
    pub max_newlines: Option<i64>,
    pub max_characters: Option<i64>,
    pub max_uppercase_percent: Option<f64>,

    pub bypass: Vec<String>,
    pub monitor_channels: Vec<String>,
    pub ignore_channels: Vec<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Automod {
    pub enabled: Option<bool>,
    pub guild_options: Option<GuildOptions>,
    pub censor: Option<Vec<Censor>>,
    pub spam: Option<Vec<Spam>>,
}

impl Default for Automod {
    fn default() -> Self {
        Automod {
            enabled: Some(false),
            guild_options: None,
            censor: None,
            spam: None,
        }
    }
}

#[derive(Debug, EnumIter)]
pub enum CensorType {
    Invites,
    Domains,
    Strings,
    IPs,
}

impl CensorType {
    pub fn get_name(&self) -> String {
        match self {
            CensorType::Invites => "Invites",
            CensorType::Domains => "Domains",
            CensorType::Strings => "BlockedStrings",
            CensorType::IPs => "IPs",
        }
        .to_string()
    }

    pub fn get_fn(&self) -> fn(&Censor, &String) -> (String, bool) {
        match self {
            CensorType::Invites => filter_invites,
            CensorType::Domains => filter_domains,
            CensorType::Strings => filter_strings,
            CensorType::IPs => filter_ips,
        }
    }
}

#[derive(Debug, EnumIter, PartialEq)]
pub enum SpamType {
    Messages,
    Mentions,
    Links,
    Attachments,
    Emojis,
    Newlines,
    Uppercase,
    MaxLength,
}

impl SpamType {
    fn get_name(&self) -> String {
        match self {
            SpamType::Messages => "MaxMessages",
            SpamType::Mentions => "Mentions",
            SpamType::Links => "Links",
            SpamType::Attachments => "Attachments",
            SpamType::Emojis => "Emojis",
            SpamType::Newlines => "Newlines",
            SpamType::Uppercase => "Uppercase",
            SpamType::MaxLength => "MaxLength",
        }
        .to_string()
    }

    fn get_fn<T: MessageTrait>(&self) -> Option<fn(&Spam, &T) -> bool> {
        match self {
            SpamType::Mentions => Some(filter_mentions),
            SpamType::Links => Some(filter_links),
            SpamType::Attachments => Some(filter_attachments),
            SpamType::Emojis => Some(filter_emojis),
            SpamType::Newlines => Some(filter_newlines),
            SpamType::Uppercase => Some(filter_uppercase),
            SpamType::MaxLength => Some(filter_max_length),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum AutomodType {
    Censor(CensorType),
    Spam(SpamType),
}

impl AutomodType {
    pub fn get_name(&self) -> String {
        match self {
            AutomodType::Censor(c) => c.get_name(),
            AutomodType::Spam(s) => s.get_name(),
        }
    }
}

#[derive(Debug)]
pub struct AutomodResult<T: MessageTrait> {
    pub typ: AutomodType,
    pub msg: Box<T>,
    pub trigger: Option<String>, // Only applicable in Censor
    pub censor: Option<Censor>,
    pub spam: Option<Spam>,
}

impl Handler {
    pub async fn automod<T: MessageTrait + Clone>(
        &self,
        conf: &Config,
        msg: &T,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Bots shouldn't need to be moderated by us.
        if msg.author().bot {
            return Ok(());
        }

        let res = match self.run_automod(&conf, msg).await {
            Some(r) => r,
            None => return Ok(()),
        };

        match res.typ {
            AutomodType::Censor(_) => {
                self.rest
                    .delete_message(*msg.channel_id(), *msg.id())
                    .await?;
                // Action
                self.issue_strike(
                    &conf,
                    &msg.guild_id().unwrap().to_string(),
                    &msg.author().id.to_string(),
                    &match self.cache.current_user() {
                        Some(u) => u.id.to_string(),
                        None => "AutoMod".to_string(),
                    },
                    Some(format!(
                        "Censor->{}({})",
                        &res.typ.get_name(),
                        res.trigger.clone().unwrap_or("unknown".to_string())
                    ))
                    .as_ref(),
                    &Duration::new(match &conf.modules {
                        Some(modules) => match &modules.moderation {
                            Some(m) => match &m.default_strike_duration {
                                Some(dur) => dur.to_string(),
                                None => "30d".to_string(),
                            },
                            None => "30d".to_string(),
                        },
                        None => "30d".to_string(),
                    }),
                )
                .await?;

                if let Some(modules) = &conf.modules {
                    if let Some(logging) = &modules.logging {
                        self.log_message_censor(logging, res).await;
                    }
                }
            }
            AutomodType::Spam(_) => {
                self.rest
                    .delete_message(*msg.channel_id(), *msg.id())
                    .await?;
                // Action
                self.issue_strike(
                    &conf,
                    &msg.guild_id().unwrap().to_string(),
                    &msg.author().id.to_string(),
                    &match self.cache.current_user() {
                        Some(u) => u.id.to_string(),
                        None => "AutoMod".to_string(),
                    },
                    Some(format!("Spam->{}", &res.typ.get_name())).as_ref(),
                    &Duration::new(match &conf.modules {
                        Some(modules) => match &modules.moderation {
                            Some(m) => match &m.default_strike_duration {
                                Some(dur) => dur.to_string(),
                                None => "30d".to_string(),
                            },
                            None => "30d".to_string(),
                        },
                        None => "30d".to_string(),
                    }),
                )
                .await?;

                if let Some(modules) = &conf.modules {
                    if let Some(logging) = &modules.logging {
                        self.log_message_spam(logging, res).await;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn run_automod<T: MessageTrait + Clone>(
        &self,
        conf: &Config,
        msg: &T,
    ) -> Option<AutomodResult<T>> {
        let automod = match &conf.modules {
            Some(modules) => match &modules.automod {
                Some(a) => a,
                None => return None,
            },
            None => return None,
        };

        let content = clean::replace_non_std_space(&msg.content().to_lowercase());

        let author_id = msg.author().id.to_string();

        let guild_id = match msg.guild_id() {
            Some(g) => *g,
            None => return None,
        };

        let roles = match self.cache.member(guild_id, msg.author().id) {
            Some(m) => m.roles().to_vec(),
            None => match self.rest.guild_member(guild_id, msg.author().id).await {
                Ok(m) => match m.model().await {
                    Ok(m) => m.roles,
                    Err(_) => return None,
                },
                Err(_) => return None,
            },
        };

        let user_groups =
            match permissions::get_user_groups_names(conf, msg.author().id, Some(&roles)) {
                Ok(groups) => groups,
                Err(_) => HashSet::new(),
            };

        let censor = automod.censor.as_ref().map_or(Vec::new(), |censors| {
            censors
                .iter()
                .filter(|censor| {
                    !censor.bypass.contains(&author_id)
                        && !user_groups
                            .iter()
                            .any(|group| censor.bypass.contains(group))
                })
                .cloned()
                .collect::<Vec<_>>()
        });

        let spam = automod.spam.as_ref().map_or(Vec::new(), |spams| {
            spams
                .iter()
                .filter(|spam| {
                    !spam.bypass.contains(&author_id)
                        && !user_groups.iter().any(|group| spam.bypass.contains(group))
                })
                .cloned()
                .collect::<Vec<_>>()
        });

        for censor in censor {
            for typ in CensorType::iter() {
                let censor_fn = typ.get_fn();
                let (trigger, ok) = censor_fn(&censor, &content);
                if !ok {
                    return Some(AutomodResult {
                        typ: AutomodType::Censor(typ),
                        msg: Box::new(msg.clone()),
                        trigger: Some(trigger),
                        censor: Some(censor.clone()),
                        spam: None,
                    });
                }
            }
        }

        for spam in spam {
            for typ in SpamType::iter() {
                match typ.get_fn() {
                    Some(spam_fn) => {
                        if !spam_fn(&spam, msg) {
                            return Some(AutomodResult {
                                typ: AutomodType::Spam(typ),
                                msg: Box::new(msg.clone()),
                                trigger: None,
                                spam: Some(spam.clone()),
                                censor: None,
                            });
                        }
                    }
                    None => (),
                }
            }

            if !self.redis.filter_messages(&spam, msg).await {
                return Some(AutomodResult {
                    typ: AutomodType::Spam(SpamType::Messages),
                    msg: Box::new(msg.clone()),
                    trigger: None,
                    spam: Some(spam.clone()),
                    censor: None,
                });
            }
        }

        None
    }
}
