use redis::ToRedisArgs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol_str::SmolStr;
use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use super::Permissions;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(u64);

impl Id {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn get(&self) -> u64 {
        self.0
    }

    pub fn clone(&self) -> Self {
        Self(self.0)
    }

    pub fn from_str(s: &str) -> Result<Self, ParseIntError> {
        s.trim_matches(|c| c == '<' || c == '>')
            .trim_start_matches(|c| c == '#' || c == '@' || c == '!' || c == '&')
            .parse()
            .map(Self)
    }
}

impl From<u64> for Id {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToRedisArgs for Id {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        self.0.write_redis_args(out);
    }
}

impl AsRef<u64> for Id {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl FromStr for Id {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Id(s.parse()?))
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IdVisitor;

        impl<'de> serde::de::Visitor<'de> for IdVisitor {
            type Value = Id;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or integer containing a Discord ID")
            }

            fn visit_str<E>(self, value: &str) -> Result<Id, E>
            where
                E: serde::de::Error,
            {
                value.parse().map(Id).map_err(serde::de::Error::custom)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Id, E>
            where
                E: serde::de::Error,
            {
                Ok(Id(value))
            }
        }

        deserializer.deserialize_any(IdVisitor)
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gateway {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload<T> {
    pub op: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Identify {
    pub token: Cow<'static, str>,
    pub properties: ConnectionProperties,
    pub intents: Intents,
    pub shard: [u32; 2],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionProperties {
    #[serde(rename = "$os")]
    pub os: Cow<'static, str>,
    #[serde(rename = "$browser")]
    pub browser: SmolStr,
    #[serde(rename = "$device")]
    pub device: SmolStr,
}

impl ConnectionProperties {
    pub fn new() -> Self {
        let version = env!("CARGO_PKG_VERSION");
        Self {
            os: Cow::Borrowed(std::env::consts::OS),
            browser: SmolStr::new(format!("black-mesa/{}", version)),
            device: SmolStr::new(format!("black-mesa/{}", version)),
        }
    }
}

// Gateway events

#[derive(Debug, Serialize, Deserialize)]
pub struct Hello {
    pub heartbeat_interval: u64, // milliseconds
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ready {
    pub v: u8,
    pub user: Value,
    #[serde(default)]
    pub guilds: Vec<PartialGuild>,
    pub session_id: Option<String>,
    pub resume_gateway_url: Option<String>,
    pub shard: Option<[u8; 2]>,
}

// REST API

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: Id,
    pub channel_id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<Member>,
    pub content: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention_everyone: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mentions: Vec<User>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mention_roles: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Id,
    pub filename: String,
    pub size: u32,
    pub url: String,
    pub proxy_url: String,
    pub height: Option<u16>,
    pub width: Option<u16>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Embed {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<EmbedFooter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<EmbedImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<EmbedThumbnail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<EmbedVideo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<EmbedProvider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<EmbedAuthor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<EmbedField>>,
}

impl Embed {
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            url: None,
            timestamp: None,
            color: None,
            footer: None,
            image: None,
            thumbnail: None,
            video: None,
            provider: None,
            author: None,
            fields: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.description.is_none()
            && self.url.is_none()
            && self.timestamp.is_none()
            && self.color.is_none()
            && self.footer.is_none()
            && self.image.is_none()
            && self.thumbnail.is_none()
            && self.video.is_none()
            && self.provider.is_none()
            && self.author.is_none()
            && self.fields.as_ref().map_or(true, |f| f.is_empty())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbedFooter {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbedImage {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbedThumbnail {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbedVideo {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbedProvider {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbedAuthor {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    pub id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Id>,
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_id: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_user: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nick: Option<String>,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub roles: HashSet<Id>,
    pub joined_at: SmolStr, // ISO8601 timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premium_since: Option<String>,
    #[serde(default)]
    pub deaf: bool,
    #[serde(default)]
    pub mute: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub communication_disabled_until: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Id,
    pub username: SmolStr,
    pub discriminator: SmolStr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<SmolStr>,
    #[serde(default)]
    pub bot: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premium_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_flags: Option<u32>,
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Role {
    pub id: Id,
    pub name: SmolStr,
    pub color: u32,
    pub hoist: bool,
    pub position: i32,
    pub permissions: Permissions,
    pub managed: bool,
    pub mentionable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unicode_emoji: Option<SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Guild {
    pub id: Id,
    pub name: SmolStr, // Max 100 characters
    pub icon: Option<SmolStr>,
    #[serde(default)]
    pub roles: HashSet<Role>,
    pub approximate_member_count: Option<u32>,
    pub owner_id: Option<Id>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartialGuild {
    pub id: Id,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub approximate_member_count: Option<u32>,
    pub approximate_presence_count: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildMemberUpdate {
    pub guild_id: Id,
    pub roles: HashSet<Id>,
    pub user: User,
    pub nick: Option<String>,
    pub joined_at: SmolStr,
    pub premium_since: Option<String>,
    pub deaf: bool,
    pub mute: bool,
}

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct Intents: u64 {
        const GUILDS = 1 << 0;
        const GUILD_MEMBERS = 1 << 1;
        const GUILD_BANS = 1 << 2;
        const GUILD_EMOJIS_AND_STICKERS = 1 << 3;
        const GUILD_INTEGRATIONS = 1 << 4;
        const GUILD_WEBHOOKS = 1 << 5;
        const GUILD_INVITES = 1 << 6;
        const GUILD_VOICE_STATES = 1 << 7;
        const GUILD_PRESENCES = 1 << 8;
        const GUILD_MESSAGES = 1 << 9;
        const GUILD_MESSAGE_REACTIONS = 1 << 10;
        const GUILD_MESSAGE_TYPING = 1 << 11;
        const DIRECT_MESSAGES = 1 << 12;
        const DIRECT_MESSAGE_REACTIONS = 1 << 13;
        const DIRECT_MESSAGE_TYPING = 1 << 14;
        const MESSAGE_CONTENT = 1 << 15;
        const GUILD_SCHEDULED_EVENTS = 1 << 16;
        const AUTO_MODERATION_CONFIGURATION = 1 << 20;
        const AUTO_MODERATION_EXECUTION = 1 << 21;

        const NONE = 0;
        const ALL = Self::GUILDS.bits()
            | Self::GUILD_MEMBERS.bits()
            | Self::GUILD_BANS.bits()
            | Self::GUILD_EMOJIS_AND_STICKERS.bits()
            | Self::GUILD_INTEGRATIONS.bits()
            | Self::GUILD_WEBHOOKS.bits()
            | Self::GUILD_INVITES.bits()
            | Self::GUILD_VOICE_STATES.bits()
            | Self::GUILD_PRESENCES.bits()
            | Self::GUILD_MESSAGES.bits()
            | Self::GUILD_MESSAGE_REACTIONS.bits()
            | Self::GUILD_MESSAGE_TYPING.bits()
            | Self::DIRECT_MESSAGES.bits()
            | Self::DIRECT_MESSAGE_REACTIONS.bits()
            | Self::DIRECT_MESSAGE_TYPING.bits()
            | Self::MESSAGE_CONTENT.bits()
            | Self::GUILD_SCHEDULED_EVENTS.bits()
            | Self::AUTO_MODERATION_CONFIGURATION.bits()
            | Self::AUTO_MODERATION_EXECUTION.bits();

        const DEFAULT = Self::GUILDS.bits()
            | Self::GUILD_MESSAGES.bits()
            | Self::GUILD_MESSAGE_REACTIONS.bits()
            | Self::DIRECT_MESSAGES.bits()
            | Self::DIRECT_MESSAGE_REACTIONS.bits();
    }
}

impl Default for Intents {
    fn default() -> Self {
        Intents::DEFAULT
    }
}
impl Serialize for Intents {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.bits())
    }
}

impl<'de> Deserialize<'de> for Intents {
    fn deserialize<D>(deserializer: D) -> Result<Intents, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u64::deserialize(deserializer)?;
        Ok(Intents::from_bits(bits).unwrap_or_else(|| Intents::NONE))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ShardConfig {
    pub shard_id: u32,
    pub num_shards: u32,
}

impl ShardConfig {
    pub fn new(shard_id: u32, num_shards: u32) -> Self {
        assert!(
            shard_id < num_shards,
            "shard_id must be less than num_shards"
        );
        Self {
            shard_id,
            num_shards,
        }
    }

    pub fn to_array(&self) -> [u32; 2] {
        [self.shard_id, self.num_shards]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayBot {
    pub url: String,
    pub shards: u32,
    pub session_start_limit: SessionStartLimit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStartLimit {
    pub total: u32,
    pub remaining: u32,
    pub reset_after: u64,
    pub max_concurrency: u32,
}
