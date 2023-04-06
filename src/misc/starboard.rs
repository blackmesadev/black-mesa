use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type Starboard = HashMap<u64, StarboardSettings>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarboardSettings {
    pub channel_id: u64,
    pub ignore_channels: Vec<u64>,
    pub emojis: Vec<String>,
    pub minimum: u64,
}
