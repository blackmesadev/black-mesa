use std::convert::TryInto;

use lazy_static::lazy_static;
use regex::Regex;
use tracing::error;

use crate::{automod::automod::{Spam}, util};
use crate::redis::redis::*;

use super::AutomodMessage;

impl Redis {
    pub async fn filter_messages(&self, spam_user : &Spam, msg: &AutomodMessage) -> bool {
        let max = match spam_user.max_messages {
            Some(max) => {
                if max == 0 {
                    return true;
                }
                max
            },
            None => return true,
        };
        let exp: usize = match spam_user.interval {
            Some(interval) => match interval.try_into() {
                Ok(interval) => interval,
                Err(_) => return true,
            },
            None => return true,
        };

        let guild_id = match msg.guild_id {
            Some(id) => id.get(),
            None => return true
        };

        match self.incr_max_messages(guild_id, msg.author.id.get(), exp, max).await {
            Some(count) => {
                if count > max {
                    return false;
                } else {
                    return true;
                }
            },
            None => {
                if exp == 0 {
                    return true;
                }
                match self.set_max_messages(guild_id,msg.author.id.get(), 1, exp).await{
                    Some(_) => {
                        return true;
                    },
                    None => {
                        error!("Error setting max messages");
                        return true;
                    }
                }
            }
        }
    }
}


pub fn filter_mentions(spam_user : &Spam, msg: &AutomodMessage) -> bool {
    let max = spam_user.max_mentions.unwrap_or(Spam::default().max_mentions.unwrap_or(0));
    if max == 0 {
        return true;
    }
    !(match &msg.mention_roles {
        Some(roles) => roles.len(),
        None => 0
    } as i64 + match &msg.mentions {
        Some(mentions) => mentions.len(),
        None => 0
    } as i64 > max)
}


pub fn filter_links(spam_user : &Spam, msg: &AutomodMessage) -> bool {
    lazy_static! {
        static ref DOMAINS_RE: Regex = Regex::new(r"[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}").unwrap();
    }
    let max = spam_user.max_links.unwrap_or(Spam::default().max_links.unwrap_or(0));
    if max == 0 {
        return true;
    }
    !(DOMAINS_RE.find_iter(match msg.content {
        Some(ref content) => content,
        None => return true
    }).count() as i64 > max)
}
pub fn filter_attachments(spam_user : &Spam, msg: &AutomodMessage) -> bool {
    let max = spam_user.max_attachments.unwrap_or(Spam::default().max_attachments.unwrap_or(0));
    if max == 0 {
        return true;
    }
    !(match &msg.attachments {
        Some(attachments) => attachments.len(),
        None => 0
    } as i64 > max)
}
pub fn filter_emojis(spam_user : &Spam, msg: &AutomodMessage) -> bool {
    let max = spam_user.max_emojis.unwrap_or(Spam::default().max_emojis.unwrap_or(0));
    if max == 0 {
        return true;
    }
    lazy_static! {
        static ref EMOJI_RE: Regex = Regex::new(r"<(a|):[A-z0-9_~]+:[0-9]{18}>").unwrap();
    }

    let msg_content = match msg.content {
        Some(ref content) => content,
        None => return true
    };
    
    let emoji_count = EMOJI_RE.find_iter(msg_content).count() as i64;
    
    if emoji_count > max {
        return false;
    }

    !(emoji_count + util::unicode::unicode_emojis_num(msg_content) > max)
    
}
pub fn filter_newlines(spam_user : &Spam, msg: &AutomodMessage) -> bool {
    let newlines = match msg.content{
        Some(ref content) => content.as_bytes(),
        None => return true
    }.iter().filter(|&&c| c == b'\n').count() as i64;

    let max = spam_user.max_newlines.unwrap_or(Spam::default().max_newlines.unwrap_or(0));
    if max == 0 {
        return true;
    }
    
    !(newlines > max)
}

pub fn filter_max_length(spam_user : &Spam, msg: &AutomodMessage) -> bool {
    let max = spam_user.max_characters.unwrap_or(Spam::default().max_characters.unwrap_or(0));
    if max == 0 {
        return true;
    }
    !(match msg.content {
        Some(ref content) => content.len(),
        None => 0
    } as i64 > max)
}

pub fn filter_uppercase(spam_user : &Spam, msg: &AutomodMessage) -> bool {
    let max = spam_user.max_uppercase_percent.unwrap_or(Spam::default().max_uppercase_percent.unwrap_or(0.0));
    let min = spam_user.min_uppercase_limit.unwrap_or(Spam::default().min_uppercase_limit.unwrap_or(0));
    if max == 0.0 {
        return true;
    }

    let msg_content = match msg.content {
        Some(ref content) => content,
        None => return true
    };
    
    if min > msg_content.len() as i64 {
        return true;
    }
    let mut uppercase_count = 0;
    for c in msg_content.chars() {
        if c.is_uppercase() {
            uppercase_count += 1;
        }
    }
    
    let uppercase_percent = uppercase_count as f64 / msg_content.len() as f64;
    
    !(uppercase_percent > max)
}