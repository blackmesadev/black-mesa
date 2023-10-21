use crate::redis::redis::*;
use crate::{automod::automod::Spam, util};

use std::convert::TryInto;

use super::MessageTrait;

impl Redis {
    pub async fn filter_messages<T: MessageTrait>(&self, spam_user: &Spam, msg: &T) -> bool {
        let max = match spam_user.max_messages {
            Some(max) => {
                if max == 0 {
                    return true;
                }
                max
            }
            None => return true,
        };
        let exp: usize = match spam_user.interval {
            Some(interval) => match interval.try_into() {
                Ok(interval) => interval,
                Err(_) => return true,
            },
            None => return true,
        };

        let guild_id = match msg.guild_id() {
            Some(id) => id.get(),
            None => return true,
        };

        match self
            .incr_max_messages(guild_id, msg.author().id.get(), exp, max)
            .await
        {
            Some(count) => {
                if count > max {
                    return false;
                } else {
                    return true;
                }
            }
            None => {
                if exp == 0 {
                    return true;
                }
                match self
                    .set_max_messages(guild_id, msg.author().id.get(), 1, exp)
                    .await
                {
                    Some(_) => {
                        return true;
                    }
                    None => {
                        tracing::error!("Error setting max messages");
                        return true;
                    }
                }
            }
        }
    }
}

pub fn filter_mentions<T: MessageTrait>(spam_user: &Spam, msg: &T) -> bool {
    let max = spam_user
        .max_mentions
        .unwrap_or(Spam::default().max_mentions.unwrap_or(0));
    if max == 0 {
        return true;
    }
    !((msg.mention_roles().len() as i64 + msg.mentions().len() as i64) > max)
}

pub fn filter_links<T: MessageTrait>(spam_user: &Spam, msg: &T) -> bool {
    let max = spam_user
        .max_links
        .unwrap_or(Spam::default().max_links.unwrap_or(0));
    if max == 0 {
        return true;
    }
    !(util::regex::DOMAINS.find_iter(msg.content()).count() as i64 > max)
}
pub fn filter_attachments<T: MessageTrait>(spam_user: &Spam, msg: &T) -> bool {
    let max = spam_user
        .max_attachments
        .unwrap_or(Spam::default().max_attachments.unwrap_or(0));
    if max == 0 {
        return true;
    }
    !(msg.attachments().len() as i64 > max)
}
pub fn filter_emojis<T: MessageTrait>(spam_user: &Spam, msg: &T) -> bool {
    let max = spam_user
        .max_emojis
        .unwrap_or(Spam::default().max_emojis.unwrap_or(0));
    if max == 0 {
        return true;
    }

    let msg_content = msg.content();

    let emoji_count = util::regex::EMOJI.find_iter(msg_content).count() as i64;

    if emoji_count > max {
        return false;
    }

    !(emoji_count + util::unicode::unicode_emojis_num(msg_content) > max)
}
pub fn filter_newlines<T: MessageTrait>(spam_user: &Spam, msg: &T) -> bool {
    let newlines = msg
        .content()
        .as_bytes()
        .iter()
        .filter(|&&c| c == b'\n')
        .count() as i64;

    let max = spam_user
        .max_newlines
        .unwrap_or(Spam::default().max_newlines.unwrap_or(0));
    if max == 0 {
        return true;
    }

    !(newlines > max)
}

pub fn filter_max_length<T: MessageTrait>(spam_user: &Spam, msg: &T) -> bool {
    let max = spam_user
        .max_characters
        .unwrap_or(Spam::default().max_characters.unwrap_or(0));
    if max == 0 {
        return true;
    }
    !(msg.content().len() as i64 > max)
}

pub fn filter_uppercase<T: MessageTrait>(spam_user: &Spam, msg: &T) -> bool {
    let max = spam_user
        .max_uppercase_percent
        .unwrap_or(Spam::default().max_uppercase_percent.unwrap_or(0.0));
    if max == 0.0 {
        return true;
    }

    let msg_content = msg.content();

    let mut uppercase_count = 0;
    for c in msg_content.chars() {
        if c.is_uppercase() {
            uppercase_count += 1;
        }
    }

    let uppercase_percent = uppercase_count as f64 / msg_content.len() as f64;

    !(uppercase_percent > max)
}
