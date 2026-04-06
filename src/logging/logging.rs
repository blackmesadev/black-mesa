use std::collections::HashMap;

use bm_lib::{
    discord::{DiscordResult, EmbedBuilder, Id},
    model::{logging::LogEvent, LogConfig},
};
use tracing::instrument;

use crate::{handler::EventHandler, logging::template::render_template};

impl EventHandler {
    pub(super) async fn send_log_embed(
        &self,
        channel_id: &Id,
        log_config: &LogConfig,
        vars: &HashMap<String, String>,
    ) {
        let title = log_config
            .embed_title
            .as_deref()
            .map(|t| render_template(t, vars))
            .unwrap_or_else(|| log_config.event.clone());

        let body = log_config
            .embed_body
            .as_deref()
            .map(|b| render_template(b, vars))
            .unwrap_or_default();

        let color = log_config.embed_color.unwrap_or(0x7289da);

        let mut builder = EmbedBuilder::new()
            .title(title)
            .description(body)
            .color(color)
            .timestamp();

        if let Some(footer) = &log_config.embed_footer {
            let rendered = render_template(footer, vars);
            builder = builder.footer(rendered, None);
        }

        let embed = builder.build();
        self.rest
            .create_message_with_embed_and_forget(channel_id, &[embed])
            .await;
    }

    pub(super) async fn send_log_text(
        &self,
        channel_id: &Id,
        log_config: &LogConfig,
        vars: &HashMap<String, String>,
    ) {
        let content = log_config
            .text_content
            .as_deref()
            .map(|t| render_template(t, vars))
            .unwrap_or_else(|| format!("[{}] event occurred", log_config.event));

        self.rest
            .create_message_and_forget(channel_id, &content)
            .await;
    }

    /// Main entry point: dispatch a typed logging event.
    /// Checks guild config `logging_enabled`, looks up `LogConfig` for the event,
    /// renders templates, and sends the log message to the configured channel.
    #[instrument(skip(self, event), fields(guild_id = %event.guild_id(), event = %event.event_type()))]
    pub async fn log_event(&self, event: LogEvent) -> DiscordResult<()> {
        let guild_id = event.guild_id().clone();
        let event_type = event.event_type();

        // Check early exits before allocating vars.
        let config = self.get_config(&guild_id).await?;
        if !config.logging_enabled {
            return Ok(());
        }

        let db_key = event_type.as_db_key();
        let log_config = match self.db.get_log_config_for_event(&guild_id, db_key).await {
            Ok(Some(lc)) => lc,
            Ok(None) => return Ok(()), // no config for this event
            Err(e) => {
                tracing::warn!(error = %e, "failed to fetch log config");
                return Ok(());
            }
        };

        let channel_id = log_config
            .channel_id
            .or(config.log_channel)
            .unwrap_or(Id::new(0));

        if channel_id.get() == 0 {
            return Ok(());
        }

        // Only allocate the vars map when we know we'll use it.
        let vars = event.into_vars();

        if log_config.embed {
            self.send_log_embed(&channel_id, &log_config, &vars).await;
        } else {
            self.send_log_text(&channel_id, &log_config, &vars).await;
        }

        Ok(())
    }
}
