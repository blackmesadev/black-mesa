// use group inheritance, assign permission nodes to groups, give a user a group
// groups will be stored similarly to levels, however they will be stored as a group object as the value where the key is the group name
// the group object will contain a list of permission nodes, and a list of groups that it inherits from
// when checking permissions, we will start with the user's group, and check if the user has the permission node, if not,
// iterate over the groups that the user's group inherits from, and check if they have the permission node as you go.

use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use twilight_model::id::{
    marker::{RoleMarker, UserMarker},
    Id,
};

use crate::config::{Config, Group};

lazy_static::lazy_static! {
    static ref DEFAULT_GROUPS: HashMap<String, Group> = {
        let mut groups = HashMap::new();
        groups.insert("default".to_string(), Group::default());
        groups
    };
}

fn get_permission_tree(check_permission: &str) -> Vec<String> {
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

    perm_tree
}

fn get_group_permissions<'a>(
    conf: &'a Config,
    group_names: &HashSet<&'a String>,
    user_id: Id<UserMarker>,
    visited: &mut HashSet<&'a String>,
) -> Result<HashSet<&'a String>, Box<dyn Error>> {
    let mut permissions_set: HashSet<&'a String> = HashSet::new();

    let conf_groups = conf.groups.as_ref().unwrap_or(&DEFAULT_GROUPS);

    for group_name in group_names.iter() {
        if visited.contains(group_name) {
            return Err("Inheritance loop detected".into());
        }

        if let Some(group) = conf_groups.get(*group_name) {
            permissions_set.extend(&group.permissions);

            visited.insert(group_name);

            let inherit_permissions =
                get_group_permissions(conf, &group.inherit.iter().collect(), user_id, visited)?;
            permissions_set.extend(inherit_permissions);
        }
    }

    if let Some(user) = conf_groups.get(&user_id.to_string()) {
        permissions_set.extend(&user.permissions);
    }

    Ok(permissions_set)
}

pub fn get_user_groups<'a>(
    conf: &'a Config,
    user_id: Id<UserMarker>,
    roles: Option<&Vec<Id<RoleMarker>>>,
) -> Result<HashMap<String, &'a Group>, Box<dyn Error>> {
    let conf_groups = conf.groups.as_ref().unwrap_or_else(|| &DEFAULT_GROUPS);

    let mut user_groups = HashMap::new();

    if let Some(conf_roles) = conf.roles.as_ref() {
        if let Some(roles) = roles {
            for role_id in roles.iter() {
                if let Some(role) = conf_roles.get(&role_id.to_string()) {
                    for group_name in &role.groups {
                        if let Some(group) = conf_groups.get(group_name) {
                            user_groups.insert(group_name.clone(), group);
                        }
                    }
                }
            }
        }
    }

    if let Some(conf_users) = conf.users.as_ref() {
        if let Some(user) = conf_users.get(&user_id.to_string()) {
            for group_name in &user.groups {
                if let Some(group) = conf_groups.get(group_name) {
                    user_groups.insert(group_name.clone(), group);
                }
            }
        }
    }

    Ok(user_groups)
}

pub fn get_user_groups_names(
    conf: &Config,
    user_id: Id<UserMarker>,
    roles: Option<&Vec<Id<RoleMarker>>>,
) -> Result<HashSet<String>, Box<dyn Error>> {
    let mut user_groups = HashSet::new();

    if let Some(conf_roles) = conf.roles.as_ref() {
        if let Some(roles) = roles {
            for role_id in roles.iter() {
                if let Some(role) = conf_roles.get(&role_id.to_string()) {
                    for group_name in &role.groups {
                        user_groups.insert(group_name.clone());
                    }
                }
            }
        }
    }

    if let Some(conf_users) = conf.users.as_ref() {
        if let Some(user) = conf_users.get(&user_id.to_string()) {
            for group_name in &user.groups {
                user_groups.insert(group_name.clone());
            }
        }
    }

    Ok(user_groups)
}

pub fn check_permission(
    conf: &Config,
    roles: Option<&Vec<Id<RoleMarker>>>,
    user_id: Id<UserMarker>,
    check_permission: &str,
) -> bool {
    if let Ok(groups_map) = get_user_groups(conf, user_id, roles) {
        let group_names: HashSet<&String> = groups_map.keys().collect();
        let mut visited: HashSet<&String> = HashSet::new();
        if let Ok(group_permissions) =
            get_group_permissions(conf, &group_names, user_id, &mut visited)
        {
            if let Some(user) = conf
                .users
                .as_ref()
                .and_then(|users| users.get(&user_id.to_string()))
            {
                let perm_tree = get_permission_tree(check_permission);
                if perm_tree
                    .iter()
                    .any(|perm| group_permissions.contains(perm) || user.permissions.contains(perm))
                {
                    return true;
                }
            }
        }
    }
    false
}

pub fn check_permission_many(
    conf: &Config,
    roles: Option<&Vec<Id<RoleMarker>>>,
    user_id: Id<UserMarker>,
    check_permissions: Vec<&str>,
) -> bool {
    if let Ok(groups_map) = get_user_groups(conf, user_id, roles) {
        let group_names: HashSet<&String> = groups_map.keys().collect();
        let mut visited: HashSet<&String> = HashSet::new();
        if let Ok(group_permissions) =
            get_group_permissions(conf, &group_names, user_id, &mut visited)
        {
            if let Some(user) = conf
                .users
                .as_ref()
                .and_then(|users| users.get(&user_id.to_string()))
            {
                for check_permission in check_permissions {
                    let perm_tree = get_permission_tree(check_permission);
                    if !perm_tree.iter().any(|perm| {
                        group_permissions.contains(perm) || user.permissions.contains(perm)
                    }) {
                        return false;
                    }
                }
                return true;
            }
        }
    }
    false
}

pub fn get_user_priority(groups: &HashMap<String, &Group>) -> u64 {
    groups
        .values()
        .map(|group| group.priority)
        .max()
        .unwrap_or(0)
}

pub fn check_inheritance_loop(conf: &Config, group_name: &str) -> Result<(), Box<dyn Error>> {
    fn check(
        conf: &Config,
        group_name: &str,
        seen_groups: &mut Vec<String>,
    ) -> Result<(), Box<dyn Error>> {
        let conf_groups = conf.groups.as_ref().ok_or("No groups found")?;

        let group = conf_groups.get(group_name).ok_or("Group not found")?;

        if seen_groups.contains(&group_name.to_string()) {
            return Err("Inheritance loop detected".into());
        }

        seen_groups.push(group_name.to_string());

        for inherit in &group.inherit {
            check(conf, inherit, seen_groups)?;
        }

        Ok(())
    }

    let mut seen_groups = Vec::new();
    check(conf, group_name, &mut seen_groups)?;

    Ok(())
}

pub const CATEGORY_MODERATION: &str = "moderation";
pub const CATEGORY_ADMIN: &str = "admin";
pub const CATEGORY_GUILD: &str = "guild";
pub const CATEGORY_ROLES: &str = "roles";
pub const CATEGORY_MUSIC: &str = "music";

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

pub const PERMISSION_SETUP: &str = "admin.setup";
pub const PERMISSION_REMOVEACTION: &str = "admin.remove";
pub const PERMISSION_UPDATE: &str = "admin.update";
pub const PERMISSION_PURGE: &str = "admin.purge";
pub const PERMISSION_DEEPSEARCH: &str = "admin.deepsearch";
pub const PERMISSION_ANTINUKE_BYPASS: &str = "admin.antinuke.bypass";

pub const PERMISSION_USERINFO: &str = "guild.userinfo";
pub const PERMISSION_USERINFOSELF: &str = "guild.userinfoself";
pub const PERMISSION_GUILDINFO: &str = "guild.guildinfo";
pub const PERMISSION_SEARCHSELF: &str = "guild.searchself";
