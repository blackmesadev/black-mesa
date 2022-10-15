use std::{error::Error, time::SystemTime, sync::Arc, str::FromStr};

use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Event;
use twilight_http::Client as HttpClient;
use twilight_model::guild::audit_log::AuditLogChange;
use twilight_model::{channel::message::AllowedMentions, guild::audit_log::AuditLogEventType};
use twilight_model::id::Id;

use crate::{mongo::mongo::*, redis::redis::*, automod::AutomodMessage};

pub struct Handler {
    pub db: Database,
    pub redis: Redis,
    pub rest: Arc<HttpClient>,
    pub cache: InMemoryCache,
    pub last_process: SystemTime,

}

impl Handler {
    pub async fn handle_event(&self, shard_id: u64, event: &Event) -> Result<(), Box<dyn Error + Send + Sync>> {
        match event {
            Event::Ready(ready) => {
                println!("{} is connected!", ready.user.name);
                
            }

            Event::MessageCreate(msg) => {

                let conf = match self.db.get_guild(&match &msg.guild_id {
                    Some(id) => id.to_string(),
                    None => return Ok(())
                }).await {
                    Ok(conf) => match conf {
                        Some(conf) => conf,
                        None => return Ok(())
                    },
                    Err(_) => {
                        return Ok(())
                    }
                };

                let automod_msg = AutomodMessage {
                    attachments: Some(msg.attachments.clone()),
                    author: msg.author.clone(),
                    channel_id: msg.channel_id,
                    content: Some(msg.content.clone()),
                    edited_timestamp: msg.edited_timestamp,
                    embeds: Some(msg.embeds.clone()),
                    guild_id: msg.guild_id,
                    id: msg.id,
                    mention_roles: Some(msg.mention_roles.clone()),
                    mentions: Some(msg.mentions.clone()),
                    timestamp: Some(msg.timestamp),
                };

                self.automod(&conf, &automod_msg).await?;
                
                self.process_cmd(&conf, msg).await?;
            }

            Event::MessageUpdate(msg_update) => {
                let conf = match self.db.get_guild(&match &msg_update.guild_id {
                    Some(id) => id.to_string(),
                    None => return Ok(())
                }).await? {
                    Some(conf) => conf,
                    None => return Ok(())
                };

                let msg = AutomodMessage {
                    attachments: msg_update.attachments.clone(),
                    author: match msg_update.author.clone() {
                        Some(author) => author,
                        None => return Ok(())
                    },
                    channel_id: msg_update.channel_id,
                    content: msg_update.content.clone(),
                    edited_timestamp: msg_update.edited_timestamp,
                    embeds: msg_update.embeds.clone(),
                    guild_id: msg_update.guild_id,
                    id: msg_update.id,
                    mention_roles: msg_update.mention_roles.clone(),
                    mentions: msg_update.mentions.clone(),
                    timestamp: msg_update.timestamp,
                };

                self.automod(&conf, &msg).await?;

                let old = match self.cache.message(msg_update.id.clone()) {
                    Some(msg) => msg,
                    None => {
                        println!("Message not found in cache");
                        return Ok(());
                    }
                };

                let log = match conf.modules.logging.log_message_edit(msg_update, old.content().to_string()) {
                    Some(l) => l,
                    None => return Ok(())
                };

                let id = match conf.modules.logging.channel_id {
                    Some(id) => id,
                    None => return Ok(())
                };

                let channel_id = Id::from_str(&id)?;

                // Seeing as we only get the ID of the author of the message from the cache,
                // its easier to just mention them and just not ping them.
                let allowed_ment = AllowedMentions::builder().build();


                self.rest.create_message(channel_id)
                .content(log.as_str())?
                .allowed_mentions(Some(&allowed_ment))
                .exec().await?;

            }

            Event::MessageDelete(msg_delete) => {
                let conf = match self.db.get_guild(&match &msg_delete.guild_id {
                    Some(id) => id.to_string(),
                    None => return Ok(())
                }).await? {
                    Some(conf) => conf,
                    None => return Ok(())
                };

                let msg = match self.cache.message(msg_delete.id) {
                    Some(msg) => msg,
                    None => {
                        println!("Message not found in cache");
                        return Ok(());
                    }
                };

                let log = match conf.modules.logging.log_message_delete(msg) {
                    Some(l) => l,
                    None => return Ok(())
                };

                let id = match conf.modules.logging.channel_id {
                    Some(id) => id,
                    None => return Ok(())
                };

                let channel_id = Id::from_str(&id)?;

                // Seeing as we only get the ID of the author of the message from the cache,
                // its easier to just mention them and just not ping them.
                let allowed_ment = AllowedMentions::builder().build();


                self.rest.create_message(channel_id)
                .content(log.as_str())?
                .allowed_mentions(Some(&allowed_ment))
                .exec().await?;
            }
            Event::BanRemove(unban) => {
                self.db.delete_ban(&unban.guild_id.to_string(), &unban.user.id.to_string()).await?;
            }

            Event::MemberAdd(member) => {
                let guild_id = &member.guild_id.to_string();
                let user_id = &member.user.id.to_string();
    
                match self.db.get_mute(guild_id, user_id).await? {
                    Some(mute) => {
                        match mute.role_id {
                            Some(role_id) => {
                                let role_id_marker = Id::from_str(&role_id)?;
                                self.rest.add_guild_member_role(member.guild_id, member.user.id, role_id_marker).exec().await?;
                            },
                            None => {}
                        }
                    }
                    None => {}
                }
            }

            Event::MemberUpdate(update) => {
                let conf = match self.db.get_guild( &update.guild_id.to_string()).await? {
                    Some(conf) => conf,
                    None => return Ok(())
                };

                let muted_role = conf.modules.moderation.mute_role;

                let audit_log = self.rest.audit_log(update.guild_id)
                .action_type(AuditLogEventType::MemberRoleUpdate)
                .limit(1)?
                .exec()
                .await?
                .model()
                .await?
                .entries;

                let entry = match audit_log.first() {
                    Some(entry) => entry,
                    None => return Ok(())
                };

                match entry.target_id {
                    Some(target_id) => {
                        if target_id.to_string() != update.user.id.to_string() {
                            return Ok(())
                        }
                    },
                    None => {}
                }

                match entry.changes.get(0) {
                    Some(change) => {
                        match change {
                            AuditLogChange::RoleRemoved { new, .. } => {
                                // check if muted_role is in the new roles
                                for role in new {
                                    if role.id.to_string() == muted_role {
                                        self.db.delete_mute(&update.guild_id.to_string(), &update.user.id.to_string()).await?;
                                    }
                                }
                            },
                            _ => {
                                return Ok(())
                            }
                        }
                    }
                    None => {
                        return Ok(())
                    }
                }

            }

            Event::ShardConnected(_) => {
                println!("Connected to shard {}", shard_id);
            }
            _ => {}
        }

        self.cache.update(event);
    
        Ok(())
    }
}