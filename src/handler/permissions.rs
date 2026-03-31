use std::collections::HashSet;

use super::EventHandler;
use bm_lib::{
    discord::{commands::Ctx, DiscordResult, Id},
    model::Config,
    permissions::Permission,
};
use tracing::instrument;

impl EventHandler {
    /// Compute the effective [`Permission`] for a guild member.
    ///
    /// Fetches the guild from cache/API to resolve Discord role permissions
    /// (when `config.inherit_discord_perms` is enabled) and combines them with
    /// any Black Mesa permission groups the member belongs to, either directly
    /// by user ID or via their Discord roles.
    pub async fn resolve_member_permissions(
        &self,
        config: &Config,
        guild_id: &Id,
        user_id: Id,
        member_roles: &HashSet<Id>,
    ) -> DiscordResult<Permission> {
        let mut perms = if config.inherit_discord_perms {
            let guild = self.get_guild(guild_id).await?;
            Permission::from_discord_permissions(&guild.roles, member_roles)
        } else {
            Permission::empty()
        };

        if let Some(groups) = &config.permission_groups {
            for group in groups {
                if group.users.contains(&user_id)
                    || group.roles.iter().any(|role| member_roles.contains(role))
                {
                    perms |= group.permissions;
                }
            }
        }

        Ok(perms)
    }
    #[instrument(
        skip(self, config, ctx),
        fields(
            guild_id = %ctx.guild_id,
            user_id = %ctx.user.id,
            permission = %perm
        )
    )]
    pub async fn check_permission(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        perm: Permission,
    ) -> DiscordResult<bool> {
        let Some(member) = ctx.message.member.as_ref() else {
            tracing::warn!("Message member is None in check_permission");
            return Ok(false);
        };

        let perms = self
            .resolve_member_permissions(config, ctx.guild_id, ctx.user.id, &member.roles)
            .await?;
        Ok(perms.has_permission(perm))
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn check_can_target(&self, ctx: &Ctx<'_>, target_id: &Id) -> DiscordResult<bool> {
        let guild = self.get_guild(ctx.guild_id).await?;

        if guild.owner_id == Some(*target_id) {
            return Ok(false);
        }

        let Some(member) = ctx.message.member.as_ref() else {
            tracing::warn!("Message member is None in check_can_target");
            return Ok(false);
        };

        let Some(role) = guild
            .roles
            .iter()
            .filter(|role| member.roles.contains(&role.id))
            .max_by_key(|role| role.position)
        else {
            return Ok(false);
        };

        let Ok(target_member) = self.get_member(ctx.guild_id, target_id).await else {
            return Ok(false); // Can't verify, assume can't target
        };

        let Some(target_role) = guild
            .roles
            .iter()
            .filter(|role| target_member.roles.contains(&role.id))
            .max_by_key(|role| role.position)
        else {
            return Ok(true); // No roles, can target
        };

        Ok(role.position > target_role.position)
    }
}
