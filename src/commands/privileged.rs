use crate::handler::EventHandler;
use bm_lib::discord::{commands::Ctx, DiscordResult};
use tracing::instrument;

impl EventHandler {
    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn shutdown_command(&self, ctx: &Ctx<'_>) -> DiscordResult<()> {
        Ok(())
    }
}
