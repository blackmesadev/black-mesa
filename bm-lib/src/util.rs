use std::time::SystemTime;

use crate::discord::{Id, DISCORD_EPOCH};

pub fn max_snowflake_now() -> u64 {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    (now - DISCORD_EPOCH) << 22
}

// expects duration in seconds
pub fn format_duration(duration: u64) -> String {
    let mut duration = duration;
    let mut result = String::new();

    let days = duration / 86400;
    if days > 0 {
        result.push_str(&format!("{}d", days));
        duration %= 86400;
    }

    let hours = duration / 3600;
    if hours > 0 {
        result.push_str(&format!("{}h", hours));
        duration %= 3600;
    }

    let minutes = duration / 60;
    if minutes > 0 {
        result.push_str(&format!("{}m", minutes));
        duration %= 60;
    }

    if duration > 0 {
        result.push_str(&format!("{}s", duration));
    }

    result
}

pub fn duration_to_unix_timestamp(duration: u64) -> u64 {
    let now = chrono::Utc::now().timestamp() as u64;

    now + duration
}

pub fn snowflake_to_timestamp(snowflake: Id) -> u64 {
    (snowflake.get() >> 22) + DISCORD_EPOCH
}
