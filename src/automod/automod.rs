#![allow(dead_code)]

use serde::Deserialize as SerdeDeserialize;
use serde_derive::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use twilight_model::channel::message::AllowedMentions;
use twilight_model::id::Id;

use crate::automod::{censor::*, clean, spam::*};
use crate::handlers::Handler;
use crate::mongo::mongo::{Config, PunishmentType};
use crate::util::duration::Duration;
use crate::util::permissions;

use super::AutomodMessage;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuildOptions {
    pub minimum_account_age: String,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Censor {
    pub filter_zalgo: Option<bool>,
    pub filter_invites: Option<bool>,
    pub filter_domains: Option<bool>,
    pub filter_strings: Option<bool>,

    #[serde(rename = "filterIPs")]
    pub filter_ips: Option<bool>,
    pub filter_regex: Option<bool>,
    pub filter_english: Option<bool>,
    pub invites_whitelist: Option<Vec<String>>,
    pub invites_blacklist: Option<Vec<String>>,
    pub domain_whitelist: Option<Vec<String>>,
    pub domain_blacklist: Option<Vec<String>>,
    pub blocked_substrings: Option<Vec<String>>,
    pub blocked_strings: Option<Vec<String>>,
    pub regex: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Spam {
    pub punishment: Option<PunishmentType>,
    pub punishment_duration: Option<i64>,
    pub count: Option<i64>,
    pub interval: Option<i64>,
    pub max_messages: Option<i64>,
    pub max_mentions: Option<i64>,
    pub max_links: Option<i64>,
    pub max_attachments: Option<i64>,
    pub max_emojis: Option<i64>,
    pub max_newlines: Option<i64>,
    pub max_duplicates: Option<i64>,
    pub max_characters: Option<i64>,
    pub max_uppercase_percent: Option<f64>,
    pub min_uppercase_limit: Option<i64>,
    pub clean: Option<bool>,
    pub clean_count: Option<i64>,
    pub clean_duration: Option<i64>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Automod {
    pub enabled: Option<bool>,
    pub guild_options: Option<GuildOptions>,
    #[serde(deserialize_with = "de_censor_levels")]
    pub censor_levels: Option<HashMap<i64, Censor>>,
    pub censor_channels: Option<HashMap<String, Censor>>,
    #[serde(deserialize_with = "de_spam_levels")]
    pub spam_levels: Option<HashMap<i64, Spam>>,
    pub spam_channels: Option<HashMap<String, Spam>>,
    pub public_humilation: Option<bool>,
    pub staff_bypass: Option<bool>,
    pub reaction_message: Option<String>,
}

fn de_censor_levels<'de, D>(deserializer: D) -> Result<Option<HashMap<i64, Censor>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut map = HashMap::new();
    let mut map2: HashMap<String, Censor> = HashMap::deserialize(deserializer)?;
    for (k, v) in map2.drain() {
        map.insert(k.parse::<i64>().unwrap(), v);
    }
    if map.is_empty() {
        Ok(None)
    } else {
        Ok(Some(map))
    }
}

fn de_spam_levels<'de, D>(deserializer: D) -> Result<Option<HashMap<i64, Spam>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut map = HashMap::new();
    let mut map2: HashMap<String, Spam> = HashMap::deserialize(deserializer)?;
    for (k, v) in map2.drain() {
        map.insert(k.parse::<i64>().unwrap(), v);
    }
    if map.is_empty() {
        Ok(None)
    } else {
        Ok(Some(map))
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

    fn get_fn(&self) -> Option<fn(&Spam, &AutomodMessage) -> bool> {
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
    pub fn get_censor(&self) -> &CensorType {
        match self {
            AutomodType::Censor(c) => c.clone(),
            _ => panic!("Not a censor type"),
        }
    }

    pub fn get_spam(&self) -> &SpamType {
        match self {
            AutomodType::Spam(s) => s.clone(),
            _ => panic!("Not a spam type"),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            AutomodType::Censor(c) => c.get_name(),
            AutomodType::Spam(s) => s.get_name(),
        }
    }
}

#[derive(Debug)]
pub struct AutomodResult {
    pub typ: AutomodType,
    pub msg: AutomodMessage,
    pub trigger: Option<String>, // Only applicable in Censor
    pub censor: Option<Censor>,
    pub spam: Option<Spam>,
}

impl Handler {
    pub async fn automod(
        &self,
        conf: &Config,
        msg: &AutomodMessage,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Bots shouldn't need to be moderated by us.
        if msg.author.bot {
            return Ok(());
        }

        let res = match self.run_automod(&conf, &msg).await {
            Some(r) => r,
            None => return Ok(()),
        };

        let id = match &conf.modules.logging.channel_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let channel_id = Id::from_str(&id)?;

        let allowed_ment = AllowedMentions::builder().build();

        match res.typ {
            AutomodType::Censor(_) => {
                self.rest.delete_message(msg.channel_id, msg.id).await?;
                // Action
                self.issue_strike(
                    &conf,
                    &msg.guild_id.unwrap().to_string(),
                    &msg.author.id.to_string(),
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
                    &Duration::new(match &conf.modules.moderation.default_strike_duration {
                        Some(d) => d.to_string(),
                        None => "30d".to_string(),
                    }),
                )
                .await?;

                // Log
                let log = match conf.modules.logging.log_message_censor(res) {
                    Some(l) => l,
                    None => return Ok(()),
                };

                self.rest
                    .create_message(channel_id)
                    .content(log.as_str())?
                    .allowed_mentions(Some(&allowed_ment))
                    .await?;
            }
            AutomodType::Spam(_) => {
                self.rest.delete_message(msg.channel_id, msg.id).await?;
                // Action
                self.issue_strike(
                    &conf,
                    &msg.guild_id.unwrap().to_string(),
                    &msg.author.id.to_string(),
                    &match self.cache.current_user() {
                        Some(u) => u.id.to_string(),
                        None => "AutoMod".to_string(),
                    },
                    Some(format!("Spam->{}", &res.typ.get_name())).as_ref(),
                    &Duration::new(match &conf.modules.moderation.default_strike_duration {
                        Some(d) => d.to_string(),
                        None => "30d".to_string(),
                    }),
                )
                .await?;

                let log = match conf.modules.logging.log_message_spam(res) {
                    Some(l) => l,
                    None => return Ok(()),
                };

                self.rest
                    .create_message(channel_id)
                    .content(log.as_str())?
                    .allowed_mentions(Some(&allowed_ment))
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn run_automod(&self, conf: &Config, msg: &AutomodMessage) -> Option<AutomodResult> {
        let content = clean::replace_non_std_space(&msg.content.as_ref()?.to_lowercase());

        let censor_levels = conf.modules.automod.censor_levels.as_ref()?;
        let spam_levels = conf.modules.automod.spam_levels.as_ref()?;

        let default_censor = &Censor::default();
        let default_spam = &Spam::default();

        let user_id = msg.author.id.to_string();

        let guild_id = match msg.guild_id {
            Some(g) => g,
            None => return None,
        };

        let roles = match self.cache.member(guild_id, msg.author.id) {
            Some(m) => m.roles().to_vec(),
            None => match self.rest.guild_member(guild_id, msg.author.id).await {
                Ok(m) => match m.model().await {
                    Ok(m) => m.roles,
                    Err(_) => return None,
                },
                Err(_) => return None,
            },
        };

        let closest_censor_lvl: i64 = permissions::get_closest_level(
            censor_levels.keys().cloned().collect(),
            permissions::get_user_level(conf, Some(&roles), &user_id),
        );
        let censor_user = match censor_levels.get(&closest_censor_lvl) {
            Some(level) => level,
            None => censor_levels.get(&0).unwrap_or(default_censor),
        };

        let closest_spam_lvl: i64 = permissions::get_closest_level(
            spam_levels.keys().cloned().collect(),
            permissions::get_user_level(conf, Some(&roles), &user_id),
        );

        let spam_user = match spam_levels.get(&closest_spam_lvl) {
            Some(level) => level,
            None => spam_levels.get(&0).unwrap_or(default_spam),
        };

        for typ in CensorType::iter() {
            let censor_fn = typ.get_fn();
            let (trigger, ok) = censor_fn(censor_user, &content);
            if !ok {
                return Some(AutomodResult {
                    typ: AutomodType::Censor(typ),
                    msg: msg.clone(),
                    trigger: Some(trigger),
                    censor: Some(censor_user.clone()),
                    spam: None,
                });
            }
        }
        for typ in SpamType::iter() {
            match typ.get_fn() {
                Some(spam_fn) => {
                    if !spam_fn(spam_user, msg) {
                        return Some(AutomodResult {
                            typ: AutomodType::Spam(typ),
                            msg: msg.clone(),
                            trigger: None,
                            spam: Some(spam_user.clone()),
                            censor: None,
                        });
                    }
                }
                None => (),
            }
        }

        // now we have to do spam message detection all alone because we need direct redis access :trolldespair:
        if !self.redis.filter_messages(spam_user, msg).await {
            return Some(AutomodResult {
                typ: AutomodType::Spam(SpamType::Messages),
                msg: msg.clone(),
                trigger: None,
                spam: Some(spam_user.clone()),
                censor: None,
            });
        }

        None
    }
}
