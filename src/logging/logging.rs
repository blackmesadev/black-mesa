#![allow(dead_code)]
use serde_derive::{Serialize, Deserialize};
use serde_with::skip_serializing_none;
use twilight_cache_inmemory::{Reference, model::CachedMessage};
use twilight_model::{id::marker::MessageMarker, gateway::payload::incoming::MessageUpdate};

use crate::{automod::automod::AutomodResult, mongo::mongo::Punishment, util::duration::Duration};

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logging {
    pub enabled: Option<bool>,
    #[serde(rename = "channelID")]
    pub channel_id: Option<String>,
    pub include_actions: Option<Vec<Actions>>,
    pub exclude_actions: Option<Vec<Actions>>,
    pub timestamps: Option<bool>,
    pub timezone: Option<String>,
    pub ignored_users: Option<Vec<String>>,
    pub ignored_channels: Option<Vec<String>>,
    pub ignored_roles: Option<Vec<String>>
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Actions {
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
}

impl Logging {
    pub fn log_message_censor(&self, res: AutomodResult) -> Option<String> {
        if !self.enabled.unwrap_or(false)
        || self.exclude_actions.as_ref().unwrap_or(&vec![Actions::None]).contains(&Actions::AutomodCensor) {
            return None;
        }

        let msg = res.msg;
        
        Some(format!("<:mesaCensoredMessage:869663511754731541> {} triggered automod rule `Censor->{}`: `{}` in channel <#{}> (`{}`)",
            msg.author.id.get(), res.typ.get_name(), res.trigger.unwrap_or("".to_string()),
            msg.channel_id.get(), msg.channel_id.get()))
    }

    pub fn log_message_spam(&self, res: AutomodResult) -> Option<String>  {
        if !self.enabled.unwrap_or(false) 
        || self.exclude_actions.as_ref().unwrap_or(&vec![Actions::None]).contains(&Actions::AutomodSpam) {
                return None;
        }

        let msg = res.msg;

        Some(format!("<:mesaMessageViolation:869663336625733635> {} triggered automod rule `Spam->{}` in channel <#{}> (`{}`)",
        msg.author.id.get(), res.typ.get_name(), msg.channel_id.get(), msg.channel_id.get()))
    }

    pub fn log_strike(&self, actor: String, target: String, reason: Option<String>, duration: Duration, uuid: String) -> Option<String> {
        if !self.enabled.unwrap_or(false) 
        || self.exclude_actions.as_ref().unwrap_or(&vec![Actions::None]).contains(&Actions::Strike) {
                return None;
        }

        if reason.is_some() {
            Some(format!("<:mesaStrike:869663336843845752> <@{}> issued strike to <@{}> expiring {}: `{}`. UUID: `{}`",
                actor, 
                target, duration.to_discord_timestamp(), reason.unwrap(), uuid))
        } else {
            Some(format!("<:mesaStrike:869663336843845752> <@{}> issued strike to <@{}> expiring {}:. UUID: `{}`",
                actor, 
                target, duration.to_discord_timestamp(), uuid))
        }
    }

    pub fn log_remove_action(&self, actor: String, punishment: &Punishment, reason: Option<String>) -> Option<String>  {
        if !self.enabled.unwrap_or(false) 
        || self.exclude_actions.as_ref().unwrap_or(&vec![Actions::None]).contains(&Actions::RemoveAction) {
                return None;
        }
        
        Some(format!("<:mesaCheck:832350526729224243> <@{}> removed action of UUID `{}` from `{}` with type `{}` for `{}`",
            actor, punishment.uuid, punishment.user_id,
            punishment.typ.pretty_string(), reason.unwrap_or("No reason provided".to_string())))
    }

    pub fn log_update_action(&self, actor: String, punishment: &Punishment, duration: Option<Duration>, reason: Option<String>) -> Option<String>  {
        if !self.enabled.unwrap_or(false) 
        || self.exclude_actions.as_ref().unwrap_or(&vec![Actions::None]).contains(&Actions::UpdateAction) {
                return None;
        }
        
        let mut log = format!("<:mesaCheck:832350526729224243> {} updated action of UUID `{}` for <@{}>.",
            actor, punishment.uuid, punishment.user_id);
        match duration {
            Some(d) => log += &format!(" New expiry: {}", d.to_discord_timestamp()),
            None => {}
        };
        match reason {
            Some(r) => log += &format!(" New reason: {}", r),
            None => {}
        };

        Some(log)
    }

    pub fn log_mute(&self, actor: String, target: String, reason: Option<String>, duration: Duration, uuid: String) -> Option<String> {
        if !self.enabled.unwrap_or(false) 
        || self.exclude_actions.as_ref().unwrap_or(&vec![Actions::None]).contains(&Actions::RemoveAction) {
                return None;
        }
        
        Some(format!("<:mesaMemberMute:869663336814497832> <@{}> muted <@{}> expiring {}: `{}`. UUID: `{}`",
            actor, target,
            duration.to_discord_timestamp(), reason.unwrap_or("No reason provided".to_string()), uuid))
    }

    pub fn log_unmute(&self, actor: String, target: String, reason: Option<String>) -> Option<String> {
        if !self.enabled.unwrap_or(false) 
        || self.exclude_actions.as_ref().unwrap_or(&vec![Actions::None]).contains(&Actions::Unmute) {
                return None;
        }
        
        Some(format!("<:mesaUnstrike:869664457788358716> <@{}> unmuted <@{}>: `{}`.",
            actor, target,
            reason.unwrap_or("No reason provided".to_string())))
    }

    pub fn log_kick(&self, actor: String, target: String, reason: Option<String>, uuid: String) -> Option<String> {
        if !self.enabled.unwrap_or(false) 
        || self.exclude_actions.as_ref().unwrap_or(&vec![Actions::None]).contains(&Actions::Kick) {
                return None;
        }
        
        Some(format!("<:mesaKick:869665034312253460> <@{}> kicked <@{}>: `{}`. UUID: `{}`",
            actor, target,
            reason.unwrap_or("No reason provided".to_string()), uuid))
    }

    pub fn log_ban(&self, actor: String, target: String, reason: Option<String>, duration: Duration, uuid: String) -> Option<String> {
        if !self.enabled.unwrap_or(false) 
        || self.exclude_actions.as_ref().unwrap_or(&vec![Actions::None]).contains(&Actions::Ban) {
                return None;
        }
        
        Some(format!("<:mesaBan:869663336625733634> <@{}> banned <@{}> expiring {}: `{}`. UUID: `{}`",
            actor, target,
            duration.to_discord_timestamp(), reason.unwrap_or("No reason provided".to_string()), uuid))
    }

    pub fn log_unban(&self, actor: String, target: String, reason: Option<String>) -> Option<String> {
        if !self.enabled.unwrap_or(false) 
        || self.exclude_actions.as_ref().unwrap_or(&vec![Actions::None]).contains(&Actions::Unban) {
                return None;
        }
        
        Some(format!("<:mesaUnban:869663336697069619> <@{}> unbanned <@{}>: `{}`",
            actor, target,
            reason.unwrap_or("No reason provided".to_string())))
    }

    pub fn log_message_delete(&self, msg: Reference<'_, twilight_model::id::Id<MessageMarker>, CachedMessage>) -> Option<String>  {
        if !self.enabled.unwrap_or(false) 
        || self.exclude_actions.as_ref().unwrap_or(&vec![Actions::None]).contains(&Actions::MessageDelete) {
                return None;
        }

        let mut log = format!("<:mesaMessageDelete:869663511977025586> Message by <@{}> deleted in channel <#{}> (`{}`): ",
        msg.author().get(), msg.channel_id().get(), msg.channel_id().get());
        
        if msg.content().len() > 0 {
            log += format!("`{}`", &msg.content()).as_str();
        }

        if msg.attachments().len() > 0 {
            log += "\n Attachments: ";
            for attachment in msg.attachments() {
                log += format!("`{}` ", attachment.url).as_str();
            }
        }

        Some(log)
    }

    pub fn log_message_edit(&self, msg: &MessageUpdate, before: String) -> Option<String> {
        if !self.enabled.unwrap_or(false) 
        || self.exclude_actions.as_ref().unwrap_or(&vec![Actions::None]).contains(&Actions::MessageEdit) {
                return None;
        }
        
        let msg_author = match &msg.author {
            Some(a) => a,
            None => return None
        };

        Some(format!("<:mesaMessageEdit:869663511834411059> Message by <@{}> edited in channel <#{}> (`{}`):\n**Before**:\n`{}`\n**After**:\n`{}`",
        msg_author.id.get(), msg.channel_id.get(), msg.channel_id.get(), before, msg.content.as_ref().unwrap_or(&"Unable to fetch message content".to_string())))
    }

}