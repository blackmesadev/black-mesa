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
        if !filter.enabled {
            return Ok(None);
        }

        for (typ, interval) in filter.filters.iter() {
            let count = match typ {
                // interval.count is irrelevant for message spam as it would mean that anything other than 1 would lead to
                // a config where every x message we would increment the counter for spam violations, which doesnt make sense for the user
                // ensure to reflect this in the frontend
                SpamType::Message => self.update_spam_counter(ctx, typ, interval).await?,
                SpamType::Newline => {
                    if ctx.message.content.matches('\n').count() as u64 >= interval.count {
                        self.update_spam_counter(ctx, typ, interval).await?
                    } else {
                        continue;
                    }
                }
            };

            tracing::debug!("{typ} spam count: {count}");

            if let Some(first_threshold) =
                filter.action.iter().filter_map(|a| Some(a.threshold)).min()
            {
                if count >= first_threshold {
                    self.rest
                        .delete_message_and_forget(ctx.channel_id, &ctx.message.id)
                        .await;

                    let violations = self.update_violation_counter(ctx, typ, interval).await?;

                    let action = filter
                        .action
                        .iter()
                        .filter(|a| violations == a.threshold)
                        .max_by_key(|a| a.threshold);

                    if let Some(action) = action {
                        let expires_at =
                            chrono::Utc::now() + chrono::Duration::milliseconds(action.duration);

                        return Ok(Some(AutomodResult::new(Infraction::new_automod(
                            *ctx.guild_id,
                            ctx.user.id,
                            ctx.user.id,
                            action.action.clone(),
                            Some(expires_at.timestamp() as u64),
                            AutomodOffense {
                                typ: OffenseType::Spam(typ.clone()),
                                message: ctx.message.content.clone(),
                                count: Some(count),
                                interval: Some(interval.interval),
                                offending_filter: None,
                            },
                            true,
                        ))));
                    }
                }
            }
        }

        Ok(None)
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id, message_id = %ctx.message.id, channel_id = %ctx.channel_id))]
    async fn update_violation_counter(
        &self,
        ctx: &Ctx<'_>,
        typ: &SpamType,
        interval: &SpamInterval,
    ) -> DiscordResult<u64> {
        let key = format!("spam_violations:{}:{}:{}", ctx.guild_id, ctx.user.id, typ);

        let now = chrono::Utc::now().timestamp_millis() as f64;
        let window_start = now - interval.interval as f64;

        self.cache.zadd(&key, now, &now.to_string()).await?;

        self.cache
            .expire(&key, Duration::from_millis(interval.interval))
            .await?;

        self.cache.zremrangebyscore(&key, 0.0, window_start).await?;

        let count = self.cache.zcard(&key).await?;

        Ok(count)
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, channel_id = %ctx.channel_id))]
    async fn update_spam_counter(
        &self,
        ctx: &Ctx<'_>,
        typ: &SpamType,
        interval: &SpamInterval,
    ) -> DiscordResult<u64> {
        let key = format!("spam:{}:{}:{}", ctx.guild_id, ctx.channel_id, typ);
        let now = chrono::Utc::now().timestamp_millis() as f64;
        let window_start = now - interval.interval as f64;

        self.cache.zadd(&key, now, &now.to_string()).await?;

        self.cache
            .expire(&key, Duration::from_millis(interval.interval))
            .await?;

        self.cache.zremrangebyscore(&key, 0.0, window_start).await?;

        let count = self.cache.zcard(&key).await?;

        Ok(count)
    }
}
