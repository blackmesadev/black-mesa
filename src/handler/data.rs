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
const API_TIMEOUT: Duration = Duration::from_secs(30);

const GUILD_COUNT_KEY: &str = "guild_count";

#[inline]
fn guild_cache_key(guild_id: &Id) -> String {
    format!("guild:{}", guild_id)
}

#[inline]
fn member_cache_key(guild_id: &Id, user_id: &Id) -> String {
    format!("member:{}:{}", guild_id, user_id)
}

#[inline]
fn user_cache_key(user_id: &Id) -> String {
    format!("user:{}", user_id)
}

#[inline]
fn roles_cache_key(guild_id: &Id, user_id: &Id) -> String {
    format!("roles:{}:{}", guild_id, user_id)
}

#[inline]
fn dm_channel_cache_key(user_id: &Id) -> String {
    format!("dm_channel:{}", user_id)
}

#[inline]
fn voice_state_cache_key(guild_id: &Id, user_id: &Id) -> String {
    format!("voice_state:{}:{}", guild_id, user_id)
}

#[inline]
fn voice_state_session_key(guild_id: &Id, user_id: &Id) -> String {
    format!("voice_state:{}:{}:session", guild_id, user_id)
}

#[inline]
pub(crate) fn voice_server_creds_key(guild_id: &Id) -> String {
    format!("voice_server_creds:{}", guild_id)
}

impl EventHandler {
    /// Helper to execute Discord API calls with timeout
    async fn api_with_timeout<F, T>(&self, future: F) -> DiscordResult<T>
    where
        F: std::future::Future<Output = DiscordResult<T>>,
    {
        tokio::time::timeout(API_TIMEOUT, future)
            .await
            .map_err(|_| {
                DiscordError::ConnectionFailed(format!(
                    "Discord API request timed out after {} seconds",
                    API_TIMEOUT.as_secs()
                ))
            })?
    }

    #[instrument(skip(self))]
    pub async fn new_config(&self, guild_id: &Id) -> DiscordResult<Config> {
        let base = Config::new(guild_id);
        let config = self.db.create_config(&base).await?;
        self.cache.set(guild_id, &config, Some(CONFIG_TTL)).await?;
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
            return Ok(config);
        }

        let config = match self.db.get_config(&guild_id).await? {
            Some(config) => config,
            None => self.new_config(guild_id).await?,
        };

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
        let guild_key = guild_cache_key(guild_id);
        let config_key = guild_id;

        self.cache.delete(&guild_key).await?;
        self.cache.delete(config_key).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_guild(&self, guild_id: &Id) -> DiscordResult<Guild> {
        let key = guild_cache_key(guild_id);
        if let Some(guild) = self.cache.get::<String, Guild>(&key).await? {
            return Ok(guild);
        }

        let guild = self.api_with_timeout(self.rest.get_guild(guild_id)).await?;

        self.cache.set(&key, &guild, None).await?;

        Ok(guild)
    }

    #[instrument(skip(self, guild), fields(guild_id = %guild.id))]
    pub async fn set_guild(&self, guild: &Guild) -> DiscordResult<()> {
        let key = guild_cache_key(&guild.id);
        self.cache.set(&key, guild, None).await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_member(&self, guild_id: &Id, user_id: &Id) -> DiscordResult<Member> {
        let key = member_cache_key(guild_id, user_id);
        if let Some(member) = self.cache.get::<String, Member>(&key).await? {
            return Ok(member);
        }

        let member = self
            .api_with_timeout(self.rest.get_member(guild_id, user_id))
            .await?;

        self.cache.set(key, &member, Some(MEMBER_TTL)).await?;

        Ok(member)
    }

    #[instrument(skip(self))]
    pub async fn get_member_roles(
        &self,
        guild_id: &Id,
        user_id: &Id,
    ) -> DiscordResult<HashSet<Id>> {
        let key = roles_cache_key(guild_id, user_id);
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
        let key = roles_cache_key(guild_id, user_id);
        self.cache.set(&key, roles, None).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_user(&self, user_id: &Id) -> DiscordResult<User> {
        let key = user_cache_key(user_id);
        if let Some(user) = self.cache.get::<String, User>(&key).await? {
            return Ok(user);
        }

        let user = self.api_with_timeout(self.rest.get_user(user_id)).await?;

        self.cache.set(key, &user, Some(USER_TTL)).await?;

        Ok(user)
    }

    #[instrument(skip(self, user))]
    pub async fn set_user(&self, user_id: &Id, user: &User) -> DiscordResult<()> {
        let key = user_cache_key(user_id);
        self.cache.set(key, user, Some(USER_TTL)).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_user_dm_channel(&self, user_id: &Id) -> DiscordResult<Id> {
        let key = dm_channel_cache_key(user_id);
        let dm_channel = self.cache.get::<String, Id>(&key).await?;
        if let Some(channel_id) = dm_channel {
            return Ok(channel_id);
        }

        let channel = self
            .api_with_timeout(self.rest.create_dm_channel(user_id))
            .await?;

        self.cache.set(key, &channel, None).await?;

        Ok(channel)
    }

    #[instrument(skip(self))]
    pub async fn get_voice_state_channel(
        &self,
        guild_id: &Id,
        user_id: &Id,
    ) -> DiscordResult<Option<Id>> {
        let key = voice_state_cache_key(guild_id, user_id);
        self.cache
            .get::<String, Id>(&key)
            .await
            .map_err(DiscordError::from)
    }

    #[instrument(skip(self))]
    pub async fn set_voice_state_channel(
        &self,
        guild_id: &Id,
        user_id: &Id,
        channel_id: Option<&Id>,
    ) -> DiscordResult<()> {
        let key = voice_state_cache_key(guild_id, user_id);
        match channel_id {
            Some(channel_id) => self
                .cache
                .set(&key, channel_id, None)
                .await
                .map_err(DiscordError::from),
            None => self.cache.delete(&key).await.map_err(DiscordError::from),
        }
    }

    #[instrument(skip(self))]
    pub async fn get_voice_state_session(
        &self,
        guild_id: &Id,
        user_id: &Id,
    ) -> DiscordResult<Option<String>> {
        let key = voice_state_session_key(guild_id, user_id);
        self.cache
            .get::<String, String>(&key)
            .await
            .map_err(DiscordError::from)
    }

    #[instrument(skip(self))]
    pub async fn set_voice_state_session(
        &self,
        guild_id: &Id,
        user_id: &Id,
        session_id: Option<&str>,
    ) -> DiscordResult<()> {
        let key = voice_state_session_key(guild_id, user_id);
        match session_id {
            Some(session_id) => self
                .cache
                .set(&key, &session_id.to_string(), None)
                .await
                .map_err(DiscordError::from),
            None => self.cache.delete(&key).await.map_err(DiscordError::from),
        }
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
