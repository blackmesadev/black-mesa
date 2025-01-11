use serde::{Deserialize, Serialize};

use crate::{discord::Id, emojis::Emoji};

use super::{automod::AutomodOffense, Uuid};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Infraction {
    #[serde(rename = "_id")]
    pub uuid: Uuid,
    pub guild_id: Id,
    pub user_id: Id,
    pub moderator_id: Id,
    pub infraction_type: InfractionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_edited: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute_role_id: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automod_offense: Option<AutomodOffense>,
    pub active: bool,
}

impl Infraction {
    pub fn new(
        guild_id: Id,
        user_id: Id,
        moderator_id: Id,
        infraction_type: InfractionType,
        reason: Option<String>,
        expires_at: Option<u64>,
        active: bool,
    ) -> Self {
        Self {
            uuid: Uuid::new(),
            guild_id,
            user_id,
            moderator_id,
            infraction_type,
            reason,
            last_edited: None,
            expires_at,
            mute_role_id: None,
            automod_offense: None,
            active,
        }
    }

    pub fn new_ban(
        guild_id: Id,
        user_id: Id,
        moderator_id: Id,
        reason: Option<String>,
        expires_at: Option<u64>,
        active: bool,
    ) -> Self {
        Self::new(
            guild_id,
            user_id,
            moderator_id,
            InfractionType::Ban,
            reason,
            expires_at,
            active,
        )
    }

    pub fn new_kick(
        guild_id: Id,
        user_id: Id,
        moderator_id: Id,
        reason: Option<String>,
        active: bool,
    ) -> Self {
        Self::new(
            guild_id,
            user_id,
            moderator_id,
            InfractionType::Kick,
            reason,
            None,
            active,
        )
    }

    pub fn new_mute(
        guild_id: Id,
        user_id: Id,
        moderator_id: Id,
        reason: Option<String>,
        expires_at: Option<u64>,
        mute_role_id: Id,
        active: bool,
    ) -> Self {
        Self {
            uuid: Uuid::new(),
            guild_id,
            user_id,
            moderator_id,
            infraction_type: InfractionType::Mute,
            reason,
            last_edited: None,
            expires_at,
            mute_role_id: Some(mute_role_id),
            automod_offense: None,
            active,
        }
    }

    pub fn new_warn(
        guild_id: Id,
        user_id: Id,
        moderator_id: Id,
        reason: Option<String>,
        active: bool,
    ) -> Self {
        Self::new(
            guild_id,
            user_id,
            moderator_id,
            InfractionType::Warn,
            reason,
            None,
            active,
        )
    }

    pub fn new_automod(
        guild_id: Id,
        user_id: Id,
        moderator_id: Id,
        infraction_type: InfractionType,
        expires_at: Option<u64>,
        automod_offense: AutomodOffense,
        active: bool,
    ) -> Self {
        Self {
            uuid: Uuid::new(),
            guild_id,
            user_id,
            moderator_id,
            infraction_type,
            reason: None,
            last_edited: None,
            expires_at,
            mute_role_id: None,
            automod_offense: Some(automod_offense),
            active,
        }
    }

    pub fn get_emoji(&self) -> &'static str {
        self.infraction_type.to_emoji()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InfractionType {
    Warn,
    Mute,
    Kick,
    Ban,
}

impl std::fmt::Display for InfractionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl InfractionType {
    #[inline]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "warn" => Some(Self::Warn),
            "mute" => Some(Self::Mute),
            "kick" => Some(Self::Kick),
            "ban" => Some(Self::Ban),
            _ => None,
        }
    }

    #[inline]
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Warn => "warn",
            Self::Mute => "mute",
            Self::Kick => "kick",
            Self::Ban => "ban",
        }
    }

    #[inline]
    pub fn to_past_tense(&self) -> &'static str {
        match self {
            Self::Warn => "Warned",
            Self::Mute => "Muted",
            Self::Kick => "Kicked",
            Self::Ban => "Banned",
        }
    }

    #[inline]
    pub fn to_verb(&self) -> &'static str {
        match self {
            Self::Warn => "Warn",
            Self::Mute => "Mute",
            Self::Kick => "Kick",
            Self::Ban => "Ban",
        }
    }

    #[inline]
    pub fn to_noun(&self) -> &'static str {
        match self {
            Self::Warn => "Warning",
            Self::Mute => "Mute",
            Self::Kick => "Kick",
            Self::Ban => "Ban",
        }
    }

    #[inline]
    pub fn to_emoji(&self) -> &'static str {
        match self {
            Self::Warn => Emoji::Warn.to_emoji(),
            Self::Mute => Emoji::Mute.to_emoji(),
            Self::Kick => Emoji::Kick.to_emoji(),
            Self::Ban => Emoji::Ban.to_emoji(),
        }
    }
}
