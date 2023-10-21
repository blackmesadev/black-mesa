use twilight_model::{
    channel::{
        message::{Embed, Mention, MessageType},
        Attachment, Message,
    },
    gateway::payload::incoming::MessageUpdate,
    id::{
        marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker},
        Id,
    },
    user::User,
    util::Timestamp,
};

mod automod;
mod censor;
mod clean;
mod spam;

pub use automod::*;
pub use censor::*;
pub use clean::*;
pub use spam::*;

pub trait MessageTrait {
    fn attachments(&self) -> &[Attachment];
    fn author(&self) -> &User;
    fn channel_id(&self) -> &Id<ChannelMarker>;
    fn content(&self) -> &str;
    fn edited_timestamp(&self) -> &Option<Timestamp>;
    fn embeds(&self) -> &[Embed];
    fn guild_id(&self) -> &Option<Id<GuildMarker>>;
    fn id(&self) -> &Id<MessageMarker>;
    fn kind(&self) -> &MessageType;
    fn mention_everyone(&self) -> bool;
    fn mention_roles(&self) -> &[Id<RoleMarker>];
    fn mentions(&self) -> &[Mention];
    fn pinned(&self) -> bool;
    fn timestamp(&self) -> &Timestamp;
    fn tts(&self) -> bool;
}

impl std::fmt::Debug for dyn MessageTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageTrait")
            .field("attachments", &self.attachments())
            .field("author", &self.author())
            .field("channel_id", &self.channel_id())
            .field("content", &self.content())
            .field("edited_timestamp", &self.edited_timestamp())
            .field("embeds", &self.embeds())
            .field("guild_id", &self.guild_id())
            .field("id", &self.id())
            .field("kind", &self.kind())
            .field("mention_everyone", &self.mention_everyone())
            .field("mention_roles", &self.mention_roles())
            .field("mentions", &self.mentions())
            .field("pinned", &self.pinned())
            .field("timestamp", &self.timestamp())
            .field("tts", &self.tts())
            .finish()
    }
}

impl MessageTrait for MessageUpdate {
    fn attachments(&self) -> &[Attachment] {
        self.attachments.as_ref().map_or(&[], |v| v.as_slice())
    }

    fn author(&self) -> &User {
        self.author.as_ref().expect("Expected author to be present")
    }

    fn channel_id(&self) -> &Id<ChannelMarker> {
        &self.channel_id
    }

    fn content(&self) -> &str {
        self.content.as_deref().unwrap_or_default()
    }

    fn edited_timestamp(&self) -> &Option<Timestamp> {
        &self.edited_timestamp
    }

    fn embeds(&self) -> &[Embed] {
        self.embeds.as_ref().map_or(&[], |v| v.as_slice())
    }

    fn guild_id(&self) -> &Option<Id<GuildMarker>> {
        &self.guild_id
    }

    fn id(&self) -> &Id<MessageMarker> {
        &self.id
    }

    fn kind(&self) -> &MessageType {
        self.kind.as_ref().expect("Expected kind to be present")
    }

    fn mention_everyone(&self) -> bool {
        self.mention_everyone.unwrap_or(false)
    }

    fn mention_roles(&self) -> &[Id<RoleMarker>] {
        self.mention_roles.as_ref().map_or(&[], |v| v.as_slice())
    }

    fn mentions(&self) -> &[Mention] {
        self.mentions.as_ref().map_or(&[], |v| v.as_slice())
    }

    fn pinned(&self) -> bool {
        self.pinned.unwrap_or(false)
    }

    fn timestamp(&self) -> &Timestamp {
        self.timestamp
            .as_ref()
            .expect("Expected timestamp to be present")
    }

    fn tts(&self) -> bool {
        self.tts.unwrap_or(false)
    }
}

impl MessageTrait for Message {
    fn attachments(&self) -> &[Attachment] {
        &self.attachments
    }

    fn author(&self) -> &User {
        &self.author
    }

    fn channel_id(&self) -> &Id<ChannelMarker> {
        &self.channel_id
    }

    fn content(&self) -> &str {
        &self.content
    }

    fn edited_timestamp(&self) -> &Option<Timestamp> {
        &self.edited_timestamp
    }

    fn embeds(&self) -> &[Embed] {
        &self.embeds
    }

    fn guild_id(&self) -> &Option<Id<GuildMarker>> {
        &self.guild_id
    }

    fn id(&self) -> &Id<MessageMarker> {
        &self.id
    }

    fn kind(&self) -> &MessageType {
        &self.kind
    }

    fn mention_everyone(&self) -> bool {
        self.mention_everyone
    }

    fn mention_roles(&self) -> &[Id<RoleMarker>] {
        &self.mention_roles
    }

    fn mentions(&self) -> &[Mention] {
        &self.mentions
    }

    fn pinned(&self) -> bool {
        self.pinned
    }

    fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    fn tts(&self) -> bool {
        self.tts
    }
}
