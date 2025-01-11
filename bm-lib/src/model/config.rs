use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{discord::Id, permissions::PermissionSet};

use super::automod::AutomodConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub id: Id, // Guild ID
    #[serde(default = "default_prefix")]
    pub prefix: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute_role: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_warn_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_channel: Option<Id>,
    #[serde(default)]
    pub prefer_embeds: bool,
    #[serde(default = "default_true")]
    pub inherit_discord_perms: bool,
    #[serde(default = "default_true")]
    pub alert_on_infraction: bool,
    #[serde(default = "default_true")]
    pub send_permission_denied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_groups: Option<Vec<Group>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automod: Option<AutomodConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_aliases: Option<HashMap<String, String>>,
}

fn default_prefix() -> String {
    "!".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub roles: HashSet<Id>,
    pub users: HashSet<Id>,
    pub permissions: PermissionSet,
}
