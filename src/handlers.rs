use std::{error::Error, str::FromStr, sync::Arc};

use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Event;
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::MessageFlags;
use twilight_model::gateway::payload::incoming::{
    BanAdd, BanRemove, ChannelDelete, InteractionCreate, MemberAdd, MemberRemove, MemberUpdate,
    MessageCreate, MessageDelete, MessageUpdate, Ready, RoleDelete,
};
use twilight_model::guild::audit_log::AuditLogChange;
use twilight_model::guild::audit_log::AuditLogEventType;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::Id;
use twilight_model::oauth::Application;
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{mongo::mongo::*, redis::redis::*};

#[derive(Debug, Clone)]
pub struct Handler {
    pub db: Database,
    pub redis: Redis,
    pub rest: Arc<HttpClient>,
    pub cache: Arc<InMemoryCache>,
    pub arc: Option<Arc<Handler>>,
}

impl Handler {
    #[tracing::instrument(skip(self))]
    pub async fn handle_event(
        &self,
        shard_id: u64,
        event: &Event,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match event {
            Event::Ready(ready) => match self.on_ready(shard_id, ready).await {
                Ok(_) => (),
                Err(e) => {
                    let span = tracing::error_span!("handle_event", shard_id = shard_id);
                    span.in_scope(|| {
                        tracing::error!(target = "on_ready", e);
                    });
                }
            },

            Event::MessageCreate(msg) => match self.message_create(shard_id, msg).await {
                Ok(_) => (),
                Err(e) => {
                    let span = tracing::error_span!("handle_event", shard_id = shard_id);
                    span.in_scope(|| {
                        tracing::error!(target = "message_create", e);
                    });
                }
            },

            Event::MessageUpdate(msg_update) => {
                match self.message_update(shard_id, msg_update).await {
                    Ok(_) => (),
                    Err(e) => {
                        let span = tracing::error_span!("handle_event", shard_id = shard_id);
                        span.in_scope(|| {
                            tracing::error!(target = "message_update", e);
                        });
                    }
                }
            }

            Event::MessageDelete(msg_delete) => {
                match self.message_delete(shard_id, msg_delete).await {
                    Ok(_) => (),
                    Err(e) => {
                        let span = tracing::error_span!("handle_event", shard_id = shard_id);
                        span.in_scope(|| {
                            tracing::error!(target = "message_delete", e);
                        });
                    }
                }
            }

            Event::InteractionCreate(interaction) => {
                match self.interaction_create(shard_id, interaction).await {
                    Ok(_) => (),
                    Err(e) => {
                        let span = tracing::error_span!("handle_event", shard_id = shard_id);
                        span.in_scope(|| {
                            tracing::error!(target = "interaction_create", e);
                        });
                    }
                }
            }

            Event::BanRemove(unban) => match self.ban_remove(shard_id, unban).await {
                Ok(_) => (),
                Err(e) => {
                    let span = tracing::error_span!("handle_event", shard_id = shard_id);
                    span.in_scope(|| {
                        tracing::error!(target = "ban_remove", e);
                    });
                }
            },

            Event::BanAdd(ban) => match self.ban_add(shard_id, ban).await {
                Ok(_) => (),
                Err(e) => {
                    let span = tracing::error_span!("handle_event", shard_id = shard_id);
                    span.in_scope(|| {
                        tracing::error!(target = "ban_add", e);
                    });
                }
            },

            Event::MemberAdd(member) => match self.member_add(shard_id, &member).await {
                Ok(_) => (),
                Err(e) => {
                    let span = tracing::error_span!("handle_event", shard_id = shard_id);
                    span.in_scope(|| {
                        tracing::error!(target = "member_add", e);
                    });
                }
            },

            Event::MemberUpdate(update) => match self.member_update(shard_id, update).await {
                Ok(_) => (),
                Err(e) => {
                    let span = tracing::error_span!("handle_event", shard_id = shard_id);
                    span.in_scope(|| {
                        tracing::error!(target = "member_update", e);
                    });
                }
            },

            Event::MemberRemove(member) => match self.member_removed(shard_id, member).await {
                Ok(_) => (),
                Err(e) => {
                    let span = tracing::error_span!("handle_event", shard_id = shard_id);
                    span.in_scope(|| {
                        tracing::error!(target = "member_remove", e);
                    });
                }
            },

            Event::RoleDelete(role) => match self.role_deleted(shard_id, role).await {
                Ok(_) => (),
                Err(e) => {
                    let span = tracing::error_span!("handle_event", shard_id = shard_id);
                    span.in_scope(|| {
                        tracing::error!(target = "role_delete", e);
                    });
                }
            },

            Event::ChannelDelete(channel) => match self.channel_deleted(shard_id, channel).await {
                Ok(_) => (),
                Err(e) => {
                    let span = tracing::error_span!("handle_event", shard_id = shard_id);
                    span.in_scope(|| {
                        tracing::error!(target = "channel_delete", e);
                    });
                }
            },

            _ => {}
        }

        self.cache.update(event);

        Ok(())
    }

    async fn on_ready(
        &self,
        shard_id: u64,
        ready: &Ready,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        tracing::info!(
            "Shard {} connected to gateway with {} guilds.",
            shard_id,
            ready.guilds.len()
        );

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn message_create(
        &self,
        _shard_id: u64,
        msg: &MessageCreate,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let conf = self
            .db
            .get_guild(&match &msg.guild_id {
                Some(id) => id.to_string(),
                None => return Ok(()),
            })
            .await?;

        if let Some(conf) = conf {
            match self.process_cmd(Some(&conf), msg).await {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!(target = "process_cmd", e);
                }
            }

            self.automod(&conf, &msg.0).await?;
        } else {
            self.process_cmd(None, msg).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn message_update(
        &self,
        _shard_id: u64,
        msg_update: &MessageUpdate,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match &msg_update.author {
            Some(author) => {
                if author.bot {
                    return Ok(());
                }
            }
            None => {}
        }
        let conf = match self
            .db
            .get_guild(&match &msg_update.guild_id {
                Some(id) => id.to_string(),
                None => return Ok(()),
            })
            .await?
        {
            Some(conf) => conf,
            None => return Ok(()),
        };

        self.automod(&conf, msg_update).await?;

        let old = match self.cache.message(msg_update.id) {
            Some(msg) => msg,
            None => {
                return Ok(());
            }
        };

        if let Some(modules) = conf.modules {
            if let Some(logging) = modules.logging {
                self.log_message_edit(&logging, msg_update, old.content().to_string())
                    .await;
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn message_delete(
        &self,
        _shard_id: u64,
        msg_delete: &MessageDelete,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let guild_id = match msg_delete.guild_id {
            Some(id) => id,
            None => return Ok(()),
        };
        let conf = match self.db.get_guild(&guild_id.to_string()).await? {
            Some(conf) => conf,
            None => return Ok(()),
        };

        let msg = match self.cache.message(msg_delete.id) {
            Some(msg) => msg,
            None => {
                return Ok(());
            }
        };

        if let Err(e) = self.antinuke(&conf, msg_delete).await {
            tracing::error!("Error in antinuke: {}", e);
        }

        let audit_log = self
            .rest
            .audit_log(guild_id)
            .action_type(AuditLogEventType::MessageDelete)
            .limit(1)?
            .await?
            .model()
            .await?
            .entries;

        let entry = match audit_log.first() {
            Some(entry) => entry,
            None => return Ok(()),
        };

        let deleted_by = match entry.target_id {
            Some(target_id) => {
                if target_id.to_string() != msg.author().to_string() {
                    None
                } else {
                    match entry.user_id {
                        Some(user_id) => Some(user_id.to_string()),
                        None => None,
                    }
                }
            }
            None => None,
        };

        if let Some(modules) = conf.modules {
            if let Some(logging) = modules.logging {
                self.log_message_delete(&logging, msg, deleted_by).await;
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn interaction_create(
        &self,
        _shard_id: u64,
        interaction: &InteractionCreate,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let conf = match self
            .db
            .get_guild(&match &interaction.guild_id {
                Some(id) => id.to_string(),
                None => return Ok(()),
            })
            .await?
        {
            Some(conf) => conf,
            None => return Ok(()),
        };

        let id = interaction.0.id.to_string();

        let id_parts = id.split("_").collect::<Vec<&str>>();

        let guild_id = match &interaction.guild_id {
            Some(id) => id.to_string(),
            None => return Ok(()),
        };

        let reason = Some(String::new()); // TODO: get reason for accept/deny

        if let Some(module_part) = id_parts.get(0) {
            match module_part {
                &"appeal" => {
                    if let Some(action) = id_parts.get(1) {
                        let uuid = id_parts.get(2).unwrap_or(&"");
                        let mut accepted = false;
                        match action {
                            &"accept" => {
                                accepted = true;
                                if let Err(e) = self
                                    .grant_appeal(
                                        Some(&conf),
                                        guild_id,
                                        uuid.to_string(),
                                        reason,
                                        true,
                                    )
                                    .await
                                {
                                    tracing::error!("Error in appeal_accept: {}", e);
                                    return Ok(());
                                }
                            }
                            &"deny" => {
                                if let Err(e) = self.deny_appeal(uuid.to_string(), reason).await {
                                    tracing::error!("Error in appeal_deny: {}", e);
                                    return Ok(());
                                }
                            }
                            _ => {}
                        }

                        // TODO: come back to this and do better interaction responses, including error handling

                        let application: Application =
                            self.rest.current_user_application().await?.model().await?;
                        let interactions = self.rest.interaction(application.id);

                        let interaction_data = InteractionResponseDataBuilder::new()
                            .content(format!(
                                "The appeal has been successfully `{}`.",
                                if accepted { "accepted" } else { "denied" }
                            ))
                            .flags(MessageFlags::EPHEMERAL)
                            .build();

                        let response = InteractionResponse {
                            kind: InteractionResponseType::ChannelMessageWithSource,
                            data: Some(interaction_data),
                        };

                        interactions
                            .create_response(interaction.id, &interaction.token, &response)
                            .await?;
                    }
                }
                &&_ => {}
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn ban_remove(
        &self,
        _shard_id: u64,
        unban: &BanRemove,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.db
            .delete_ban(&unban.guild_id.to_string(), &unban.user.id.to_string())
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn ban_add(
        &self,
        _shard_id: u64,
        ban: &BanAdd,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let conf = match self.db.get_guild(&ban.guild_id.to_string()).await? {
            Some(conf) => conf,
            None => return Ok(()),
        };

        if let Err(e) = self.antinuke(&conf, ban).await {
            tracing::error!("Error in antinuke: {}", e);
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn member_add(
        &self,
        _shard_id: u64,
        member: &MemberAdd,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let guild_id = &member.guild_id.to_string();
        let user_id = &member.user.id.to_string();

        match self.db.get_mute(guild_id, user_id).await? {
            Some(mute) => match mute.role_id {
                Some(role_id) => {
                    let role_id_marker = Id::from_str(&role_id)?;
                    self.rest
                        .add_guild_member_role(member.guild_id, member.user.id, role_id_marker)
                        .await?;
                    Ok(())
                }
                None => Ok(()),
            },
            None => Ok(()),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn member_update(
        &self,
        _shard_id: u64,
        update: &MemberUpdate,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conf = match self.db.get_guild(&update.guild_id.to_string()).await? {
            Some(conf) => conf,
            None => return Ok(()),
        };

        let audit_log = self
            .rest
            .audit_log(update.guild_id)
            .action_type(AuditLogEventType::MemberRoleUpdate)
            .limit(1)?
            .await?
            .model()
            .await?
            .entries;

        let entry = match audit_log.first() {
            Some(entry) => entry,
            None => return Ok(()),
        };

        match entry.target_id {
            Some(target_id) => {
                if target_id.to_string() != update.user.id.to_string() {
                    return Ok(());
                }
            }
            None => {}
        }

        match entry.changes.get(0) {
            Some(change) => match change {
                AuditLogChange::RoleRemoved { new, .. } => {
                    if let Some(modules) = conf.modules {
                        let muted_role = match modules.moderation {
                            Some(moderation) => moderation.mute_role,
                            None => return Ok(()),
                        };
                        for role in new {
                            if role.id.to_string() == muted_role {
                                self.db
                                    .delete_mute(
                                        &update.guild_id.to_string(),
                                        &update.user.id.to_string(),
                                    )
                                    .await?;
                            }
                        }
                    }
                }
                _ => return Ok(()),
            },
            None => return Ok(()),
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn member_removed(
        &self,
        _shard_id: u64,
        member: &MemberRemove,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conf = match self.db.get_guild(&member.guild_id.to_string()).await? {
            Some(conf) => conf,
            None => return Ok(()),
        };

        if let Err(e) = self.antinuke(&conf, member).await {
            tracing::error!("Error in antinuke: {}", e);
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn role_deleted(
        &self,
        _shard_id: u64,
        role_delete: &RoleDelete,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let conf = match self.db.get_guild(&role_delete.guild_id.to_string()).await? {
            Some(conf) => conf,
            None => return Ok(()),
        };

        if let Err(e) = self.antinuke(&conf, role_delete).await {
            tracing::error!("Error in antinuke: {}", e);
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn channel_deleted(
        &self,
        _shard_id: u64,
        channel_deleted: &ChannelDelete,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let guild_id = match channel_deleted.guild_id {
            Some(id) => id,
            None => return Ok(()),
        };
        let conf = match self.db.get_guild(&guild_id.to_string()).await? {
            Some(conf) => conf,
            None => return Ok(()),
        };

        if let Err(e) = self.antinuke(&conf, channel_deleted).await {
            tracing::error!("Error in antinuke: {}", e);
        }

        Ok(())
    }
}
