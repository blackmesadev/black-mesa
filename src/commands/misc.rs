use bm_lib::{
    discord::{
        commands::{Arg, Args, Ctx},
        DiscordError, DiscordResult, EmbedBuilder,
    },
    emojis::Emoji,
    model::Config,
    permissions::Permission,
    util,
};

use tracing::instrument;

use crate::{check_permission, EventHandler, AUTHOR_COLON_THREE, GOAT_ID, SERVICE_NAME};

const DOCS_URL: &str = "";
const HELP_STRING: &str = "Help can be found via the documentation at ";

impl EventHandler {
    #[instrument(skip(self, config, ctx))]
    pub async fn reset_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        // Check if they are the owner first - emergency override in case of misconfiguration
        let guild = self.get_guild(ctx.guild_id).await?;
        if Some(ctx.user.id) != guild.owner_id {
            // Check permissions if not owner
            check_permission!(self, config, ctx, Permission::CONFIG_EDIT);
        }

        let confirmed = matches!(args.get(0), Some(Arg::Text(s)) if *s == "confirm");
        let correct_guild = matches!(args.get(1), Some(Arg::Id(id)) if *id == *ctx.guild_id);

        if !confirmed || !correct_guild {
            self.rest
                .create_message(
                    &ctx.channel_id,
                    &format!(
                        "⚠️ This will reset **all** server configuration to defaults. To confirm, run: `{}reset confirm {}`",
                        config.prefix,
                        ctx.guild_id.get()
                    ),
                )
                .await?;
            return Ok(());
        }

        self.reset_config(ctx.guild_id).await?;

        self.rest
            .create_message(
                &ctx.channel_id,
                &format!(
                    "{} Server configuration has been reset to defaults.",
                    Emoji::Check
                ),
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx))]
    pub async fn permissions_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::CONFIG_VIEW);

        let guild = match self.get_guild(ctx.guild_id).await {
            Ok(g) => g,
            Err(e) => {
                tracing::error!("Failed to fetch guild for permissions command: {}", e);
                self.send_error(&ctx.channel_id, e).await?;
                return Ok(());
            }
        };

        let Some(member) = ctx.message.member.as_ref() else {
            tracing::warn!("Message member is None in permissions_command");
            self.rest
                .create_message_and_forget(&ctx.channel_id, "Failed to get member info")
                .await;
            return Ok(());
        };
        let member_roles = &member.roles;

        let (lookup_id, roles) = match args.get(0) {
            Some(arg) => match arg {
                Arg::Id(id) | Arg::Role(id) | Arg::User(id) => {
                    if guild.roles.iter().any(|r| r.id == *id) {
                        (*id, vec![*id])
                    } else {
                        match self.get_member(ctx.guild_id, id).await {
                            Ok(member) => (*id, member.roles),
                            Err(e) => {
                                tracing::warn!(error = ?e, "Failed to get member");
                                self.rest
                                    .create_message_and_forget(
                                        &ctx.channel_id,
                                        "Invalid ID or user not in server",
                                    )
                                    .await;
                                return Ok(());
                            }
                        }
                    }
                }
                _ => {
                    self.rest
                        .create_message_and_forget(&ctx.channel_id, "Invalid ID format")
                        .await;
                    return Ok(());
                }
            },
            None => (ctx.user.id, member_roles.clone()),
        };

        let perms = self
            .resolve_member_permissions(config, ctx.guild_id, lookup_id, &roles)
            .await?;

        let all_perms = Permission::all_permissions_vec();
        let total_pages = (all_perms.len() + 24) / 25;

        for (page_idx, chunk) in all_perms.chunks(25).enumerate() {
            let mut embed = EmbedBuilder::new()
                .title(format!(
                    "Permissions (Page {}/{})",
                    page_idx + 1,
                    total_pages
                ))
                .description(format!("Permissions for `{}`", lookup_id.get()));

            for perm in chunk {
                let status = if perms.has_permission(*perm) {
                    "✅"
                } else {
                    "❌"
                };
                embed = embed.field(perm.to_string(), status, true);
            }

            let embed = embed
                .color(0xFF8C00)
                .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
                .build();

            if let Err(e) = self
                .rest
                .create_message_with_embed(&ctx.channel_id, &[embed])
                .await
            {
                tracing::error!("Failed to send permissions embed: {}", e);
                self.send_error(&ctx.channel_id, e).await?;
                return Ok(());
            }
        }

        Ok(())
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn ping_command(&self, ctx: &Ctx<'_>) -> DiscordResult<()> {
        let ping = self.ping_nanos.load(std::sync::atomic::Ordering::Relaxed) / 1_000_000;

        self.rest
            .create_message(
                &ctx.channel_id,
                &format!("{} Pong! `{}ms`", Emoji::Check, ping),
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn botinfo_command(&self, ctx: &Ctx<'_>) -> DiscordResult<()> {
        let rust_ver = env!("RUSTC_VERSION");

        let uptime = util::format_duration(self.start_time.elapsed().as_secs());

        let guild_count = match self.get_guild_count().await {
            Ok(count) => count,
            Err(e) => {
                tracing::error!("Failed to fetch guild count: {}", e);
                self.send_error(&ctx.channel_id, e).await?;
                return Ok(());
            }
        };

        let embed = EmbedBuilder::new()
            .title("Black Mesa")
            .description("Black Mesa is a Discord Moderation bot designed with Performance, Reliability and Customisation in mind.")
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
            .field(
                "Bot Version",
                format!("`v{}`", env!("CARGO_PKG_VERSION")),
                true,
            )
            .field("Rust Version", format!("`v{rust_ver}`"), true)
            .field("Lib Version", format!("`v{}`", bm_lib::LIB_VERSION), true)
            .field("Uptime", format!("`{}`", uptime), true)
            .field("Guilds", format!("`{}`", guild_count), true)
            .field("Documentation", format!("[Here]({DOCS_URL})"), true)
            .build();

        self.rest
            .create_message_with_embed(&ctx.channel_id, &[embed])
            .await?;

        Ok(())
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn userinfo_command(&self, ctx: &Ctx<'_>, args: &Args<'_>) -> DiscordResult<()> {
        let lookup_id = match args.get(0) {
            Some(arg) => match arg {
                Arg::Id(id) | Arg::User(id) => *id,
                _ => {
                    self.rest
                        .create_message_and_forget(&ctx.channel_id, "Invalid ID format")
                        .await;
                    return Ok(());
                }
            },
            None => ctx.user.id,
        };

        let (user_result, member_result, guild_result) = tokio::join!(
            self.get_user(&lookup_id),
            self.get_member(ctx.guild_id, &lookup_id),
            self.get_guild(ctx.guild_id),
        );

        let user = match user_result {
            Ok(u) => u,
            Err(e) => { self.send_error(&ctx.channel_id, e).await?; return Ok(()); }
        };
        let member = match member_result {
            Ok(m) => m,
            Err(e) => { self.send_error(&ctx.channel_id, e).await?; return Ok(()); }
        };
        let guild = match guild_result {
            Ok(g) => g,
            Err(e) => { self.send_error(&ctx.channel_id, e).await?; return Ok(()); }
        };

        let created_at = util::snowflake_to_timestamp(user.id) / 1000;
        let joined_at = chrono::DateTime::parse_from_rfc3339(&member.joined_at)
            .map_err(|e| {
                DiscordError::ParseError(format!("Failed to parse joined_at timestamp: {e}"))
            })?
            .timestamp();

        let highest_role_id = guild
            .roles
            .iter()
            .filter(|r| member.roles.contains(&r.id))
            .max_by_key(|r| r.position)
            .map(|r| r.id)
            .unwrap_or(guild.id);

        let mut embed = EmbedBuilder::new()
            .title(format!("User Info for {}", user.username))
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
            .field("User ID", format!("`{}`", user.id), true)
            .field("Username", format!("`{}`", user.username), true);

        if user.discriminator != "0" {
            embed = embed.field("Discriminator", format!("`#{}`", user.discriminator), true);
        }

        embed = embed
            .field("Highest Role", format!("<@&{}>", highest_role_id), true)
            .field("Account created", format!("<t:{}:R>", created_at), true)
            .field("Joined", format!("<t:{}:R>", joined_at), true);

        if user.bot {
            embed = embed.field("Bot", "🤖", true);
        }

        if lookup_id == GOAT_ID {
            embed = embed.field("Goat", "🐐", true);
        }

        if let Some(nick) = &member.nick {
            embed = embed.field("Nickname", format!("`{}`", nick), true);
        }

        let embed = embed.build();

        self.rest
            .create_message_with_embed(ctx.channel_id, &[embed])
            .await?;

        Ok(())
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn help_command(&self, ctx: &Ctx<'_>) -> DiscordResult<()> {
        self.rest
            .create_message(&ctx.channel_id, &format!("{HELP_STRING}{DOCS_URL}"))
            .await?;
        Ok(())
    }
}
