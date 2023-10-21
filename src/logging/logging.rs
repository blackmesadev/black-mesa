#![allow(dead_code)]
use serde_derive::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::str::FromStr;
use twilight_cache_inmemory::{model::CachedMessage, Reference};
use twilight_http::Response;
use twilight_model::channel::message::AllowedMentions;
use twilight_model::id::Id;
use twilight_model::{
    channel::Message, gateway::payload::incoming::MessageUpdate, id::marker::MessageMarker,
};

use crate::automod::MessageTrait;
use crate::{
    automod::AutomodResult, handlers::Handler, moderation::moderation::Punishment,
    util::duration::Duration,
};

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Logging {
    pub enabled: Option<bool>,
    pub channel_id: Option<String>,
    pub include_events: Option<Vec<Event>>,
    pub ignored_users: Option<Vec<String>>,
    pub ignored_channels: Option<Vec<String>>,
}

impl Default for Logging {
    fn default() -> Self {
        Self {
            enabled: Some(false),
            channel_id: None,
            include_events: None,
            ignored_users: None,
            ignored_channels: None,
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    None,
    AutomodCensor,
    AutomodSpam,
    Strike,
    Mute,
    Unmute,
    Kick,
    Ban,
    Unban,
    RemoveAction,
    UpdateAction,

    MessageDelete,
    MessageEdit,
    ChannelDelete,
    ChannelEdit,
    ChannelCreate,

    AuditLog,
    MessageLog,
    ModLog,
    AutomodLog,

    All,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Log {
    pub event: Event,
    pub target: Option<String>,
    pub actor: Option<String>,
    pub reason: Option<String>,
    pub duration: Option<Duration>,
    pub uuid: Option<String>,
}

impl Log {
    pub fn new(
        event: Event,
        target: Option<String>,
        actor: Option<String>,
        reason: Option<String>,
        duration: Option<Duration>,
        uuid: Option<String>,
    ) -> Self {
        Self {
            event,
            target,
            actor,
            reason,
            duration,
            uuid,
        }
    }
}

impl Handler {
    pub async fn log_message_censor<T: MessageTrait>(
        &self,
        logging: &Logging,
        res: AutomodResult<T>,
    ) -> Option<Response<Message>> {
        let events = match logging.include_events.as_ref() {
            Some(events) => events,
            None => return None,
        };

        if !logging.enabled.unwrap_or(false)
            || events.contains(&Event::AutomodCensor)
            || events.contains(&Event::AutomodLog)
            || events.contains(&Event::All)
        {
            return None;
        }

        let msg = res.msg;

        let content = format!("<:mesaCensoredMessage:869663511754731541> <@{}> (`{}`) triggered automod rule `Censor->{}`: `{}` in channel <#{}> (`{}`)",
            msg.author().id.get(), msg.author().id.get(), res.typ.get_name(), res.trigger.unwrap_or("".to_string()),
            msg.channel_id().get(), msg.channel_id().get());

        let allowed_mentions = AllowedMentions::default();

        self.rest
            .create_message(match &logging.channel_id {
                Some(id) => Id::from_str(id.as_str()).ok()?,
                None => return None,
            })
            .content(&content)
            .ok()?
            .allowed_mentions(Some(&allowed_mentions))
            .await
            .ok()
    }

    pub async fn log_message_spam<T: MessageTrait>(
        &self,
        logging: &Logging,
        res: AutomodResult<T>,
    ) -> Option<Response<Message>> {
        let events = match logging.include_events.as_ref() {
            Some(events) => events,
            None => return None,
        };

        if !logging.enabled.unwrap_or(false)
            || events.contains(&Event::AutomodSpam)
            || events.contains(&Event::AutomodLog)
            || events.contains(&Event::All)
        {
            return None;
        }

        let msg = res.msg;

        let content = format!("<:mesaMessageViolation:869663336625733635> <@{}> (`{}`) triggered automod rule `Spam->{}` in channel <#{}> (`{}`)",
        msg.author().id.get(), msg.author().id.get(), res.typ.get_name(), msg.channel_id().get(), msg.channel_id().get());

        let allowed_mentions = AllowedMentions::default();

        self.rest
            .create_message(match &logging.channel_id {
                Some(id) => Id::from_str(id.as_str()).ok()?,
                None => return None,
            })
            .content(&content)
            .ok()?
            .allowed_mentions(Some(&allowed_mentions))
            .await
            .ok()
    }

    pub async fn log_strike(
        &self,
        logging: &Logging,
        actor: &String,
        target: &String,
        reason: Option<&String>,
        duration: &Duration,
        uuid: &String,
    ) -> Option<Response<Message>> {
        let events = match logging.include_events.as_ref() {
            Some(events) => events,
            None => return None,
        };

        if !logging.enabled.unwrap_or(false)
            || events.contains(&Event::Strike)
            || events.contains(&Event::ModLog)
            || events.contains(&Event::All)
        {
            return None;
        }

        let content = if reason.is_some() {
            format!("<:mesaStrike:869663336843845752> <@{}> (`{}`)  issued strike to <@{}> expiring {}: `{}`. UUID: `{}`",
                actor, actor, target, duration.to_discord_timestamp(), reason.unwrap(), uuid)
        } else {
            format!("<:mesaStrike:869663336843845752> <@{}> (`{}`) issued strike to <@{}> expiring {}:. UUID: `{}`",
                actor, actor, target, duration.to_discord_timestamp(), uuid)
        };

        let allowed_mentions = AllowedMentions::default();

        self.rest
            .create_message(match &logging.channel_id {
                Some(id) => Id::from_str(id.as_str()).ok()?,
                None => return None,
            })
            .content(&content)
            .ok()?
            .allowed_mentions(Some(&allowed_mentions))
            .await
            .ok()
    }

    pub async fn log_remove_action(
        &self,
        logging: &Logging,
        actor: &String,
        punishment: &Punishment,
        reason: Option<&String>,
    ) -> Option<Response<Message>> {
        let events = match logging.include_events.as_ref() {
            Some(events) => events,
            None => return None,
        };

        if !logging.enabled.unwrap_or(false)
            || events.contains(&Event::RemoveAction)
            || events.contains(&Event::AuditLog)
            || events.contains(&Event::All)
        {
            return None;
        }

        let content = format!("<:mesaCheck:832350526729224243> <@{}> (`{}`) removed action of UUID `{}` from `{}` with type `{}` for `{}`",
            actor, actor, punishment.uuid, punishment.user_id, punishment.typ.pretty_string(), reason.unwrap_or(&"No reason provided".to_string()));

        let allowed_mentions = AllowedMentions::default();

        self.rest
            .create_message(match &logging.channel_id {
                Some(id) => Id::from_str(id.as_str()).ok()?,
                None => return None,
            })
            .content(&content)
            .ok()?
            .allowed_mentions(Some(&allowed_mentions))
            .await
            .ok()
    }

    pub async fn log_update_action(
        &self,
        logging: &Logging,
        actor: &String,
        punishment: &Punishment,
        duration: Option<&Duration>,
        reason: Option<&String>,
    ) -> Option<Response<Message>> {
        let events = match logging.include_events.as_ref() {
            Some(events) => events,
            None => return None,
        };

        if !logging.enabled.unwrap_or(false)
            || events.contains(&Event::UpdateAction)
            || events.contains(&Event::AuditLog)
            || events.contains(&Event::All)
        {
            return None;
        }

        let mut content = format!(
            "<:mesaCheck:832350526729224243> <@{}> (`{}`) updated action of UUID `{}` for <@{}>.",
            actor, actor, punishment.uuid, punishment.user_id
        );
        match duration {
            Some(d) => content += &format!(" New expiry: {}", d.to_discord_timestamp()),
            None => {}
        };
        match reason {
            Some(r) => content += &format!(" New reason: {}", r),
            None => {}
        };

        let allowed_mentions = AllowedMentions::default();

        self.rest
            .create_message(match &logging.channel_id {
                Some(id) => Id::from_str(id.as_str()).ok()?,
                None => return None,
            })
            .content(&content)
            .ok()?
            .allowed_mentions(Some(&allowed_mentions))
            .await
            .ok()
    }

    pub async fn log_mute(
        &self,
        logging: &Logging,
        actor: &String,
        target: &String,
        reason: Option<&String>,
        duration: &Duration,
        uuid: &String,
    ) -> Option<Response<Message>> {
        let events = match logging.include_events.as_ref() {
            Some(events) => events,
            None => return None,
        };

        if !logging.enabled.unwrap_or(false)
            || events.contains(&Event::RemoveAction)
            || events.contains(&Event::AuditLog)
            || events.contains(&Event::All)
        {
            return None;
        }

        let content = format!("<:mesaMemberMute:869663336814497832> <@{}> (`{}`) muted <@{}> (`{}`) expiring {}: `{}`. UUID: `{}`",
            actor, actor, target, target,
            duration.to_discord_timestamp(), reason.unwrap_or(&"No reason provided".to_string()), uuid);

        let allowed_mentions = AllowedMentions::default();

        self.rest
            .create_message(match &logging.channel_id {
                Some(id) => Id::from_str(id.as_str()).ok()?,
                None => return None,
            })
            .content(&content)
            .ok()?
            .allowed_mentions(Some(&allowed_mentions))
            .await
            .ok()
    }

    pub async fn log_unmute(
        &self,
        logging: &Logging,
        actor: &String,
        target: &String,
        reason: Option<&String>,
    ) -> Option<Response<Message>> {
        let events = match logging.include_events.as_ref() {
            Some(events) => events,
            None => return None,
        };

        if !logging.enabled.unwrap_or(false)
            || events.contains(&Event::Unmute)
            || events.contains(&Event::ModLog)
            || events.contains(&Event::All)
        {
            return None;
        }

        let content = format!(
            "<:mesaUnstrike:869664457788358716> <@{}> (`{}`) unmuted <@{}> (`{}`): `{}`.",
            actor,
            actor,
            target,
            target,
            reason.unwrap_or(&"No reason provided".to_string())
        );

        let allowed_mentions = AllowedMentions::default();

        self.rest
            .create_message(match &logging.channel_id {
                Some(id) => Id::from_str(id.as_str()).ok()?,
                None => return None,
            })
            .content(&content)
            .ok()?
            .allowed_mentions(Some(&allowed_mentions))
            .await
            .ok()
    }

    pub async fn log_kick(
        &self,
        logging: &Logging,
        actor: &String,
        target: &String,
        reason: Option<&String>,
        uuid: &String,
    ) -> Option<Response<Message>> {
        let events = match logging.include_events.as_ref() {
            Some(events) => events,
            None => return None,
        };

        if !logging.enabled.unwrap_or(false)
            || events.contains(&Event::Kick)
            || events.contains(&Event::ModLog)
            || events.contains(&Event::All)
        {
            return None;
        }

        let content = format!(
            "<:mesaKick:869665034312253460> <@{}> (`{}`) kicked <@{}> (`{}`): `{}`. UUID: `{}`",
            actor,
            actor,
            target,
            target,
            reason.unwrap_or(&"No reason provided".to_string()),
            uuid
        );

        let allowed_mentions = AllowedMentions::default();

        self.rest
            .create_message(match &logging.channel_id {
                Some(id) => Id::from_str(id.as_str()).ok()?,
                None => return None,
            })
            .content(&content)
            .ok()?
            .allowed_mentions(Some(&allowed_mentions))
            .await
            .ok()
    }

    pub async fn log_softban(
        &self,
        logging: &Logging,
        actor: &String,
        target: &String,
        reason: Option<&String>,
        duration: &Duration,
        uuid: &String,
    ) -> Option<Response<Message>> {
        let events = match logging.include_events.as_ref() {
            Some(events) => events,
            None => return None,
        };

        if !logging.enabled.unwrap_or(false)
            || events.contains(&Event::Ban)
            || events.contains(&Event::ModLog)
            || events.contains(&Event::All)
        {
            return None;
        }

        let content = format!("<:mesaBan:869663336625733634> <@{}> (`{}`) softbanned <@{}> (`{}`) deleting {} worth of messages: `{}`. UUID: `{}`",
            actor, actor, target, target,
            duration.to_discord_timestamp(), reason.unwrap_or(&"No reason provided".to_string()), uuid);

        let allowed_mentions = AllowedMentions::default();

        self.rest
            .create_message(match &logging.channel_id {
                Some(id) => Id::from_str(id.as_str()).ok()?,
                None => return None,
            })
            .content(&content)
            .ok()?
            .allowed_mentions(Some(&allowed_mentions))
            .await
            .ok()
    }

    pub async fn log_ban(
        &self,
        logging: &Logging,
        actor: &String,
        target: &String,
        reason: Option<&String>,
        duration: &Duration,
        uuid: &String,
    ) -> Option<Response<Message>> {
        let events = match logging.include_events.as_ref() {
            Some(events) => events,
            None => return None,
        };

        if !logging.enabled.unwrap_or(false)
            || events.contains(&Event::Ban)
            || events.contains(&Event::ModLog)
            || events.contains(&Event::All)
        {
            return None;
        }

        let content = format!("<:mesaBan:869663336625733634> <@{}> (`{}`) banned <@{}> (`{}`) expiring {}: `{}`. UUID: `{}`",
            actor, actor, target, target,
            duration.to_discord_timestamp(), reason.unwrap_or(&"No reason provided".to_string()), uuid);

        let allowed_mentions = AllowedMentions::default();

        self.rest
            .create_message(match &logging.channel_id {
                Some(id) => Id::from_str(id.as_str()).ok()?,
                None => return None,
            })
            .content(&content)
            .ok()?
            .allowed_mentions(Some(&allowed_mentions))
            .await
            .ok()
    }

    pub async fn log_unban(
        &self,
        logging: &Logging,
        actor: &String,
        target: &String,
        reason: Option<&String>,
    ) -> Option<Response<Message>> {
        let events = match logging.include_events.as_ref() {
            Some(events) => events,
            None => return None,
        };

        if !logging.enabled.unwrap_or(false)
            || events.contains(&Event::Unban)
            || events.contains(&Event::ModLog)
            || events.contains(&Event::All)
        {
            return None;
        }

        let content = format!(
            "<:mesaUnban:869663336697069619> <@{}> (`{}`) unbanned <@{}> (`{}`): `{}`",
            actor,
            actor,
            target,
            target,
            reason.unwrap_or(&"No reason provided".to_string())
        );

        let allowed_mentions = AllowedMentions::default();

        self.rest
            .create_message(match &logging.channel_id {
                Some(id) => Id::from_str(id.as_str()).ok()?,
                None => return None,
            })
            .content(&content)
            .ok()?
            .allowed_mentions(Some(&allowed_mentions))
            .await
            .ok()
    }

    pub async fn log_message_delete(
        &self,
        logging: &Logging,
        msg: Reference<'_, twilight_model::id::Id<MessageMarker>, CachedMessage>,
        actor: Option<String>,
    ) -> Option<Response<Message>> {
        let events = match logging.include_events.as_ref() {
            Some(events) => events,
            None => return None,
        };

        if !logging.enabled.unwrap_or(false)
            || events.contains(&Event::MessageDelete)
            || events.contains(&Event::MessageLog)
            || events.contains(&Event::All)
        {
            return None;
        }

        let mut content = match actor {
            Some(actor) => {
                format!("<:mesaMessageDelete:869663511977025586> Message by <@{}> (`{}`) deleted by <@{}> (`{}`) in channel <#{}> (`{}`): ",
                    msg.author().get(), msg.author().get(), actor, actor, msg.channel_id().get(), msg.channel_id().get())
            }
            None => {
                format!("<:mesaMessageDelete:869663511977025586> Message by <@{}> (`{}`) deleted in channel <#{}> (`{}`): ",
                    msg.author().get(), msg.author().get(), msg.channel_id().get(), msg.channel_id().get())
            }
        };

        if msg.content().len() > 0 {
            content += format!("`{}`", &msg.content()).as_str();
        }

        if msg.attachments().len() > 0 {
            content += "\n Attachments: ";
            for attachment in msg.attachments() {
                content += format!("`{}` ", attachment.url).as_str();
            }
        }

        let allowed_mentions = AllowedMentions::default();

        self.rest
            .create_message(match &logging.channel_id {
                Some(id) => Id::from_str(id.as_str()).ok()?,
                None => return None,
            })
            .content(&content)
            .ok()?
            .allowed_mentions(Some(&allowed_mentions))
            .await
            .ok()
    }

    pub async fn log_message_edit(
        &self,
        logging: &Logging,
        msg: &MessageUpdate,
        before: String,
    ) -> Option<Response<Message>> {
        let events = match logging.include_events.as_ref() {
            Some(events) => events,
            None => return None,
        };

        if !logging.enabled.unwrap_or(false)
            || events.contains(&Event::MessageEdit)
            || events.contains(&Event::MessageLog)
            || events.contains(&Event::All)
        {
            return None;
        }

        let msg_author = match &msg.author {
            Some(a) => a,
            None => return None,
        };

        let content = format!("<:mesaMessageEdit:869663511834411059> Message by <@{}> (`{}`) edited in channel <#{}> (`{}`):\n**Before**:\n`{}`\n**After**:\n`{}`",
        msg_author.id.get(),msg_author.id.get(), msg.channel_id.get(), msg.channel_id.get(), before,
        msg.content.as_ref().unwrap_or(&"Unable to fetch message content".to_string()));

        let allowed_mentions = AllowedMentions::default();

        self.rest
            .create_message(match &logging.channel_id {
                Some(id) => Id::from_str(id.as_str()).ok()?,
                None => return None,
            })
            .content(&content)
            .ok()?
            .allowed_mentions(Some(&allowed_mentions))
            .await
            .ok()
    }
}
