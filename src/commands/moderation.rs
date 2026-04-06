use futures::future::try_join_all;
use tracing::instrument;

use crate::{
    check_can_target, check_permission,
    commands::schema,
    handler::{moderation::DEFAULT_WARN_LENGTH, EventHandler, ZWSP},
    AUTHOR_COLON_THREE, SERVICE_NAME,
};
use bm_lib::{
    discord::{
        commands::{Args, Ctx},
        DiscordResult, EmbedBuilder,
    },
    model::{automod::OffenseType, Config, Uuid},
    permissions::Permission,
};

impl EventHandler {
    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn kick_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MODERATION_KICK);

        let targets = args.get_targets();

        if targets.is_empty() {
            self.missing_parameters(config, ctx, args, schema::USER_TARGET)
                .await?;
            return Ok(());
        }

        check_can_target!(self, config, ctx, &targets);

        let reason = args.get_first_text();

        let infractions = match try_join_all(targets.iter().map(|target| {
            self.kick_user(
                ctx.guild_id,
                target,
                &ctx.user.id,
                reason.map(std::borrow::Cow::Borrowed),
            )
        }))
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to kick user: {}", e);
                self.send_error(&ctx.channel_id, e).await?;
                return Ok(());
            }
        };

        if let Err(e) = self
            .send_infraction_channel(ctx.channel_id, &infractions, config.prefer_embeds)
            .await
        {
            tracing::error!("Failed to send infraction channel message: {}", e);
            self.send_error(&ctx.channel_id, e).await?;
        }

        Ok(())
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn ban_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MODERATION_BAN);

        let targets = args.get_targets();

        if targets.is_empty() {
            self.missing_parameters(config, ctx, args, schema::USER_TARGET)
                .await?;
            return Ok(());
        }

        check_can_target!(self, config, ctx, &targets);

        let duration = args.get_first_duration();
        let reason = args.get_first_text();

        let infractions = match try_join_all(targets.iter().map(|target| {
            self.ban_user(
                ctx.guild_id,
                target,
                &ctx.user.id,
                duration,
                reason.map(std::borrow::Cow::Borrowed),
            )
        }))
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to ban user: {}", e);
                self.send_error(&ctx.channel_id, e).await?;
                return Ok(());
            }
        };

        if let Err(e) = self
            .send_infraction_channel(ctx.channel_id, &infractions, config.prefer_embeds)
            .await
        {
            tracing::error!("Failed to send infraction channel message: {}", e);
            self.send_error(&ctx.channel_id, e).await?;
        }

        Ok(())
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn mute_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MODERATION_MUTE);

        let Some(mute_role) = config.mute_role.as_ref() else {
            return Ok(()); // No mute role configured
        };

        let targets = args.get_targets();

        if targets.is_empty() {
            self.missing_parameters(config, ctx, args, schema::USER_TARGET)
                .await?;
            return Ok(());
        }

        check_can_target!(self, config, ctx, &targets);

        let duration = args.get_first_duration();
        let reason = args.get_first_text();

        let infractions = match try_join_all(targets.iter().map(|target| {
            self.mute_user(
                ctx.guild_id,
                target,
                &ctx.user.id,
                mute_role,
                duration,
                reason.map(std::borrow::Cow::Borrowed),
            )
        }))
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to mute user: {}", e);
                self.send_error(&ctx.channel_id, e).await?;
                return Ok(());
            }
        };

        if let Err(e) = self
            .send_infraction_channel(ctx.channel_id, &infractions, config.prefer_embeds)
            .await
        {
            tracing::error!("Failed to send infraction channel message: {}", e);
            self.send_error(&ctx.channel_id, e).await?;
        }

        Ok(())
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn warn_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MODERATION_WARN);

        let targets = args.get_targets();

        if targets.is_empty() {
            self.missing_parameters(config, ctx, args, schema::USER_TARGET)
                .await?;
            return Ok(());
        }

        check_can_target!(self, config, ctx, &targets);

        let duration = args.get_first_duration().unwrap_or_else(|| {
            config
                .default_warn_duration
                .unwrap_or_else(|| DEFAULT_WARN_LENGTH)
        });

        let reason = args.get_first_text();

        let infractions = match try_join_all(targets.iter().map(|target| {
            self.warn_user(
                ctx.guild_id,
                target,
                &ctx.user.id,
                Some(duration),
                reason.map(std::borrow::Cow::Borrowed),
            )
        }))
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to warn user: {}", e);
                self.send_error(&ctx.channel_id, e).await?;
                return Ok(());
            }
        };

        if let Err(e) = self
            .send_infraction_channel(ctx.channel_id, &infractions, config.prefer_embeds)
            .await
        {
            tracing::error!("Failed to send infraction channel message: {}", e);
            self.send_error(&ctx.channel_id, e).await?;
        }

        Ok(())
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn unban_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MODERATION_UNBAN);

        let targets = args.get_targets();

        if targets.is_empty() {
            self.missing_parameters(config, ctx, args, schema::USER_TARGET)
                .await?;
            return Ok(());
        }

        let reason = args.get_first_text();

        if let Err(e) = try_join_all(targets.iter().map(|target| {
            self.unban_user(
                ctx.guild_id,
                target,
                &ctx.user.id,
                reason.map(std::borrow::Cow::Borrowed),
            )
        }))
        .await
        {
            tracing::error!("Failed to unban user: {}", e);
            self.send_error(&ctx.channel_id, e).await?;
            return Ok(());
        }

        let mentions = targets
            .iter()
            .map(|id| format!("<@{}>", id))
            .collect::<Vec<_>>()
            .join(", ");
        self.rest
            .create_message_no_ping(
                &ctx.channel_id,
                &format!(
                    "{} Successfully unbanned {}",
                    bm_lib::emojis::Emoji::Check,
                    mentions
                ),
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn unmute_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MODERATION_UNMUTE);

        let targets = args.get_targets();

        if targets.is_empty() {
            self.missing_parameters(config, ctx, args, schema::USER_TARGET)
                .await?;
            return Ok(());
        }

        let reason = args.get_first_text();

        if let Err(e) = try_join_all(targets.iter().map(|target| {
            self.unmute_user(
                ctx.guild_id,
                target,
                &ctx.user.id,
                reason.map(std::borrow::Cow::Borrowed),
            )
        }))
        .await
        {
            tracing::error!("Failed to unmute user: {}", e);
            self.send_error(&ctx.channel_id, e).await?;
            return Ok(());
        }

        let mentions = targets
            .iter()
            .map(|id| format!("<@{}>", id))
            .collect::<Vec<_>>()
            .join(", ");
        self.rest
            .create_message_no_ping(
                &ctx.channel_id,
                &format!(
                    "{} Successfully unmuted {}",
                    bm_lib::emojis::Emoji::Check,
                    mentions
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn pardon_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MODERATION_PARDON);

        let targets: Vec<Uuid> = match args
            .raw_args()
            .iter()
            .map(|s| Uuid::from_string(s))
            .collect()
        {
            Some(targets) => targets,
            None => {
                self.missing_parameters(config, ctx, args, schema::UUID_TARGET)
                    .await?;

                return Ok(());
            }
        };

        if targets.is_empty() {
            tracing::info!("No targets provided to pardon");
            return Ok(());
        }

        let mut embed = EmbedBuilder::new()
            .title("Pardon")
            .description("Pardoned the following infractions")
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None);

        let mut infractions = Vec::with_capacity(targets.len());

        for target in targets {
            match self.pardon(ctx.guild_id, &target, &ctx.user.id, None).await {
                Ok(Some(infraction)) => {
                    // Send DM notification
                    if let Err(e) = self.send_infraction_remove_dm(&infraction).await {
                        tracing::warn!("Failed to send DM for pardoned infraction: {}", e);
                    }
                    infractions.push(infraction);
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::error!("Failed to pardon infraction: {}", e);
                    self.send_error(&ctx.channel_id, e).await?;
                    return Ok(());
                }
            }
        }

        if infractions.is_empty() {
            embed = embed.field("No infractions found", ZWSP, false);
        } else {
            for infraction in &infractions {
                let dur_str = match infraction.expires_at {
                    Some(expires) => format!("<t:{}:R>", expires),
                    None => "`Never`".to_string(),
                };

                let reason = infraction
                    .reason
                    .as_deref()
                    .map(|s| {
                        if s.len() <= 40 {
                            s.to_string()
                        } else {
                            format!("{}...", &s[..40])
                        }
                    })
                    .unwrap_or_else(|| String::from("No reason provided"));

                let uuid = infraction.uuid.to_string();

                embed = embed.field(
                    infraction.infraction_type.to_verb(),
                    format!(
                        "**Reason:** `{}`\n**Expires:** {}\n**UUID:** `{}`",
                        reason, dur_str, uuid
                    ),
                    true,
                );
            }
        }

        let embed = embed.build();

        if let Err(e) = self
            .rest
            .create_message_with_embed(ctx.channel_id, &[embed])
            .await
        {
            tracing::error!("Failed to send pardon message: {}", e);
            self.send_error(&ctx.channel_id, e).await?;
            return Ok(());
        }

        // Send channel notifications for each pardoned infraction
        for infraction in &infractions {
            if config.prefer_embeds {
                if let Err(e) = self
                    .send_infraction_remove_channel(ctx.channel_id, infraction)
                    .await
                {
                    tracing::warn!(
                        "Failed to send channel notification for pardoned infraction: {}",
                        e
                    );
                }
            }
        }

        Ok(())
    }

    pub async fn lookup_user_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        let mut targets = args.get_targets();

        if targets.is_empty() {
            check_permission!(self, config, ctx, Permission::UTILITY_SELFLOOKUP);
            targets.push(ctx.user.id);
        } else {
            check_permission!(self, config, ctx, Permission::MODERATION_LOOKUP);
        }

        let infractions = self
            .get_member_infractions(ctx.guild_id, &targets[0])
            .await?;

        let mut embed = EmbedBuilder::new()
            .title("Infractions")
            .description(format!(
                "A list of infractions for the user <@{}>",
                targets[0].get()
            ))
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None);

        for infraction in infractions {
            let dur_str = match infraction.expires_at {
                Some(expires) => format!("<t:{}:R>", expires),
                None => "`Never`".to_string(),
            };

            let uuid = infraction.uuid.to_string();

            let reason = match infraction.automod_offense {
                Some(offense) => match offense.typ {
                    OffenseType::Spam(ref spam_typ) => {
                        format!(
                            "**Automod:** `{}-{}`\n**Count:** `{}`\n**Interval:** `{}`\n",
                            spam_typ,
                            offense.typ,
                            offense.count.unwrap_or(0),
                            offense.interval.unwrap_or(0)
                        )
                    }
                    OffenseType::Censor(ref censor_typ) => {
                        format!(
                            "**Automod:** `{}-{}`\n**Word:** `{}`\n",
                            censor_typ,
                            offense.typ,
                            offense
                                .offending_filter
                                .as_deref()
                                .map(|word: &str| {
                                    if word.len() <= 16 {
                                        let word_truncated = &word[..word.len().min(16)];
                                        if word_truncated.len() <= 2 {
                                            "*".repeat(word_truncated.len())
                                        } else {
                                            format!(
                                                "{}{}{}",
                                                &word_truncated[..1],
                                                "*".repeat(word_truncated.len() - 2),
                                                &word_truncated[word_truncated.len() - 1..]
                                            )
                                        }
                                    } else {
                                        format!("{}{}{}", &word[..1], "*".repeat(14), &word[15..16])
                                    }
                                })
                                .unwrap_or_else(|| String::from("No word provided"))
                        )
                    }
                },
                None => {
                    let reason = infraction
                        .reason
                        .as_deref()
                        .map(|s: &str| {
                            if s.len() <= 61 {
                                s.to_string()
                            } else {
                                format!("{}...", &s[..61])
                            }
                        })
                        .unwrap_or_else(|| String::from("No reason provided"));

                    format!("**Reason:** `{}`\n", reason)
                }
            };

            embed = embed.field(
                infraction.infraction_type.to_verb(),
                format!("{}**Expires:** {}\n**UUID:** `{}`", reason, dur_str, uuid),
                true,
            );
        }

        let embed = embed.build();

        self.rest
            .create_message_with_embed(ctx.channel_id, &[embed])
            .await?;

        Ok(())
    }
}
