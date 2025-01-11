use bm_lib::{
    discord::{
        commands::{Args, Ctx},
        DiscordResult, EmbedBuilder, Id,
    },
    emojis::Emoji,
    model::Config,
    permissions::Permission,
};

use tracing::instrument;

use crate::{
    check_permission, get_raw_arg, handler::EventHandler, AUTHOR_COLON_THREE, SERVICE_NAME,
};

use super::schema;

impl EventHandler {
    #[instrument(skip(self, config, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn group_command(
        &self,
        config: &mut Config,
        ctx: &Ctx<'_>,
        args: &mut Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::ConfigEdit);

        let subcommand = match args.pop_subcommand() {
            Some(subcommand) => subcommand,
            None => {
                self.rest
                    .create_message(ctx.channel_id, format!("{} Missing subcommand. Try `create`, `delete`, `list`, `add`, `remove`, `users`, `roles`, `grant`, `revoke`", Emoji::Cross).as_str())
                    .await?;
                return Ok(());
            }
        };

        match subcommand {
            "create" => self.create_group_subcommand(config, ctx, args).await?,
            "delete" => self.delete_group_subcommand(config, ctx, args).await?,
            "list" => self.list_groups(config, ctx, args).await?,
            "add" => self.add_to_group_subcommand(config, ctx, args).await?,
            "remove" => self.remove_from_group_subcommand(config, ctx, args).await?,
            "users" => {
                self.list_users_in_group_subcommand(config, ctx, args)
                    .await?
            }
            "roles" => {
                self.list_roles_in_group_subcommand(config, ctx, args)
                    .await?
            }
            "grant" => self.grant_permissions_subcommand(config, ctx, args).await?,
            "revoke" => {
                self.revoke_permissions_subcommand(config, ctx, args)
                    .await?
            }
            _ => {
                self.rest
                .create_message(ctx.channel_id, format!("{} Invalid subcommand. Try `create`, `delete`, `list`, `add`, `remove`, `users`, `roles`, `grant`, `revoke`", Emoji::Cross).as_str())
                    .await?;
            }
        }

        Ok(())
    }

    #[instrument(skip(self, config, ctx))]
    async fn create_group_subcommand(
        &self,
        config: &mut Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        let name = get_raw_arg!(self, config, ctx, args, 0, schema::PERMISSION_GROUP).to_string();

        self.new_group(config, ctx.guild_id, &name).await?;

        self.rest
            .create_message(
                ctx.channel_id,
                format!("{} Successfully created Group `{name}`", Emoji::Check).as_str(),
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx))]
    async fn delete_group_subcommand(
        &self,
        config: &mut Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        let name = get_raw_arg!(self, config, ctx, args, 0, schema::PERMISSION_GROUP).to_string();

        self.delete_group(config, ctx.guild_id, &name).await?;

        self.rest
            .create_message(
                ctx.channel_id,
                format!("{} Successfully deleted Group `{name}`", Emoji::Check).as_str(),
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx))]
    async fn list_groups(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        if args.raw_args().is_empty() {
            self.list_groups_subcommand(config, ctx).await
        } else {
            let group_name =
                get_raw_arg!(self, config, ctx, args, 0, schema::PERMISSION_GROUP).to_string();
            self.list_group_subcommand(config, ctx, group_name).await
        }
    }

    #[instrument(skip(self, config, ctx))]
    async fn list_groups_subcommand(&self, config: &Config, ctx: &Ctx<'_>) -> DiscordResult<()> {
        let groups = config.permission_groups.as_ref().map_or_else(
            || Vec::new(),
            |groups| {
                groups
                    .iter()
                    .map(|group| format!("`{}`", group.name))
                    .collect()
            },
        );

        let groups = groups.join(", ");

        self.rest
            .create_message(
                ctx.channel_id,
                format!("{} Fetched Groups: {groups}", Emoji::Check).as_str(),
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx))]
    async fn list_group_subcommand(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        group_name: String,
    ) -> DiscordResult<()> {
        let group = config
            .permission_groups
            .as_ref()
            .and_then(|groups| groups.iter().find(|group| group.name == group_name));

        if let Some(group) = group {
            let permissions = group
                .permissions
                .iter()
                .map(|perm| format!("`{}`", perm))
                .collect::<Vec<_>>()
                .join(", ");

            let embed = EmbedBuilder::new()
                .title(format!("Group `{}`", group.name))
                .field("User count", format!("{}", group.users.len()), true)
                .field("Role count", format!("{}", group.roles.len()), true)
                .field("Permissions", permissions, false)
                .color(0xFF8C00)
                .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
                .build();

            self.rest
                .create_message_with_embed(ctx.channel_id, &vec![embed])
                .await?;
        } else {
            self.rest
                .create_message(
                    ctx.channel_id,
                    format!("{} Group `{}` not found", Emoji::Cross, group_name).as_str(),
                )
                .await?;
        }

        Ok(())
    }

    #[instrument(skip(self, config, ctx))]
    async fn add_to_group_subcommand(
        &self,
        config: &mut Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        let group_name =
            get_raw_arg!(self, config, ctx, args, 0, schema::PERMISSION_GROUP).to_string();
        let target_id = match Id::from_str(get_raw_arg!(
            self,
            config,
            ctx,
            args,
            1,
            schema::USER_TARGET
        )) {
            Ok(id) => id,
            Err(_) => {
                self.rest
                    .create_message(
                        ctx.channel_id,
                        format!("{} Invalid user ID", Emoji::Cross).as_str(),
                    )
                    .await?;
                return Ok(());
            }
        };

        let is_role = self
            .get_guild(ctx.guild_id)
            .await?
            .roles
            .iter()
            .any(|role| role.id == target_id);

        let target = match is_role {
            true => {
                self.add_role_to_group(config, ctx.guild_id, &group_name, &target_id)
                    .await?;
                format!("<@&{}>", target_id)
            }
            false => {
                self.add_user_to_group(config, ctx.guild_id, &group_name, &target_id)
                    .await?;
                format!("<@{}>", target_id)
            }
        };

        self.rest
            .create_message_no_ping(
                ctx.channel_id,
                format!(
                    "{} Successfully added {target} to `{group_name}`",
                    Emoji::Check
                )
                .as_str(),
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx))]
    async fn remove_from_group_subcommand(
        &self,
        config: &mut Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        let group_name =
            get_raw_arg!(self, config, ctx, args, 0, schema::PERMISSION_GROUP).to_string();
        let target_id = match Id::from_str(get_raw_arg!(
            self,
            config,
            ctx,
            args,
            1,
            schema::USER_TARGET
        )) {
            Ok(id) => id,
            Err(_) => {
                self.rest
                    .create_message(
                        ctx.channel_id,
                        format!("{} Invalid user ID", Emoji::Cross).as_str(),
                    )
                    .await?;
                return Ok(());
            }
        };

        let is_role = self
            .get_guild(ctx.guild_id)
            .await?
            .roles
            .iter()
            .any(|role| role.id == target_id);

        let target = match is_role {
            true => {
                self.remove_role_from_group(config, ctx.guild_id, &group_name, &target_id)
                    .await?;
                format!("<@&{}>", target_id)
            }
            false => {
                self.remove_user_from_group(config, ctx.guild_id, &group_name, &target_id)
                    .await?;
                format!("<@{}>", target_id)
            }
        };

        self.rest
            .create_message_no_ping(
                ctx.channel_id,
                format!(
                    "{} Successfully removed {target} from `{group_name}`",
                    Emoji::Check
                )
                .as_str(),
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx))]
    async fn list_users_in_group_subcommand(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        let group_name =
            get_raw_arg!(self, config, ctx, args, 0, schema::PERMISSION_GROUP).to_string();

        let users = match self.get_users_in_group(config, &group_name).await? {
            Some(users) => users
                .iter()
                .map(|id| format!("<@{}>", id))
                .collect::<Vec<_>>()
                .join(", "),
            None => {
                self.rest
                    .create_message(
                        ctx.channel_id,
                        format!(
                            "{} No users present in group `{}`",
                            Emoji::Cross,
                            group_name
                        )
                        .as_str(),
                    )
                    .await?;
                return Ok(());
            }
        };

        let embed = EmbedBuilder::new()
            .title(format!("Users in Group `{}`", group_name))
            .description(users)
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
            .build();

        self.rest
            .create_message_with_embed(ctx.channel_id, &vec![embed])
            .await?;
        Ok(())
    }

    #[instrument(skip(self, config, ctx))]
    async fn list_roles_in_group_subcommand(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        let group_name =
            get_raw_arg!(self, config, ctx, args, 0, schema::PERMISSION_GROUP).to_string();

        let roles = match self.list_roles_in_group(config, &group_name).await? {
            Some(roles) => roles
                .iter()
                .map(|id| format!("<@&{}>", id))
                .collect::<Vec<_>>()
                .join(", "),
            None => {
                self.rest
                    .create_message(
                        ctx.channel_id,
                        format!(
                            "{} No roles present in group `{}`",
                            Emoji::Cross,
                            group_name
                        )
                        .as_str(),
                    )
                    .await?;
                return Ok(());
            }
        };

        let embed = EmbedBuilder::new()
            .title(format!("Roles in Group `{}`", group_name))
            .description(roles)
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
            .build();

        self.rest
            .create_message_with_embed(ctx.channel_id, &vec![embed])
            .await?;
        Ok(())
    }

    #[instrument(skip(self, config, ctx))]
    async fn grant_permissions_subcommand(
        &self,
        config: &mut Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        let group_name =
            get_raw_arg!(self, config, ctx, args, 0, schema::PERMISSION_GROUP).to_string();

        let permissions: Vec<Permission> = match args
            .iter_raw_args()
            .skip(1)
            .map(|arg| Permission::from_str(arg))
            .collect()
        {
            Some(permissions) => permissions,
            None => {
                self.rest
                    .create_message(
                        ctx.channel_id,
                        format!("{} Invalid permission specified", Emoji::Cross).as_str(),
                    )
                    .await?;
                return Ok(());
            }
        };

        let permissions_str = permissions
            .iter()
            .map(|perm| format!("`{}`", perm.to_string()))
            .collect::<Vec<_>>()
            .join(", ");

        self.grant_permissions(config, ctx.guild_id, &group_name, permissions)
            .await?;

        self.rest
            .create_message(
                ctx.channel_id,
                format!(
                    "{} Successfully granted permissions {permissions_str} to `{group_name}`",
                    Emoji::Check
                )
                .as_str(),
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self, config, ctx))]
    async fn revoke_permissions_subcommand(
        &self,
        config: &mut Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        let group_name =
            get_raw_arg!(self, config, ctx, args, 0, schema::PERMISSION_GROUP).to_string();

        let permissions: Vec<Permission> = match args
            .iter_raw_args()
            .skip(1)
            .map(|arg| Permission::from_str(arg))
            .collect()
        {
            Some(permissions) => permissions,
            None => {
                self.rest
                    .create_message(
                        ctx.channel_id,
                        format!("{} Invalid permission specified", Emoji::Cross).as_str(),
                    )
                    .await?;
                return Ok(());
            }
        };

        let permissions_str = permissions
            .iter()
            .map(|perm| format!("`{}`", perm.to_string()))
            .collect::<Vec<_>>()
            .join(", ");

        self.revoke_permissions(config, ctx.guild_id, &group_name, permissions)
            .await?;

        self.rest
            .create_message(
                ctx.channel_id,
                format!(
                    "{} Successfully revoked permissions {permissions_str} from `{group_name}`",
                    Emoji::Check
                )
                .as_str(),
            )
            .await?;

        Ok(())
    }
}
