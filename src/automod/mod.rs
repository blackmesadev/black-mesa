use serde::{Deserialize, Serialize};
use twilight_model::{
    channel::{
        message::{Embed, Mention},
        Attachment,
    },
    id::{
        marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker},
        Id,
    },
    user::User,
    util::Timestamp,
};

pub mod automod;
pub mod censor;
pub mod clean;
pub mod spam;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomodMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
    pub author: User,
    pub channel_id: Id<ChannelMarker>,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_timestamp: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Id<GuildMarker>>,
    pub id: Id<MessageMarker>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention_roles: Option<Vec<Id<RoleMarker>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<Mention>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<Timestamp>,
}
