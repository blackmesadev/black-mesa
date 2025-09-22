use crate::handler::EventHandler;
use bm_lib::{
    discord::{commands::Ctx, DiscordResult},
    model::{
        automod::{AutomodOffense, Censor, CensorType, OffenseType},
        Infraction,
    },
};
use tracing::instrument;

use super::AutomodResult;

#[derive(Clone)]
enum Pattern {
    Simple(Box<str>),
    Wildcard {
        start: Box<str>,
        end: Box<str>,
        require_middle: bool,
    },
}

impl Pattern {
    #[inline]
    fn matches(&self, text: &str) -> bool {
        match self {
            Pattern::Simple(pattern) => text.contains(pattern.as_ref()),
            Pattern::Wildcard {
                start,
                end,
                require_middle,
            } => {
                if start.is_empty() && end.is_empty() {
                    return true;
                }

                if !text.starts_with(start.as_ref()) || !text.ends_with(end.as_ref()) {
                    return false;
                }

                if *require_middle {
                    text.len() > start.len() + end.len()
                } else {
                    true
                }
            }
        }
    }

    #[inline]
    fn from_str(pattern: &str) -> Self {
        if pattern.contains("...") {
            let parts: Vec<&str> = pattern.split("...").collect();
            if parts.len() == 2 {
                return Pattern::Wildcard {
                    start: parts[0].into(),
                    end: parts[1].into(),
                    require_middle: true,
                };
            }
        } else if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return Pattern::Wildcard {
                    start: parts[0].into(),
                    end: parts[1].into(),
                    require_middle: false,
                };
            }
        }

        Pattern::Simple(pattern.into())
    }
}

impl EventHandler {
    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id, message_id = %ctx.message.id, channel_id = %ctx.channel_id))]
    pub(crate) async fn handle_censor(
        &self,
        ctx: &Ctx<'_>,
        typ: &CensorType,
        censor: &Censor,
    ) -> DiscordResult<Option<AutomodResult>> {
        if !censor.enabled || censor.filters.is_empty() {
            return Ok(None);
        }

        let content = if censor.ignore_whitespace {
            ctx.message
                .content
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>()
                .to_lowercase()
        } else {
            ctx.message.content.to_lowercase()
        };

        let found_filter = match typ {
            CensorType::Word => {
                // Early exit if content is empty
                if content.is_empty() {
                    return Ok(None);
                }

                // Pre-compile patterns once
                let patterns: Vec<Pattern> = censor
                    .filters
                    .iter()
                    .map(|p| Pattern::from_str(p))
                    .collect();

                // Check patterns against content - optimized for common case
                let mut matched_word = None;

                // First try checking full content for patterns
                for (i, pattern) in patterns.iter().enumerate() {
                    if pattern.matches(&content) {
                        // Found a match, now find which word specifically matched
                        let words: Vec<&str> = content.split_whitespace().collect();
                        for word in &words {
                            if pattern.matches(word) {
                                matched_word = Some(word.to_string());
                                break;
                            }
                        }
                        // If no single word matched, but pattern matched full content,
                        // return the filter itself
                        if matched_word.is_none() {
                            matched_word = Some(censor.filters[i].clone());
                        }
                        break;
                    }
                }

                matched_word
            }

            CensorType::Link => {
                // Optimized URL parsing - avoid unnecessary allocations
                content.split_whitespace().find_map(|s| {
                    url::Url::parse(s).ok().and_then(|url| {
                        url.host_str().and_then(|domain| {
                            if censor.filters.iter().any(|filter| filter == domain) {
                                Some(domain.to_string())
                            } else {
                                None
                            }
                        })
                    })
                })
            }

            CensorType::Invite => {
                // Optimized Discord invite parsing
                content.split_whitespace().find_map(|s| {
                    url::Url::parse(s).ok().and_then(|url| {
                        if url.host_str() == Some("discord.gg") {
                            url.path_segments()
                                .and_then(|segments| segments.last())
                                .and_then(|invite| {
                                    if censor.filters.iter().any(|filter| filter == invite) {
                                        Some(invite.to_string())
                                    } else {
                                        None
                                    }
                                })
                        } else {
                            None
                        }
                    })
                })
            }
        };

        if (censor.whitelist && found_filter.is_none())
            || (!censor.whitelist && found_filter.is_some())
        {
            self.rest
                .delete_message_and_forget(ctx.channel_id, &ctx.message.id)
                .await;

            let expires_at =
                chrono::Utc::now() + chrono::Duration::milliseconds(censor.action.duration);

            return Ok(Some(AutomodResult::new(Infraction::new_automod(
                *ctx.guild_id,
                ctx.user.id,
                ctx.user.id,
                censor.action.action.clone(),
                Some(expires_at.timestamp() as u64),
                AutomodOffense {
                    typ: OffenseType::Censor(typ.clone()),
                    message: ctx.message.content.clone(),
                    count: None,
                    interval: None,
                    offending_filter: found_filter,
                },
                true,
            ))));
        }

        Ok(None)
    }
}
