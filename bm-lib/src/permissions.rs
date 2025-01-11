use crate::discord::{Id, Permissions, Role};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::HashSet, fmt};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Permission {
    // Root level permissions (wildcards)
    All,
    Moderation,
    Music,
    Config,
    Utility,

    // Moderation permissions
    ModerationKick,
    ModerationBan,
    ModerationUnban,
    ModerationMute,
    ModerationUnmute,
    ModerationWarn,
    ModerationPardon,
    ModerationPurge,
    ModerationLookup,

    // Music permissions
    MusicPlay,
    MusicSkip,
    MusicStop,
    MusicPause,
    MusicResume,
    MusicClear,
    MusicVolume,
    MusicShuffle,

    // Config permissions
    ConfigView,
    ConfigEdit,

    // Utility permissions
    UtilityInfo,
    UtilityUserinfo,
    UtilityServerinfo,
    UtilityHelp,
    UtilityPing,
    UtilityInvite,
    UtilitySelfLookup,
}

impl Permission {
    pub fn from_str(s: &str) -> Option<Permission> {
        let p = match s {
            "all" => Permission::All,
            "moderation" => Permission::Moderation,
            "music" => Permission::Music,
            "config" => Permission::Config,
            "utility" => Permission::Utility,
            "moderation.kick" => Permission::ModerationKick,
            "moderation.ban" => Permission::ModerationBan,
            "moderation.unban" => Permission::ModerationUnban,
            "moderation.mute" => Permission::ModerationMute,
            "moderation.unmute" => Permission::ModerationUnmute,
            "moderation.warn" => Permission::ModerationWarn,
            "moderation.pardon" => Permission::ModerationPardon,
            "moderation.purge" => Permission::ModerationPurge,
            "moderation.lookup" => Permission::ModerationLookup,
            "music.play" => Permission::MusicPlay,
            "music.skip" => Permission::MusicSkip,
            "music.stop" => Permission::MusicStop,
            "music.pause" => Permission::MusicPause,
            "music.resume" => Permission::MusicResume,
            "music.clear" => Permission::MusicClear,
            "music.volume" => Permission::MusicVolume,
            "music.shuffle" => Permission::MusicShuffle,
            "config.view" => Permission::ConfigView,
            "config.edit" => Permission::ConfigEdit,
            "utility.info" => Permission::UtilityInfo,
            "utility.userinfo" => Permission::UtilityUserinfo,
            "utility.serverinfo" => Permission::UtilityServerinfo,
            "utility.help" => Permission::UtilityHelp,
            "utility.ping" => Permission::UtilityPing,
            "utility.invite" => Permission::UtilityInvite,
            "utility.selflookup" => Permission::UtilitySelfLookup,
            _ => return None,
        };

        Some(p)
    }

    /// Get all child permissions for a given parent permission
    pub fn children(&self) -> HashSet<Permission> {
        use Permission::*;
        match self {
            All => Permission::all_permissions(),
            Moderation => HashSet::from([
                ModerationKick,
                ModerationBan,
                ModerationUnban,
                ModerationMute,
                ModerationUnmute,
                ModerationWarn,
                ModerationPardon,
                ModerationPurge,
            ]),
            Music => HashSet::from([
                MusicPlay,
                MusicSkip,
                MusicStop,
                MusicPause,
                MusicResume,
                MusicClear,
                MusicVolume,
                MusicShuffle,
            ]),
            Config => HashSet::from([ConfigView, ConfigEdit]),
            Utility => HashSet::from([
                UtilityInfo,
                UtilityUserinfo,
                UtilityServerinfo,
                UtilityHelp,
                UtilityPing,
                UtilityInvite,
            ]),
            _ => HashSet::new(),
        }
    }

    /// Get the parent permission for a given permission
    #[inline]
    pub fn parent(&self) -> Option<Permission> {
        use Permission::*;
        match self {
            All => None,
            Moderation | Music | Config | Utility => Some(All),
            ModerationKick | ModerationBan | ModerationUnban | ModerationMute
            | ModerationUnmute | ModerationWarn | ModerationPardon | ModerationPurge
            | ModerationLookup => Some(Moderation),
            MusicPlay | MusicSkip | MusicStop | MusicPause | MusicResume | MusicClear
            | MusicVolume | MusicShuffle => Some(Music),
            ConfigView | ConfigEdit => Some(Config),
            UtilityInfo | UtilityUserinfo | UtilityServerinfo | UtilityHelp | UtilityPing
            | UtilityInvite | UtilitySelfLookup => Some(Utility),
        }
    }

    /// Get all permissions
    #[inline]
    pub fn all_permissions() -> HashSet<Permission> {
        use Permission::*;
        HashSet::from([
            All,
            Moderation,
            Music,
            Config,
            Utility,
            ModerationKick,
            ModerationBan,
            ModerationUnban,
            ModerationMute,
            ModerationUnmute,
            ModerationWarn,
            ModerationPardon,
            ModerationPurge,
            MusicPlay,
            MusicSkip,
            MusicStop,
            MusicPause,
            MusicResume,
            MusicClear,
            MusicVolume,
            MusicShuffle,
            ConfigView,
            ConfigEdit,
            UtilityInfo,
            UtilityUserinfo,
            UtilityServerinfo,
            UtilityHelp,
            UtilityPing,
            UtilityInvite,
        ])
    }

    /// Get all permissions as a vector
    #[inline]
    pub fn all_permissions_vec() -> Vec<Permission> {
        use Permission::*;
        vec![
            All,
            Moderation,
            Music,
            Config,
            Utility,
            ModerationKick,
            ModerationBan,
            ModerationUnban,
            ModerationMute,
            ModerationUnmute,
            ModerationWarn,
            ModerationPardon,
            ModerationPurge,
            MusicPlay,
            MusicSkip,
            MusicStop,
            MusicPause,
            MusicResume,
            MusicClear,
            MusicVolume,
            MusicShuffle,
            ConfigView,
            ConfigEdit,
            UtilityInfo,
            UtilityUserinfo,
            UtilityServerinfo,
            UtilityHelp,
            UtilityPing,
            UtilityInvite,
        ]
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::All => "all",
                Self::Moderation => "moderation",
                Self::Music => "music",
                Self::Config => "config",
                Self::Utility => "utility",
                Self::ModerationKick => "moderation.kick",
                Self::ModerationBan => "moderation.ban",
                Self::ModerationUnban => "moderation.unban",
                Self::ModerationMute => "moderation.mute",
                Self::ModerationUnmute => "moderation.unmute",
                Self::ModerationWarn => "moderation.warn",
                Self::ModerationPardon => "moderation.pardon",
                Self::ModerationPurge => "moderation.purge",
                Self::ModerationLookup => "moderation.lookup",
                Self::MusicPlay => "music.play",
                Self::MusicSkip => "music.skip",
                Self::MusicStop => "music.stop",
                Self::MusicPause => "music.pause",
                Self::MusicResume => "music.resume",
                Self::MusicClear => "music.clear",
                Self::MusicVolume => "music.volume",
                Self::MusicShuffle => "music.shuffle",
                Self::ConfigView => "config.view",
                Self::ConfigEdit => "config.edit",
                Self::UtilityInfo => "utility.info",
                Self::UtilityUserinfo => "utility.userinfo",
                Self::UtilityServerinfo => "utility.serverinfo",
                Self::UtilityHelp => "utility.help",
                Self::UtilityPing => "utility.ping",
                Self::UtilityInvite => "utility.invite",
                Self::UtilitySelfLookup => "utility.selflookup",
            }
        )
    }
}

impl Serialize for Permission {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Permission {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "all" => Ok(Self::All),
            "moderation" => Ok(Self::Moderation),
            "music" => Ok(Self::Music),
            "config" => Ok(Self::Config),
            "utility" => Ok(Self::Utility),
            "moderation.kick" => Ok(Self::ModerationKick),
            "moderation.ban" => Ok(Self::ModerationBan),
            "moderation.unban" => Ok(Self::ModerationUnban),
            "moderation.mute" => Ok(Self::ModerationMute),
            "moderation.unmute" => Ok(Self::ModerationUnmute),
            "moderation.warn" => Ok(Self::ModerationWarn),
            "moderation.pardon" => Ok(Self::ModerationPardon),
            "moderation.purge" => Ok(Self::ModerationPurge),
            "music.play" => Ok(Self::MusicPlay),
            "music.skip" => Ok(Self::MusicSkip),
            "music.stop" => Ok(Self::MusicStop),
            "music.pause" => Ok(Self::MusicPause),
            "music.resume" => Ok(Self::MusicResume),
            "music.clear" => Ok(Self::MusicClear),
            "music.volume" => Ok(Self::MusicVolume),
            "music.shuffle" => Ok(Self::MusicShuffle),
            "config.view" => Ok(Self::ConfigView),
            "config.edit" => Ok(Self::ConfigEdit),
            "utility.info" => Ok(Self::UtilityInfo),
            "utility.userinfo" => Ok(Self::UtilityUserinfo),
            "utility.serverinfo" => Ok(Self::UtilityServerinfo),
            "utility.help" => Ok(Self::UtilityHelp),
            "utility.ping" => Ok(Self::UtilityPing),
            "utility.invite" => Ok(Self::UtilityInvite),
            _ => Err(D::Error::custom("invalid permission")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
}

impl Serialize for PermissionSet {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let perms: Vec<String> = self.permissions.iter().map(|p| p.to_string()).collect();
        perms.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PermissionSet {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let perms: Vec<String> = Vec::deserialize(deserializer)?;
        let permissions = perms
            .iter()
            .map(|p| Permission::from_str(p).ok_or(D::Error::custom("invalid permission")))
            .collect::<Result<HashSet<Permission>, D::Error>>()?;
        Ok(Self { permissions })
    }
}

impl PermissionSet {
    #[inline]
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }

    pub fn with_permissions(permissions: HashSet<Permission>) -> Self {
        Self { permissions }
    }

    pub fn add(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    pub fn remove(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    pub fn extend(&mut self, other: PermissionSet) {
        self.permissions.extend(other.permissions);
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Permission) -> bool,
    {
        self.permissions.retain(f);
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        if self.permissions.contains(&Permission::All) {
            return true;
        }

        if self.permissions.contains(permission) {
            return true;
        }

        if let Some(parent) = permission.parent() {
            if self.permissions.contains(&parent) {
                return true;
            }
        }

        false
    }

    pub fn clear(&mut self) {
        self.permissions.clear();
    }

    pub fn permissions(&self) -> &HashSet<Permission> {
        &self.permissions
    }

    pub fn from_vec(permissions: Vec<Permission>) -> Self {
        Self {
            permissions: permissions.into_iter().collect(),
        }
    }

    /// Create a new PermissionSet from Discord permissions
    pub fn from_discord_permissions(roles: &HashSet<Role>, present: &HashSet<Id>) -> Self {
        let perms = roles.iter().fold(Permissions::empty(), |acc, role| {
            if present.contains(&role.id) {
                acc | role.permissions.clone()
            } else {
                acc
            }
        });

        tracing::debug!(
            "Creating permission set from Discord permissions: {:?}",
            perms
        );
        let mut set = Self::new();

        // Administrator gives all permissions
        if perms.contains(Permissions::ADMINISTRATOR) {
            set.add(Permission::All);
            return set;
        }

        // Individual moderation permissions
        if perms.contains(Permissions::KICK_MEMBERS) {
            set.add(Permission::ModerationKick);
        }
        if perms.contains(Permissions::BAN_MEMBERS) {
            set.add(Permission::ModerationBan);
            set.add(Permission::ModerationUnban);
        }
        if perms.contains(Permissions::MANAGE_MESSAGES) {
            set.add(Permission::ModerationMute);
            set.add(Permission::ModerationUnmute);
            set.add(Permission::ModerationWarn);
            set.add(Permission::ModerationPardon);
        }
        if perms.contains(Permissions::MANAGE_CHANNELS | Permissions::MANAGE_MESSAGES) {
            set.add(Permission::ModerationPurge);
        }

        // Music Permissions
        if perms.contains(Permissions::CONNECT) {
            set.add(Permission::Music);
            set.add(Permission::MusicPlay);
        }

        // Voice moderation powers grant intrusive music controls
        if perms.contains(Permissions::MUTE_MEMBERS)
            || perms.contains(Permissions::DEAFEN_MEMBERS)
            || perms.contains(Permissions::MOVE_MEMBERS)
        {
            set.add(Permission::MusicStop);
            set.add(Permission::MusicPause);
            set.add(Permission::MusicResume);
            set.add(Permission::MusicClear);
            set.add(Permission::MusicVolume);
            set.add(Permission::MusicShuffle);
            set.add(Permission::MusicSkip);
        }

        // Config Permissions
        if perms.contains(Permissions::MANAGE_CHANNELS) {
            set.add(Permission::Config);
            set.add(Permission::ConfigEdit);
        }
        if perms.contains(Permissions::VIEW_CHANNEL) {
            set.add(Permission::ConfigView);
        }

        // Utility Permissions (basic permissions everyone should have)
        if perms.contains(Permissions::VIEW_CHANNEL) {
            set.add(Permission::Utility);
            set.add(Permission::UtilityInfo);
            set.add(Permission::UtilityUserinfo);
            set.add(Permission::UtilityServerinfo);
            set.add(Permission::UtilityHelp);
            set.add(Permission::UtilityPing);
            set.add(Permission::UtilityInvite);
        }

        set
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<Permission> {
        self.permissions.iter()
    }
}

impl Default for PermissionSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionOverride {
    pub groups: Vec<String>,
    pub roles: Vec<Id>,
    pub users: Vec<Id>,
}
