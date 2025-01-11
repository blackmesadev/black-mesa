use std::collections::HashSet;

use crate::{
    discord::Id,
    model::{Config, Group},
};

impl Config {
    pub fn new(id: &Id) -> Self {
        Self {
            id: id.clone(),
            prefix: "!".to_string(),
            mute_role: None,
            default_warn_duration: None,
            log_channel: None,
            prefer_embeds: false,
            inherit_discord_perms: true,
            alert_on_infraction: true,
            send_permission_denied: true,
            permission_groups: None,
            automod: None,
            command_aliases: None,
        }
    }
}

impl Group {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            roles: HashSet::new(),
            users: HashSet::new(),
            permissions: Default::default(),
        }
    }
}
