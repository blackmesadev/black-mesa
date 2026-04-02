use crate::{handler::EventHandler, GOAT_ID};
use bm_lib::{
    discord::{
        commands::{Args, Ctx},
        DiscordResult,
    },
    model::Config,
};
use tracing::instrument;
impl EventHandler {
    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id, message_id = %ctx.message.id, channel_id = %ctx.channel_id))]
    pub async fn handle_command(
        &self,
        config: &mut Config,
        ctx: &Ctx<'_>,
        command: &str,
        args: &mut Args<'_>,
    ) -> DiscordResult<()> {
        if ctx.user.id == GOAT_ID {
            self.handle_privileged_command(config, ctx, command, args)
                .await?;
        }

        match command {
            // Utility commands
            "ping" => self.ping_command(ctx).await,

            "botinfo" => self.botinfo_command(ctx).await,
            "userinfo" => self.userinfo_command(ctx, args).await,
            "help" => self.help_command(ctx).await,

            // Configuration commands
            "resetconfig" => self.resetconfig_command(config, ctx).await,
            "reset" => self.reset_command(config, ctx, args).await,
            "setprefix" => self.setprefix_command(config, ctx, args).await,
            "setconfig" => self.setconfig_command(config, ctx, args).await,
            "addalias" => self.add_alias_command(config, ctx, args).await,
            "removealias" => self.remove_alias_command(config, ctx, args).await,
            "aliases" => self.list_aliases_command(config, ctx).await,
            "group" => self.group_command(config, ctx, args).await,

            // Moderation commands
            "kick" => self.kick_command(config, ctx, args).await,
            "ban" => self.ban_command(config, ctx, args).await,
            "unban" => self.unban_command(config, ctx, args).await,
            "mute" => self.mute_command(config, ctx, args).await,
            "unmute" => self.unmute_command(config, ctx, args).await,
            "warn" => self.warn_command(config, ctx, args).await,
            "pardon" => self.pardon_command(config, ctx, args).await,

            "lookup" => self.lookup_user_command(config, ctx, args).await,

            // Music commands
            "enqueue" => self.enqueue_command(config, ctx, args).await,
            "queue" => self.queue_command(config, ctx, args).await,
            "clearqueue" => self.clearqueue_command(config, ctx, args).await,
            "playlistsave" => self.playlistsave_command(config, ctx, args).await,
            "playlistenqueue" => self.playlistenqueue_command(config, ctx, args).await,
            "play" => self.play_command(config, ctx, args).await,
            "pause" => self.pause_command(config, ctx, args).await,
            "resume" => self.resume_command(config, ctx, args).await,
            "skip" => self.skip_command(config, ctx, args).await,
            "stop" => self.stop_command(config, ctx, args).await,
            "seek" => self.seek_command(config, ctx, args).await,
            "volume" => self.volume_command(config, ctx, args).await,
            "current" => self.current_command(config, ctx, args).await,

            _ => {
                tracing::debug!("Unknown command: {}", command);
                return Ok(());
            }
        }
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id, message_id = %ctx.message.id, channel_id = %ctx.channel_id))]
    async fn handle_privileged_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        command: &str,
        args: &mut Args<'_>,
    ) -> DiscordResult<()> {
        match command {
            "clearcache" => self.clear_cache_command(ctx).await,
            "permissions" => self.permissions_command(config, ctx, args).await,
            "shutdown" => self.shutdown_command(ctx).await,
            _ => Ok(()),
        }
    }
}
