use std::time::Duration;

use bm_lib::{
    clients::MesastreamError,
    discord::{
        DiscordError, DiscordResult, Id, VoiceConnectionDetails, VoiceServerUpdate,
        VoiceStateUpdate,
    },
};
use tracing::instrument;

use crate::handler::data::voice_server_creds_key;

use super::EventHandler;

const VOICE_JOIN_POLL_INTERVAL: Duration = Duration::from_millis(100);
const VOICE_JOIN_STATE_TTL: Duration = Duration::from_secs(120);
/// TTL for cached voice server credentials (token + endpoint).
/// Set generously so credentials survive the ~45-min Discord voice server rotation.
const VOICE_CREDS_TTL: Duration = Duration::from_secs(7200);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PendingVoiceJoinState {
    channel_id: Id,
    user_id: Id,
    session_id: Option<String>,
    token: Option<String>,
    endpoint: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct VoiceJoinDetailsState {
    guild_id: Id,
    player_id: Id,
    channel_id: Id,
    user_id: Id,
    session_id: String,
    token: String,
    endpoint: String,
}

impl VoiceJoinDetailsState {
    /// Converts the state into a VoiceConnectionDetails struct.
    fn into_connection_details(self) -> VoiceConnectionDetails {
        VoiceConnectionDetails {
            guild_id: self.guild_id,
            player_id: self.player_id,
            channel_id: self.channel_id,
            user_id: self.user_id,
            session_id: self.session_id,
            token: self.token,
            endpoint: self.endpoint,
        }
    }
}

/// Cached voice server credentials from the most recent VOICE_SERVER_UPDATE.
/// Allows `voice_join` to skip the leave/rejoin dance when the bot is already
/// in the target channel.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct CachedVoiceServerCreds {
    pub token: String,
    pub endpoint: String,
}

/// Generate Redis cache key for pending voice join state.
#[inline]
fn pending_voice_join_key(guild_id: &Id) -> String {
    format!("voice_join:pending:{}", guild_id)
}

/// Generate Redis cache key for completed voice join result.
#[inline]
fn voice_join_result_key(guild_id: &Id) -> String {
    format!("voice_join:result:{}", guild_id)
}

impl EventHandler {
    /// Send a VOICE_STATE_UPDATE gateway opcode (op 4) to join or leave a voice channel.
    #[instrument(skip(self), fields(guild_id = %guild_id))]
    pub async fn send_voice_update(
        &self,
        guild_id: &Id,
        channel_id: Option<&Id>,
    ) -> DiscordResult<()> {
        let guard = self.gateway.lock().await;
        let Some(sender) = guard.as_ref() else {
            return Err(DiscordError::NotConnected);
        };

        sender
            .update_voice_state(guild_id, channel_id, false, false)
            .await
    }

    /// Cache the most recent voice server credentials from VOICE_SERVER_UPDATE so
    /// `voice_join` can build connection details without leaving/rejoining.
    async fn cache_voice_server_creds(
        &self,
        guild_id: &Id,
        token: &str,
        endpoint: &str,
    ) -> DiscordResult<()> {
        let key = voice_server_creds_key(guild_id);
        let creds = CachedVoiceServerCreds {
            token: token.to_string(),
            endpoint: endpoint.to_string(),
        };
        self.cache
            .set(&key, &creds, Some(VOICE_CREDS_TTL))
            .await
            .map_err(DiscordError::from)
    }

    /// Retrieve cached voice server credentials for a guild.
    pub(crate) async fn get_cached_voice_server_creds(
        &self,
        guild_id: &Id,
    ) -> DiscordResult<Option<CachedVoiceServerCreds>> {
        let key = voice_server_creds_key(guild_id);
        self.cache
            .get::<String, CachedVoiceServerCreds>(&key)
            .await
            .map_err(DiscordError::from)
    }

    /// Invalidate cached voice server credentials for a guild.
    ///
    /// Called after a mesastream player is destroyed so the next `voice_join`
    /// is forced to do a full leave/rejoin and obtain a fresh token from
    /// Discord.  Without this, a stale token (e.g. from a prior 4006
    /// disconnect) would be reused, causing an immediate 4006 again.
    pub(crate) async fn clear_cached_voice_server_creds(&self, guild_id: &Id) {
        let key = voice_server_creds_key(guild_id);
        if let Err(e) = self.cache.delete(&key).await {
            tracing::warn!(
                guild_id = %guild_id,
                error = ?e,
                "failed to clear cached voice server credentials"
            );
        }
    }

    /// Persists pending voice join state to Redis with TTL.
    async fn persist_pending_voice_join(
        &self,
        guild_id: &Id,
        pending: &PendingVoiceJoinState,
    ) -> DiscordResult<()> {
        let pending_key = pending_voice_join_key(guild_id);
        self.cache
            .set(&pending_key, pending, Some(VOICE_JOIN_STATE_TTL))
            .await
            .map_err(DiscordError::from)
    }

    /// Attempts to complete a pending voice join by checking for required gateway events.
    /// Returns `true` when completion payload is written and pending state is removed.
    async fn try_complete_voice_join(
        &self,
        guild_id: &Id,
        pending: &PendingVoiceJoinState,
    ) -> DiscordResult<bool> {
        let pending_key = pending_voice_join_key(guild_id);
        match (&pending.session_id, &pending.token, &pending.endpoint) {
            (Some(session_id), Some(token), Some(endpoint)) => {
                let details = VoiceJoinDetailsState {
                    guild_id: *guild_id,
                    player_id: *guild_id,
                    channel_id: pending.channel_id,
                    user_id: pending.user_id,
                    session_id: session_id.clone(),
                    token: token.clone(),
                    endpoint: endpoint.clone(),
                };

                let result_key = voice_join_result_key(guild_id);
                self.cache
                    .set(&result_key, &details, Some(VOICE_JOIN_STATE_TTL))
                    .await
                    .map_err(DiscordError::from)?;
                self.cache
                    .delete(&pending_key)
                    .await
                    .map_err(DiscordError::from)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Poll Redis for completed voice join result until available.
    #[instrument(skip(self), fields(guild_id = %guild_id))]
    async fn await_voice_join_result(
        &self,
        guild_id: &Id,
    ) -> DiscordResult<VoiceConnectionDetails> {
        let result_key = voice_join_result_key(guild_id);
        let timeout = tokio::time::Duration::from_secs(10);
        let start = tokio::time::Instant::now();

        loop {
            if let Some(details) = self
                .cache
                .get::<String, VoiceJoinDetailsState>(&result_key)
                .await
                .map_err(DiscordError::from)?
            {
                self.cache
                    .delete(&result_key)
                    .await
                    .map_err(DiscordError::from)?;
                return Ok(details.into_connection_details());
            }

            if start.elapsed() > timeout {
                tracing::error!(guild_id = %guild_id, "voice join timed out waiting for gateway events");
                // Clean up pending state
                let pending_key = pending_voice_join_key(guild_id);
                self.cache.delete(&pending_key).await.ok();
                self.cache.delete(&result_key).await.ok();
                return Err(DiscordError::Voice(
                    "Voice join timed out - Discord gateway events did not arrive".to_string(),
                ));
            }

            tokio::time::sleep(VOICE_JOIN_POLL_INTERVAL).await;
        }
    }

    /// Joins the user's voice channel, creating pending join state and awaiting gateway events.
    /// Returns voice connection details once both VOICE_STATE_UPDATE and VOICE_SERVER_UPDATE arrive.
    ///
    /// When the bot is already in the target channel (e.g. mesastream player was
    /// auto-destroyed but the bot stayed in VC), we first try to return cached
    /// voice server credentials so the bot stays connected without leaving.
    /// Only falls back to the full leave/rejoin + op 4 dance when cached
    /// credentials are unavailable.
    #[tracing::instrument(skip(self), fields(guild_id = %guild_id, user_id = %user_id))]
    pub async fn voice_join(
        &self,
        guild_id: &Id,
        user_id: &Id,
    ) -> DiscordResult<VoiceConnectionDetails> {
        let Some(channel_id) = self.get_voice_state_channel(guild_id, user_id).await? else {
            return Err(DiscordError::Voice(
                "You must be in a voice channel".to_string(),
            ));
        };

        let bot_id = self
            .bot_id
            .get()
            .copied()
            .ok_or(DiscordError::NotConnected)?;

        // Fast-path: bot is already in the target channel - use cached
        // credentials from the most recent VOICE_SERVER_UPDATE instead of
        // leaving/rejoining (which is disruptive and wastes time).
        //
        // Guard: only trust cached credentials when `voice_guilds` confirms the
        // bot joined this guild during the *current* process lifetime.  After a
        // restart the in-memory set is empty, so we fall through to the full
        // leave/rejoin flow which obtains fresh tokens from Discord.
        let bot_in_guild_this_session = self.voice_guilds.lock().await.contains(guild_id);
        let bot_channel = self.get_voice_state_channel(guild_id, &bot_id).await?;
        if bot_channel.as_ref() == Some(&channel_id) {
            if bot_in_guild_this_session {
                if let (Ok(Some(session_id)), Ok(Some(creds))) = (
                    self.get_voice_state_session(guild_id, &bot_id).await,
                    self.get_cached_voice_server_creds(guild_id).await,
                ) {
                    tracing::info!(
                        guild_id = %guild_id,
                        channel_id = %channel_id,
                        endpoint = %creds.endpoint,
                        "bot already in target channel - using cached voice credentials"
                    );
                    return Ok(VoiceConnectionDetails {
                        guild_id: *guild_id,
                        player_id: *guild_id,
                        channel_id,
                        user_id: bot_id,
                        session_id,
                        token: creds.token,
                        endpoint: creds.endpoint,
                    });
                }
            } else {
                tracing::info!(
                    guild_id = %guild_id,
                    channel_id = %channel_id,
                    "bot appears to be in channel (stale Redis state) but hasn't joined this session - forcing fresh gateway events"
                );
            }
            // No cached credentials or stale session - fall through to
            // leave/rejoin so Discord emits fresh VOICE_STATE_UPDATE +
            // VOICE_SERVER_UPDATE.
            tracing::info!(
                guild_id = %guild_id,
                channel_id = %channel_id,
                "leaving channel to force fresh gateway events"
            );
            self.send_voice_update(guild_id, None).await?;
            tokio::time::sleep(Duration::from_millis(250)).await;
        }

        let pending_key = pending_voice_join_key(guild_id);
        let result_key = voice_join_result_key(guild_id);

        self.cache
            .delete(&result_key)
            .await
            .map_err(DiscordError::from)?;
        self.cache
            .delete(&pending_key)
            .await
            .map_err(DiscordError::from)?;

        self.persist_pending_voice_join(
            guild_id,
            &PendingVoiceJoinState {
                channel_id,
                user_id: bot_id,
                session_id: None,
                token: None,
                endpoint: None,
            },
        )
        .await?;

        if let Err(e) = self.send_voice_update(guild_id, Some(&channel_id)).await {
            self.cache
                .delete(&pending_key)
                .await
                .map_err(DiscordError::from)?;
            return Err(e);
        }

        self.await_voice_join_result(guild_id).await
    }

    /// Leaves the voice channel and cleans up any pending join state.
    #[tracing::instrument(skip(self), fields(guild_id = %guild_id))]
    pub async fn voice_leave(&self, guild_id: &Id) -> DiscordResult<()> {
        let pending_key = pending_voice_join_key(guild_id);
        let result_key = voice_join_result_key(guild_id);

        self.send_voice_update(guild_id, None).await?;
        self.cache
            .delete(&pending_key)
            .await
            .map_err(DiscordError::from)?;
        self.cache
            .delete(&result_key)
            .await
            .map_err(DiscordError::from)
    }

    /// Handles VOICE_STATE_UPDATE events from Discord gateway.
    /// Updates cached voice states and tries to complete pending voice joins.
    #[tracing::instrument(skip(self, vs), fields(guild_id = %vs.guild_id.unwrap_or(Id::new(0)), user_id = %vs.user_id))]
    pub async fn on_voice_state_update(&self, vs: &VoiceStateUpdate) -> DiscordResult<()> {
        let Some(guild_id) = vs.guild_id else {
            return Ok(());
        };

        self.set_voice_state_channel(&guild_id, &vs.user_id, vs.channel_id.as_ref())
            .await?;

        // Cache session_id for the bot user (needed for connection updates).
        // Clear it when the bot leaves a channel so stale sessions are not reused.
        if self
            .bot_id
            .get()
            .is_some_and(|&bot_id| bot_id == vs.user_id)
        {
            let session = if vs.channel_id.is_some() {
                Some(vs.session_id.as_str())
            } else {
                None
            };
            self.set_voice_state_session(&guild_id, &vs.user_id, session)
                .await?;

            // Track which guilds have the bot in a VC for mesastream reconnect.
            if vs.channel_id.is_some() {
                self.voice_guilds.lock().await.insert(guild_id);
            } else {
                self.voice_guilds.lock().await.remove(&guild_id);
            }
        }

        if !self
            .bot_id
            .get()
            .is_some_and(|&bot_id| bot_id == vs.user_id)
        {
            return Ok(());
        }

        let pending_key = pending_voice_join_key(&guild_id);
        if let Some(mut pending) = self
            .cache
            .get::<String, PendingVoiceJoinState>(&pending_key)
            .await
            .map_err(DiscordError::from)?
        {
            pending.session_id = Some(vs.session_id.clone());
            if !self.try_complete_voice_join(&guild_id, &pending).await? {
                self.persist_pending_voice_join(&guild_id, &pending).await?;
            }
        }

        Ok(())
    }

    /// Handles VOICE_SERVER_UPDATE gateway events.
    /// Caches the credentials, merges into pending join state, and reconciles with mesastream.
    #[tracing::instrument(skip(self, vs), fields(guild_id = %vs.guild_id))]
    pub async fn on_voice_server_update(&self, vs: &VoiceServerUpdate) -> DiscordResult<()> {
        let Some(endpoint) = &vs.endpoint else {
            return Ok(());
        };

        // Always cache the latest voice server credentials so `voice_join` can
        // reuse them without forcing a leave/rejoin.
        if let Err(e) = self
            .cache_voice_server_creds(&vs.guild_id, &vs.token, endpoint)
            .await
        {
            tracing::warn!(
                guild_id = %vs.guild_id,
                error = ?e,
                "failed to cache voice server credentials"
            );
        }

        let pending_key = pending_voice_join_key(&vs.guild_id);
        if let Some(mut pending) = self
            .cache
            .get::<String, PendingVoiceJoinState>(&pending_key)
            .await
            .map_err(DiscordError::from)?
        {
            pending.token = Some(vs.token.clone());
            pending.endpoint = Some(endpoint.clone());
            if !self.try_complete_voice_join(&vs.guild_id, &pending).await? {
                self.persist_pending_voice_join(&vs.guild_id, &pending)
                    .await?;
            }
        } else {
            // No pending join, but Discord sent VOICE_SERVER_UPDATE (happens during gateway reconnects
            // or ~45-min voice server rotations).
            // Check if bot is in a VC and either update or create the player in mesastream.
            self.reconcile_voice_connection(&vs.guild_id, &vs.token, endpoint)
                .await;
        }

        Ok(())
    }

    /// Reconcile the voice connection state with mesastream after a gateway reconnect
    /// or voice-server endpoint rotation.
    ///
    /// Called from `on_voice_server_update` when no pending voice join exists.
    /// Checks if the bot is in a VC with cached session info, then either:
    /// - Updates the existing player's connection in mesastream, or
    /// - Creates a new player if none exists (e.g. after mesastream restart).
    ///
    /// On failure, just logs a warning.  The next user command will trigger
    /// `ensure_player` which handles recovery via `voice_join`.  The bot stays
    /// in the voice channel at all times.
    #[tracing::instrument(skip(self), fields(guild_id = %guild_id))]
    async fn reconcile_voice_connection(&self, guild_id: &Id, token: &str, endpoint: &str) {
        let Some(&bot_id) = self.bot_id.get() else {
            return;
        };

        let bot_channel = match self.get_voice_state_channel(guild_id, &bot_id).await {
            Ok(Some(ch)) => ch,
            _ => return,
        };

        let session_id = match self.get_voice_state_session(guild_id, &bot_id).await {
            Ok(Some(s)) => s,
            _ => {
                tracing::warn!(
                    guild_id = %guild_id,
                    "VOICE_SERVER_UPDATE: bot in VC but no cached session_id"
                );
                return;
            }
        };

        let details = VoiceConnectionDetails {
            guild_id: *guild_id,
            player_id: *guild_id,
            channel_id: bot_channel,
            user_id: bot_id,
            session_id,
            token: token.to_string(),
            endpoint: endpoint.to_string(),
        };

        self.ensure_mesastream_player(guild_id, details).await;
    }

    /// Ensure mesastream has an active player for the given guild with the
    /// provided voice connection details.
    ///
    /// - If a player already exists → update its voice connection (hot-swap).
    /// - If no player (404) → create a new one (restores queue + position from Redis).
    ///
    /// Used by both `reconcile_voice_connection` (VOICE_SERVER_UPDATE / bot
    /// restart) and `reconcile_all_voice_players` (mesastream restart).
    #[tracing::instrument(skip(self, details), fields(guild_id = %details.guild_id))]
    pub(crate) async fn ensure_mesastream_player(
        &self,
        guild_id: &Id,
        details: VoiceConnectionDetails,
    ) {
        let endpoint = details.endpoint.clone();
        let channel_id = details.channel_id;

        match self.mesastream.get_player(guild_id).await {
            Ok(player) => {
                // If the player's voice transport is already healthy, don't
                // push credentials that may be stale from a delayed
                // VOICE_SERVER_UPDATE - the connection is fine as-is.
                if player.voice_connected {
                    tracing::debug!(
                        guild_id = %guild_id,
                        player_id = %player.player_id,
                        "mesastream player already voice-connected - skipping update_connection"
                    );
                    return;
                }

                // Player exists but voice is disconnected - push new credentials.
                tracing::info!(
                    guild_id = %guild_id,
                    player_id = %player.player_id,
                    endpoint = %endpoint,
                    "updating mesastream connection for disconnected player"
                );

                let payload = details.into_bridge_payload();
                if let Err(e) = self.mesastream.update_connection(guild_id, &payload).await {
                    tracing::warn!(
                        guild_id = %guild_id,
                        error = ?e,
                        "update_connection failed - next user command will retry"
                    );
                }
            }
            Err(MesastreamError::Api { status: 404, .. }) => {
                // No player in mesastream but bot is in a VC - create one so
                // playback can resume from the persisted queue.
                tracing::info!(
                    guild_id = %guild_id,
                    channel_id = %channel_id,
                    endpoint = %endpoint,
                    "bot in VC with no mesastream player, creating player"
                );

                let payload = details.into_bridge_payload();
                match self.mesastream.create_player(&payload).await {
                    Ok(snapshot) => {
                        tracing::info!(
                            guild_id = %guild_id,
                            player_id = %snapshot.player_id,
                            "created mesastream player via reconciliation"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            guild_id = %guild_id,
                            error = ?e,
                            "failed to create mesastream player via reconciliation"
                        );
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    guild_id = %guild_id,
                    error = ?e,
                    "failed to check mesastream player state"
                );
            }
        }
    }
}
