use bm_lib::{
    discord::{
        commands::{Args, Ctx},
        DiscordResult, EmbedBuilder,
    },
    emojis::Emoji,
    model::Config,
};
use tracing::instrument;

use crate::{AUTHOR_COLON_THREE, SERVICE_NAME};

use super::{EventHandler, ZWSP};

impl EventHandler {
    #[instrument(skip(self, ctx))]
    pub async fn missing_parameters_embed(
        &self,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
        missing: &str,
    ) -> DiscordResult<()> {
        let embed = EmbedBuilder::new()
            .title("Missing Parameters")
            .description("You are missing required parameters for this command")
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
            .field(ZWSP, format!("**Required:** `{}`", missing).as_str(), true)
            .build();

        self.rest
            .create_message_with_embed(ctx.channel_id, &vec![embed])
            .await?;

        Ok(())
    }

    #[instrument(skip(self, ctx))]
    pub async fn missing_parameters_text(&self, ctx: &Ctx<'_>, missing: &str) -> DiscordResult<()> {
        self.rest
            .create_message(
                ctx.channel_id,
                &format!(
                    "{} You are missing required parameters for this command `{}`",
                    Emoji::Cross,
                    missing
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn missing_parameters(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
        missing: &str,
    ) -> DiscordResult<()> {
        if config.prefer_embeds {
            self.missing_parameters_embed(ctx, args, missing).await
        } else {
            self.missing_parameters_text(ctx, missing).await
        }
    }

    #[instrument(skip(self, ctx))]
    pub async fn incorrect_parameter_type_embed(
        &self,
        ctx: &Ctx<'_>,
        arg: &str,
        expected: &str,
    ) -> DiscordResult<()> {
        let embed = EmbedBuilder::new()
            .title("Incorrect Parameter Type")
            .description("You have provided an incorrect parameter type for this command")
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
            .field(ZWSP, format!("**Expected:** `{}`", expected).as_str(), true)
            .field(ZWSP, format!("**Provided:** `{}`", arg).as_str(), true)
            .build();

        self.rest
            .create_message_with_embed(ctx.channel_id, &vec![embed])
            .await?;

        Ok(())
    }
}
