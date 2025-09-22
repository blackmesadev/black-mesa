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
        check_permission!(self, config, ctx, Permission::ModerationKick);

        let targets = args.get_targets();

        if targets.is_empty() {
            self.missing_parameters(config, ctx, args, schema::USER_TARGET)
                .await?;
            return Ok(());
        }

        check_can_target!(self, config, ctx, &targets);

        let reason = args.get_first_text();

        let mut infractions = Vec::with_capacity(targets.len());

        for target in targets {
            let reason_ref = reason.map(std::borrow::Cow::Borrowed);
            let infraction = self
                .kick_user(ctx.guild_id, &target, &ctx.user.id, reason_ref)
                .await?;
            infractions.push(infraction);
        }

        self.send_infraction_channel(ctx.channel_id, &infractions, config.prefer_embeds)
            .await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn ban_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::ModerationBan);

        let targets = args.get_targets();

        if targets.is_empty() {
            self.missing_parameters(config, ctx, args, schema::USER_TARGET)
                .await?;
            return Ok(());
        }

        check_can_target!(self, config, ctx, &targets);

        let duration = args.get_first_duration();
        let reason = args.get_first_text();

        let mut infractions = Vec::with_capacity(targets.len());

        for target in targets {
            let reason_ref = reason.map(std::borrow::Cow::Borrowed);
            let infraction = self
                .ban_user(
                    ctx.guild_id,
                    &target,
                    &ctx.user.id,
                    duration,
                    reason_ref,
                )
                .await?;
            infractions.push(infraction);
        }

        self.send_infraction_channel(ctx.channel_id, &infractions, config.prefer_embeds)
            .await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn mute_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::ModerationMute);

        let mute_role = match config.mute_role.as_ref() {
            Some(role) => role,
            None => {
                tracing::info!("No mute role set");
                return Ok(());
            }
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

        let mut infractions = Vec::with_capacity(targets.len());

        for target in targets {
            let reason_ref = reason.map(std::borrow::Cow::Borrowed);
            let infraction = self
                .mute_user(
                    ctx.guild_id,
                    &target,
                    &ctx.user.id,
                    mute_role,
                    duration,
                    reason_ref,
                )
                .await?;
            infractions.push(infraction);
        }

        self.send_infraction_channel(ctx.channel_id, &infractions, config.prefer_embeds)
            .await?;

        Ok(())
    }

    pub async fn warn_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::ModerationWarn);

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

        let mut infractions = Vec::with_capacity(targets.len());

        for target in targets {
            let reason_ref = reason.map(std::borrow::Cow::Borrowed);
            let infraction = self
                .warn_user(
                    ctx.guild_id,
                    &target,
                    &ctx.user.id,
                    Some(duration),
                    reason_ref,
                )
                .await?;
            infractions.push(infraction);
        }

        self.send_infraction_channel(ctx.channel_id, &infractions, config.prefer_embeds)
            .await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn unban_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::ModerationUnban);

        let targets = args.get_targets();

        if targets.is_empty() {
            self.missing_parameters(config, ctx, args, schema::USER_TARGET)
                .await?;
            return Ok(());
        }

        let reason = args.get_first_text();

        for target in targets {
            let reason_ref = reason.map(std::borrow::Cow::Borrowed);
            self.unban_user(ctx.guild_id, &target, &ctx.user.id, reason_ref)
                .await?;
        }

        Ok(())
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn unmute_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::ModerationUnmute);

        let targets = args.get_targets();

        if targets.is_empty() {
            self.missing_parameters(config, ctx, args, schema::USER_TARGET)
                .await?;
            return Ok(());
        }

        let reason = args.get_first_text();

        for target in targets {
            let reason_ref = reason.map(std::borrow::Cow::Borrowed);
            self.unmute_user(ctx.guild_id, &target, &ctx.user.id, reason_ref)
                .await?;
        }

        Ok(())
    }

    pub async fn pardon_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::ModerationPardon);

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
            if let Some(infraction) = self
                .pardon(ctx.guild_id, &target, &ctx.user.id, None)
                .await?
            {
                infractions.push(infraction);
            }
        }

        if infractions.is_empty() {
            embed = embed.field("No infractions found", ZWSP, false);
        } else {
            for infraction in infractions {
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

        self.rest
            .create_message_with_embed(ctx.channel_id, &vec![embed])
            .await?;

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
            check_permission!(self, config, ctx, Permission::UtilitySelfLookup);
            targets.push(ctx.user.id);
        } else {
            check_permission!(self, config, ctx, Permission::ModerationLookup);
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
                                .map(|word| {
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
                                        format!(
                                            "{}{}{}",
                                            &word[..1],
                                            "*".repeat(14),
                                            &word[15..16]
                                        )
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
                        .map(|s| {
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
            .create_message_with_embed(ctx.channel_id, &vec![embed])
            .await?;

        Ok(())
    }
}
