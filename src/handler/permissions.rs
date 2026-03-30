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
            if let Ok(guild) = self.get_guild(ctx.guild_id).await {
                let roles = &guild.roles;
                let Some(member) = ctx.message.member.as_ref() else {
                    tracing::warn!("Message member is None in check_permission");
                    return Ok(false);
                };
                let present = &member.roles;

                let perms = PermissionSet::from_discord_permissions(roles, present);

                if perms.has_permission(&perm) {
                    return Ok(true);
                }
            }
        }

        if let Some(groups) = &config.permission_groups {
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

        Ok(false)
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
