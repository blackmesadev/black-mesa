use bson::oid::ObjectId;
use uuid::Uuid;

use crate::{handlers::Handler, util::duration::Duration, mongo::mongo::{Punishment, PunishmentType, Config}};

impl Handler {
    pub async fn issue_strike(&self,
        conf: &Config,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        reason: &Option<String>,
        duration: &Duration) -> Result<Punishment, Box<dyn std::error::Error + Send + Sync>> {

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
            reason: reason.clone(),
            uuid: infraction_uuid,
            expired: false
        };

        self.send_punishment_embed(guild_id, user_id, issuer, reason, Some(duration),
            &PunishmentType::Strike).await?;

        self.db.add_punishment(&strike).await?;

        self.escalate_strike(conf, guild_id, user_id).await?;

        Ok(strike)
    }

    pub async fn escalate_strike(&self, conf: &Config, guild_id: &String, user_id: &String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let strikes = self.db.get_strikes(guild_id, user_id).await?;
        let esc = &conf.modules.moderation.strike_escalation;
        let esc_to = match esc.get(&(strikes.len() as i64)) {
            Some(esc_to) => esc_to,
            None => return Ok(()),
        };

        match esc_to.typ {
            PunishmentType::Mute => {
                let issuer = &match self.cache.current_user() {
                    Some(user) => user.id.to_string(),
                    None => return Ok(()),
                };

                self.mute_user(
                    conf,
                    guild_id,
                    user_id,
                    issuer,
                    &Duration::new(esc_to.duration.clone()),
                    &Some("Exceeded strike limit".to_string())
                ).await?;
            },

            PunishmentType::Kick => {
                let issuer = &match self.cache.current_user() {
                    Some(user) => user.id.to_string(),
                    None => return Ok(()),
                };

                self.kick_user(
                    guild_id,
                    user_id,
                    issuer,
                    &Some("Exceeded strike limit".to_string())
                ).await?;
            },

            PunishmentType::Ban => {
                let issuer = &match self.cache.current_user() {
                    Some(user) => user.id.to_string(),
                    None => return Ok(()),
                };

                self.ban_user(
                    guild_id,
                    user_id,
                    issuer,
                    &Duration::new(esc_to.duration.clone()),
                    &Some("Exceeded strike limit".to_string())
                ).await?;
            },
            _ => {}
        }
        Ok(())
    }
} 