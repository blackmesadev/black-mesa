use bson::Bson;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::vec;

use crate::antinuke::*;
use crate::antiraid::*;
use crate::appeals;
use crate::automod;
use crate::logging::*;
use crate::moderation::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub prefix: Option<String>,
    pub users: Option<HashMap<String, User>>,
    pub roles: Option<HashMap<String, Role>>,
    pub groups: Option<HashMap<String, Group>>,
    pub modules: Option<Modules>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Modules {
    pub antinuke: Option<antinuke::Antinuke>,
    pub antiraid: Option<antiraid::Antiraid>,
    pub appeals: Option<appeals::Appeals>,
    pub automod: Option<automod::Automod>,
    pub logging: Option<logging::Logging>,
    pub moderation: Option<moderation::Moderation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Group {
    pub permissions: Vec<String>,
    pub inherit: Vec<String>,
    pub priority: u64,
}

impl Default for Group {
    fn default() -> Self {
        Group {
            permissions: vec![String::from("guild")],
            inherit: Vec::new(),
            priority: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub groups: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Role {
    pub groups: Vec<String>,
    pub permissions: Vec<String>,
}

impl From<Config> for Bson {
    fn from(config: Config) -> Self {
        bson::to_bson(&config).unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            prefix: Some("!".to_string()),
            users: Some(HashMap::new()),
            roles: Some(HashMap::new()),
            groups: Some(HashMap::new()),
            modules: Some(Modules {
                antinuke: Some(antinuke::Antinuke::default()),
                antiraid: Some(antiraid::Antiraid::default()),
                appeals: Some(appeals::Appeals::default()),
                automod: Some(automod::Automod::default()),
                logging: Some(logging::Logging::default()),
                moderation: Some(moderation::Moderation::default()),
            }),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Guild {
    pub config: Config,
    pub guild_id: String,
}

impl Guild {
    pub fn new(guild_id: String) -> Self {
        Guild {
            config: Config::default(),
            guild_id,
        }
    }
}
