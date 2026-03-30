//! Handles real-time events from the mesastream WebSocket stream.
//!
//! The primary use case is detecting mesastream restarts (`Connected` event)
//! and automatically recreating players for every guild where the bot is
//! currently in a voice channel.

use bm_lib::{
    discord::{Id, VoiceConnectionDetails},
    model::mesastream::{MesastreamEvent, Track},
};
use tokio::sync::mpsc;
use tracing::{info, instrument, warn};

use super::EventHandler;

impl EventHandler {
    /// Spawns a task that reads from the mesastream WS event channel and
    /// handles each event.  Runs until the channel is closed.
    pub fn spawn_mesastream_event_handler(
        self: &std::sync::Arc<Self>,
        mut rx: mpsc::Receiver<MesastreamEvent>,
    ) {
        let handler = std::sync::Arc::clone(self);
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                handler.handle_mesastream_event(event).await;
            }
            info!("mesastream event handler stopped");
        });
    }

    async fn handle_mesastream_event(&self, event: MesastreamEvent) {
        match event {
            MesastreamEvent::Connected => self.on_connected().await,
            MesastreamEvent::PlayerCreated {
                guild_id,
                player_id,
                restored_queue_len,
                restored_position_ms,
            } => {
                self.on_player_created(
                    guild_id,
                    player_id,
                    restored_queue_len,
                    restored_position_ms,
                )
                .await
            }
            MesastreamEvent::PlayerDestroyed {
                guild_id,
                player_id,
                was_stopped,
            } => {
                self.on_player_destroyed(guild_id, player_id, was_stopped)
                    .await
            }
            MesastreamEvent::TrackStarted {
                guild_id,
                player_id,
                track,
                position_ms,
            } => {
                self.on_track_started(guild_id, player_id, track, position_ms)
                    .await
            }
            MesastreamEvent::TrackEnded {
                guild_id,
                player_id,
                track_id,
            } => self.on_track_ended(guild_id, player_id, track_id).await,
            MesastreamEvent::Goodbye => self.on_goodbye().await,
            MesastreamEvent::VoiceDisconnected {
                guild_id,
                player_id,
                reason,
            } => {
                self.on_voice_disconnected(guild_id, player_id, reason)
                    .await
            }
        }
    }

    #[instrument(skip(self))]
    async fn on_connected(&self) {
        info!("mesastream ws: connected — reconciling voice players");
        self.reconcile_all_voice_players().await;
    }

    #[instrument(skip(self), fields(guild_id = %guild_id, player_id = %player_id))]
    async fn on_player_created(
        &self,
        guild_id: Id,
        player_id: Id,
        restored_queue_len: usize,
        restored_position_ms: u64,
    ) {
        info!(
            restored_queue_len,
            restored_position_ms, "mesastream player created"
        );
    }

    #[instrument(skip(self), fields(guild_id = %guild_id, player_id = %player_id))]
    async fn on_player_destroyed(&self, guild_id: Id, player_id: Id, was_stopped: bool) {
        if was_stopped {
            // Explicit stop — leave the voice channel and clear credentials.
            info!(
                was_stopped,
                "mesastream player stopped — leaving voice channel"
            );

            // Invalidate cached voice server credentials so the next
            // voice_join is forced to request a fresh token from Discord.
            // This prevents reusing a stale token when the voice gateway
            // was closed with 4006 (session no longer valid) before the
            // player was destroyed.
            self.clear_cached_voice_server_creds(&guild_id).await;

            // Disconnect from the voice channel now that the player is gone.
            if let Err(e) = self.voice_leave(&guild_id).await {
                warn!(error = %e, "failed to leave voice channel after player destroy");
            }
        } else {
            // Auto-destroy (queue drained) — stay in the voice channel so
            // the user can immediately `!play` another song without the bot
            // having to rejoin.  Keeping the voice connection alive also
            // avoids unnecessary gateway op-4 churn that can interfere with
            // other guilds' voice state when multiple players are active.
            info!(
                was_stopped,
                "mesastream player auto-destroyed (queue empty) — staying in voice channel"
            );
        }
    }

    #[instrument(skip(self, track), fields(guild_id = %guild_id, player_id = %player_id, track_id = %track.id))]
    async fn on_track_started(&self, guild_id: Id, player_id: Id, track: Track, position_ms: u64) {
        info!(
            artist = %track.metadata.artist,
            title = %track.metadata.title,
            position_ms,
            "track started"
        );
    }

    #[instrument(skip(self), fields(guild_id = %guild_id, player_id = %player_id, track_id = %track_id))]
    async fn on_track_ended(&self, guild_id: Id, player_id: Id, track_id: String) {
        info!("track ended");
    }

    #[instrument(skip(self))]
    async fn on_goodbye(&self) {
        info!("mesastream ws: server shutting down — will reconnect automatically");
    }

    #[instrument(skip(self), fields(guild_id = %guild_id, player_id = %player_id))]
    async fn on_voice_disconnected(&self, guild_id: Id, player_id: Id, reason: String) {
        warn!(reason = %reason, "mesastream voice disconnected — requesting fresh voice credentials");
        self.handle_voice_disconnected(&guild_id).await;
    }

    /// After a mesastream (re)connection, iterate all guilds where the bot is
    /// in a voice channel and ensure a player exists in mesastream.
    ///
    /// Delegates to `ensure_mesastream_player` which either updates an existing
    /// player's connection or creates a new one that restores queue & position
    /// from Redis.
    #[instrument(skip(self))]
    async fn reconcile_all_voice_players(&self) {
        let guild_ids: Vec<Id> = self.voice_guilds.lock().await.iter().copied().collect();
        if guild_ids.is_empty() {
            return;
        }

        info!(
            guild_count = guild_ids.len(),
            "reconciling mesastream players for active voice guilds"
        );

        let Some(&bot_id) = self.bot_id.get() else {
            warn!("bot_id not set — cannot reconcile voice players");
            return;
        };

        for guild_id in guild_ids {
            // Get the bot's current channel & session
            let channel_id = match self.get_voice_state_channel(&guild_id, &bot_id).await {
                Ok(Some(ch)) => ch,
                _ => continue,
            };

            let session_id = match self.get_voice_state_session(&guild_id, &bot_id).await {
                Ok(Some(s)) => s,
                _ => {
                    warn!(
                        guild_id = %guild_id,
                        "no cached session_id — skipping guild"
                    );
                    continue;
                }
            };

            // Get cached voice server credentials
            let creds = match self.get_cached_voice_server_creds(&guild_id).await {
                Ok(Some(c)) => c,
                _ => {
                    warn!(
                        guild_id = %guild_id,
                        "no cached voice server credentials — skipping guild"
                    );
                    continue;
                }
            };

            let details = VoiceConnectionDetails {
                guild_id,
                player_id: guild_id,
                channel_id,
                user_id: bot_id,
                session_id,
                token: creds.token,
                endpoint: creds.endpoint,
            };

            self.ensure_mesastream_player(&guild_id, details).await;
        }
    }

    /// Handle a voice disconnect from mesastream by re-sending a voice state
    /// update for the current channel.  This triggers Discord to issue a fresh
    /// `VOICE_SERVER_UPDATE` with new credentials without the bot visibly
    /// leaving.  The creds flow through `on_voice_server_update` →
    /// `reconcile_voice_connection` → `ensure_mesastream_player` →
    /// `update_connection`, giving mesastream the new token it needs.
    #[instrument(skip(self), fields(guild_id = %guild_id))]
    async fn handle_voice_disconnected(&self, guild_id: &Id) {
        let Some(&bot_id) = self.bot_id.get() else {
            warn!("bot_id not set — cannot handle voice disconnect");
            return;
        };

        // Find the channel the bot is currently in
        let channel_id = match self.get_voice_state_channel(guild_id, &bot_id).await {
            Ok(Some(ch)) => ch,
            Ok(None) => {
                warn!(guild_id = %guild_id, "bot not in a voice channel — nothing to reconnect");
                return;
            }
            Err(e) => {
                warn!(guild_id = %guild_id, error = ?e, "failed to look up bot voice state");
                return;
            }
        };

        // Clear stale cached credentials so the new VOICE_SERVER_UPDATE is used
        self.clear_cached_voice_server_creds(guild_id).await;

        // Re-send voice state update for the same channel — Discord responds
        // with fresh VOICE_STATE_UPDATE + VOICE_SERVER_UPDATE without the bot
        // visibly leaving the channel.
        if let Err(e) = self.send_voice_update(guild_id, Some(&channel_id)).await {
            warn!(guild_id = %guild_id, error = ?e, "failed to send voice state update for credential refresh");
        } else {
            info!(
                guild_id = %guild_id,
                channel_id = %channel_id,
                "voice state update sent — awaiting fresh credentials from Discord"
            );
        }
    }
}
