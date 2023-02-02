use std::str::FromStr;

use bson::oid::ObjectId;
use twilight_model::{channel::message::AllowedMentions, id::Id};
use uuid::Uuid;

use crate::{
    handlers::Handler,
    mongo::mongo::{Config, Punishment, PunishmentType},
    util::duration::Duration,
};

impl Handler {
    pub async fn issue_strike(
        &self,
        conf: &Config,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        reason: Option<&String>,
        duration: &Duration,
    ) -> Result<Punishment, Box<dyn std::error::Error + Send + Sync>> {
        let infraction_uuid = Uuid::new_v4().to_string();

        let dur = duration.to_unix_expiry();

        let strike = Punishment {
            oid: ObjectId::new(),
            guild_id: guild_id.to_string(),
            user_id: user_id.to_string(),
            issuer: issuer.to_string(),
            typ: PunishmentType::Strike,
            expires: dur,
            role_id: None,
            weight: None,
            reason: reason.cloned(),
            uuid: infraction_uuid,
            expired: false,
        };

        self.send_punishment_embed(
            guild_id,
            user_id,
            issuer,
            reason,
            Some(duration),
            &PunishmentType::Strike,
        )
        .await?;

        self.db.add_punishment(&strike).await?;

        self.escalate_strike(conf, guild_id, user_id).await?;

        Ok(strike)
    }

    pub async fn escalate_strike(
        &self,
        conf: &Config,
        guild_id: &String,
        user_id: &String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let strikes = self.db.get_strikes(guild_id, user_id).await?;
        let esc = &conf.modules.moderation.strike_escalation;
        let esc_to = match esc.get(&(strikes.len() as i64)) {
            Some(esc_to) => esc_to,
            None => return Ok(()),
        };

        let id = match &conf.modules.logging.channel_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let channel_id = Id::from_str(&id)?;

        let allowed_ment = AllowedMentions::builder().build();

        match esc_to.typ {
            PunishmentType::Mute => {
                let issuer = &match self.cache.current_user() {
                    Some(user) => user.id.to_string(),
                    None => return Ok(()),
                };

                let duration = &Duration::new(esc_to.duration.clone());
                let reason = Some("Exceeded strike limit".to_string());

                let punishment = self
                    .mute_user(conf, guild_id, user_id, issuer, duration, reason.as_ref())
                    .await?;

                let log = match conf.modules.logging.log_mute(
                    issuer,
                    user_id,
                    reason.as_ref(),
                    duration,
                    &punishment.uuid,
                ) {
                    Some(log) => log,
                    None => return Ok(()),
                };

                self.rest
                    .create_message(channel_id)
                    .content(log.as_str())?
                    .allowed_mentions(Some(&allowed_ment))
                    .await?;
            }

            PunishmentType::Kick => {
                let issuer = &match self.cache.current_user() {
                    Some(user) => user.id.to_string(),
                    None => return Ok(()),
                };

                let reason = Some("Exceeded strike limit".to_string());

                let punishment = self
                    .kick_user(guild_id, user_id, issuer, reason.as_ref())
                    .await?;

                let log = match conf.modules.logging.log_kick(
                    issuer,
                    user_id,
                    reason.as_ref(),
                    &punishment.uuid,
                ) {
                    Some(log) => log,
                    None => return Ok(()),
                };

                self.rest
                    .create_message(channel_id)
                    .content(log.as_str())?
                    .allowed_mentions(Some(&allowed_ment))
                    .await?;
            }

            PunishmentType::Ban => {
                let issuer = &match self.cache.current_user() {
                    Some(user) => user.id.to_string(),
                    None => return Ok(()),
                };

                let duration = &Duration::new(esc_to.duration.clone());
                let reason = Some("Exceeded strike limit".to_string());

                let punishment = self
                    .ban_user(guild_id, user_id, issuer, duration, reason.as_ref())
                    .await?;

                let log = match conf.modules.logging.log_ban(
                    issuer,
                    user_id,
                    reason.as_ref(),
                    duration,
                    &punishment.uuid,
                ) {
                    Some(log) => log,
                    None => return Ok(()),
                };

                self.rest
                    .create_message(channel_id)
                    .content(log.as_str())?
                    .allowed_mentions(Some(&allowed_ment))
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
