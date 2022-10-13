use std::{collections::HashMap, str::FromStr};

use bson::oid::ObjectId;
use serde::Deserialize as SerdeDeserialize;
use serde_derive::{Serialize, Deserialize};
use twilight_model::{id::Id, channel::embed::{Embed, EmbedField}};
use twilight_http::request::AuditLogReason;
use uuid::Uuid;

use crate::{mongo::mongo::{PunishmentType, Punishment, Config}, handlers::Handler, util::duration::Duration};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Moderation {
    pub censor_searches: bool,
    pub censor_staff_searches: bool,
    pub confirm_actions_message: bool,
    pub confirm_actions_message_expiry: i64,
    pub confirm_actions_reaction: bool,
    pub default_strike_duration: Option<String>,
    pub display_no_permission: bool,
    pub mute_role: String,
    pub remove_roles_on_mute: bool,
    pub reason_edit_level: i64,
    pub notify_actions: bool,
    pub show_moderator_on_notify: bool,
    pub silence_level: i64,
    pub strike_cushioning: i64,
    #[serde(deserialize_with = "de_strike_esc")]
    pub strike_escalation: HashMap<i64, StrikeEscalation>,
    #[serde(default)]
    pub update_higher_level_reason: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
        reason: &Option<String>,
        typ: &PunishmentType,
        role_id: Option<String>, // for mute
    ) -> Result<Punishment, Box<dyn std::error::Error + Send + Sync>> {

        let punishment = Punishment{
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
            reason: reason.clone(),
            uuid: Uuid::new_v4().to_string(),
            expired: false
        };
        
        if typ == &PunishmentType::Mute {
            let current = self.db.get_mute(guild_id, user_id).await?;
            match current {
                Some(_) => {
                    if issuer == &match self.cache.current_user() {
                        Some(user) => user.id.to_string(),
                        None => return Err("No current user".into()),
                    } {
                        // we don't want to overwrite the mute of a moderator and it can't be us if they're already muted
                        Err("already muted during automod")?
                    }

                    self.db.delete_mute(guild_id, user_id).await?;

                    self.db.add_punishment(&punishment).await?;
                },
                None => {
                    self.db.add_punishment(&punishment).await?;
                }
            };
        }
        else {
            self.db.add_punishment(&punishment).await?;
        }

        Ok(punishment)
    }

    pub async fn send_punishment_embed(&self,
        guild_id: &String,
        user_id: &String,
        issuer_id: &String,
        reason: &Option<String>,
        duration: Option<&Duration>,
        typ: &PunishmentType,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let guild = self.cache.guild(Id::from_str(guild_id)?);
        let issuer = self.cache.user(Id::from_str(&issuer_id)?);

        let mut fields = vec![
            EmbedField{
                name: "Server Name".to_string(),
                value: match guild {
                    Some(guild) => guild.name().to_string(),
                    None => guild_id.to_string(),
                },
                inline: true
            },
            EmbedField{
                name: "Actioned by".to_string(),
                value: match issuer {
                    Some(user) => user.name.to_string(),
                    None => format!("<@{}>", issuer_id).to_string(),
                },
                inline: false
            },
            EmbedField{
                name: "Reason".to_string(),
                value: match reason {
                    Some(reason) => reason.to_string(),
                    None => "No reason provided".to_string()
                },
                inline: false
            }
        ];
        
        match duration {
            Some(duration) => {
                fields.push(EmbedField{
                    name: "Expires".to_string(),
                    value: duration.to_discord_timestamp(),
                    inline: false
                })
            }
            None => {}
        }

        let embeds = vec![Embed {
            title: Some(format!("You have been {}.", typ.past_tense_string())),
            description: None,
            color: Some(0),
            footer: Some(twilight_model::channel::embed::EmbedFooter { 
                icon_url: None,
                proxy_icon_url: None,
                text: "Black Mesa Rust Beta by Tyler#0911 running on rustc 1.63.0".to_string()
            }),
            fields,
            kind: "rich".to_string(),
            author: None,
            image: None,
            provider: None,
            thumbnail: None,
            timestamp: None,
            url: Some("https://blackmesa.bot".to_string()),
            video: None
        }];

        let dm_channel = match self.rest.create_private_channel(Id::from_str(user_id)?)
        .exec()
        .await {
            Ok(channel) => {
                match channel.model().await {
                    Ok(channel) => channel,
                    Err(_) => return Ok(())
                }
            },
            Err(_) => return Ok(())
        };

        match self.rest.create_message(dm_channel.id).embeds(&embeds) {
            Ok(m) => {
                match m.exec().await {
                    Ok(_) => return Ok(()),
                    Err(_) => return Ok(())
                };
            },
            Err(_) => return Ok(())
        }

    }

    pub async fn kick_user(&self,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        reason: &Option<String>,
    ) -> Result<Punishment, Box<dyn std::error::Error + Send + Sync>> {
        let punishment = self.add_punishment(guild_id, user_id, issuer, None, reason,
            &PunishmentType::Kick, None).await?;
            

        self.send_punishment_embed(guild_id, user_id, issuer, reason, None,
            &PunishmentType::Kick).await?;

        match self.rest.remove_guild_member(
            Id::from_str(guild_id)?,
            Id::from_str(user_id)?
        ).reason(format!("{} - {}", issuer, match reason {
            Some(r) => r.to_string(),
            None => "No reason provided".to_string()
        }).as_str()) {
            Ok(k) => {
                k.exec()
                .await?;
                Ok(punishment)
            },
            Err(e) => Err(e)?
        }
    }

    pub async fn ban_user(&self,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        duration: &Duration,
        reason: &Option<String>,
    ) -> Result<Punishment, Box<dyn std::error::Error + Send + Sync>> {
        let punishment = self.add_punishment(guild_id, user_id, issuer, Some(duration), reason,
            &PunishmentType::Ban, None).await?;

        self.send_punishment_embed(guild_id, user_id, issuer, reason, Some(duration),
            &PunishmentType::Ban).await?;

        match self.rest.create_ban(
            Id::from_str(guild_id)?,
            Id::from_str(user_id)?
        ).reason(format!("{} - {}", issuer, match reason {
            Some(r) => r.to_string(),
            None => "No reason provided".to_string()
        }).as_str()) {
            Ok(k) => {
                k.exec()
                .await?;
                Ok(punishment)
            },
            Err(e) => Err(e)?
        }
    }

    pub async fn unban_user(&self,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        reason: &Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.db.delete_ban(guild_id, user_id).await?;

        // might be nice to have some sort of embed sent to the user on an unban / any other punishment removal
        
        match self.rest.delete_ban(
            Id::from_str(guild_id)?,
            Id::from_str(user_id)?
        ).reason(format!("{} - {}", issuer, match reason {
            Some(r) => r.to_string(),
            None => "No reason provided".to_string()
        }).as_str()) {
            Ok(k) => {
                k.exec()
                .await?;
                Ok(())
            },
            Err(e) => Err(e)?
        }
    }

    pub async fn mute_user(&self,
        conf: &Config,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        duration: &Duration,
        reason: &Option<String>
    ) -> Result<Punishment, Box<dyn std::error::Error + Send + Sync>>{
        let mute_id = &conf.modules.moderation.mute_role;
        let punishment = self.add_punishment(guild_id, user_id, issuer, Some(duration), reason,
            &PunishmentType::Mute, Some(mute_id.to_string())).await?;
        
        self.send_punishment_embed(guild_id, user_id, issuer, reason, Some(duration),
            &PunishmentType::Mute).await?;
        
        match self.rest.add_guild_member_role(
            Id::from_str(guild_id)?,
            Id::from_str(user_id)?,
            Id::from_str(mute_id)?
        ).reason(format!("{} - {}", issuer, match reason {
            Some(r) => r.to_string(),
            None => "No reason provided".to_string()
        }).as_str()) {
            Ok(k) => {
                k.exec()
                .await?;
                Ok(punishment)
            },
            Err(e) => Err(e)?
        }
    }

    // rather conf or role_id MUST be specified or this will error.
    pub async fn unmute_user(&self,
        conf: Option<&Config>,
        mute_role_id: Option<String>,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        reason: &Option<String>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>{
        let role_id = match conf {
            Some(conf) => conf.modules.moderation.mute_role.clone(),
            None => match mute_role_id {
                Some(mute_role_id) => mute_role_id,
                None => return Err("No mute role specified".into())
            }
        };

        // might be nice to have some sort of embed sent to the user on an unmute / any other punishment removal

        self.db.delete_mute(guild_id, user_id).await?;

        match self.rest.remove_guild_member_role(
            Id::from_str(guild_id)?,
            Id::from_str(user_id)?,
            Id::from_str(&role_id)?
        ).reason(format!("{} - {}", issuer, match reason {
            Some(r) => r.to_string(),
            None => "No reason provided".to_string()
        }).as_str()) {
            Ok(k) => {
                k.exec()
                .await?;
                Ok(())
            },
            Err(e) => Err(e)?
        }
    }
}