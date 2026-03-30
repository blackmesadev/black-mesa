use std::sync::Arc;
use std::time::Duration;
use tracing::{field, Instrument, Span};

use bm_lib::discord::{
    commands::{Args, Ctx},
    DiscordError, DiscordResult, DiscordWebsocket, Event, Guild, GuildMemberUpdate, Id, Message,
    Ready, ResumeState, ShardConfig,
};
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
            Event::GuildMemberUpdate(u) => tracing::info_span!(
                parent: None, "GuildMemberUpdate",
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
                Event::GuildMemberUpdate(u) => self.on_member_update(u).await?,
                Event::VoiceStateUpdate(vs) => self.on_voice_state_update(vs).await?,
                Event::VoiceServerUpdate(vs) => self.on_voice_server_update(vs).await?,
                Event::MessageUpdate(_) => {} // Not handled yet
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
                                let event_name = event.event_name();
                                tokio::spawn(async move {
                                    let handle = tokio::spawn(async move {
                                        if let Err(e) = handler.handle_event(&event).await {
                                            tracing::error!(error = ?e, "event handler failed");
                                        }
                                    });
                                    if let Err(e) = handle.await {
                                        if e.is_panic() {
                                            tracing::error!(
                                                event_type = %event_name,
                                                "Event handler panicked: {:?}",
                                                e
                                            );
                                        }
                                    }
                                });
                            }
                        }
                    }
                    Ok(None) => {
                        tracing::warn!("event channel closed (RX task exited), reconnecting");
                        // Try to resume — preserve voice sessions
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
                        "bot in voice channel on GUILD_CREATE — re-joining to refresh voice credentials"
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
        self.increment_guild_count().await
    }

    #[tracing::instrument(skip(self, guild), fields(guild_id = %guild.id, member_count = guild.approximate_member_count.unwrap_or(0)))]
    async fn on_guild_update(&self, guild: &Guild) -> DiscordResult<()> {
        self.set_guild(guild).await
    }

    #[tracing::instrument(skip(self, member_update), fields(guild_id = %member_update.guild_id))]
    async fn on_member_update(&self, member_update: &GuildMemberUpdate) -> DiscordResult<()> {
        self.set_member_roles(
            &member_update.guild_id,
            &member_update.user.id,
            &member_update.roles,
        )
        .await
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

        if let Err(e) = self.handle_automod(&config, ctx).await {
            tracing::error!("Automod processing failed: {}", e);
            // Don't send error for automod failures - let them pass silently
            // as automod is a background process
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
