use crate::handler::EventHandler;
use bm_lib::{
    discord::{commands::Ctx, DiscordResult},
    model::{
        automod::{AutomodOffense, OffenseType, SpamFilter, SpamInterval, SpamType},
        Infraction,
    },
};
use std::time::Duration;
use tracing::instrument;

use super::AutomodResult;

impl EventHandler {
    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id, message_id = %ctx.message.id, channel_id = %ctx.channel_id))]
    pub(crate) async fn handle_spam(
        &self,
        ctx: &Ctx<'_>,
        filter: &SpamFilter,
    ) -> DiscordResult<Option<AutomodResult>> {
        if !filter.enabled || filter.filters.is_empty() {
            return Ok(None);
        }

        let message_content = &ctx.message.content;
        let newline_count = if filter.filters.contains_key(&SpamType::Newline) {
            message_content.matches('\n').count() as u64
        } else {
            0
        };

        let min_threshold = filter
            .action
            .iter()
            .map(|a| a.threshold)
            .min()
            .unwrap_or(u64::MAX);

        for (spam_type, interval) in &filter.filters {
            let should_check = match spam_type {
                SpamType::Message => true, // Always check message spam
                SpamType::Newline => newline_count >= interval.count,
            };

            if !should_check {
                continue;
            }

            let count = self.update_spam_counter(ctx, spam_type, interval).await?;

            if count < min_threshold {
                continue;
            }

            self.rest
                .delete_message_and_forget(ctx.channel_id, &ctx.message.id)
                .await;

            let violations = self
                .update_violation_counter(ctx, spam_type, interval)
                .await?;

            if let Some(action) = filter
                .action
                .iter()
                .filter(|a| violations >= a.threshold)
                .max_by_key(|a| a.threshold)
            {
                let expires_at =
                    chrono::Utc::now() + chrono::Duration::milliseconds(action.duration);

                return Ok(Some(AutomodResult::new(Infraction::new_automod(
                    *ctx.guild_id,
                    ctx.user.id,
                    ctx.user.id,
                    action.action.clone(),
                    Some(expires_at.timestamp() as u64),
                    AutomodOffense {
                        typ: OffenseType::Spam(spam_type.clone()),
                        message: message_content.clone(),
                        count: Some(count),
                        interval: Some(interval.interval),
                        offending_filter: None,
                    },
                    true,
                ))));
            }
        }

        Ok(None)
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id, message_id = %ctx.message.id, channel_id = %ctx.channel_id))]
    #[inline]
    async fn update_violation_counter(
        &self,
        ctx: &Ctx<'_>,
        spam_type: &SpamType,
        interval: &SpamInterval,
    ) -> DiscordResult<u64> {
        let key = format!(
            "spam_violations:{}:{}:{:?}",
            ctx.guild_id, ctx.user.id, spam_type
        );
        self.update_redis_counter(&key, interval.interval).await
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, channel_id = %ctx.channel_id))]
    #[inline]
    async fn update_spam_counter(
        &self,
        ctx: &Ctx<'_>,
        spam_type: &SpamType,
        interval: &SpamInterval,
    ) -> DiscordResult<u64> {
        let key = format!("spam:{}:{}:{:?}", ctx.guild_id, ctx.channel_id, spam_type);
        self.update_redis_counter(&key, interval.interval).await
    }

    #[inline]
    async fn update_redis_counter(&self, key: &str, interval_ms: u64) -> DiscordResult<u64> {
        let now = chrono::Utc::now().timestamp_millis() as f64;
        let window_start = now - interval_ms as f64;
        let now_str = now.to_string();

        self.cache.zadd(&key, now, &now_str).await?;
        self.cache
            .expire(&key, Duration::from_millis(interval_ms))
            .await?;
        self.cache.zremrangebyscore(&key, 0.0, window_start).await?;

        let count = self.cache.zcard(&key).await?;
        Ok(count)
    }
}
