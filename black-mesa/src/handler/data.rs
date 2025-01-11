use std::{collections::HashSet, time::Duration};

use bm_lib::{
    discord::{DiscordError, DiscordResult, Guild, Id, Member, User},
    model::{Config, Infraction},
};
use tracing::instrument;

use super::EventHandler;

const CONFIG_TTL: Duration = Duration::from_secs(60);
const USER_TTL: Duration = Duration::from_secs(180);
const MEMBER_TTL: Duration = Duration::from_secs(600);

const GUILD_COUNT_KEY: &str = "guild_count";

impl EventHandler {
    #[instrument(skip(self))]
    pub async fn new_config(&self, guild_id: &Id) -> DiscordResult<Config> {
        let config = Config::new(guild_id);
        self.cache.set(guild_id, &config, Some(CONFIG_TTL)).await?;
        self.db.create_config(&config).await?;
        Ok(config)
    }

    #[instrument(skip(self))]
    pub async fn reset_config(&self, guild_id: &Id) -> DiscordResult<()> {
        let config = Config::new(guild_id);
        self.cache.set(guild_id, &config, Some(CONFIG_TTL)).await?;
        self.db.update_config(guild_id, &config).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_config(&self, guild_id: &Id) -> DiscordResult<Config> {
        if let Some(config) = self.cache.get::<Id, Config>(&guild_id).await? {
            tracing::debug!("Config cache hit");
            return Ok(config);
        }

        tracing::debug!("Config cache miss, loading from database");
        let config = match self.db.get_config(&guild_id).await? {
            Some(config) => {
                tracing::debug!("Config loaded from database");
                config
            }
            None => {
                tracing::debug!("Creating new config");
                self.new_config(guild_id).await?
            }
        };

        tracing::debug!("Caching config");
        self.cache.set(guild_id, &config, Some(CONFIG_TTL)).await?;

        Ok(config)
    }

    #[instrument(skip(self, config))]
    pub async fn set_config(&self, guild_id: &Id, config: &Config) -> DiscordResult<()> {
        self.cache.set(guild_id, config, Some(CONFIG_TTL)).await?;
        self.db.update_config(guild_id, config).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn clear_cache(&self, guild_id: &Id) -> DiscordResult<()> {
        let guild_key = format!("guild:{}", guild_id);
        let config_key = guild_id;

        self.cache.delete(&guild_key).await?;
        self.cache.delete(config_key).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_guild(&self, guild_id: &Id) -> DiscordResult<Guild> {
        let key = format!("guild:{}", guild_id);
        if let Some(guild) = self.cache.get::<String, Guild>(&key).await? {
            return Ok(guild);
        }

        let guild = self.rest.get_guild(&guild_id).await?;

        self.cache.set(&key, &guild, None).await?;

        Ok(guild)
    }

    #[instrument(skip(self, guild), fields(guild_id = %guild.id))]
    pub async fn set_guild(&self, guild: &Guild) -> DiscordResult<()> {
        let key = format!("guild:{}", guild.id);
        self.cache.set(&key, guild, None).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_member(&self, guild_id: &Id, user_id: &Id) -> DiscordResult<Member> {
        let key = format!("member:{}:{}", guild_id, user_id);
        if let Some(member) = self.cache.get::<String, Member>(&key).await? {
            return Ok(member);
        }

        let member = self.rest.get_member(&guild_id, &user_id).await?;

        self.cache.set(key, &member, Some(MEMBER_TTL)).await?;

        Ok(member)
    }

    #[instrument(skip(self))]
    pub async fn refresh_member(&self, guild_id: &Id, user_id: &Id) -> DiscordResult<Member> {
        let member = self.rest.get_member(&guild_id, &user_id).await?;

        self.cache.set(user_id, &member, Some(MEMBER_TTL)).await?;

        Ok(member)
    }

    #[instrument(skip(self))]
    pub async fn get_member_roles(
        &self,
        guild_id: &Id,
        user_id: &Id,
    ) -> DiscordResult<HashSet<Id>> {
        let key = format!("roles:{}:{}", guild_id, user_id);
        if let Some(roles) = self.cache.get::<String, HashSet<Id>>(&key).await? {
            return Ok(roles);
        }

        let roles = self.get_member(guild_id, user_id).await?.roles;

        self.cache.set(&key, &roles, None).await?;

        Ok(roles)
    }

    #[instrument(skip(self))]
    pub async fn set_member_roles(
        &self,
        guild_id: &Id,
        user_id: &Id,
        roles: &HashSet<Id>,
    ) -> DiscordResult<()> {
        let key = format!("roles:{}:{}", guild_id, user_id);
        self.cache.set(&key, roles, None).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_user(&self, user_id: &Id) -> DiscordResult<User> {
        let key = format!("user:{}", user_id);
        if let Some(user) = self.cache.get::<String, User>(&key).await? {
            return Ok(user);
        }

        let user = self.rest.get_user(&user_id).await?;

        self.cache.set(key, &user, Some(USER_TTL)).await?;

        Ok(user)
    }

    #[instrument(skip(self, user))]
    pub async fn set_user(&self, user_id: &Id, user: &User) -> DiscordResult<()> {
        let key = format!("user:{}", user_id);
        self.cache.set(key, user, Some(USER_TTL)).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_user_dm_channel(&self, user_id: &Id) -> DiscordResult<Id> {
        let key = format!("dm_channel:{}", user_id);
        let dm_channel = self.cache.get::<String, Id>(&key).await?;
        if let Some(channel_id) = dm_channel {
            return Ok(channel_id);
        }

        let channel = self.rest.create_dm_channel(user_id).await?;

        self.cache.set(key, &channel, None).await?;

        Ok(channel)
    }

    #[instrument(skip(self))]
    pub async fn get_member_infractions(
        &self,
        guild_id: &Id,
        user_id: &Id,
    ) -> DiscordResult<Vec<Infraction>> {
        self.db
            .get_active_infractions(guild_id, user_id, None)
            .await
            .map_err(DiscordError::from)
    }

    #[instrument(skip(self))]
    pub async fn get_guild_count(&self) -> DiscordResult<usize> {
        if let Some(count) = self.cache.get(&GUILD_COUNT_KEY).await? {
            Ok(count)
        } else {
            Ok(0)
        }
    }

    #[instrument(skip(self), fields(guild_count = %count))]
    pub async fn set_guild_count(&self, count: usize) -> DiscordResult<()> {
        self.cache.set(GUILD_COUNT_KEY, &count, None).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn increment_guild_count(&self) -> DiscordResult<()> {
        self.cache.incr(&GUILD_COUNT_KEY, None).await?;
        Ok(())
    }
}
