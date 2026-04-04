use std::borrow::Cow;
use std::collections::HashMap;

use bm_lib::{
    discord::{DiscordError, DiscordResult, Id},
    model::{
        logging::{LogEventType, MesaLogEvent},
        Infraction, Uuid,
    },
    util::duration_to_unix_timestamp,
};
use tracing::instrument;

use super::EventHandler;

pub const DEFAULT_WARN_LENGTH: u64 = 604800;

impl EventHandler {
    #[instrument(skip(self))]
    pub async fn kick_user(
        &self,
        guild_id: &Id,
        user_id: &Id,
        moderator_id: &Id,
        reason: Option<Cow<'_, str>>,
    ) -> DiscordResult<Infraction> {
        let infraction = Infraction::new_kick(
            *guild_id,
            *user_id,
            *moderator_id,
            reason.as_ref().map(|r| r.to_string()),
            false,
        );

        let (kick_result, dm_result, db_result) = tokio::join!(
            self.rest.kick_member(&guild_id, &user_id, reason.clone()),
            self.send_infraction_dm(&infraction),
            self.db.create_infraction(&infraction)
        );

        kick_result?;
        dm_result?;
        db_result?;

        let mut vars = HashMap::new();
        vars.insert("user_id".into(), user_id.to_string());
        vars.insert("username".into(), format!("<@{}>", user_id));
        vars.insert("moderator_id".into(), moderator_id.to_string());
        vars.insert("guild_id".into(), guild_id.to_string());
        vars.insert(
            "reason".into(),
            reason
                .as_ref()
                .map(|r| r.as_ref())
                .unwrap_or("No reason")
                .to_string(),
        );
        vars.insert("infraction_id".into(), infraction.uuid.to_string());
        let _ = self
            .log_event(
                guild_id,
                &LogEventType::Mesa(MesaLogEvent::ModerationKick),
                vars,
            )
            .await;

        Ok(infraction)
    }

    #[instrument(skip(self))]
    pub async fn ban_user(
        &self,
        guild_id: &Id,
        user_id: &Id,
        moderator_id: &Id,
        duration: Option<u64>,
        reason: Option<Cow<'_, str>>,
    ) -> DiscordResult<Infraction> {
        let infraction = Infraction::new_ban(
            *guild_id,
            *user_id,
            *moderator_id,
            reason.as_ref().map(|r| r.to_string()),
            duration.map(duration_to_unix_timestamp),
            true,
        );

        let (ban_result, dm_result, db_result) = tokio::join!(
            self.rest.ban_member(&guild_id, &user_id, reason.clone(), 0),
            self.send_infraction_dm(&infraction),
            self.db.create_infraction(&infraction)
        );

        ban_result?;
        dm_result?;
        db_result?;

        let mut vars = HashMap::new();
        vars.insert("user_id".into(), user_id.to_string());
        vars.insert("username".into(), format!("<@{}>", user_id));
        vars.insert("moderator_id".into(), moderator_id.to_string());
        vars.insert("guild_id".into(), guild_id.to_string());
        vars.insert(
            "reason".into(),
            reason.as_deref().unwrap_or("No reason").to_string(),
        );
        vars.insert(
            "duration".into(),
            duration
                .map(|d| d.to_string())
                .unwrap_or_else(|| "Permanent".into()),
        );
        vars.insert("infraction_id".into(), infraction.uuid.to_string());
        let _ = self
            .log_event(
                guild_id,
                &LogEventType::Mesa(MesaLogEvent::ModerationBan),
                vars,
            )
            .await;

        Ok(infraction)
    }

    #[instrument(skip(self))]
    pub async fn mute_user(
        &self,
        guild_id: &Id,
        user_id: &Id,
        moderator_id: &Id,
        mute_role: &Id,
        duration: Option<u64>,
        reason: Option<Cow<'_, str>>,
    ) -> DiscordResult<Infraction> {
        let infraction = Infraction::new_mute(
            *guild_id,
            *user_id,
            *moderator_id,
            reason.clone().map(|r| r.to_string()),
            duration.map(duration_to_unix_timestamp),
            *mute_role,
            true,
        );

        let reason_str = reason
            .as_ref()
            .map(|r| r.as_ref().to_string())
            .unwrap_or_else(|| "No reason".into());

        let (role_result, dm_result, db_result) = tokio::join!(
            self.rest.add_role(guild_id, user_id, mute_role, reason),
            self.send_infraction_dm(&infraction),
            self.db.create_infraction(&infraction)
        );

        role_result?;
        dm_result?;
        db_result?;

        let mut vars = HashMap::new();
        vars.insert("user_id".into(), user_id.to_string());
        vars.insert("username".into(), format!("<@{}>", user_id));
        vars.insert("moderator_id".into(), moderator_id.to_string());
        vars.insert("guild_id".into(), guild_id.to_string());
        vars.insert("reason".into(), reason_str);
        vars.insert(
            "duration".into(),
            duration
                .map(|d| d.to_string())
                .unwrap_or_else(|| "Permanent".into()),
        );
        vars.insert("infraction_id".into(), infraction.uuid.to_string());
        let _ = self
            .log_event(
                guild_id,
                &LogEventType::Mesa(MesaLogEvent::ModerationMute),
                vars,
            )
            .await;

        Ok(infraction)
    }

    #[instrument(skip(self))]
    pub async fn warn_user(
        &self,
        guild_id: &Id,
        user_id: &Id,
        moderator_id: &Id,
        duration: Option<u64>,
        reason: Option<Cow<'_, str>>,
    ) -> DiscordResult<Infraction> {
        let reason_str = reason
            .as_ref()
            .map(|r| r.as_ref().to_string())
            .unwrap_or_else(|| "No reason".into());

        let infraction = Infraction::new(
            *guild_id,
            *user_id,
            *moderator_id,
            bm_lib::model::InfractionType::Warn,
            reason.map(|r| r.into_owned()),
            duration.map(duration_to_unix_timestamp),
            true,
        );

        let (dm_result, db_result) = tokio::join!(
            self.send_infraction_dm(&infraction),
            self.db.create_infraction(&infraction)
        );

        dm_result?;
        db_result?;

        let mut vars = HashMap::new();
        vars.insert("user_id".into(), user_id.to_string());
        vars.insert("username".into(), format!("<@{}>", user_id));
        vars.insert("moderator_id".into(), moderator_id.to_string());
        vars.insert("guild_id".into(), guild_id.to_string());
        vars.insert("reason".into(), reason_str);
        vars.insert(
            "duration".into(),
            duration
                .map(|d| d.to_string())
                .unwrap_or_else(|| "Permanent".into()),
        );
        vars.insert("infraction_id".into(), infraction.uuid.to_string());
        let _ = self
            .log_event(
                guild_id,
                &LogEventType::Mesa(MesaLogEvent::ModerationWarn),
                vars,
            )
            .await;

        Ok(infraction)
    }

    #[instrument(skip(self))]
    pub async fn unban_user(
        &self,
        guild_id: &Id,
        user_id: &Id,
        moderator_id: &Id,
        reason: Option<Cow<'_, str>>,
    ) -> DiscordResult<()> {
        let reason_str = reason
            .as_ref()
            .map(|r| r.as_ref().to_string())
            .unwrap_or_else(|| "No reason".into());

        let (unban_result, infractions) = tokio::join!(
            self.rest.unban_member(&guild_id, &user_id, reason),
            self.db.get_active_infractions(
                guild_id,
                user_id,
                Some(bm_lib::model::InfractionType::Ban)
            )
        );

        unban_result?;
        let infractions = infractions?;

        for infraction in infractions {
            let (dm_result, db_result) = tokio::join!(
                self.send_infraction_dm(&infraction),
                self.db.deactivate_infraction(&infraction.uuid)
            );
            dm_result?;
            db_result?;
        }

        let mut vars = HashMap::new();
        vars.insert("user_id".into(), user_id.to_string());
        vars.insert("username".into(), format!("<@{}>", user_id));
        vars.insert("moderator_id".into(), moderator_id.to_string());
        vars.insert("guild_id".into(), guild_id.to_string());
        vars.insert("reason".into(), reason_str);
        let _ = self
            .log_event(
                guild_id,
                &LogEventType::Mesa(MesaLogEvent::ModerationUnban),
                vars,
            )
            .await;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn unmute_user(
        &self,
        guild_id: &Id,
        user_id: &Id,
        moderator_id: &Id,
        reason: Option<Cow<'_, str>>,
    ) -> DiscordResult<()> {
        let infractions = self
            .db
            .get_active_infractions(guild_id, user_id, Some(bm_lib::model::InfractionType::Mute))
            .await?;

        for infraction in infractions {
            if let Some(role_id) = infraction.mute_role_id {
                let (role_result, dm_result, db_result) = tokio::join!(
                    self.rest
                        .remove_role(guild_id, user_id, &role_id, reason.clone()),
                    self.send_infraction_remove_dm(&infraction),
                    self.db.deactivate_infraction(&infraction.uuid)
                );
                role_result?;
                dm_result?;
                db_result?;
            }
        }

        let mut vars = HashMap::new();
        vars.insert("user_id".into(), user_id.to_string());
        vars.insert("username".into(), format!("<@{}>", user_id));
        vars.insert("moderator_id".into(), moderator_id.to_string());
        vars.insert("guild_id".into(), guild_id.to_string());
        vars.insert(
            "reason".into(),
            reason
                .as_ref()
                .map(|r| r.as_ref())
                .unwrap_or("No reason")
                .to_string(),
        );
        let _ = self
            .log_event(
                guild_id,
                &LogEventType::Mesa(MesaLogEvent::ModerationUnmute),
                vars,
            )
            .await;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn pardon(
        &self,
        guild_id: &Id,
        warn_id: &Uuid,
        moderator_id: &Id,
        reason: Option<Cow<'_, str>>,
    ) -> DiscordResult<Option<Infraction>> {
        let result = self
            .db
            .delete_infraction(guild_id, warn_id)
            .await
            .map_err(DiscordError::from)?;

        if let Some(ref infraction) = result {
            let mut vars = HashMap::new();
            vars.insert("user_id".into(), infraction.user_id.to_string());
            vars.insert("username".into(), format!("<@{}>", infraction.user_id));
            vars.insert("moderator_id".into(), moderator_id.to_string());
            vars.insert("guild_id".into(), guild_id.to_string());
            vars.insert(
                "reason".into(),
                reason
                    .as_ref()
                    .map(|r| r.as_ref())
                    .unwrap_or("No reason")
                    .to_string(),
            );
            vars.insert("infraction_id".into(), warn_id.to_string());
            let _ = self
                .log_event(
                    guild_id,
                    &LogEventType::Mesa(MesaLogEvent::ModerationPardon),
                    vars,
                )
                .await;
        }

        Ok(result)
    }
}
