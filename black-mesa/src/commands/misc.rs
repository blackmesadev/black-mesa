use std::collections::HashSet;

use bm_lib::{
    discord::{
        commands::{Arg, Args, Ctx},
        DiscordError, DiscordResult, EmbedBuilder,
    },
    emojis::Emoji,
    model::Config,
    permissions::{Permission, PermissionSet},
    util,
};

use tracing::instrument;

use crate::{check_permission, EventHandler, AUTHOR_COLON_THREE, GOAT_ID, SERVICE_NAME};

const DOCS_URL: &str = "";
const HELP_STRING: &str = "Help can be found via the documentation at ";

impl EventHandler {
    #[instrument(skip(self, config, ctx))]
    pub async fn permissions_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::ConfigView);

        let guild = self.get_guild(ctx.guild_id).await?;
        let (lookup_id, roles) = match args.get(0) {
            Some(arg) => match arg {
                Arg::Id(id) | Arg::Role(id) | Arg::User(id) => {
                    if let Some(_) = guild.roles.iter().find(|r| r.id == *id) {
                        let mut hashset = HashSet::new();
                        hashset.insert(*id);
                        (*id, hashset)
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
            None => (
                ctx.user.id,
                ctx.message.member.as_ref().unwrap().roles.clone(),
            ),
        };

        let mut perms = PermissionSet::new();
        perms.extend(PermissionSet::from_discord_permissions(
            &guild.roles,
            &roles,
        ));

        if let Some(groups) = &config.permission_groups {
            for group in groups {
                if group.users.contains(&lookup_id)
                    || group.roles.iter().any(|role| roles.contains(role))
                {
                    perms.extend(group.permissions.clone());
                }
            }
        }

        let all_perms = Permission::all_permissions_vec();
        let chunks = all_perms.chunks(25);
        let total_pages = (all_perms.len() + 24) / 25;

        for (page_idx, chunk) in chunks.enumerate() {
            let mut embed = EmbedBuilder::new()
                .title(format!(
                    "Permissions (Page {}/{})",
                    page_idx + 1,
                    total_pages
                ))
                .description(format!("Permissions for `{}`", lookup_id.get()).as_str());

            for perm in chunk {
                if perms.has_permission(perm) {
                    embed = embed.field(perm.to_string(), "✅", true);
                } else {
                    embed = embed.field(perm.to_string(), "❌", true);
                }
            }

            let embed = embed
                .color(0xFF8C00)
                .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
                .build();

            self.rest
                .create_message_with_embed(&ctx.channel_id, &vec![embed])
                .await?;
        }

        Ok(())
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn ping_command(&self, ctx: &Ctx<'_>) -> DiscordResult<()> {
        let ping = self.ping.read().await.as_millis();

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
        let rust_ver = rustc_version::version()
            .map_err(|e| DiscordError::Other(format!("Failed to get rustc version {e}")))?;

        let uptime = util::format_duration(self.start_time.elapsed().as_secs());

        let guild_count = self.get_guild_count().await?;

        let embed = EmbedBuilder::new()
            .title("Black Mesa")
            .description("Performance, reliability, and quality.")
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
            .field(
                "Bot Version",
                format!("`v{}`", env!("CARGO_PKG_VERSION")),
                true,
            )
            .field("Rust Version", format!("`v{rust_ver}`"), true)
            .field("Uptime", format!("`{}`", uptime), true)
            .field("Guilds", format!("`{}`", guild_count), true)
            .field("Documentation", format!("[Here]({DOCS_URL})"), true)
            .build();

        self.rest
            .create_message_with_embed(ctx.channel_id, &vec![embed])
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

        let user = match self.get_user(&lookup_id).await {
            Ok(user) => user,
            Err(e) => {
                self.rest
                    .create_message_and_forget(&ctx.channel_id, "Failed to get user info")
                    .await;
                return Err(e);
            }
        };

        let member = match self.get_member(ctx.guild_id, &user.id).await {
            Ok(member) => member,
            Err(e) => {
                self.rest
                    .create_message_and_forget(&ctx.channel_id, "Failed to get user info")
                    .await;
                return Err(e);
            }
        };

        let guild = self.get_guild(ctx.guild_id).await?;

        let created_at = util::snowflake_to_timestamp(user.id) / 1000;
        let joined_at = chrono::DateTime::parse_from_rfc3339(&member.joined_at)
            .map_err(|e| DiscordError::Other(format!("Failed to parse joined_at timestamp: {e}")))?
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
            .create_message_with_embed(ctx.channel_id, &vec![embed])
            .await?;

        Ok(())
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn help_command(&self, ctx: &Ctx<'_>) -> DiscordResult<()> {
        match self.rest
            .create_message(
                &ctx.channel_id,
                &HELP_STRING,
            )
            .await {
            Ok(_) => {
                self.rest
                    .create_message(
                        &ctx.channel_id,
                        &format!("{} Help has been sent to your DMs", Emoji::Check),
                    )
                    .await?;
            }
            Err(e) => {
                tracing::warn!(error = ?e, "Failed to send help message");
                self.rest
                    .create_message(
                        &ctx.channel_id,
                        &format!("{} Failed to send help message to DM", Emoji::Cross),
                    )
                    .await?;
            }
        }        

        Ok(())
    }
}