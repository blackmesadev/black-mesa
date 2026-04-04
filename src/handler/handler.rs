use std::sync::Arc;
use std::time::Duration;
use tracing::{field, Instrument, Span};

use bm_lib::discord::{
    commands::{Args, Ctx},
    Channel, DiscordError, DiscordResult, DiscordWebsocket, Event, Guild, GuildBanEvent,
    GuildMember, GuildMemberRemove, GuildRoleDeleteEvent, GuildRoleEvent, Id, InviteCreateEvent,
    InviteDeleteEvent, Message, MessageDelete, Ready, ResumeState, ShardConfig,
};
use bm_lib::model::logging::{DiscordLogEvent, LogEventType};
use tracing::instrument;

use super::EventHandler;

/// Calculates exponential backoff delay with a maximum cap.
fn reconnect_delay(base: Duration, max: Duration, attempts: u32) -> Duration {
    std::cmp::min(base * 2_u32.pow(attempts.min(6)), max)
}

impl EventHandler {
    /// Parses a command from message content, checking for prefix or bot mention.
    /// Returns the command name and remaining arguments string.
    fn parse_command<'a>(
        content: &'a str,
        prefix: &str,
        bot_mention: &str,
    ) -> Option<(&'a str, &'a str)> {
        let trim_len = if !bot_mention.is_empty() && content.starts_with(bot_mention) {
            bot_mention.len()
        } else if content.starts_with(prefix) {
            prefix.len()
        } else {
            return None;
        };

        let trimmed = content.get(trim_len..)?.trim_start();
        let (command, args) = trimmed
            .split_once(char::is_whitespace)
            .map(|(cmd, rest)| (cmd, rest.trim_start()))
            .unwrap_or((trimmed, ""));

        if command.is_empty() {
            return None;
        }

        Some((command, args))
    }

    /// Creates a tracing span for the given event with appropriate fields.
    fn event_span(event: &Event) -> Span {
        match event {
            Event::Ready(_) => tracing::info_span!(parent: None, "Ready"),
            Event::MessageCreate(message) => tracing::info_span!(
                parent: None,
                "MessageCreate",
                message_id = %message.id,
                channel_id = %message.channel_id
            ),
            Event::GuildCreate(guild) => {
                tracing::info_span!(parent: None, "GuildCreate", guild_id = %guild.id)
            }
            Event::GuildUpdate(guild) => {
                tracing::info_span!(parent: None, "GuildUpdate", guild_id = %guild.id)
            }
            Event::GuildMemberAdd(u) => tracing::info_span!(
                "GuildMemberAdd",
                guild_id = %u.guild_id, user_id = %u.user.id
            ),
            Event::GuildMemberUpdate(u) => tracing::info_span!(
                "GuildMemberUpdate",
                guild_id = %u.guild_id, user_id = %u.user.id
            ),
            Event::GuildMemberRemove(u) => tracing::info_span!(
                "GuildMemberRemove",
                guild_id = %u.guild_id, user_id = %u.user.id
            ),
            Event::VoiceStateUpdate(vs) => {
                let guild_id = vs.guild_id.unwrap_or(Id::new(0));
                tracing::info_span!(
                    parent: None, "VoiceStateUpdate",
                    guild_id = %guild_id,
                    user_id = %vs.user_id
                )
            }
            Event::VoiceServerUpdate(vs) => tracing::info_span!(
                parent: None, "VoiceServerUpdate", guild_id = %vs.guild_id
            ),
            Event::MessageUpdate(message) => tracing::info_span!(
                parent: None, "MessageUpdate",
                message_id = %message.id, channel_id = %message.channel_id
            ),
            Event::MessageDelete(d) => tracing::info_span!(
                parent: None, "MessageDelete",
                message_id = %d.id, channel_id = %d.channel_id
            ),
            Event::ChannelCreate(ch) => tracing::info_span!(
                parent: None, "ChannelCreate",
                channel_id = %ch.id, guild_id = ?ch.guild_id
            ),
            Event::ChannelUpdate(ch) => tracing::info_span!(
                parent: None, "ChannelUpdate",
                channel_id = %ch.id, guild_id = ?ch.guild_id
            ),
            Event::ChannelDelete(ch) => tracing::info_span!(
                parent: None, "ChannelDelete",
                channel_id = %ch.id, guild_id = ?ch.guild_id
            ),
            Event::GuildRoleCreate(r) => tracing::info_span!(
                parent: None, "GuildRoleCreate",
                guild_id = %r.guild_id, role_id = %r.role.id
            ),
            Event::GuildRoleUpdate(r) => tracing::info_span!(
                parent: None, "GuildRoleUpdate",
                guild_id = %r.guild_id, role_id = %r.role.id
            ),
            Event::GuildRoleDelete(r) => tracing::info_span!(
                parent: None, "GuildRoleDelete",
                guild_id = %r.guild_id, role_id = %r.role_id
            ),
            Event::GuildBanAdd(b) => tracing::info_span!(
                parent: None, "GuildBanAdd",
                guild_id = %b.guild_id, user_id = %b.user.id
            ),
            Event::GuildBanRemove(b) => tracing::info_span!(
                parent: None, "GuildBanRemove",
                guild_id = %b.guild_id, user_id = %b.user.id
            ),
            Event::InviteCreate(inv) => tracing::info_span!(
                parent: None, "InviteCreate",
                guild_id = ?inv.guild_id, code = %inv.code
            ),
            Event::InviteDelete(inv) => tracing::info_span!(
                parent: None, "InviteDelete",
                guild_id = ?inv.guild_id, code = %inv.code
            ),
        }
    }

    /// Dispatches a Discord gateway event to the appropriate handler method.
    pub async fn handle_event(&self, event: &Event) -> DiscordResult<()> {
        let span = Self::event_span(event);

        async {
            match event {
                Event::Ready(ready) => self.on_ready(ready).await?,
                Event::MessageCreate(message) => self.on_message_create(message).await?,
                Event::GuildCreate(guild) => self.on_guild_create(guild).await?,
                Event::GuildUpdate(guild) => self.on_guild_update(guild).await?,
                Event::GuildMemberAdd(member) => {
                    self.on_member_update(member).await?;
                    // Log member add event
                    let mut vars = std::collections::HashMap::new();
                    vars.insert("user_id".into(), member.user.id.to_string());
                    vars.insert("username".into(), member.user.username.to_string());
                    vars.insert("guild_id".into(), member.guild_id.to_string());
                    let _ = self
                        .log_event(
                            &member.guild_id,
                            &LogEventType::Discord(DiscordLogEvent::GuildMemberAdd),
                            vars,
                        )
                        .await;
                }
                Event::GuildMemberUpdate(member) => self.on_member_update(member).await?,
                Event::GuildMemberRemove(member) => self.on_member_remove(member).await?,
                Event::VoiceStateUpdate(vs) => self.on_voice_state_update(vs).await?,
                Event::VoiceServerUpdate(vs) => self.on_voice_server_update(vs).await?,
                Event::MessageUpdate(_) => {} // Not handled yet
                Event::MessageDelete(d) => self.on_message_delete(d).await?,
                Event::ChannelCreate(ch) => self.on_channel_create(ch).await?,
                Event::ChannelUpdate(ch) => self.on_channel_update(ch).await?,
                Event::ChannelDelete(ch) => self.on_channel_delete(ch).await?,
                Event::GuildRoleCreate(r) => self.on_guild_role_create(r).await?,
                Event::GuildRoleUpdate(r) => self.on_guild_role_update(r).await?,
                Event::GuildRoleDelete(r) => self.on_guild_role_delete(r).await?,
                Event::GuildBanAdd(b) => self.on_guild_ban_add(b).await?,
                Event::GuildBanRemove(b) => self.on_guild_ban_remove(b).await?,
                Event::InviteCreate(inv) => self.on_invite_create(inv).await?,
                Event::InviteDelete(inv) => self.on_invite_delete(inv).await?,
            }
            Ok(())
        }
        .instrument(span)
        .await
    }

    /// Main event loop that connects to Discord gateway and handles all events.
    /// Automatically reconnects with exponential backoff on connection loss.
    pub async fn listen(
        self: Arc<Self>,
        token: &str,
        shard_config: ShardConfig,
    ) -> DiscordResult<()> {
        const BASE_DELAY: Duration = Duration::from_secs(1);
        const MAX_DELAY: Duration = Duration::from_secs(15);

        let mut reconnect_attempts = 0u32;
        let mut resume_state: Option<ResumeState> = None;

        loop {
            tracing::info!(
                shard_id = shard_config.shard_id,
                attempt = reconnect_attempts + 1,
                resuming = resume_state.is_some(),
                "connecting to Discord gateway"
            );

            let mut ws = match DiscordWebsocket::connect(
                self.rest.clone(),
                token,
                shard_config,
                self.ping_nanos.clone(),
                resume_state.take(),
            )
            .await
            {
                Ok(ws) => ws,
                Err(e) => {
                    tracing::error!(error = ?e, "WebSocket connect failed");
                    reconnect_attempts += 1;
                    tokio::time::sleep(reconnect_delay(BASE_DELAY, MAX_DELAY, reconnect_attempts))
                        .await;
                    continue;
                }
            };

            if let Err(e) = ws.handle_initial_connection().await {
                tracing::error!(error = ?e, "initial connection setup failed");
                reconnect_attempts += 1;
                tokio::time::sleep(reconnect_delay(BASE_DELAY, MAX_DELAY, reconnect_attempts))
                    .await;
                continue;
            }

            // Hand off the sender to the struct
            match ws.gateway_sender() {
                Some(sender) => {
                    *self.gateway.lock().await = Some(sender);
                }
                None => {
                    tracing::error!("gateway_sender unavailable after handle_initial_connection");
                    reconnect_attempts += 1;
                    tokio::time::sleep(reconnect_delay(BASE_DELAY, MAX_DELAY, reconnect_attempts))
                        .await;
                    continue;
                }
            }

            reconnect_attempts = 0;
            tracing::info!("gateway connected");

            loop {
                match ws.next_event().await {
                    Ok(Some(event)) => {
                        // Voice events must be processed synchronously to avoid race conditions
                        // with voice join polling mechanism
                        match &event {
                            // Ready and voice events are processed inline (synchronously)
                            // to guarantee ordering: bot_id must be set before
                            // GUILD_CREATE tasks run, and voice events must be
                            // serial to avoid race conditions with the pending-join
                            // polling mechanism.
                            Event::Ready(_)
                            | Event::VoiceStateUpdate(_)
                            | Event::VoiceServerUpdate(_) => {
                                if let Err(e) = self.handle_event(&event).await {
                                    tracing::error!(error = ?e, "voice event handler failed");
                                }
                            }
                            _ => {
                                let handler = Arc::clone(&self);
                                tokio::spawn(async move {
                                    if let Err(e) = handler.handle_event(&event).await {
                                        tracing::error!(error = ?e, "event handler failed");
                                    }
                                });
                            }
                        }
                    }
                    Ok(None) => {
                        tracing::warn!("event channel closed (RX task exited), reconnecting");
                        // Try to resume - preserve voice sessions
                        resume_state = ws.resume_state();
                        break;
                    }
                    Err(ref e) => {
                        tracing::warn!(error = ?e, "gateway connection lost");
                        resume_state = match e {
                            // Discord asked us to reconnect → always try RESUME
                            DiscordError::Reconnect => ws.resume_state(),
                            // Session is still valid → try RESUME
                            DiscordError::InvalidSession(true) => ws.resume_state(),
                            // Session is dead → must do full IDENTIFY
                            DiscordError::InvalidSession(false) => None,
                            // Other errors → try RESUME if we have state
                            _ => ws.resume_state(),
                        };
                        break;
                    }
                }
            }

            // We are now disconnected - clean up and prepare for reconnect

            *self.gateway.lock().await = None;

            // If we have a resume state we can try quickly; otherwise back off.
            if resume_state.is_some() {
                reconnect_attempts = 0;
            } else {
                reconnect_attempts += 1;
            }
            let delay = reconnect_delay(BASE_DELAY, MAX_DELAY, reconnect_attempts);
            tracing::warn!(attempt = reconnect_attempts + 1, delay = ?delay, resuming = resume_state.is_some(), "reconnecting");
            tokio::time::sleep(delay).await;
        }
    }

    #[tracing::instrument(skip(self, ready), fields(username, user_id))]
    async fn on_ready(&self, ready: &Ready) -> DiscordResult<()> {
        let username = ready.user["username"]
            .as_str()
            .expect("username not found in ready payload");
        let id = ready.user["id"]
            .as_str()
            .expect("id not found in ready payload");

        Span::current()
            .record("username", &field::display(username))
            .record("user_id", &field::display(id));

        self.set_guild_count(ready.guilds.len()).await?;

        if let Ok(parsed_id) = id.parse::<u64>() {
            // Only set bot_id if not already set (Discord can send multiple READY events)
            if self.bot_id.get().is_none() {
                self.bot_id.set(Id::new(parsed_id)).ok();
            }
        }

        tracing::info!("connected as {} ({})", username, id);
        if self.bot_mention.get().is_none() {
            self.bot_mention.set(format!("<@{}> ", id)).ok();
        }

        Ok(())
    }

    #[tracing::instrument(
        skip(self, message),
        fields(
            message_id = %message.id,
            channel_id = %message.channel_id,
            guild_id = tracing::field::Empty,
            author_id = tracing::field::Empty
        )
    )]
    async fn on_message_create(&self, message: &Message) -> DiscordResult<()> {
        let Some(guild_id) = message.guild_id else {
            return Ok(()); // Ignore DM messages
        };

        let Some(author) = message.author.as_ref() else {
            tracing::warn!("Message {} has no author", message.id);
            return Ok(());
        };

        if author.bot {
            return Ok(()); // Ignore bot messages
        }

        let start_time = std::time::Instant::now();

        let roles = match self.get_member_roles(&guild_id, &author.id).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Failed to fetch member roles: {}", e);
                return Ok(());
            }
        };

        if let Err(e) = self.update_member_guilds_cache(&guild_id, &author.id).await {
            tracing::warn!(error = ?e, "Failed to update member guilds cache");
        }

        if let Err(e) = self.set_user(&author.id, author).await {
            tracing::warn!("Failed to cache user: {}", e);
            // Continue anyway - caching failure shouldn't block command processing
        }

        let Some(ctx) = Ctx::new(message, &roles) else {
            tracing::error!("Failed to create context for message {}", message.id);
            return Ok(());
        };

        let result = self.handle_message(&ctx).await;

        if let Err(e) = result {
            tracing::error!(
                error = ?e,
                message_id = ?ctx.message.id,
                channel_id = ?ctx.channel_id,
                guild_id = ?ctx.guild_id,
                user_id = ?ctx.user.id,
                "handle_message failed"
            );
            return Err(e);
        }

        let elapsed = start_time.elapsed();
        tracing::debug!(elapsed_ms = elapsed.as_millis(), "Message processed");
        if elapsed > Duration::from_secs(5) {
            tracing::warn!(elapsed = ?elapsed, "Message processing took unusually long");
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, guild), fields(guild_id = %guild.id, member_count = guild.approximate_member_count.unwrap_or(0)))]
    async fn on_guild_create(&self, guild: &Guild) -> DiscordResult<()> {
        let bot_id = self.bot_id.get().copied();

        // Seed voice state cache from GUILD_CREATE payload.
        for vs in &guild.voice_states {
            if let Some(channel_id) = vs.channel_id {
                self.set_voice_state_channel(&guild.id, &vs.user_id, Some(&channel_id))
                    .await?;
            }
            // Cache session_id for the bot so VOICE_SERVER_UPDATE can rebuild connections
            if bot_id.is_some_and(|bid| bid == vs.user_id) {
                self.set_voice_state_session(&guild.id, &vs.user_id, Some(&vs.session_id))
                    .await?;

                // Populate voice_guilds so mesastream reconnect reconciliation
                // knows the bot is in a VC even if only GUILD_CREATE has fired.
                if vs.channel_id.is_some() {
                    self.voice_guilds.lock().await.insert(guild.id);
                }

                tracing::debug!(
                    guild_id = %guild.id,
                    channel_id = ?vs.channel_id,
                    "cached bot voice session from GUILD_CREATE"
                );

                // After a gateway reconnect (IDENTIFY, not RESUME), the old voice
                // session is invalidated but Discord does NOT automatically send a
                // VOICE_SERVER_UPDATE.  Re-send op 4 to rejoin the channel so Discord
                // issues fresh VOICE_STATE_UPDATE + VOICE_SERVER_UPDATE, which
                // reconcile_voice_connection will forward to mesastream.
                if let Some(channel_id) = vs.channel_id {
                    tracing::info!(
                        guild_id = %guild.id,
                        channel_id = %channel_id,
                        "bot in voice channel on GUILD_CREATE - re-joining to refresh voice credentials"
                    );
                    if let Err(e) = self.send_voice_update(&guild.id, Some(&channel_id)).await {
                        tracing::warn!(
                            guild_id = %guild.id,
                            error = ?e,
                            "failed to send voice rejoin on GUILD_CREATE"
                        );
                    }
                }
            }
        }
        self.set_guild(guild).await?;

        // Populate channels cache
        self.get_channels(&guild.id).await?;

        self.increment_guild_count().await
    }

    #[tracing::instrument(skip(self, guild), fields(guild_id = %guild.id, member_count = guild.approximate_member_count.unwrap_or(0)))]
    async fn on_guild_update(&self, guild: &Guild) -> DiscordResult<()> {
        self.set_guild(guild).await?;

        // Log guild update event
        let mut vars = std::collections::HashMap::new();
        vars.insert("guild_id".into(), guild.id.to_string());
        vars.insert("guild_name".into(), guild.name.to_string());
        let _ = self
            .log_event(
                &guild.id,
                &LogEventType::Discord(DiscordLogEvent::GuildUpdate),
                vars,
            )
            .await;

        Ok(())
    }

    #[tracing::instrument(skip(self, member_update), fields(guild_id = %member_update.guild_id))]
    async fn on_member_update(&self, member_update: &GuildMember) -> DiscordResult<()> {
        self.set_member_roles(
            &member_update.guild_id,
            &member_update.user.id,
            &member_update.roles,
        )
        .await?;

        // Update member guilds reverse index - member is still in this guild
        if let Err(e) = self
            .update_member_guilds_cache(&member_update.guild_id, &member_update.user.id)
            .await
        {
            tracing::warn!(error = ?e, "Failed to update member guilds cache");
        }

        // Log guild member update event
        let mut vars = std::collections::HashMap::new();
        vars.insert("user_id".into(), member_update.user.id.to_string());
        vars.insert("username".into(), member_update.user.username.to_string());
        vars.insert("guild_id".into(), member_update.guild_id.to_string());
        vars.insert(
            "roles".into(),
            member_update
                .roles
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        );
        let _ = self
            .log_event(
                &member_update.guild_id,
                &LogEventType::Discord(DiscordLogEvent::GuildMemberUpdate),
                vars,
            )
            .await;

        Ok(())
    }

    #[tracing::instrument(skip(self, member_remove), fields(guild_id = %member_remove.guild_id, user_id = %member_remove.user.id))]
    async fn on_member_remove(&self, member_remove: &GuildMemberRemove) -> DiscordResult<()> {
        // Remove this guild from the user's member guilds cache
        if let Err(e) = self
            .remove_from_member_guilds_cache(&member_remove.guild_id, &member_remove.user.id)
            .await
        {
            tracing::warn!(error = ?e, "Failed to remove from member guilds cache");
        }

        // Log member remove event
        let mut vars = std::collections::HashMap::new();
        vars.insert("user_id".into(), member_remove.user.id.to_string());
        vars.insert("username".into(), member_remove.user.username.to_string());
        vars.insert("guild_id".into(), member_remove.guild_id.to_string());
        let _ = self
            .log_event(
                &member_remove.guild_id,
                &LogEventType::Discord(DiscordLogEvent::GuildMemberRemove),
                vars,
            )
            .await;

        Ok(())
    }

    #[tracing::instrument(skip(self, channel), fields(channel_id = %channel.id, guild_id = ?channel.guild_id))]
    async fn on_channel_create(&self, channel: &Channel) -> DiscordResult<()> {
        let Some(guild_id) = channel.guild_id else {
            return Ok(()); // Ignore DM channels
        };

        // Update channels cache
        if let Ok(mut channels) = self.get_channels(&guild_id).await {
            channels.push((*channel).clone());
            self.set_channels(&guild_id, &channels).await?;
        }

        // Log channel create event
        let mut vars = std::collections::HashMap::new();
        vars.insert("channel_id".into(), channel.id.to_string());
        vars.insert("channel_name".into(), channel.name.to_string());
        vars.insert("guild_id".into(), guild_id.to_string());
        let _ = self
            .log_event(
                &guild_id,
                &LogEventType::Discord(DiscordLogEvent::ChannelCreate),
                vars,
            )
            .await;

        Ok(())
    }

    #[tracing::instrument(skip(self, channel), fields(channel_id = %channel.id, guild_id = ?channel.guild_id))]
    async fn on_channel_update(&self, channel: &Channel) -> DiscordResult<()> {
        let Some(guild_id) = channel.guild_id else {
            return Ok(()); // Ignore DM channels
        };

        // Update channels cache
        if let Ok(mut channels) = self.get_channels(&guild_id).await {
            if let Some(existing) = channels.iter_mut().find(|c| c.id == channel.id) {
                *existing = (*channel).clone();
                self.set_channels(&guild_id, &channels).await?;
            }
        }

        // Log channel update event
        let mut vars = std::collections::HashMap::new();
        vars.insert("channel_id".into(), channel.id.to_string());
        vars.insert("channel_name".into(), channel.name.to_string());
        vars.insert("guild_id".into(), guild_id.to_string());
        let _ = self
            .log_event(
                &guild_id,
                &LogEventType::Discord(DiscordLogEvent::ChannelUpdate),
                vars,
            )
            .await;

        Ok(())
    }

    #[tracing::instrument(skip(self, channel), fields(channel_id = %channel.id, guild_id = ?channel.guild_id))]
    async fn on_channel_delete(&self, channel: &Channel) -> DiscordResult<()> {
        let Some(guild_id) = channel.guild_id else {
            return Ok(()); // Ignore DM channels
        };

        // Update channels cache
        if let Ok(mut channels) = self.get_channels(&guild_id).await {
            channels.retain(|c| c.id != channel.id);
            self.set_channels(&guild_id, &channels).await?;
        }

        // Log channel delete event
        let mut vars = std::collections::HashMap::new();
        vars.insert("channel_id".into(), channel.id.to_string());
        vars.insert("channel_name".into(), channel.name.to_string());
        vars.insert("guild_id".into(), guild_id.to_string());
        let _ = self
            .log_event(
                &guild_id,
                &LogEventType::Discord(DiscordLogEvent::ChannelDelete),
                vars,
            )
            .await;

        Ok(())
    }

    #[tracing::instrument(skip(self, message_delete), fields(message_id = %message_delete.id, channel_id = %message_delete.channel_id, guild_id = ?message_delete.guild_id))]
    async fn on_message_delete(&self, message_delete: &MessageDelete) -> DiscordResult<()> {
        let Some(guild_id) = message_delete.guild_id else {
            return Ok(()); // Ignore DM messages
        };

        let mut vars = std::collections::HashMap::new();
        vars.insert("message_id".into(), message_delete.id.to_string());
        vars.insert("channel_id".into(), message_delete.channel_id.to_string());
        vars.insert("guild_id".into(), guild_id.to_string());
        let _ = self
            .log_event(
                &guild_id,
                &LogEventType::Discord(DiscordLogEvent::MessageDelete),
                vars,
            )
            .await;

        Ok(())
    }

    #[tracing::instrument(skip(self, role_event), fields(guild_id = %role_event.guild_id, role_id = %role_event.role.id))]
    async fn on_guild_role_create(&self, role_event: &GuildRoleEvent) -> DiscordResult<()> {
        let mut vars = std::collections::HashMap::new();
        vars.insert("role_id".into(), role_event.role.id.to_string());
        vars.insert("role_name".into(), role_event.role.name.to_string());
        vars.insert("guild_id".into(), role_event.guild_id.to_string());
        let _ = self
            .log_event(
                &role_event.guild_id,
                &LogEventType::Discord(DiscordLogEvent::RoleCreate),
                vars,
            )
            .await;

        Ok(())
    }

    #[tracing::instrument(skip(self, role_event), fields(guild_id = %role_event.guild_id, role_id = %role_event.role.id))]
    async fn on_guild_role_update(&self, role_event: &GuildRoleEvent) -> DiscordResult<()> {
        let mut vars = std::collections::HashMap::new();
        vars.insert("role_id".into(), role_event.role.id.to_string());
        vars.insert("role_name".into(), role_event.role.name.to_string());
        vars.insert("guild_id".into(), role_event.guild_id.to_string());
        let _ = self
            .log_event(
                &role_event.guild_id,
                &LogEventType::Discord(DiscordLogEvent::RoleUpdate),
                vars,
            )
            .await;

        Ok(())
    }

    #[tracing::instrument(skip(self, role_delete), fields(guild_id = %role_delete.guild_id, role_id = %role_delete.role_id))]
    async fn on_guild_role_delete(&self, role_delete: &GuildRoleDeleteEvent) -> DiscordResult<()> {
        let mut vars = std::collections::HashMap::new();
        vars.insert("role_id".into(), role_delete.role_id.to_string());
        vars.insert("role_name".into(), String::new()); // name unavailable in delete event
        vars.insert("guild_id".into(), role_delete.guild_id.to_string());
        let _ = self
            .log_event(
                &role_delete.guild_id,
                &LogEventType::Discord(DiscordLogEvent::RoleDelete),
                vars,
            )
            .await;

        Ok(())
    }

    #[tracing::instrument(skip(self, ban_event), fields(guild_id = %ban_event.guild_id, user_id = %ban_event.user.id))]
    async fn on_guild_ban_add(&self, ban_event: &GuildBanEvent) -> DiscordResult<()> {
        let mut vars = std::collections::HashMap::new();
        vars.insert("user_id".into(), ban_event.user.id.to_string());
        vars.insert("username".into(), ban_event.user.username.to_string());
        vars.insert("guild_id".into(), ban_event.guild_id.to_string());
        let _ = self
            .log_event(
                &ban_event.guild_id,
                &LogEventType::Discord(DiscordLogEvent::GuildBanAdd),
                vars,
            )
            .await;

        Ok(())
    }

    #[tracing::instrument(skip(self, ban_event), fields(guild_id = %ban_event.guild_id, user_id = %ban_event.user.id))]
    async fn on_guild_ban_remove(&self, ban_event: &GuildBanEvent) -> DiscordResult<()> {
        let mut vars = std::collections::HashMap::new();
        vars.insert("user_id".into(), ban_event.user.id.to_string());
        vars.insert("username".into(), ban_event.user.username.to_string());
        vars.insert("guild_id".into(), ban_event.guild_id.to_string());
        let _ = self
            .log_event(
                &ban_event.guild_id,
                &LogEventType::Discord(DiscordLogEvent::GuildBanRemove),
                vars,
            )
            .await;

        Ok(())
    }

    #[tracing::instrument(skip(self, invite), fields(guild_id = ?invite.guild_id, code = %invite.code))]
    async fn on_invite_create(&self, invite: &InviteCreateEvent) -> DiscordResult<()> {
        let Some(guild_id) = invite.guild_id else {
            return Ok(()); // Ignore DM invites
        };

        let mut vars = std::collections::HashMap::new();
        vars.insert("channel_id".into(), invite.channel_id.to_string());
        vars.insert("code".into(), invite.code.to_string());
        vars.insert("guild_id".into(), guild_id.to_string());
        vars.insert(
            "inviter_id".into(),
            invite
                .inviter
                .as_ref()
                .map(|u| u.id.to_string())
                .unwrap_or_default(),
        );
        let _ = self
            .log_event(
                &guild_id,
                &LogEventType::Discord(DiscordLogEvent::InviteCreate),
                vars,
            )
            .await;

        Ok(())
    }

    #[tracing::instrument(skip(self, invite), fields(guild_id = ?invite.guild_id, code = %invite.code))]
    async fn on_invite_delete(&self, invite: &InviteDeleteEvent) -> DiscordResult<()> {
        let Some(guild_id) = invite.guild_id else {
            return Ok(()); // Ignore DM invites
        };

        let mut vars = std::collections::HashMap::new();
        vars.insert("channel_id".into(), invite.channel_id.to_string());
        vars.insert("code".into(), invite.code.to_string());
        vars.insert("guild_id".into(), guild_id.to_string());
        let _ = self
            .log_event(
                &guild_id,
                &LogEventType::Discord(DiscordLogEvent::InviteDelete),
                vars,
            )
            .await;

        Ok(())
    }

    /// Processes a message, running automod checks and dispatching commands.
    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id, message_id = %ctx.message.id, channel_id = %ctx.channel_id))]
    pub async fn handle_message(&self, ctx: &Ctx<'_>) -> DiscordResult<()> {
        let mut config = match self.get_config(ctx.guild_id).await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to load guild config: {}", e);
                self.send_error(&ctx.channel_id, e).await?;
                return Ok(());
            }
        };

        if config.automod_enabled {
            if let Err(e) = self.handle_automod(&config, ctx).await {
                tracing::error!("Automod processing failed: {}", e);
            }
        }

        let bot_mention = self.bot_mention.get().map(|s| s.as_str()).unwrap_or("");
        let Some((command_str, args_str)) =
            Self::parse_command(&ctx.message.content, &config.prefix, bot_mention)
        else {
            return Ok(());
        };

        let command = command_str.to_lowercase();
        let raw_args: Vec<&str> = args_str.split_whitespace().collect();
        let parsed_args = bm_lib::discord::commands::parse_args(raw_args.iter().copied()).await;
        let mut args = Args::new(&parsed_args, &raw_args);

        self.handle_command(&mut config, ctx, &command, &mut args)
            .await
    }
}
