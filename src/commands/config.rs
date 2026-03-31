use crate::{
    check_permission, commands::schema, get_raw_arg, handler::EventHandler, AUTHOR_COLON_THREE,
    SERVICE_NAME,
};
use bm_lib::{
    discord::{
        commands::{parse_duration, Arg, Args, Ctx},
        DiscordResult, EmbedBuilder, Id,
    },
    emojis::Emoji,
    model::Config,
    permissions::Permission,
};
use tracing::instrument;

macro_rules! set_bool_config {
    ($self:expr, $ctx:expr, $value:expr, $field:expr) => {
        match $value.parse().ok() {
            Some(v) => $field = v,
            None => {
                $self
                    .incorrect_parameter_type_embed($ctx, "text", "bool")
                    .await?;
                return Ok(());
            }
        }
    };
}

macro_rules! set_id_config {
    ($self:expr, $ctx:expr, $value:expr, $field:expr) => {
        match $value.parse::<u64>().ok() {
            Some(v) => $field = Some(Id::new(v)),
            None => {
                $self
                    .incorrect_parameter_type_embed($ctx, "text", "id")
                    .await?;
                return Ok(());
            }
        }
    };
}

macro_rules! set_duration_config {
    ($self:expr, $ctx:expr, $value:expr, $field:expr) => {
        match parse_duration($value) {
            Some(v) => $field = Some(v),
            None => {
                $self
                    .incorrect_parameter_type_embed($ctx, "text", "duration")
                    .await?;
                return Ok(());
            }
        }
    };
}

impl EventHandler {
    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn resetconfig_command(&self, config: &Config, ctx: &Ctx<'_>) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::CONFIG_EDIT);
        self.reset_config(ctx.guild_id).await
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn setprefix_command(
        &self,
        config: &mut Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::CONFIG_EDIT);

        let Some(Arg::Text(prefix)) = args.get(0) else {
            self.missing_parameters(config, ctx, args, schema::PREFIX)
                .await?;
            return Ok(());
        };

        config.prefix = prefix.to_string();

        self.set_config(ctx.guild_id, &config).await?;

        let embed = EmbedBuilder::new()
            .title("Prefix Set")
            .description(format!("The prefix has been set to `{}`", prefix))
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
            .build();

        self.rest
            .create_message_with_embed(ctx.channel_id, &[embed])
            .await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn setconfig_command(
        &self,
        config: &mut Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::CONFIG_EDIT);

        let key = get_raw_arg!(self, &config, ctx, args, 0, schema::SET_CONFIG);
        let value = get_raw_arg!(self, &config, ctx, args, 1, schema::SET_CONFIG);

        match key.to_string().as_str() {
            "prefix" => config.prefix = value.to_string(),
            "mute_role" => set_id_config!(self, ctx, value, config.mute_role),
            "default_warn_duration" => {
                set_duration_config!(self, ctx, value, config.default_warn_duration)
            }
            "log_channel" => set_id_config!(self, ctx, value, config.log_channel),
            "prefer_embeds" => set_bool_config!(self, ctx, value, config.prefer_embeds),
            "inherit_discord_perms" => {
                set_bool_config!(self, ctx, value, config.inherit_discord_perms)
            }
            "alert_on_infraction" => set_bool_config!(self, ctx, value, config.alert_on_infraction),
            "send_permission_denied" => {
                set_bool_config!(self, ctx, value, config.send_permission_denied)
            }
            _ => {
                self.rest
                    .create_message(
                        &ctx.channel_id,
                        format!("{} Invalid key", Emoji::Cross).as_str(),
                    )
                    .await?;
                return Ok(());
            }
        }

        self.set_config(ctx.guild_id, config).await?;

        let msg = format!(
            "{} Successfully updated the config key `{}` with value `{}`",
            Emoji::Check,
            key,
            value
        );

        self.rest.create_message(&ctx.channel_id, &msg).await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn add_alias_command(
        &self,
        config: &mut Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::CONFIG_EDIT);

        let alias = get_raw_arg!(self, &config, ctx, args, 0, schema::ADD_ALIAS);
        let command = get_raw_arg!(self, &config, ctx, args, 1, schema::ADD_ALIAS);

        if let Some(aliases) = config.command_aliases.as_mut() {
            aliases.insert(alias.to_string(), command.to_string());
        } else {
            config.command_aliases =
                Some(std::iter::once((alias.to_string(), command.to_string())).collect());
        }

        self.set_config(ctx.guild_id, &config).await?;

        let msg = format!(
            "{} Successfully added the alias `{}` for the command `{}`",
            Emoji::Check,
            alias,
            command
        );

        self.rest.create_message(&ctx.channel_id, &msg).await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn remove_alias_command(
        &self,
        config: &mut Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::CONFIG_EDIT);

        let alias = get_raw_arg!(self, &config, ctx, args, 0, schema::REMOVE_ALIAS);

        if let Some(aliases) = config.command_aliases.as_mut() {
            if aliases.remove(&alias.to_string()).is_none() {
                self.rest
                    .create_message(
                        &ctx.channel_id,
                        &format!("{} Alias `{}` not found", Emoji::Cross, alias),
                    )
                    .await?;
                return Ok(());
            }
        } else {
            self.rest
                .create_message(
                    &ctx.channel_id,
                    &format!("{} Alias `{}` not found", Emoji::Cross, alias),
                )
                .await?;
            return Ok(());
        }

        self.set_config(ctx.guild_id, &config).await?;

        let msg = format!(
            "{} Successfully removed the alias `{}`",
            Emoji::Check,
            alias
        );

        self.rest.create_message(&ctx.channel_id, &msg).await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn list_aliases_command(&self, config: &Config, ctx: &Ctx<'_>) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::CONFIG_VIEW);

        let Some(aliases) = &config.command_aliases else {
            self.rest
                .create_message(
                    &ctx.channel_id,
                    &format!("{} No aliases found", Emoji::Cross),
                )
                .await?;
            return Ok(());
        };

        let mut msg = format!("{} Found {} Aliases:\n", Emoji::Check, aliases.len());
        for (alias, command) in aliases {
            msg.push_str(&format!("`{}` -> `{}`\n", alias, command));
        }

        self.rest.create_message(&ctx.channel_id, &msg).await?;

        Ok(())
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn clear_cache_command(&self, ctx: &Ctx<'_>) -> DiscordResult<()> {
        // No permission check as this command is only available to the bot owner
        self.clear_cache(ctx.guild_id).await?;

        self.rest
            .create_message(
                &ctx.channel_id,
                &format!("{} Successfully cleared the guild cache", Emoji::Check),
            )
            .await?;

        Ok(())
    }
}
