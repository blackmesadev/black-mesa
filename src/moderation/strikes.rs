use bson::oid::ObjectId;
use uuid::Uuid;

use crate::{
    config::Config,
    handlers::Handler,
    moderation::moderation::{Punishment, PunishmentType},
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

        let appealable = (*conf)
            .modules
            .as_ref()
            .and_then(|modules| modules.appeals.as_ref())
            .map(|appeals| appeals.enabled)
            .unwrap_or(false);

        let notif_id = match self
            .send_punishment_embed(
                guild_id,
                user_id,
                issuer,
                reason,
                Some(duration),
                &PunishmentType::Strike,
                appealable,
            )
            .await?
        {
            Some(msg) => Some(msg.id.to_string()),
            None => None,
        };

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
            uuid: infraction_uuid.clone(),
            escalation_uuid: None,
            expired: false,
            expired_reason: None,
            appeal_status: None,
            notif_id,
        };

        let appealable = (*conf)
            .modules
            .as_ref()
            .and_then(|modules| modules.appeals.as_ref())
            .map(|appeals| appeals.enabled)
            .unwrap_or(false);

        self.send_punishment_embed(
            guild_id,
            user_id,
            issuer,
            reason,
            Some(duration),
            &PunishmentType::Strike,
            appealable,
        )
        .await?;

        self.db.add_punishment(&strike).await?;

        self.escalate_strike(conf, guild_id, user_id, infraction_uuid)
            .await?;

        Ok(strike)
    }

    pub async fn escalate_strike(
        &self,
        conf: &Config,
        guild_id: &String,
        user_id: &String,
        escalation_uuid: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // this shouldn't even be possible but just in case
        let modules = match &conf.modules {
            Some(m) => m,
            None => return Err("Modules not set".into()),
        };

        let moderation = match &modules.moderation {
            Some(moderation) => moderation,
            None => return Ok(()),
        };

        let strikes = self.db.get_strikes(guild_id, user_id).await?;
        let esc = &moderation.strike_escalation;
        let esc_to = match esc.get(&(strikes.len() as i64)) {
            Some(esc_to) => esc_to,
            None => return Ok(()),
        };

        let logging = &modules.logging;

        let appealable = (*conf)
            .modules
            .as_ref()
            .and_then(|modules| modules.appeals.as_ref())
            .map(|appeals| appeals.enabled)
            .unwrap_or(false);

        match esc_to.typ {
            PunishmentType::Mute => {
                let issuer = &match self.cache.current_user() {
                    Some(user) => user.id.to_string(),
                    None => return Ok(()),
                };

                let duration = &Duration::new(esc_to.duration.clone());
                let reason = Some("Exceeded strike limit".to_string());

                let punishment = self
                    .mute_user(
                        conf,
                        guild_id,
                        user_id,
                        issuer,
                        duration,
                        reason.as_ref(),
                        Some(escalation_uuid),
                        appealable,
                    )
                    .await?;

                if let Some(logging) = logging {
                    self.log_mute(
                        logging,
                        issuer,
                        user_id,
                        reason.as_ref(),
                        duration,
                        &punishment.uuid,
                    )
                    .await;
                }
            }

            PunishmentType::Kick => {
                let issuer = &match self.cache.current_user() {
                    Some(user) => user.id.to_string(),
                    None => return Ok(()),
                };

                let reason = Some("Exceeded strike limit".to_string());

                let punishment = self
                    .kick_user(
                        guild_id,
                        user_id,
                        issuer,
                        reason.as_ref(),
                        Some(escalation_uuid),
                        appealable,
                    )
                    .await?;

                if let Some(logging) = logging {
                    self.log_kick(logging, issuer, user_id, reason.as_ref(), &punishment.uuid)
                        .await;
                }
            }

            PunishmentType::Ban => {
                let issuer = &match self.cache.current_user() {
                    Some(user) => user.id.to_string(),
                    None => return Ok(()),
                };

                let duration = &Duration::new(esc_to.duration.clone());
                let reason = Some("Exceeded strike limit".to_string());

                let punishment = self
                    .ban_user(
                        guild_id,
                        user_id,
                        issuer,
                        duration,
                        reason.as_ref(),
                        Some(escalation_uuid),
                        appealable,
                    )
                    .await?;

                if let Some(logging) = logging {
                    self.log_ban(
                        logging,
                        issuer,
                        user_id,
                        reason.as_ref(),
                        duration,
                        &punishment.uuid,
                    )
                    .await;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
