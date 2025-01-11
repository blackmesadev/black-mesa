use crate::discord::{
    error::DiscordResult,
    model::{Channel, Gateway, Member, Role, User},
};
use dashmap::DashMap;
use reqwest::{header::HeaderMap, Client, Method, Response};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    borrow::Cow,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::instrument;

use super::{DiscordError, Embed, Guild, Id, Message};

const API_BASE: &str = "https://discord.com/api/v10";
const AUTH_PREFIX: &str = "Bot ";

#[derive(Debug, Clone)]
struct RateLimit {
    remaining: u32,
    reset_after: f64,
    is_global: bool,
}

impl RateLimit {
    fn from_headers(headers: &HeaderMap) -> Option<Self> {
        let remaining = headers
            .get("x-ratelimit-remaining")?
            .to_str()
            .ok()?
            .parse()
            .ok()?;
        let reset_after = headers
            .get("x-ratelimit-reset-after")?
            .to_str()
            .ok()?
            .parse()
            .ok()?;
        let is_global = headers.contains_key("x-ratelimit-global");

        Some(RateLimit {
            remaining,
            reset_after,
            is_global,
        })
    }

    fn should_wait(&self) -> Option<Duration> {
        if self.remaining == 0 {
            return Some(Duration::from_secs_f64(self.reset_after));
        }
        None
    }
}

pub struct DiscordRestClient {
    client: Client,
    token: Cow<'static, str>,
    rate_limits: DashMap<String, RateLimit>,
    global_rate_limit: RwLock<Option<(Instant, Duration)>>,
}

impl DiscordRestClient {
    pub fn new(token: Cow<'static, str>) -> Self {
        Self {
            client: Client::new(),
            token,
            rate_limits: DashMap::new(),
            global_rate_limit: RwLock::new(None),
        }
    }

    #[instrument(skip(self, data), fields(method = ?method, path = path))]
    async fn make_request<T>(
        &self,
        method: Method,
        path: &str,
        data: Option<T>,
        headers: Option<HeaderMap>,
        bucket: &str,
    ) -> DiscordResult<Response>
    where
        T: Serialize,
    {
        tracing::debug!(bucket = bucket, "Making API request");

        loop {
            if let Some((reset_time, wait)) = *self.global_rate_limit.read().await {
                if Instant::now() < reset_time {
                    tracing::warn!(wait_ms = wait.as_millis(), "Waiting for global rate limit");
                    sleep(reset_time - Instant::now()).await;
                }
            }

            if let Some(rate_limit) = self.rate_limits.get(bucket) {
                if let Some(wait_time) = rate_limit.should_wait() {
                    tracing::warn!(
                        bucket = bucket,
                        wait_ms = wait_time.as_millis(),
                        "Waiting for bucket rate limit"
                    );
                    sleep(wait_time).await;
                }
            }

            let auth_header = format!("{}{}", AUTH_PREFIX, self.token.as_ref());
            let url = format!("{}{}", API_BASE, path);

            let request = match method {
                Method::GET => self.client.get(&url),
                Method::POST => {
                    let request = self.client.post(&url);
                    if let Some(ref data) = data {
                        request.json(&data)
                    } else {
                        request
                    }
                }
                Method::PUT => self.client.put(&url),
                Method::PATCH => {
                    let request = self.client.patch(&url);
                    if let Some(ref data) = data {
                        request.json(&data)
                    } else {
                        request
                    }
                }
                Method::DELETE => self.client.delete(&url),
                _ => self.client.get(&url),
            };

            let mut req = request.header("Authorization", auth_header);

            if let Some(ref headers) = headers {
                for (key, value) in headers.iter() {
                    req = req.header(key, value);
                }
            }

            let response = req.send().await.map_err(|e| {
                tracing::error!(error = %e, "Discord API request failed");
                e
            })?;

            if response.status() == 429 {
                tracing::warn!(bucket = bucket, status = 429, "Rate limited by Discord API");
                if let Some(rate_limit) = RateLimit::from_headers(response.headers()) {
                    if rate_limit.is_global {
                        let wait_time = Duration::from_secs_f64(rate_limit.reset_after);
                        *self.global_rate_limit.write().await =
                            Some((Instant::now() + wait_time, wait_time)); // Changed to .await
                    } else {
                        self.rate_limits.insert(bucket.to_string(), rate_limit);
                    }
                }
                continue;
            }

            if !response.status().is_success() {
                tracing::error!(
                    status = response.status().as_u16(),
                    bucket = bucket,
                    "Discord API request failed"
                );
                return Err(response.error_for_status().unwrap_err().into());
            }

            tracing::debug!(
                status = response.status().as_u16(),
                bucket = bucket,
                "Discord API request successful"
            );

            let rate_limit = RateLimit::from_headers(response.headers());

            if let Some(rate_limit) = rate_limit {
                self.rate_limits.insert(bucket.to_string(), rate_limit);
            }

            return Ok(response);
        }
    }

    pub async fn get_gateway(&self) -> DiscordResult<Gateway> {
        tracing::debug!("Fetching gateway URL");
        let response = self
            .make_request(Method::GET, "/gateway", Option::<()>::None, None, "gateway")
            .await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn get_channel(&self, channel_id: &Id) -> DiscordResult<Channel> {
        let response = self
            .make_request(
                Method::GET,
                &format!("/channels/{}", channel_id),
                Option::<()>::None,
                None,
                "channel",
            )
            .await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn get_guild(&self, guild_id: &Id) -> DiscordResult<Guild> {
        let response = self
            .make_request(
                Method::GET,
                &format!("/guilds/{}", guild_id),
                Option::<()>::None,
                None,
                "guild",
            )
            .await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn get_guild_channels(&self, guild_id: &Id) -> DiscordResult<Vec<Channel>> {
        let response = self
            .make_request(
                Method::GET,
                &format!("/guilds/{}/channels", guild_id),
                Option::<()>::None,
                None,
                "guild",
            )
            .await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn get_guild_members(&self, guild_id: &Id) -> DiscordResult<Vec<Member>> {
        let response = self
            .make_request(
                Method::GET,
                &format!("/guilds/{}/members", guild_id),
                Option::<()>::None,
                None,
                "guild",
            )
            .await?;
        let members = response.json().await?;

        Ok(members)
    }

    pub async fn get_member(&self, guild_id: &Id, user_id: &Id) -> DiscordResult<Member> {
        let response = self
            .make_request(
                Method::GET,
                &format!("/guilds/{}/members/{}", guild_id, user_id),
                Option::<()>::None,
                None,
                "guild",
            )
            .await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn get_roles(&self, guild_id: &Id) -> DiscordResult<Vec<Role>> {
        let response = self
            .make_request(
                Method::GET,
                &format!("/guilds/{}/roles", guild_id),
                Option::<()>::None,
                None,
                "guild",
            )
            .await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn get_user(&self, user_id: &Id) -> DiscordResult<User> {
        let response = self
            .make_request(
                Method::GET,
                &format!("/users/{}", user_id),
                Option::<()>::None,
                None,
                "user",
            )
            .await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn get_current_user(&self) -> DiscordResult<User> {
        let response = self
            .make_request(Method::GET, "/users/@me", Option::<()>::None, None, "user")
            .await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn create_message(&self, channel_id: &Id, content: &str) -> DiscordResult<Message> {
        tracing::debug!(channel_id = %channel_id, "Creating message");
        let data = json!({ "content": content });
        let response = self
            .make_request(
                Method::POST,
                &format!("/channels/{}/messages", channel_id),
                Some(&data),
                None,
                "channel",
            )
            .await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn edit_message(
        &self,
        channel_id: &Id,
        message_id: &Id,
        content: &str,
    ) -> DiscordResult<Message> {
        let data = json!({ "content": content });
        let response = self
            .make_request(
                Method::PATCH,
                &format!("/channels/{}/messages/{}", channel_id, message_id),
                Some(&data),
                None,
                "channel",
            )
            .await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn create_message_no_ping(
        &self,
        channel_id: &Id,
        content: &str,
    ) -> DiscordResult<Message> {
        let data = json!({ "content": content, "allowed_mentions": { "parse": [] } });
        let response = self
            .make_request(
                Method::POST,
                &format!("/channels/{}/messages", channel_id),
                Some(&data),
                None,
                "channel",
            )
            .await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn create_message_and_forget(&self, channel_id: &Id, content: &str) {
        let data = json!({ "content": content });
        let client = self.client.clone();
        let token = self.token.clone();
        let channel_id = channel_id.clone();
        tokio::spawn(async move {
            if let Err(e) = client
                .post(&format!("{}/channels/{}/messages", API_BASE, channel_id))
                .header("Authorization", format!("{}{}", AUTH_PREFIX, token))
                .json(&data)
                .send()
                .await
            {
                tracing::error!("Failed to send message: {:?}", e);
            }
        });
    }

    pub async fn create_message_with_embed(
        &self,
        channel_id: &Id,
        embeds: &Vec<Embed>,
    ) -> DiscordResult<Message> {
        let data = json!({ "embeds": embeds });
        let response = self
            .make_request(
                Method::POST,
                &format!("/channels/{}/messages", channel_id),
                Some(&data),
                None,
                "channel",
            )
            .await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn create_message_with_embed_and_forget(&self, channel_id: &Id, embeds: &Vec<Embed>) {
        let data = json!({ "embeds": embeds });
        let client = self.client.clone();
        let token = self.token.clone();
        let channel_id = channel_id.clone();
        tokio::spawn(async move {
            if let Err(e) = client
                .post(&format!("{}/channels/{}/messages", API_BASE, channel_id))
                .header("Authorization", format!("{}{}", AUTH_PREFIX, token))
                .json(&data)
                .send()
                .await
            {
                tracing::error!("Failed to send message: {:?}", e);
            }
        });
    }

    pub async fn create_dm_channel(&self, user_id: &Id) -> DiscordResult<Id> {
        let data = json!({ "recipient_id": user_id });
        let response = self
            .make_request(
                Method::POST,
                "/users/@me/channels",
                Some(&data),
                None,
                "channel",
            )
            .await?;
        let channel: Value = response.json().await?;
        let channel_id = channel["id"]
            .as_str()
            .ok_or(DiscordError::KeyNotFound("Channel ID not found".into()))?;
        Ok(channel_id
            .parse()
            .map_err(|_| DiscordError::ParseError("Channel ID".into()))?)
    }

    pub async fn kick_member(
        &self,
        guild_id: &Id,
        user_id: &Id,
        reason: Option<Cow<'_, str>>,
    ) -> DiscordResult<()> {
        let headers = reason.map(|r| {
            let mut headers = HeaderMap::new();
            headers.insert("X-Audit-Log-Reason", r.parse().unwrap());
            headers
        });

        self.make_request(
            Method::DELETE,
            &format!("/guilds/{}/members/{}", guild_id, user_id),
            Option::<()>::None,
            headers,
            "guild",
        )
        .await?;
        Ok(())
    }

    pub async fn ban_member(
        &self,
        guild_id: &Id,
        user_id: &Id,
        reason: Option<Cow<'_, str>>,
        delete_message_days: u8,
    ) -> DiscordResult<()> {
        let headers = reason.map(|r| {
            let mut headers = HeaderMap::new();
            headers.insert("X-Audit-Log-Reason", r.parse().unwrap());
            headers
        });

        let data = json!({ "delete_message_days": delete_message_days });
        self.make_request(
            Method::PUT,
            &format!("/guilds/{}/bans/{}", guild_id, user_id),
            Some(&data),
            headers,
            "guild",
        )
        .await?;
        Ok(())
    }

    pub async fn unban_member(
        &self,
        guild_id: &Id,
        user_id: &Id,
        reason: Option<Cow<'_, str>>,
    ) -> DiscordResult<()> {
        let headers = reason.map(|r| {
            let mut headers = HeaderMap::new();
            headers.insert("X-Audit-Log-Reason", r.parse().unwrap());
            headers
        });

        self.make_request(
            Method::DELETE,
            &format!("/guilds/{}/bans/{}", guild_id, user_id),
            Option::<()>::None,
            headers,
            "guild",
        )
        .await?;
        Ok(())
    }

    pub async fn add_role(
        &self,
        guild_id: &Id,
        user_id: &Id,
        role_id: &Id,
        reason: Option<Cow<'_, str>>,
    ) -> DiscordResult<()> {
        let headers = reason.map(|r| {
            let mut headers = HeaderMap::new();
            headers.insert("X-Audit-Log-Reason", r.parse().unwrap());
            headers
        });

        self.make_request(
            Method::PUT,
            &format!("/guilds/{}/members/{}/roles/{}", guild_id, user_id, role_id),
            Option::<()>::None,
            headers,
            "guild",
        )
        .await?;
        Ok(())
    }

    pub async fn remove_role(
        &self,
        guild_id: &Id,
        user_id: &Id,
        role_id: &Id,
        reason: Option<Cow<'_, str>>,
    ) -> DiscordResult<()> {
        let headers = reason.map(|r| {
            let mut headers = HeaderMap::new();
            headers.insert("X-Audit-Log-Reason", r.parse().unwrap());
            headers
        });

        self.make_request(
            Method::DELETE,
            &format!("/guilds/{}/members/{}/roles/{}", guild_id, user_id, role_id),
            Option::<()>::None,
            headers,
            "guild",
        )
        .await?;
        Ok(())
    }

    pub async fn delete_message(&self, channel_id: &Id, message_id: &Id) -> DiscordResult<()> {
        self.make_request(
            Method::DELETE,
            &format!("/channels/{}/messages/{}", channel_id, message_id),
            Option::<()>::None,
            None,
            "channel",
        )
        .await?;
        Ok(())
    }

    pub async fn delete_message_and_forget(&self, channel_id: &Id, message_id: &Id) {
        let client = self.client.clone();
        let token = self.token.clone();
        let channel_id = channel_id.clone();
        let message_id = message_id.clone();
        tokio::spawn(async move {
            if let Err(e) = client
                .delete(&format!(
                    "{}/channels/{}/messages/{}",
                    API_BASE, channel_id, message_id
                ))
                .header("Authorization", format!("{}{}", AUTH_PREFIX, token))
                .send()
                .await
            {
                tracing::error!("Failed to delete message: {:?}", e);
            }
        });
    }
}
