use super::EventHandler;
use bm_lib::{
    discord::{commands::Ctx, DiscordResult, Id},
    model::Config,
    permissions::{Permission, PermissionSet},
};
use tracing::instrument;

impl EventHandler {
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
        if config.inherit_discord_perms {
            tracing::debug!("Checking Discord permissions");
            if let Ok(guild) = self.get_guild(ctx.guild_id).await {
                let roles = &guild.roles;
                let present = &ctx.message.member.as_ref().unwrap().roles;

                let perms = PermissionSet::from_discord_permissions(roles, present);

                if perms.has_permission(&perm) {
                    tracing::debug!("Permission granted via Discord roles");
                    return Ok(true);
                }
            }
        }

        if let Some(groups) = &config.permission_groups {
            tracing::debug!(group_count = groups.len(), "Checking permission groups");
            if groups
                .iter()
                .any(|group| group.users.contains(&ctx.user.id))
                && groups
                    .iter()
                    .any(|group| group.permissions.has_permission(&perm))
            {
                return Ok(true);
            }
        }

        tracing::debug!("Permission check failed");
        Ok(false)
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn check_can_target(&self, ctx: &Ctx<'_>, target_id: &Id) -> DiscordResult<bool> {
        let guild = self.get_guild(ctx.guild_id).await?;

        if guild.owner_id == Some(*target_id) {
            return Ok(false);
        }

        let highest_user_role = match guild
            .roles
            .iter()
            .filter(|role| {
                ctx.message
                    .member
                    .as_ref()
                    .unwrap()
                    .roles
                    .contains(&role.id)
            })
            .max_by_key(|role| role.position)
        {
            Some(role) => role,
            None => return Ok(false),
        };

        let target_member = match self.get_member(ctx.guild_id, target_id).await {
            Ok(member) => member,
            Err(e) => {
                tracing::warn!(error = ?e, "Failed to get member");
                return Ok(false);
            }
        };

        let target_highest_role = match guild
            .roles
            .iter()
            .filter(|role| target_member.roles.contains(&role.id))
            .max_by_key(|role| role.position)
        {
            Some(role) => role,
            None => return Ok(true), // No roles, so no permissions
        };

        Ok(highest_user_role.position > target_highest_role.position)
    }
}
