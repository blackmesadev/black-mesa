use std::collections::HashSet;

use bm_lib::{
    discord::{DiscordResult, Id},
    model::{Config, Group},
    permissions::{Permission, PermissionSet},
};
use tracing::instrument;

use super::EventHandler;
impl EventHandler {
    #[instrument(skip(self))]
    pub async fn new_group<'a>(
        &self,
        config: &'a mut Config,
        guild_id: &Id,
        name: &str,
    ) -> DiscordResult<&'a Config> {
        let group = Group::new(name);

        if let Some(groups) = config.permission_groups.as_mut() {
            groups.push(group);
        } else {
            config.permission_groups = Some(vec![group]);
        }

        self.set_config(guild_id, &config).await?;

        Ok(config)
    }

    #[instrument(skip(self))]
    pub async fn delete_group<'a>(
        &self,
        config: &'a mut Config,
        guild_id: &Id,
        name: &str,
    ) -> DiscordResult<&'a Config> {
        if let Some(groups) = config.permission_groups.as_mut() {
            groups.retain(|group| group.name != name);
        }

        self.set_config(guild_id, &config).await?;

        Ok(config)
    }

    #[instrument(skip(self))]
    pub async fn grant_permissions<'a>(
        &self,
        config: &'a mut Config,
        guild_id: &Id,
        group_name: &str,
        permissions: Vec<Permission>,
    ) -> DiscordResult<&'a Config> {
        if let Some(groups) = config.permission_groups.as_mut() {
            if let Some(group) = groups.iter_mut().find(|group| group.name == group_name) {
                group
                    .permissions
                    .extend(PermissionSet::from_vec(permissions));
            }
        }

        self.set_config(guild_id, &config).await?;

        Ok(config)
    }

    #[instrument(skip(self))]
    pub async fn revoke_permissions<'a>(
        &self,
        config: &'a mut Config,
        guild_id: &Id,
        group_name: &str,
        permissions: Vec<Permission>,
    ) -> DiscordResult<&'a Config> {
        if let Some(groups) = config.permission_groups.as_mut() {
            if let Some(group) = groups.iter_mut().find(|group| group.name == group_name) {
                group
                    .permissions
                    .retain(|perm| !permissions.contains(&perm));
            }
        }

        self.set_config(guild_id, &config).await?;

        Ok(config)
    }

    #[instrument(skip(self))]
    pub async fn add_user_to_group<'a>(
        &self,
        config: &'a mut Config,
        guild_id: &Id,
        group_name: &str,
        user_id: &Id,
    ) -> DiscordResult<&'a Config> {
        if let Some(groups) = config.permission_groups.as_mut() {
            if let Some(group) = groups.iter_mut().find(|group| group.name == group_name) {
                group.users.insert(*user_id);
            }
        }

        self.set_config(guild_id, &config).await?;

        Ok(config)
    }

    #[instrument(skip(self))]
    pub async fn remove_user_from_group<'a>(
        &self,
        config: &'a mut Config,
        guild_id: &Id,
        group_name: &str,
        user_id: &Id,
    ) -> DiscordResult<&'a Config> {
        if let Some(groups) = config.permission_groups.as_mut() {
            if let Some(group) = groups.iter_mut().find(|group| group.name == group_name) {
                group.users.retain(|id| id != user_id);
            }
        }

        self.set_config(guild_id, &config).await?;

        Ok(config)
    }

    #[instrument(skip(self))]
    pub async fn get_users_in_group<'a>(
        &self,
        config: &'a Config,
        group_name: &str,
    ) -> DiscordResult<Option<&'a HashSet<Id>>> {
        if let Some(groups) = &config.permission_groups {
            if let Some(group) = groups.iter().find(|group| group.name == group_name) {
                return Ok(Some(&group.users));
            }
        }

        Ok(None)
    }

    #[instrument(skip(self))]
    pub async fn add_role_to_group<'a>(
        &self,
        config: &'a mut Config,
        guild_id: &Id,
        group_name: &str,
        role_id: &Id,
    ) -> DiscordResult<&'a Config> {
        if let Some(groups) = config.permission_groups.as_mut() {
            if let Some(group) = groups.iter_mut().find(|group| group.name == group_name) {
                group.roles.insert(*role_id);
            }
        }

        self.set_config(guild_id, &config).await?;

        Ok(config)
    }

    #[instrument(skip(self))]
    pub async fn remove_role_from_group<'a>(
        &self,
        config: &'a mut Config,
        guild_id: &Id,
        group_name: &str,
        role_id: &Id,
    ) -> DiscordResult<&'a Config> {
        if let Some(groups) = config.permission_groups.as_mut() {
            if let Some(group) = groups.iter_mut().find(|group| group.name == group_name) {
                group.roles.retain(|id| id != role_id);
            }
        }

        self.set_config(guild_id, &config).await?;

        Ok(config)
    }

    #[instrument(skip(self))]
    pub async fn list_roles_in_group<'a>(
        &self,
        config: &'a Config,
        group_name: &str,
    ) -> DiscordResult<Option<&'a HashSet<Id>>> {
        if let Some(groups) = &config.permission_groups {
            if let Some(group) = groups.iter().find(|group| group.name == group_name) {
                return Ok(Some(&group.roles));
            }
        }

        Ok(None)
    }

    #[instrument(skip(self))]
    pub async fn get_user_groups<'a>(&self, config: &'a Config, user_id: &Id) -> Vec<&'a Group> {
        let mut user_groups = Vec::new();

        if let Some(groups) = &config.permission_groups {
            for group in groups {
                if group.users.contains(user_id) {
                    user_groups.push(group);
                }
            }
        }

        user_groups
    }
}
