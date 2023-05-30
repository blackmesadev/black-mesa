#![allow(dead_code)]

use std::error::Error;

use twilight_model::id::{marker::RoleMarker, Id};

use crate::mongo::mongo::Config;

pub const CATEGORY_MODERATION: &str = "moderation";
pub const CATEGORY_ADMIN: &str = "admin";
pub const CATEGORY_GUILD: &str = "guild";
pub const CATEGORY_ROLES: &str = "roles";
pub const CATEGORY_MUSIC: &str = "music";
pub const CATEGORY_VOTING: &str = "voting";

pub const PERMISSION_BAN: &str = "moderation.ban";
pub const PERMISSION_KICK: &str = "moderation.kick";
pub const PERMISSION_MUTE: &str = "moderation.mute";
pub const PERMISSION_REMOVEACTIONSELF: &str = "moderation.removeself";
pub const PERMISSION_SEARCH: &str = "moderation.search";
pub const PERMISSION_SOFTBAN: &str = "moderation.softban";
pub const PERMISSION_STRIKE: &str = "moderation.strike";
pub const PERMISSION_UNBAN: &str = "moderation.unban";
pub const PERMISSION_UNMUTE: &str = "moderation.unmute";
pub const PERMISSION_UPDATESELF: &str = "moderation.updateself";

pub const PERMISSION_MAKEMUTE: &str = "admin.makemute";
pub const PERMISSION_SETUP: &str = "admin.setup";
pub const PERMISSION_REMOVEACTION: &str = "admin.remove";
pub const PERMISSION_UPDATE: &str = "admin.update";
pub const PERMISSION_PURGE: &str = "admin.purge";
pub const PERMISSION_DEEPSEARCH: &str = "admin.deepsearch";

pub const PERMISSION_VIEWCMDLEVEL: &str = "guild.viewcommandlevel";
pub const PERMISSION_VIEWUSERLEVEL: &str = "guild.viewuserlevel";
pub const PERMISSION_USERINFO: &str = "guild.userinfo";
pub const PERMISSION_USERINFOSELF: &str = "guild.userinfoself";
pub const PERMISSION_GUILDINFO: &str = "guild.guildinfo";
pub const PERMISSION_SEARCHSELF: &str = "guild.searchself";

pub const PERMISSION_ROLEADD: &str = "roles.add";
pub const PERMISSION_ROLEREMOVE: &str = "roles.remove";
pub const PERMISSION_ROLECREATE: &str = "roles.create";
pub const PERMISSION_ROLERMROLE: &str = "roles.rmrole";
pub const PERMISSION_ROLEUPDATE: &str = "roles.update";
pub const PERMISSION_ROLELIST: &str = "roles.list";

pub const PERMISSION_PLAY: &str = "music.play";
pub const PERMISSION_STOP: &str = "music.stop";
pub const PERMISSION_SKIP: &str = "music.skip";
pub const PERMISSION_REMOVE: &str = "music.remove";
pub const PERMISSION_DC: &str = "music.dc";
pub const PERMISSION_SEEK: &str = "music.seek";
pub const PERMISSION_VOLUME: &str = "music.volume";

pub const PERMISSION_VOTEMUTE: &str = "voting.mute";

pub fn get_closest_level(i: Vec<i64>, target: i64) -> i64 {
    let mut closest = i[0];
    for x in i {
        if (target - x).abs() < (target - closest).abs() {
            closest = x;
        }
    }
    closest
}

pub fn get_permission(
    conf: &Config,
    check_permission: &str,
) -> Result<i64, Box<dyn Error + Send + Sync>> {
    let temp_tree: Vec<&str> = check_permission.split(".").collect();
    let mut perm_tree: Vec<String> = Vec::new();

    for (pk, _) in temp_tree.iter().enumerate() {
        let mut node = temp_tree[0].to_string();
        if temp_tree.len() > 1 {
            let mut j = 1;
            while j <= pk {
                node = format!("{}.{}", node, temp_tree[pk]);
                j += 1;
            }
        }
        perm_tree.push(node);
    }

    let mut perm_value: i64 = -1;

    for perm in perm_tree {
        let perm_val = conf.permissions.get(&perm);
        match perm_val {
            Some(val) => {
                perm_value = *val;
            }
            None => {}
        }
    }

    if perm_value == -1 {
        return Err("Permission not found".into());
    }

    Ok(perm_value)
}

pub fn check_permission(
    conf: &Config,
    roles: Option<&Vec<Id<RoleMarker>>>,
    user_id: &String,
    check_permissions: Vec<&str>,
) -> bool {
    for check_permission in check_permissions {
        let perm_value = &get_permission(conf, check_permission).unwrap();
        let user_level = &get_user_level(conf, roles, user_id);

        if perm_value > user_level {
            return false;
        }
    }
    true
}

pub fn get_user_level(conf: &Config, roles: Option<&Vec<Id<RoleMarker>>>, user_id: &String) -> i64 {
    let mut user_level = conf.levels.get(user_id).unwrap_or(&0);
    let mut role_level: &i64 = &0;
    match roles {
        Some(roles) => {
            for role in roles {
                let new = conf.levels.get(&role.to_string()).unwrap_or(&0);
                if new > role_level {
                    role_level = new;
                }
            }
            if role_level > user_level {
                user_level = role_level;
            }
        }
        None => {}
    }

    *user_level
}
