mod censor;
mod macros;
mod spam;

use std::collections::HashMap;

use crate::{check_bypass, handler::EventHandler};

use bm_lib::{
    discord::{commands::Ctx, DiscordResult, EmbedBuilder},
    model::{
        automod::{AutomodSettings, OffenseType},
        logging::{LogEventType, MesaLogEvent},
        Config, Infraction,
    },
};
use tracing::instrument;

#[derive(Debug)]
pub struct AutomodResult {
    pub infraction: Infraction,
}

impl AutomodResult {
    pub fn new(infraction: Infraction) -> Self {
        Self { infraction }
    }
}

/// Merge global and channel settings when inherit_global is true.
/// Channel settings take precedence over global settings.
fn merge_settings(global: &AutomodSettings, channel: &AutomodSettings) -> AutomodSettings {
    let mut merged = AutomodSettings {
        name: channel.name.clone(),
        enabled: channel.enabled,
        inherit_global: channel.inherit_global,
        censors: None,
        spam: None,
        bypass: channel.bypass.clone(),
    };

    // Merge censors: start with global, override with channel-specific
    let mut censors = HashMap::new();
    if let Some(global_censors) = &global.censors {
        censors.extend(global_censors.clone());
    }
    if let Some(channel_censors) = &channel.censors {
        censors.extend(channel_censors.clone());
    }
    if !censors.is_empty() {
        merged.censors = Some(censors);
    }

    // Merge spam: channel takes precedence if present, otherwise use global
    merged.spam = channel.spam.clone().or_else(|| global.spam.clone());

    // Merge bypass: channel takes precedence if present, otherwise use global
    if merged.bypass.is_none() {
        merged.bypass = global.bypass.clone();
    }

    merged
}

impl EventHandler {
    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id, message_id = %ctx.message.id, channel_id = %ctx.channel_id))]
    pub async fn handle_automod(&self, config: &Config, ctx: &Ctx<'_>) -> DiscordResult<()> {
        let Some(automod) = &config.automod else {
            return Ok(());
        };

        let global = automod.global.as_ref();
        let channel = automod.channels.get(&ctx.channel_id);

        // Determine which settings to use
        let settings = match (global, channel) {
            // Channel override exists
            (Some(g), Some(c)) if c.enabled => {
                if c.inherit_global {
                    // Merge global and channel settings
                    Some(merge_settings(g, c))
                } else {
                    // Use only channel settings
                    Some(c.clone())
                }
            }
            // No channel override, use global if enabled
            (Some(g), None) if g.enabled => Some(g.clone()),
            // Channel exists but not enabled, check global
            (Some(g), Some(c)) if !c.enabled && g.enabled => Some(g.clone()),
            // No valid settings
            _ => None,
        };

        if let Some(settings) = settings {
            if let Some(result) = self.process_automod(config, ctx, &settings).await? {
                self.process_automod_infraction(config, ctx, &result.infraction)
                    .await?;
            }
        }

        Ok(())
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id, message_id = %ctx.message.id, channel_id = %ctx.channel_id))]
    async fn process_automod(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        automod: &AutomodSettings,
    ) -> DiscordResult<Option<AutomodResult>> {
        check_bypass!(self, config, ctx, &automod.bypass);

        if let Some(censors) = &automod.censors {
            for (typ, censor) in censors {
                if let Some(result) = self.handle_censor(ctx, typ, censor).await? {
                    return Ok(Some(result));
                }
            }
        }

        if let Some(spam) = &automod.spam {
            check_bypass!(self, config, ctx, &spam.bypass);
            if let Some(result) = self.handle_spam(ctx, spam).await? {
                return Ok(Some(result));
            }
        }

        Ok(None)
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, channel_id = %ctx.channel_id))]
    async fn process_automod_infraction(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        infraction: &Infraction,
    ) -> DiscordResult<()> {
        // For None infraction type, skip creating the infraction record
        // (message deletion and log event still happen below)
        use bm_lib::model::InfractionType;
        if infraction.infraction_type != InfractionType::None {
            self.db.create_infraction(infraction).await?;
        }

        let (event_type, reason) = if let Some(offense) = infraction.automod_offense.as_ref() {
            match &offense.typ {
                OffenseType::Spam(typ) => {
                    let typ_str = typ.to_pretty_string();
                    let reason = format!(
                        "{typ_str} spam detection triggered: {} occurrences in {}ms",
                        offense.count.unwrap_or(0),
                        offense.interval.unwrap_or(0)
                    );
                    (MesaLogEvent::AutomodSpam, reason)
                }
                OffenseType::Censor(typ) => {
                    let typ_str = typ.to_pretty_string();

                    let offending_word = offense.offending_filter.clone().unwrap_or_default();
                    let message_content = ctx.message.content.as_str();

                    let censored = if offending_word.len() > 2 {
                        format!(
                            "{}{}{}",
                            &offending_word[..1],
                            "*".repeat(offending_word.len() - 2),
                            &offending_word[offending_word.len() - 1..]
                        )
                    } else {
                        "*".repeat(offending_word.len())
                    };

                    let reason = if let Some(pos) = message_content
                        .to_lowercase()
                        .find(&offending_word.to_lowercase())
                    {
                        let start = pos.saturating_sub(15);
                        let end = (pos + offending_word.len() + 15).min(message_content.len());
                        let context =
                            message_content[start..end].replace(&offending_word, &censored);
                        format!("{typ_str} censor triggered: \"...{context}...\"")
                    } else {
                        format!("{typ_str} censor triggered: {censored}")
                    };
                    (MesaLogEvent::AutomodCensor, reason)
                }
            }
        } else {
            let reason = infraction
                .reason
                .clone()
                .unwrap_or_else(|| "Automod action triggered".to_string());
            (MesaLogEvent::AutomodCensor, reason)
        };

        // Build placeholder vars for the logging system
        let mut vars = HashMap::new();
        vars.insert("user_id".into(), ctx.user.id.to_string());
        vars.insert("username".into(), ctx.user.username.to_string());
        vars.insert("channel_id".into(), ctx.channel_id.to_string());
        vars.insert("guild_id".into(), ctx.guild_id.to_string());
        vars.insert("reason".into(), reason.clone());

        if let Some(offense) = infraction.automod_offense.as_ref() {
            match &offense.typ {
                OffenseType::Spam(typ) => {
                    vars.insert("spam_type".into(), typ.to_pretty_string());
                    vars.insert("count".into(), offense.count.unwrap_or(0).to_string());
                    vars.insert("interval".into(), offense.interval.unwrap_or(0).to_string());
                }
                OffenseType::Censor(typ) => {
                    vars.insert("filter_type".into(), typ.to_pretty_string());
                    vars.insert(
                        "offending_content".into(),
                        offense.offending_filter.clone().unwrap_or_default(),
                    );
                }
            }
        }

        // Try the logging system first
        let log_event = LogEventType::Mesa(event_type);
        let dispatched = self.log_event(ctx.guild_id, &log_event, vars).await;

        // Fallback to the legacy log_channel embed if logging system didn't send
        // (i.e. no log_config found for this event, or logging disabled)
        if dispatched.is_err() || !config.logging_enabled {
            if let Some(log_channel) = config.log_channel {
                let embed = EmbedBuilder::new()
                    .title("Automod Action")
                    .description(reason)
                    .color(0xff0000)
                    .build();
                self.rest
                    .create_message_with_embed_and_forget(&log_channel, &[embed])
                    .await;
            }
        }

        Ok(())
    }
}
