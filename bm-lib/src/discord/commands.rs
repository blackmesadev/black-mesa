use std::collections::HashSet;

use tracing::instrument;

use super::{Id, Message, User, DISCORD_EPOCH};
use crate::util;

#[derive(Debug)]
pub enum Arg {
    Id(Id),
    User(Id),
    Channel(Id),
    Role(Id),
    Text(String),
    Duration(u64),
    Number(u64),
}

impl Arg {
    pub fn to_string(&self) -> String {
        match self {
            Arg::Id(id) => id.to_string(),
            Arg::User(id) => format!("<@{}>", id),
            Arg::Channel(id) => format!("<#{}>", id),
            Arg::Role(id) => format!("<@&{}>", id),
            Arg::Text(text) => text.clone(),
            Arg::Duration(duration) => util::format_duration(*duration),
            Arg::Number(num) => num.to_string(),
        }
    }

    pub fn get_type(&self) -> &'static str {
        match self {
            Arg::Id(_) => "id",
            Arg::User(_) => "user",
            Arg::Channel(_) => "channel",
            Arg::Role(_) => "role",
            Arg::Text(_) => "text",
            Arg::Duration(_) => "duration",
            Arg::Number(_) => "number",
        }
    }

    pub fn as_id(&self) -> Option<Id> {
        match self {
            Arg::Id(id) | Arg::User(id) | Arg::Channel(id) | Arg::Role(id) => Some(*id),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Arg::Text(text) => Some(text.as_str()),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<u64> {
        match self {
            Arg::Number(num) => Some(*num),
            _ => None,
        }
    }

    pub fn as_duration(&self) -> Option<u64> {
        match self {
            Arg::Duration(duration) => Some(*duration),
            _ => None,
        }
    }

    pub fn as_user(&self) -> Option<Id> {
        match self {
            Arg::User(id) => Some(*id),
            _ => None,
        }
    }

    pub fn as_channel(&self) -> Option<Id> {
        match self {
            Arg::Channel(id) => Some(*id),
            _ => None,
        }
    }

    pub fn as_role(&self) -> Option<Id> {
        match self {
            Arg::Role(id) => Some(*id),
            _ => None,
        }
    }
}
#[derive(Debug)]
pub struct Args<'a>(&'a [Arg], &'a [&'a str]);

impl Args<'_> {
    pub fn new<'a>(args: &'a [Arg], msg: &'a [&'a str]) -> Args<'a> {
        Args(args, &msg)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arg> {
        self.0.iter()
    }

    pub fn iter_raw_args(&self) -> impl Iterator<Item = &str> {
        self.1.iter().copied()
    }

    pub fn get(&self, index: usize) -> Option<&Arg> {
        self.0.get(index)
    }

    pub fn get_raw(&self, index: usize) -> Option<&str> {
        self.1.get(index).copied()
    }

    pub fn raw_args(&self) -> &[&str] {
        self.1
    }

    pub fn to_string(&self) -> String {
        self.0
            .iter()
            .map(Arg::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn get_subcommand(&self) -> Option<&str> {
        self.0.get(0).and_then(|arg| arg.as_text())
    }

    pub fn pop_subcommand(&mut self) -> Option<&str> {
        if self.1.is_empty() {
            None
        } else {
            let sub = self.1[0];
            self.1 = &self.1[1..];
            Some(sub)
        }
    }

    pub fn get_text(&self) -> Vec<String> {
        self.0
            .iter()
            .filter_map(|arg| match arg {
                Arg::Text(text) => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn get_first_number(&self) -> Option<u64> {
        self.0.iter().find_map(|arg| match arg {
            Arg::Number(num) => Some(*num),
            _ => None,
        })
    }

    pub fn get_first_duration(&self) -> Option<u64> {
        self.0.iter().find_map(|arg| match arg {
            Arg::Duration(duration) => Some(*duration),
            _ => None,
        })
    }

    pub fn get_first_id(&self) -> Option<Id> {
        self.0.iter().find_map(|arg| match arg {
            Arg::Id(id) => Some(*id),
            _ => None,
        })
    }

    pub fn get_first_user(&self) -> Option<Id> {
        self.0.iter().find_map(|arg| match arg {
            Arg::User(id) => Some(*id),
            _ => None,
        })
    }

    pub fn get_first_channel(&self) -> Option<Id> {
        self.0.iter().find_map(|arg| match arg {
            Arg::Channel(id) => Some(*id),
            _ => None,
        })
    }

    pub fn get_first_role(&self) -> Option<Id> {
        self.0.iter().find_map(|arg| match arg {
            Arg::Role(id) => Some(*id),
            _ => None,
        })
    }

    pub fn get_first_text(&self) -> Option<&str> {
        self.0.iter().find_map(|arg| match arg {
            Arg::Text(text) => Some(text.as_str()),
            _ => None,
        })
    }

    pub fn get_targets(&self) -> Vec<Id> {
        self.0
            .iter()
            .filter_map(|arg| match arg {
                Arg::Id(id) | Arg::User(id) => Some(*id),
                _ => None,
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty() && self.1.is_empty()
    }
}

pub struct Ctx<'a> {
    pub channel_id: &'a Id,
    pub guild_id: &'a Id,
    pub user: &'a User,
    pub roles: &'a HashSet<Id>,
    pub message: &'a Message,
}

impl<'a> Ctx<'a> {
    pub fn new(message: &'a Message, roles: &'a HashSet<Id>) -> Option<Self> {
        Some(Self {
            channel_id: &message.channel_id,
            guild_id: message.guild_id.as_ref()?,
            user: message.author.as_ref()?,
            roles,
            message,
        })
    }
}

#[instrument(skip(args))]
pub async fn parse_args(mut args: impl Iterator<Item = &'_ str>) -> Vec<Arg> {
    tracing::debug!("Parsing command arguments");
    let mut parsed_args = Vec::new();
    let mut text_buffer = Vec::new();
    let mut text_mode = false;

    let max_snowflake = util::max_snowflake_now();

    while let Some(arg) = args.next() {
        if text_mode {
            text_buffer.push(arg);
            continue;
        }

        if let Some(id) = parse_mention(arg) {
            parsed_args.push(id);
            continue;
        }

        if let Ok(num) = arg.parse::<u64>() {
            if num > DISCORD_EPOCH && num < max_snowflake {
                parsed_args.push(Arg::Id(num.into()));
            } else {
                parsed_args.push(Arg::Number(num));
            }
            continue;
        }

        if let Some(duration) = parse_duration(arg) {
            parsed_args.push(Arg::Duration(duration));
            continue;
        }

        text_mode = true;
        text_buffer.push(arg);
    }

    if !text_buffer.is_empty() {
        parsed_args.push(Arg::Text(text_buffer.join(" ")));
    }

    tracing::debug!(arg_count = parsed_args.len(), "Finished parsing arguments");
    parsed_args
}

#[instrument]
pub fn parse_mention(arg: &str) -> Option<Arg> {
    tracing::debug!(arg = arg, "Parsing mention");
    if (!arg.starts_with('<')) || (!arg.ends_with('>')) {
        return None;
    }

    if let Some(channel_id) = arg.strip_prefix("<#").and_then(|s| s.strip_suffix(">")) {
        return channel_id.parse::<Id>().ok().map(Arg::Channel);
    }

    if let Some(user_id) = arg.strip_prefix("<@").and_then(|s| s.strip_suffix(">")) {
        return user_id.parse::<Id>().ok().map(Arg::User);
    }

    if let Some(role_id) = arg.strip_prefix("<@&").and_then(|s| s.strip_suffix(">")) {
        return role_id.parse::<Id>().ok().map(Arg::Role);
    }

    None
}

#[instrument]
pub fn parse_duration(arg: &str) -> Option<u64> {
    tracing::debug!(arg = arg, "Parsing duration");
    if !arg.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    let mut total_seconds = 0u64;
    let mut current_num = 0u64;
    let mut has_valid_unit = false;

    for c in arg.chars() {
        match c {
            '0'..='9' => {
                current_num = current_num
                    .saturating_mul(10)
                    .saturating_add((c as u64) - ('0' as u64));
            }
            unit => {
                let multiplier = match unit {
                    'y' | 'Y' => Some(31_536_000),
                    'w' | 'W' => Some(604_800),
                    'd' | 'D' => Some(86_400),
                    'h' | 'H' => Some(3_600),
                    'm' | 'M' => Some(60),
                    's' | 'S' => Some(1),
                    _ => None,
                };

                if let Some(mult) = multiplier {
                    has_valid_unit = true;
                    total_seconds = total_seconds.saturating_add(current_num.saturating_mul(mult));
                    current_num = 0;
                }
            }
        }
    }

    total_seconds = total_seconds.saturating_add(current_num);

    if (total_seconds > 0) && (has_valid_unit || current_num > 0) {
        Some(total_seconds)
    } else {
        None
    }
}
