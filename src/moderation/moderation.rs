use core::fmt;
use std::{collections::HashMap, str::FromStr};

use bson::{oid::ObjectId, serde_helpers::serialize_object_id_as_hex_string};
use serde::Deserialize as SerdeDeserialize;
use serde_aux::prelude::bool_true;
use serde_derive::{Deserialize, Serialize};
use twilight_http::request::AuditLogReason;
use twilight_model::{
    channel::{
        message::{
            embed::{self, EmbedField},
            Embed,
        },
        Message,
    },
    id::Id,
};
use uuid::Uuid;

use crate::{
    appeals::AppealStatus, config::Config, handlers::Handler, util::duration::Duration, VERSION,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Punishment {
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
    #[serde(rename = "_id")]
    pub oid: ObjectId,
    pub guild_id: String,
    pub user_id: String,
    pub issuer: String,
    #[serde(rename = "type")]
    pub typ: PunishmentType,
    pub expires: Option<i64>,
    pub role_id: Option<String>,
    pub weight: Option<i64>,
    pub reason: Option<String>,
    pub uuid: String,
    pub escalation_uuid: Option<String>,
    #[serde(default = "bool_true")]
    pub expired: bool,
    pub expired_reason: Option<String>,
    pub appeal_status: Option<AppealStatus>,
    pub notif_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum PunishmentType {
    #[default]
    Unknown,
    None,
    Strike,
    Mute,
    Kick,
    Ban,
    Softban,
}
impl FromStr for PunishmentType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "strike" => Ok(PunishmentType::Strike),
            "mute" => Ok(PunishmentType::Mute),
            "kick" => Ok(PunishmentType::Kick),
            "ban" => Ok(PunishmentType::Ban),
            _ => Ok(PunishmentType::Unknown),
        }
    }
}

impl fmt::Display for PunishmentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PunishmentType::Strike => write!(f, "strike"),
            PunishmentType::Mute => write!(f, "mute"),
            PunishmentType::Kick => write!(f, "kick"),
            PunishmentType::Ban => write!(f, "ban"),
            _ => write!(f, "unknown"),
        }
    }
}

impl PunishmentType {
    pub fn pretty_string(&self) -> String {
        match self {
            PunishmentType::Unknown => "Unknown",
            PunishmentType::None => "None",
            PunishmentType::Strike => "Strike",
            PunishmentType::Mute => "Mute",
            PunishmentType::Kick => "Kick",
            PunishmentType::Ban => "Ban",
            PunishmentType::Softban => "Softban",
        }
        .to_string()
    }

    pub fn past_tense_string(&self) -> String {
        match self {
            PunishmentType::Unknown => "Unknown",
            PunishmentType::None => "None",
            PunishmentType::Strike => "Striked",
            PunishmentType::Mute => "Muted",
            PunishmentType::Kick => "Kicked",
            PunishmentType::Ban => "Banned",
            PunishmentType::Softban => "Softbanned",
        }
        .to_string()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Moderation {
    pub censor_searches: bool,
    pub default_strike_duration: Option<String>,
    pub display_no_permission: bool,
    pub mute_role: String,
    pub notify_actions: bool,
    pub show_moderator_on_notify: bool,
    #[serde(deserialize_with = "de_strike_esc")]
    pub strike_escalation: HashMap<i64, StrikeEscalation>,
    pub update_higher_level_action: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrikeEscalation {
    #[serde(rename = "type")]
    pub typ: PunishmentType,
    pub duration: String,
}

fn de_strike_esc<'de, D>(deserializer: D) -> Result<HashMap<i64, StrikeEscalation>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut map = HashMap::new();
    let mut map2: HashMap<String, StrikeEscalation> = HashMap::deserialize(deserializer)?;
    for (k, v) in map2.drain() {
        map.insert(k.parse::<i64>().unwrap(), v);
    }
    Ok(map)
}

impl Handler {
    pub async fn add_punishment(
        &self,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        duration: Option<&Duration>,
        reason: Option<&String>,
        typ: &PunishmentType,
        role_id: Option<String>, // for mute
        escalation_uuid: Option<String>,
        notif_id: Option<String>,
    ) -> Result<Punishment, Box<dyn std::error::Error + Send + Sync>> {
        let punishment = Punishment {
            oid: ObjectId::new(),
            guild_id: guild_id.to_string(),
            user_id: user_id.to_string(),
            issuer: issuer.to_string(),
            typ: typ.clone(),
            expires: match duration {
                Some(dur) => dur.to_unix_expiry(),
                None => None,
            },
            role_id,
            weight: None,
            reason: reason.cloned(),
            uuid: Uuid::new_v4().to_string(),
            escalation_uuid,
            expired: false,
            expired_reason: None,
            appeal_status: None,
            notif_id,
        };

        if typ == &PunishmentType::Mute {
            let current = self.db.get_mute(guild_id, user_id).await?;
            match current {
                Some(_) => {
                    if issuer
                        == &match self.cache.current_user() {
                            Some(user) => user.id.to_string(),
                            None => return Err("No current user".into()),
                        }
                    {
                        // we don't want to overwrite the mute of a moderator and it can't be us if they're already muted
                        Err("already muted during automod")?
                    }

                    self.db.delete_mute(guild_id, user_id).await?;

                    self.db.add_punishment(&punishment).await?;
                }
                None => {
                    self.db.add_punishment(&punishment).await?;
                }
            };
        } else {
            self.db.add_punishment(&punishment).await?;
        }

        Ok(punishment)
    }

    pub async fn send_punishment_embed(
        &self,
        guild_id: &String,
        user_id: &String,
        issuer_id: &String,
        reason: Option<&String>,
        duration: Option<&Duration>,
        typ: &PunishmentType,
        appealable: bool,
    ) -> Result<Option<Message>, Box<dyn std::error::Error + Send + Sync>> {
        let guild = self.cache.guild(Id::from_str(guild_id)?);
        let issuer = self.cache.user(Id::from_str(&issuer_id)?);

        let mut fields = vec![
            EmbedField {
                name: "Guild Name".to_string(),
                value: match guild {
                    Some(guild) => guild.name().to_string(),
                    None => guild_id.to_string(),
                },
                inline: true,
            },
            EmbedField {
                name: "Actioned by".to_string(),
                value: match issuer {
                    Some(user) => user.name.to_string(),
                    None => format!("<@{}>", issuer_id).to_string(),
                },
                inline: false,
            },
            EmbedField {
                name: "Reason".to_string(),
                value: match reason {
                    Some(reason) => reason.to_string(),
                    None => "No reason provided".to_string(),
                },
                inline: false,
            },
            EmbedField {
                name: "Appeal".to_string(),
                value: String::from(if appealable {
                    "You can appeal this punishment by replying to this message with `!appeal`."
                } else {
                    "You cannot appeal this punishment."
                }),
                inline: false,
            },
        ];

        match duration {
            Some(duration) => fields.push(EmbedField {
                name: "Expires".to_string(),
                value: duration.to_discord_timestamp(),
                inline: false,
            }),
            None => {}
        }

        let embeds = vec![Embed {
            title: Some(format!("You have been {}.", typ.past_tense_string())),
            description: None,
            color: Some(0),
            footer: Some(embed::EmbedFooter {
                icon_url: None,
                proxy_icon_url: None,
                text: format!("Black Mesa v{}", VERSION),
            }),
            fields,
            kind: "rich".to_string(),
            author: None,
            image: None,
            provider: None,
            thumbnail: None,
            timestamp: None,
            url: Some("https://blackmesa.bot".to_string()),
            video: None,
        }];

        let dm_channel = match self
            .rest
            .create_private_channel(Id::from_str(user_id)?)
            .await
        {
            Ok(channel) => match channel.model().await {
                Ok(channel) => channel,
                Err(_) => return Ok(None),
            },
            Err(_) => return Ok(None),
        };

        match self
            .rest
            .create_message(dm_channel.id)
            .embeds(&embeds)?
            .await
        {
            Ok(msg) => Ok(Some(msg.model().await?)),
            Err(_) => return Ok(None),
        }
    }

    pub async fn kick_user(
        &self,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        reason: Option<&String>,
        escalation_uuid: Option<String>,
        appealable: bool,
    ) -> Result<Punishment, Box<dyn std::error::Error + Send + Sync>> {
        let notif_id = match self
            .send_punishment_embed(
                guild_id,
                user_id,
                issuer,
                reason,
                None,
                &PunishmentType::Kick,
                appealable,
            )
            .await?
        {
            Some(msg) => Some(msg.id.to_string()),
            None => None,
        };

        let punishment = self
            .add_punishment(
                guild_id,
                user_id,
                issuer,
                None,
                reason,
                &PunishmentType::Kick,
                None,
                escalation_uuid,
                notif_id,
            )
            .await?;

        match self
            .rest
            .remove_guild_member(Id::from_str(guild_id)?, Id::from_str(user_id)?)
            .reason(
                format!(
                    "{} - {}",
                    issuer,
                    match reason {
                        Some(r) => r.to_string(),
                        None => "No reason provided".to_string(),
                    }
                )
                .as_str(),
            ) {
            Ok(k) => {
                k.await?;
                Ok(punishment)
            }
            Err(e) => Err(e)?,
        }
    }

    pub async fn softban_user(
        &self,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        duration: &Duration,
        reason: Option<&String>,
        appealable: bool,
    ) -> Result<Punishment, Box<dyn std::error::Error + Send + Sync>> {
        let notif_id = match self
            .send_punishment_embed(
                guild_id,
                user_id,
                issuer,
                reason,
                Some(duration),
                &PunishmentType::Softban,
                appealable,
            )
            .await?
        {
            Some(msg) => Some(msg.id.to_string()),
            None => None,
        };

        let punishment = self
            .add_punishment(
                guild_id,
                user_id,
                issuer,
                Some(duration),
                reason,
                &PunishmentType::Softban,
                None,
                None,
                notif_id,
            )
            .await?;

        match self
            .rest
            .create_ban(Id::from_str(guild_id)?, Id::from_str(user_id)?)
            .reason(
                format!(
                    "{} - {}",
                    issuer,
                    match reason {
                        Some(r) => r.to_string(),
                        None => "No reason provided".to_string(),
                    }
                )
                .as_str(),
            )?
            .delete_message_seconds(duration.seconds.try_into().unwrap())
        {
            Ok(k) => {
                k.await?;
            }
            Err(e) => Err(e)?,
        }

        match self
            .rest
            .delete_ban(Id::from_str(guild_id)?, Id::from_str(user_id)?)
            .reason(format!("Softban ban removal - `{}`", punishment.uuid).as_str())
        {
            Ok(k) => {
                k.await?;
                Ok(punishment)
            }
            Err(e) => Err(e)?,
        }
    }

    pub async fn ban_user(
        &self,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        duration: &Duration,
        reason: Option<&String>,
        escalation_uuid: Option<String>,
        appealable: bool,
    ) -> Result<Punishment, Box<dyn std::error::Error + Send + Sync>> {
        let notif_id = match self
            .send_punishment_embed(
                guild_id,
                user_id,
                issuer,
                reason,
                Some(duration),
                &PunishmentType::Ban,
                appealable,
            )
            .await?
        {
            Some(msg) => Some(msg.id.to_string()),
            None => None,
        };

        let punishment = self
            .add_punishment(
                guild_id,
                user_id,
                issuer,
                Some(duration),
                reason,
                &PunishmentType::Ban,
                None,
                escalation_uuid,
                notif_id,
            )
            .await?;

        match self
            .rest
            .create_ban(Id::from_str(guild_id)?, Id::from_str(user_id)?)
            .reason(
                format!(
                    "{} - {}",
                    issuer,
                    match reason {
                        Some(r) => r.to_string(),
                        None => "No reason provided".to_string(),
                    }
                )
                .as_str(),
            ) {
            Ok(k) => {
                k.await?;
                Ok(punishment)
            }
            Err(e) => Err(e)?,
        }
    }

    pub async fn unban_user(
        &self,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        reason: Option<&String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.db.delete_ban(guild_id, user_id).await?;

        // might be nice to have some sort of embed sent to the user on an unban / any other punishment removal

        match self
            .rest
            .delete_ban(Id::from_str(guild_id)?, Id::from_str(user_id)?)
            .reason(
                format!(
                    "{} - {}",
                    issuer,
                    match reason {
                        Some(r) => r.to_string(),
                        None => "No reason provided".to_string(),
                    }
                )
                .as_str(),
            ) {
            Ok(k) => {
                k.await?;
                Ok(())
            }
            Err(e) => Err(e)?,
        }
    }

    pub async fn mute_user(
        &self,
        conf: &Config,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        duration: &Duration,
        reason: Option<&String>,
        escalation_uuid: Option<String>,
        appealable: bool,
    ) -> Result<Punishment, Box<dyn std::error::Error + Send + Sync>> {
        let modules = match &conf.modules {
            Some(m) => m,
            None => return Err("Modules not set".into()),
        };

        let moderation = match modules.moderation {
            Some(ref m) => m,
            None => return Err("Moderation module not enabled".into()),
        };

        if moderation.mute_role.is_empty() {
            return Err("Mute role is not set".into());
        }

        let mute_id = &moderation.mute_role;

        let notif_id = match self
            .send_punishment_embed(
                guild_id,
                user_id,
                issuer,
                reason,
                Some(duration),
                &PunishmentType::Mute,
                appealable,
            )
            .await?
        {
            Some(msg) => Some(msg.id.to_string()),
            None => None,
        };

        let punishment = self
            .add_punishment(
                guild_id,
                user_id,
                issuer,
                Some(duration),
                reason,
                &PunishmentType::Mute,
                Some(mute_id.to_string()),
                escalation_uuid,
                notif_id,
            )
            .await?;

        match self
            .rest
            .add_guild_member_role(
                Id::from_str(guild_id)?,
                Id::from_str(user_id)?,
                Id::from_str(mute_id)?,
            )
            .reason(
                format!(
                    "{} - {}",
                    issuer,
                    match reason {
                        Some(r) => r.to_string(),
                        None => "No reason provided".to_string(),
                    }
                )
                .as_str(),
            ) {
            Ok(k) => {
                k.await?;
                Ok(punishment)
            }
            Err(e) => Err(e)?,
        }
    }

    // rather conf or role_id MUST be specified or this will error.
    pub async fn unmute_user(
        &self,
        conf: Option<&Config>,
        mute_role_id: Option<String>,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        reason: Option<&String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let role_id = match conf {
            Some(conf) => match &conf.modules {
                Some(modules) => match &modules.moderation {
                    Some(m) => {
                        if m.mute_role.is_empty() {
                            mute_role_id
                        } else {
                            Some(m.mute_role.clone())
                        }
                    }
                    None => mute_role_id,
                },
                None => mute_role_id,
            },
            None => mute_role_id,
        };

        let role_id = match role_id {
            Some(r) => r,
            None => return Err("No mute role specified".into()),
        };

        // might be nice to have some sort of embed sent to the user on an unmute / any other punishment removal

        self.db.delete_mute(guild_id, user_id).await?;

        match self
            .rest
            .remove_guild_member_role(
                Id::from_str(guild_id)?,
                Id::from_str(user_id)?,
                Id::from_str(&role_id)?,
            )
            .reason(
                format!(
                    "{} - {}",
                    issuer,
                    match reason {
                        Some(r) => r.to_string(),
                        None => "No reason provided".to_string(),
                    }
                )
                .as_str(),
            ) {
            Ok(k) => {
                k.await?;
                Ok(())
            }
            Err(e) => Err(e)?,
        }
    }
}
