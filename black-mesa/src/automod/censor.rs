use crate::handler::EventHandler;
use bm_lib::{
    discord::{commands::Ctx, DiscordResult},
    model::{automod::{AutomodOffense, Censor, CensorType, OffenseType}, Infraction},
};
use tracing::instrument;
use std::collections::HashMap;

use super::AutomodResult;

lazy_static::lazy_static! {
    static ref LETTER_SUBSTITUTIONS: HashMap<char, Vec<&'static str>> = {
        let mut m = HashMap::new();
        m.insert('a', vec!["a", "@", "4", "^"]);
        m.insert('e', vec!["e", "3"]);
        m.insert('g', vec!["g", "9", "6"]);
        m.insert('i', vec!["i", "1", "!", "|"]);
        m.insert('l', vec!["l", "1", "|"]);
        m.insert('o', vec!["o", "0"]);
        m.insert('s', vec!["s", "$", "5"]);
        m.insert('t', vec!["t", "7"]);
        m
    };
}

#[derive(Clone)]
enum Pattern {
    Simple(String),
    Wildcard {
        start: String,
        end: String,
        require_middle: bool,
    },
    Substitution(String),
}

impl Pattern {
    fn generate_variations(text: &str) -> Vec<String> {
        let mut result = vec![text.to_string()];
        
        for (pos, ch) in text.chars().enumerate() {
            if let Some(substitutions) = LETTER_SUBSTITUTIONS.get(&ch.to_ascii_lowercase()) {
                let current_count = result.len();
                for i in 0..current_count {
                    for sub in substitutions.iter().skip(1) {
                        let mut new_var = result[i].clone();
                        new_var.replace_range(pos..pos+1, sub);
                        result.push(new_var);
                    }
                }
            }
        }
        
        result
    }

    fn matches(&self, text: &str) -> bool {
        match self {
            Pattern::Simple(pattern) => text.contains(pattern),
            Pattern::Substitution(pattern) => {
                let variations = Self::generate_variations(pattern);
                variations.iter().any(|var| text.contains(var))
            },
            Pattern::Wildcard { start, end, require_middle } => {
                let start_vars = Self::generate_variations(start);
                let end_vars = Self::generate_variations(end);

                if start.is_empty() && end.is_empty() {
                    return true;
                }

                for start_var in &start_vars {
                    for end_var in &end_vars {
                        if !text.starts_with(start_var) || !text.ends_with(end_var) {
                            continue;
                        }

                        if *require_middle {
                            if text.len() > start_var.len() + end_var.len() {
                                return true;
                            }
                        } else {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }

    fn from_str(pattern: &str) -> Self {
        if pattern.contains('%') {
            return Pattern::Substitution(pattern.replace('%', ""));
        }

        if !pattern.contains("...") && !pattern.contains('*') {
            return Pattern::Simple(pattern.to_string());
        }

        if pattern.contains("...") {
            let parts: Vec<&str> = pattern.split("...").collect();
            if parts.len() == 2 {
                Pattern::Wildcard {
                    start: parts[0].to_string(),
                    end: parts[1].to_string(),
                    require_middle: true,
                }
            } else {
                Pattern::Simple(pattern.to_string())
            }
        } else {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                Pattern::Wildcard {
                    start: parts[0].to_string(),
                    end: parts[1].to_string(),
                    require_middle: false,
                }
            } else {
                Pattern::Simple(pattern.to_string())
            }
        }
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
        if !censor.enabled {
            return Ok(None);
        }

        let content = if censor.ignore_whitespace {
            ctx.message.content.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_lowercase()
        } else {
            ctx.message.content.to_lowercase()
        };

        let found_filter = match typ {
            CensorType::Word => {
                let patterns: Vec<Pattern> = censor.filters.iter()
                    .map(|p| Pattern::from_str(p))
                    .collect();

                let has_match = patterns.iter().any(|p| p.matches(&content));
                if has_match {
                    content.split_whitespace()
                        .find(|word| patterns.iter().any(|p| p.matches(word)))
                        .map(String::from)
                } else {
                    None
                }
            },
            
            CensorType::Link => {
                let domains: Vec<String> = content.split_whitespace()
                    .filter_map(|s| url::Url::parse(s).ok()?.host_str().map(String::from))
                    .collect();
                
                if domains.iter().any(|domain| censor.filters.contains(domain)) {
                    domains.into_iter()
                        .find(|domain| censor.filters.contains(domain))
                } else {
                    None
                }
            },
            
            CensorType::Invite => {
                let invites: Vec<String> = content.split_whitespace()
                    .filter_map(|s| {
                        let url = url::Url::parse(s).ok()?;
                        if url.host_str()? == "discord.gg" {
                            url.path_segments()?.last().map(String::from)
                        } else {
                            None
                        }
                    })
                    .collect();

                if invites.iter().any(|invite| censor.filters.contains(invite)) {
                    invites.into_iter()
                        .find(|invite| censor.filters.contains(invite))
                } else {
                    None
                }
            },
        };

        if (censor.whitelist && found_filter.is_none()) || (!censor.whitelist && found_filter.is_some()) {
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
