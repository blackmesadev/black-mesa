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
        config: &Config,
        ctx: &Ctx<'_>,
        command: &str,
        args: &mut Args<'_>,
    ) -> DiscordResult<()> {
        if ctx.user.id == GOAT_ID {
            self.handle_privileged_command(config, ctx, command, args)
                .await?;
        }

        match command {
            "ping" => self.ping_command(ctx).await,

            "botinfo" => self.botinfo_command(ctx).await,
            "userinfo" => self.userinfo_command(ctx, args).await,

            "resetconfig" => self.resetconfig_command(&mut config.clone(), ctx).await,
            "setprefix" => self.setprefix_command(&mut config.clone(), ctx, args).await,
            "setconfig" => self.setconfig_command(&mut config.clone(), ctx, args).await,
            "addalias" => self.add_alias_command(&mut config.clone(), ctx, args).await,
            "removealias" => {
                self.remove_alias_command(&mut config.clone(), ctx, args)
                    .await
            }
            "aliases" => self.list_aliases_command(&mut config.clone(), ctx).await,
            "group" => self.group_command(&mut config.clone(), ctx, args).await,

            "kick" => self.kick_command(config, ctx, args).await,
            "ban" => self.ban_command(config, ctx, args).await,
            "unban" => self.unban_command(config, ctx, args).await,
            "mute" => self.mute_command(config, ctx, args).await,
            "unmute" => self.unmute_command(config, ctx, args).await,
            "warn" => self.warn_command(config, ctx, args).await,
            "pardon" => self.pardon_command(config, ctx, args).await,

            "lookup" => self.lookup_user_command(config, ctx, args).await,

            _ => {
                tracing::debug!("Unknown command: {}", command);
                return Ok(());
            }
        }
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id, message_id = %ctx.message.id, channel_id = %ctx.channel_id))]
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
