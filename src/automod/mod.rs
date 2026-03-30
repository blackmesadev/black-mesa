mod censor;
mod macros;
mod spam;

use crate::{check_bypass, handler::EventHandler};

use bm_lib::{
    discord::{commands::Ctx, DiscordResult, EmbedBuilder},
    model::{
        automod::{AutomodSettings, OffenseType},
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

impl EventHandler {
    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id, message_id = %ctx.message.id, channel_id = %ctx.channel_id))]
    pub async fn handle_automod(&self, config: &Config, ctx: &Ctx<'_>) -> DiscordResult<()> {
        let Some(automod) = &config.automod else {
            return Ok(());
        };

        if !automod.enabled {
            return Ok(());
        }

        let Some(global) = &automod.global else {
            return Ok(());
        };

        if global.enabled {
            if let Some(result) = self.process_automod(config, ctx, global).await? {
                self.process_automod_infraction(config, ctx, &result.infraction)
                    .await?;
                return Ok(());
            }
        }

        let Some(channel) = automod.channels.get(&ctx.channel_id) else {
            return Ok(());
        };

        if channel.enabled {
            if let Some(result) = self.process_automod(config, ctx, channel).await? {
                self.process_automod_infraction(config, ctx, &result.infraction)
                    .await?;
                return Ok(());
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
        self.db.create_infraction(infraction).await?;
        if let Some(log_channel) = config.log_channel {
            let reason = if let Some(offense) = infraction.automod_offense.as_ref() {
                match &offense.typ {
                    OffenseType::Spam(typ) => {
                        let typ = typ.to_pretty_string();
                        format!(
                            "{typ} spam detection triggered: {} occurrences in {}ms",
                            offense.count.unwrap_or(0),
                            offense.interval.unwrap_or(0)
                        )
                    }
                    OffenseType::Censor(typ) => {
                        let typ = typ.to_pretty_string();

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

                        if let Some(pos) = message_content
                            .to_lowercase()
                            .find(&offending_word.to_lowercase())
                        {
                            let start = pos.saturating_sub(15);
                            let end = (pos + offending_word.len() + 15).min(message_content.len());
                            let context =
                                message_content[start..end].replace(&offending_word, &censored);
                            format!("{typ} censor triggered: \"...{context}...\"")
                        } else {
                            format!("{typ} censor triggered: {censored}")
                        }
                    }
                }
            } else {
                infraction
                    .reason
                    .clone()
                    .unwrap_or_else(|| "Automod action triggered".to_string())
            };

            let embed = EmbedBuilder::new()
                .title("Automod Action")
                .description(reason)
                .color(0xff0000)
                .build();
            self.rest
                .create_message_with_embed_and_forget(&log_channel, &vec![embed])
                .await;
        }
        Ok(())
    }
}
