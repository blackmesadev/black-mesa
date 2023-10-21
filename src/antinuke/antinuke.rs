use serde::{Deserialize, Serialize};
use twilight_model::{
    gateway::payload::incoming::{BanAdd, ChannelDelete, MemberRemove, MessageDelete, RoleDelete},
    guild::audit_log::{AuditLog, AuditLogEventType},
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
};

use crate::{
    config::Config,
    handlers::Handler,
    util::{duration::Duration, permissions},
};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum AntinukeAction {
    Ban,
    Kick,
    RemovePermission,
    #[default]
    None,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub enum AntinukeTrigger {
    MessageDelete,
    RoleDelete,
    ChannelDelete,
    MemberBan,
    MemberKick,
    #[default]
    None,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Trigger {
    pub trigger: AntinukeTrigger,
    pub count: u32,
    pub time: Duration,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Antinuke {
    pub enabled: Option<bool>,
    pub action: Option<AntinukeAction>,
    pub allow_bypass: Option<bool>,
    pub bypass_ids: Option<Vec<String>>,
    pub triggers: Vec<Trigger>,
}

pub trait AntinukeTriggerTrait {
    fn guild_id(&self) -> Option<Id<GuildMarker>>;

    fn id(&self) -> String;

    fn typ(&self) -> AntinukeTrigger;
}

impl AntinukeTriggerTrait for &MessageDelete {
    fn guild_id(&self) -> Option<Id<GuildMarker>> {
        self.guild_id
    }

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn typ(&self) -> AntinukeTrigger {
        AntinukeTrigger::MessageDelete
    }
}

impl AntinukeTriggerTrait for &RoleDelete {
    fn guild_id(&self) -> Option<Id<GuildMarker>> {
        Some(self.guild_id)
    }

    fn id(&self) -> String {
        self.role_id.to_string()
    }

    fn typ(&self) -> AntinukeTrigger {
        AntinukeTrigger::RoleDelete
    }
}

impl AntinukeTriggerTrait for &ChannelDelete {
    fn guild_id(&self) -> Option<Id<GuildMarker>> {
        self.guild_id
    }

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn typ(&self) -> AntinukeTrigger {
        AntinukeTrigger::ChannelDelete
    }
}

impl AntinukeTriggerTrait for &BanAdd {
    fn guild_id(&self) -> Option<Id<GuildMarker>> {
        Some(self.guild_id)
    }

    fn id(&self) -> String {
        self.user.id.to_string()
    }

    fn typ(&self) -> AntinukeTrigger {
        AntinukeTrigger::MemberBan
    }
}

impl AntinukeTriggerTrait for &MemberRemove {
    fn guild_id(&self) -> Option<Id<GuildMarker>> {
        Some(self.guild_id)
    }

    fn id(&self) -> String {
        self.user.id.to_string()
    }

    fn typ(&self) -> AntinukeTrigger {
        AntinukeTrigger::MemberKick
    }
}

impl Handler {
    pub async fn antinuke<T>(
        &self,
        conf: &Config,
        trigger: T,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        T: AntinukeTriggerTrait,
    {
        //if let Some(modules) = &conf.modules {
        //    if let Some(antinuke) = &modules.antinuke {
        //        if let Some(enabled) = antinuke.enabled {
        //            if !enabled {
        //                return Ok(());
        //            }
        //        }
        //
        //        if antinuke.triggers.is_empty() {
        //            return Ok(());
        //        }
        //
        //        if !antinuke.triggers.iter().any(|t| t.trigger == trigger.typ()) {
        //            return Ok(());
        //        }
        //
        //        let guild_id = trigger.guild_id();
        //        let id = trigger.id();
        //        let typ = trigger.typ();
        //
        //        let user_id = self.get_user_audit(trigger).await?;
        //
        //        if !permissions::check_permission(
        //            conf,
        //            None,
        //            user_id,
        //            permissions::PERMISSION_ANTINUKE_BYPASS,
        //        ) {
        //            return Ok(());
        //        }
        //
        //        // TODO: implement antinuke fully
        //    }
        //};

        Ok(())
    }

    async fn get_user_audit<T>(
        &self,
        trigger: T,
    ) -> Result<Id<UserMarker>, Box<dyn std::error::Error>>
    where
        T: AntinukeTriggerTrait,
    {
        let guild_id = match trigger.guild_id() {
            Some(id) => id,
            None => return Err("No guild id found in get_user_audit".into()),
        };

        let audit: AuditLog = self.rest.audit_log(guild_id).await?.model().await?;

        match trigger.typ() {
            AntinukeTrigger::MessageDelete => audit
                .entries
                .iter()
                .find(|entry| entry.action_type == AuditLogEventType::MessageDelete)
                .map(|entry| {
                    entry
                        .user_id
                        .ok_or("No user id found in get_user_audit".into())
                })
                .unwrap_or(Err(
                    "No matching audit log entry found in get_user_audit".into()
                )),
            AntinukeTrigger::RoleDelete => audit
                .entries
                .iter()
                .find(|entry| entry.action_type == AuditLogEventType::RoleDelete)
                .map(|entry| {
                    entry
                        .user_id
                        .ok_or("No user id found in get_user_audit".into())
                })
                .unwrap_or(Err(
                    "No matching audit log entry found in get_user_audit".into()
                )),
            AntinukeTrigger::ChannelDelete => audit
                .entries
                .iter()
                .find(|entry| entry.action_type == AuditLogEventType::ChannelDelete)
                .map(|entry| {
                    entry
                        .user_id
                        .ok_or("No user id found in get_user_audit".into())
                })
                .unwrap_or(Err(
                    "No matching audit log entry found in get_user_audit".into()
                )),
            AntinukeTrigger::MemberBan => audit
                .entries
                .iter()
                .find(|entry| entry.action_type == AuditLogEventType::MemberBanAdd)
                .map(|entry| {
                    entry
                        .user_id
                        .ok_or("No user id found in get_user_audit".into())
                })
                .unwrap_or(Err(
                    "No matching audit log entry found in get_user_audit".into()
                )),
            AntinukeTrigger::MemberKick => audit
                .entries
                .iter()
                .find(|entry| entry.action_type == AuditLogEventType::MemberKick)
                .map(|entry| {
                    entry
                        .user_id
                        .ok_or("No user id found in get_user_audit".into())
                })
                .unwrap_or(Err(
                    "No matching audit log entry found in get_user_audit".into()
                )),
            _ => Err("Invalid trigger type in get_user_audit".into()),
        }
    }
}
