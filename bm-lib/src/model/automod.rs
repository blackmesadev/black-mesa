use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{discord::Id, model::InfractionType, permissions::PermissionOverride};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomodConfig {
    pub enabled: bool,
    pub global: Option<AutomodSettings>,
    #[serde(default)]
    pub channels: HashMap<Id, AutomodSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomodSettings {
    pub name: String,
    pub enabled: bool,
    pub censors: Option<HashMap<CensorType, Censor>>,
    pub spam: Option<SpamFilter>,
    pub bypass: Option<PermissionOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Censor {
    pub enabled: bool,
    #[serde(default)]
    pub whitelist: bool,
    pub filters: Vec<String>,
    #[serde(default)]
    pub ignore_whitespace: bool,
    pub bypass: Option<PermissionOverride>,
    pub action: CensorAutomodAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CensorAutomodAction {
    pub action: InfractionType,
    pub duration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamFilter {
    pub enabled: bool,
    pub filters: HashMap<SpamType, SpamInterval>,
    pub bypass: Option<PermissionOverride>,
    pub action: Vec<SpamAutomodAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamInterval {
    pub interval: u64, // Millis, if 0 then it applies to all messages
    pub count: u64,    // frequency of the matching item
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamAutomodAction {
    pub action: InfractionType,
    pub duration: i64,
    pub threshold: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomodOffense {
    #[serde(rename = "type")]
    pub typ: OffenseType,
    pub message: String,
    pub count: Option<u64>,
    pub interval: Option<u64>,
    pub offending_filter: Option<String>,
}

#[derive(Debug, Clone)]
pub enum OffenseType {
    Spam(SpamType),
    Censor(CensorType),
}

impl std::fmt::Display for OffenseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OffenseType::Spam(spam) => write!(f, "{}-spam", spam),
            OffenseType::Censor(censor) => write!(f, "{}-censor", censor),
        }
    }
}

impl Serialize for OffenseType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            OffenseType::Spam(spam) => spam.serialize(serializer),
            OffenseType::Censor(censor) => censor.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for OffenseType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "message-spam" => Ok(OffenseType::Spam(SpamType::Message)),
            "newline-spam" => Ok(OffenseType::Spam(SpamType::Newline)),
            "word-censor" => Ok(OffenseType::Censor(CensorType::Word)),
            "link-censor" => Ok(OffenseType::Censor(CensorType::Link)),
            "invite-censor" => Ok(OffenseType::Censor(CensorType::Invite)),
            _ => Err(serde::de::Error::custom("invalid offense type")),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpamType {
    Message,
    Newline,
}

impl std::fmt::Display for SpamType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpamType::Message => write!(f, "message"),
            SpamType::Newline => write!(f, "newline"),
        }
    }
}

impl SpamType {
    pub fn to_pretty_string(&self) -> String {
        match self {
            SpamType::Message => "Message",
            SpamType::Newline => "Newline",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CensorType {
    Word,
    Link,
    Invite,
}

impl std::fmt::Display for CensorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CensorType::Word => write!(f, "word"),
            CensorType::Link => write!(f, "link"),
            CensorType::Invite => write!(f, "invite"),
        }
    }
}

impl CensorType {
    pub fn to_pretty_string(&self) -> String {
        match self {
            CensorType::Word => "Word",
            CensorType::Link => "Link",
            CensorType::Invite => "Invite",
        }
        .to_string()
    }
}