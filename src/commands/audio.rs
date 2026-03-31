use bm_lib::{
    clients::MesastreamError,
    discord::{
        commands::{Arg, Args, Ctx},
        DiscordError, DiscordResult, EmbedBuilder, Id, VoiceConnectionDetails,
    },
    emojis::Emoji,
    model::{
        mesastream::{CreatePlayerRequest, PlayerStateSnapshot, Track},
        Config,
    },
    permissions::Permission,
};
use tracing::instrument;

use crate::{
    check_permission, commands::schema, handler::EventHandler, AUTHOR_COLON_THREE, SERVICE_NAME,
};

macro_rules! audio_call {
    ($self:expr, $ctx:expr, $expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                $self.send_audio_error($ctx.channel_id, e).await.ok();
                return Ok(());
            }
        }
    };
}

fn truncate(mut s: String, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        s = s.chars().take(max_chars - 3).collect();
        s.push_str("...");
    }
    s
}

fn format_position_ms(ms: u64) -> String {
    let total_secs = ms / 1000;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

fn parse_player_id(s: &str) -> DiscordResult<Id> {
    s.parse::<Id>()
        .map_err(|e| DiscordError::ParseError(format!("invalid player_id: {e}")))
}

fn details_to_request(details: VoiceConnectionDetails) -> CreatePlayerRequest {
    let gateway_url = details.voice_gateway_url();
    CreatePlayerRequest {
        guild_id: details.guild_id,
        player_id: details.guild_id,
        channel_id: details.channel_id,
        user_id: details.user_id,
        session_id: details.session_id,
        token: details.token,
        endpoint: details.endpoint,
        gateway_url,
    }
}

/// Checks whether a mesastream error indicates a stale Discord voice session
/// (e.g. expired token, session rotated, bot was kicked).
fn is_stale_session_error(error: &MesastreamError) -> bool {
    matches!(
        error,
        MesastreamError::Api { status: 503, message, .. }
            if {
                let msg = message.to_ascii_lowercase();
                msg.contains("4006")
                    || msg.contains("session no longer valid")
                    || msg.contains("stale voice token")
            }
    )
}

impl EventHandler {
    #[instrument(skip(self, config, ctx, args), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id, index))]
    async fn resolve_player_id(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
        index: usize,
    ) -> DiscordResult<Option<Id>> {
        if let Some(id) = args.get(index).and_then(|a| a.as_id()) {
            return Ok(Some(id));
        }
        if let Some(raw) = args.get_raw(index) {
            return Ok(Some(parse_player_id(raw)?));
        }

        if self
            .get_voice_state_channel(ctx.guild_id, &ctx.user.id)
            .await?
            .is_some()
        {
            return Ok(Some(*ctx.guild_id));
        }

        self.missing_parameters(config, ctx, args, schema::AUDIO_PLAYER_ID)
            .await?;
        Ok(None)
    }

    async fn parse_connection_request(
        &self,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
        start: usize,
    ) -> Option<CreatePlayerRequest> {
        let (channel_id, offset) = match args.get(start).and_then(|a| a.as_id()) {
            Some(channel_id) => (channel_id, 1usize),
            None => {
                let inferred = self
                    .get_voice_state_channel(ctx.guild_id, &ctx.user.id)
                    .await
                    .ok()
                    .flatten()?;
                (inferred, 0usize)
            }
        };

        let session_id = args.get_raw(start + offset)?.to_string();
        let token = args.get_raw(start + offset + 1)?.to_string();
        let endpoint = args.get_raw(start + offset + 2)?.to_string();
        let gateway_url = format!("wss://{}/?v=8&encoding=json", endpoint);

        Some(CreatePlayerRequest {
            guild_id: *ctx.guild_id,
            player_id: *ctx.guild_id,
            channel_id,
            user_id: ctx.user.id,
            session_id,
            token,
            endpoint,
            gateway_url,
        })
    }

    fn display_track_title(track: &Track) -> String {
        let raw_title = track.metadata.title.trim();
        if raw_title.is_empty()
            || raw_title.starts_with("http://")
            || raw_title.starts_with("https://")
        {
            return "Loading title...".to_string();
        }

        truncate(raw_title.to_string(), 64)
    }

    fn format_duration_ms(ms: u64) -> String {
        let total_secs = ms / 1000;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{mins}:{secs:02}")
    }

    fn format_track(track: &Track) -> String {
        format!(
            "[{}]({}) • {} • {}",
            Self::display_track_title(track),
            track.metadata.url,
            truncate(track.metadata.artist.clone(), 48),
            Self::format_duration_ms(track.metadata.duration_ms),
        )
    }

    fn snapshot_summary(s: &PlayerStateSnapshot) -> String {
        format!(
            "player_id=`{}` status=`{:?}` queue_len=`{}`",
            s.player_id,
            s.status,
            s.queue.len()
        )
    }

    #[instrument(skip(self, emoji, description), fields(channel_id = %channel_id))]
    async fn send_audio_embed(
        &self,
        channel_id: &Id,
        emoji: Emoji,
        title: &str,
        description: &str,
    ) -> DiscordResult<()> {
        let embed = EmbedBuilder::new()
            .title(format!("{} {}", emoji, title))
            .description(description)
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
            .build();

        self.rest
            .create_message_with_embed(channel_id, &[embed])
            .await?;
        Ok(())
    }

    #[instrument(skip(self, emoji, snapshot), fields(channel_id = %channel_id, player_id = %snapshot.player_id))]
    async fn send_snapshot_embed(
        &self,
        channel_id: &Id,
        emoji: Emoji,
        title: &str,
        snapshot: &PlayerStateSnapshot,
        queue_hint: Option<String>,
    ) -> DiscordResult<()> {
        let used_queue_as_now_playing =
            snapshot.current_track.is_none() && snapshot.queue.len() == 1;

        let now_playing = snapshot
            .current_track
            .as_ref()
            .or_else(|| {
                if used_queue_as_now_playing {
                    snapshot.queue.first()
                } else {
                    None
                }
            })
            .map(Self::format_track)
            .unwrap_or_else(|| "Nothing is playing right now.".to_string());

        let mut embed = EmbedBuilder::new()
            .title(format!("{} {}", emoji, title))
            .description("Audio updated")
            .field("Now Playing", now_playing, false)
            .field("Queue", format!("{} track(s)", snapshot.queue.len()), true)
            .field("Progress", format_position_ms(snapshot.position_ms), true)
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None);

        let next_up = if used_queue_as_now_playing {
            snapshot.queue.get(1)
        } else {
            snapshot.queue.first()
        };

        if let Some(next_up) = next_up {
            embed = embed.field("Up Next", Self::format_track(next_up), false);
        }

        if let Some(hint) = queue_hint {
            embed = embed.field("Queue Position", hint, false);
        }

        let embed = embed.build();
        self.rest
            .create_message_with_embed(channel_id, &[embed])
            .await?;
        Ok(())
    }

    #[instrument(skip(self, error), fields(channel_id = %channel_id))]
    async fn send_audio_error(&self, channel_id: &Id, error: MesastreamError) -> DiscordResult<()> {
        let description = match &error {
            MesastreamError::Api { message, .. } => message.clone(),
            other => other.to_string(),
        };
        tracing::warn!(error = ?error, "mesastream error");
        self.send_audio_embed(
            channel_id,
            Emoji::Cross,
            "Playback Error",
            &format!(
                "Something went wrong while handling your audio request.\n\n{}",
                description
            ),
        )
        .await
        .ok();
        Ok(())
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    async fn ensure_player(&self, ctx: &Ctx<'_>) -> DiscordResult<String> {
        let player_id = ctx.guild_id.to_string();

        match self.mesastream.get_player(ctx.guild_id).await {
            Ok(_) => {
                // Player exists — check if the bot is still in a voice channel.
                // If it was kicked/disconnected, rejoin and refresh mesastream's
                // connection so subsequent play/resume calls use fresh credentials.
                let bot_id = self
                    .bot_id
                    .get()
                    .copied()
                    .ok_or(DiscordError::NotConnected)?;
                let bot_in_vc = self
                    .get_voice_state_channel(ctx.guild_id, &bot_id)
                    .await?
                    .is_some();

                if !bot_in_vc {
                    tracing::info!(guild_id = %ctx.guild_id, "bot not in VC but player exists — rejoining");
                    let details = self.voice_join(ctx.guild_id, &ctx.user.id).await?;
                    let payload = details.into_bridge_payload();
                    self.mesastream
                        .update_connection(ctx.guild_id, &payload)
                        .await
                        .map_err(|e| DiscordError::ConnectionFailed(e.to_string()))?;
                }

                return Ok(player_id);
            }
            Err(MesastreamError::Api { status: 404, .. }) => {}
            Err(e) => return Err(DiscordError::ConnectionFailed(e.to_string())),
        }

        let channel_id = self
            .get_voice_state_channel(ctx.guild_id, &ctx.user.id)
            .await?;

        let Some(channel_id) = channel_id else {
            return Err(DiscordError::Voice(
                "You must be in a voice channel".to_string(),
            ));
        };

        let details = self.voice_join(ctx.guild_id, &ctx.user.id).await?;

        let payload = details_to_request(details);

        let snapshot = match self.mesastream.create_player(&payload).await {
            Ok(snapshot) => snapshot,
            Err(MesastreamError::Api { status: 409, .. }) => self
                .mesastream
                .get_player(ctx.guild_id)
                .await
                .map_err(|e| DiscordError::ConnectionFailed(e.to_string()))?,
            Err(e) => return Err(DiscordError::ConnectionFailed(e.to_string())),
        };

        tracing::info!(player_id, channel_id = %channel_id, "created player: {}", Self::snapshot_summary(&snapshot));
        Ok(player_id)
    }

    #[instrument(skip(self, config, ctx, args, permission), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id, endpoint))]
    async fn player_action(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
        permission: Permission,
        endpoint: &str,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, permission);

        let Some(player_id) = self.resolve_player_id(config, ctx, args, 0).await? else {
            return Ok(());
        };

        let snapshot = match endpoint {
            "pause" => audio_call!(self, ctx, self.mesastream.pause(&player_id).await),
            "resume" => audio_call!(self, ctx, self.mesastream.resume(&player_id).await),
            "skip" => audio_call!(self, ctx, self.mesastream.skip(&player_id).await),
            "stop" => audio_call!(self, ctx, self.mesastream.stop(&player_id).await),
            other => {
                tracing::error!(endpoint = other, "unknown player_action endpoint");
                return Err(DiscordError::Other(format!("unknown action: {other}")));
            }
        };

        let title = match endpoint {
            "pause" => "Paused",
            "resume" => "Resumed",
            "skip" => "Skipped",
            "stop" => "Stopped",
            _ => "Playback Updated",
        };

        self.send_snapshot_embed(ctx.channel_id, Emoji::Check, title, &snapshot, None)
            .await
    }

    #[instrument(skip(self, config, ctx, args), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn play_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MUSIC_PLAY);

        let player_id = match self.ensure_player(ctx).await {
            Ok(id) => id,
            Err(e) => {
                self.send_error(ctx.channel_id, e).await.ok();
                return Ok(());
            }
        };
        let player_id = match parse_player_id(&player_id) {
            Ok(id) => id,
            Err(e) => {
                self.send_error(ctx.channel_id, e).await.ok();
                return Ok(());
            }
        };

        if let Some(url) = args.get_raw(0) {
            audio_call!(self, ctx, self.mesastream.enqueue(&player_id, url).await);
        }

        let response = match self.mesastream.play(&player_id).await {
            Ok(snapshot) => snapshot,
            Err(e) if is_stale_session_error(&e) => {
                // Voice session is stale — rejoin to get fresh credentials and retry.
                tracing::info!(player_id = %player_id, "play returned stale session error, rejoining voice");
                let details = match self.voice_join(ctx.guild_id, &ctx.user.id).await {
                    Ok(d) => d,
                    Err(voice_err) => {
                        self.send_error(ctx.channel_id, voice_err).await.ok();
                        return Ok(());
                    }
                };
                let payload = details.into_bridge_payload();
                if let Err(e) = self
                    .mesastream
                    .update_connection(&player_id, &payload)
                    .await
                {
                    self.send_audio_error(ctx.channel_id, e).await.ok();
                    return Ok(());
                }
                audio_call!(self, ctx, self.mesastream.play(&player_id).await)
            }
            Err(e) => {
                self.send_audio_error(ctx.channel_id, e).await.ok();
                return Ok(());
            }
        };
        self.send_snapshot_embed(
            ctx.channel_id,
            Emoji::Check,
            "Playback Started",
            &response,
            None,
        )
        .await
    }

    #[instrument(skip(self, config, ctx, args), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn pause_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        self.player_action(config, ctx, args, Permission::MUSIC_PAUSE, "pause")
            .await
    }

    #[instrument(skip(self, config, ctx, args), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn resume_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        self.player_action(config, ctx, args, Permission::MUSIC_RESUME, "resume")
            .await
    }

    #[instrument(skip(self, config, ctx, args), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn skip_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        self.player_action(config, ctx, args, Permission::MUSIC_SKIP, "skip")
            .await
    }

    #[instrument(skip(self, config, ctx, args), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn stop_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        self.player_action(config, ctx, args, Permission::MUSIC_STOP, "stop")
            .await
    }

    #[instrument(skip(self, config, ctx, args), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn seek_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MUSIC_PLAY);

        let Some(position_ms) = (match args.get(0) {
            Some(Arg::Number(n)) => Some(*n as u64),
            _ => None,
        }) else {
            self.missing_parameters(config, ctx, args, schema::AUDIO_SEEK)
                .await?;
            return Ok(());
        };

        let Some(player_id) = self.resolve_player_id(config, ctx, args, 1).await? else {
            return Ok(());
        };

        let response = audio_call!(
            self,
            ctx,
            self.mesastream.seek(&player_id, position_ms).await
        );
        self.send_snapshot_embed(
            ctx.channel_id,
            Emoji::Check,
            "Position Updated",
            &response,
            None,
        )
        .await
    }

    #[instrument(skip(self, config, ctx, args), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn volume_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MUSIC_VOLUME);

        let Some(raw) = args.get_raw(0) else {
            self.missing_parameters(config, ctx, args, schema::AUDIO_VOLUME)
                .await?;
            return Ok(());
        };

        let Ok(volume) = raw.parse::<f32>() else {
            self.incorrect_parameter_type_embed(ctx, raw, "float (0.0 - 2.0)")
                .await?;
            return Ok(());
        };

        if !(0.0..=2.0).contains(&volume) {
            self.incorrect_parameter_type_embed(ctx, raw, "float (0.0 - 2.0)")
                .await?;
            return Ok(());
        }

        let Some(player_id) = self.resolve_player_id(config, ctx, args, 1).await? else {
            return Ok(());
        };

        let response = audio_call!(
            self,
            ctx,
            self.mesastream.set_volume(&player_id, volume).await
        );
        self.send_snapshot_embed(
            ctx.channel_id,
            Emoji::Check,
            "Volume Updated",
            &response,
            None,
        )
        .await
    }

    #[instrument(skip(self, config, ctx, args), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn current_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MUSIC_PLAY);

        let Some(player_id) = self.resolve_player_id(config, ctx, args, 0).await? else {
            return Ok(());
        };

        let track = audio_call!(
            self,
            ctx,
            self.mesastream.get_current_track(&player_id).await
        );

        let Some(current) = track else {
            return self
                .send_audio_embed(
                    ctx.channel_id,
                    Emoji::Cross,
                    "Now Playing",
                    "No track is currently playing.",
                )
                .await;
        };

        let description = format!(
            "{}\n{} / {} ({}  remaining)",
            Self::format_track(&current.track),
            current.position,
            current.duration,
            current.remaining,
        );

        self.send_audio_embed(ctx.channel_id, Emoji::Check, "Now Playing", &description)
            .await
    }

    #[instrument(skip(self, config, ctx, args), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn enqueue_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MUSIC_PLAY);

        let Some(url) = args.get_raw(0) else {
            self.missing_parameters(config, ctx, args, schema::AUDIO_ENQUEUE)
                .await?;
            return Ok(());
        };

        let player_id = match self.ensure_player(ctx).await {
            Ok(id) => id,
            Err(e) => {
                self.send_error(ctx.channel_id, e).await.ok();
                return Ok(());
            }
        };
        let player_id = match parse_player_id(&player_id) {
            Ok(id) => id,
            Err(e) => {
                self.send_error(ctx.channel_id, e).await.ok();
                return Ok(());
            }
        };

        let response = audio_call!(self, ctx, self.mesastream.enqueue(&player_id, url).await);
        let tracks_ahead = response.queue.len().saturating_sub(1);
        let queue_hint = if tracks_ahead == 0 {
            "Your track is up next.".to_string()
        } else {
            format!("Your track will play after {} song(s).", tracks_ahead)
        };

        self.send_snapshot_embed(
            ctx.channel_id,
            Emoji::Check,
            "Added to Queue",
            &response,
            Some(queue_hint),
        )
        .await
    }

    #[instrument(skip(self, config, ctx, args), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn queue_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MUSIC_PLAY);

        let Some(player_id) = self.resolve_player_id(config, ctx, args, 0).await? else {
            return Ok(());
        };

        let queue = audio_call!(self, ctx, self.mesastream.get_queue(&player_id).await);

        if queue.is_empty() {
            return self
                .send_audio_embed(
                    ctx.channel_id,
                    Emoji::Cross,
                    "Queue",
                    "The queue is currently empty.",
                )
                .await;
        }

        const LIMIT: usize = 20;
        let mut body = queue
            .iter()
            .take(LIMIT)
            .enumerate()
            .map(|(i, t)| format!("{}. {}", i + 1, Self::format_track(t)))
            .collect::<Vec<_>>()
            .join("\n");

        if queue.len() > LIMIT {
            body.push_str(&format!("\n... and {} more", queue.len() - LIMIT));
        }

        self.send_audio_embed(
            ctx.channel_id,
            Emoji::Check,
            "Queue",
            &format!("{} track(s) in queue\n\n{}", queue.len(), body),
        )
        .await
    }

    #[instrument(skip(self, config, ctx, args), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn clearqueue_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MUSIC_CLEAR);

        let Some(player_id) = self.resolve_player_id(config, ctx, args, 0).await? else {
            return Ok(());
        };

        let response = audio_call!(self, ctx, self.mesastream.clear_queue(&player_id).await);
        self.send_snapshot_embed(
            ctx.channel_id,
            Emoji::Check,
            "Queue Cleared",
            &response,
            None,
        )
        .await
    }

    #[instrument(skip(self, config, ctx, args), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn playlistsave_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MUSIC_PLAY);

        let Some(name) = args.get_raw(0) else {
            self.missing_parameters(config, ctx, args, schema::AUDIO_PLAYLIST)
                .await?;
            return Ok(());
        };

        let Some(player_id) = self.resolve_player_id(config, ctx, args, 1).await? else {
            return Ok(());
        };

        audio_call!(
            self,
            ctx,
            self.mesastream.save_playlist(&player_id, name).await
        );
        self.send_audio_embed(
            ctx.channel_id,
            Emoji::Check,
            "Playlist Saved",
            &format!("Saved the current queue as playlist `{}`.", name),
        )
        .await
    }

    #[instrument(skip(self, config, ctx, args), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id))]
    pub async fn playlistenqueue_command(
        &self,
        config: &Config,
        ctx: &Ctx<'_>,
        args: &Args<'_>,
    ) -> DiscordResult<()> {
        check_permission!(self, config, ctx, Permission::MUSIC_PLAY);

        let Some(name) = args.get_raw(0) else {
            self.missing_parameters(config, ctx, args, schema::AUDIO_PLAYLIST)
                .await?;
            return Ok(());
        };

        let Some(player_id) = self.resolve_player_id(config, ctx, args, 1).await? else {
            return Ok(());
        };

        let response = audio_call!(
            self,
            ctx,
            self.mesastream.enqueue_playlist(&player_id, name).await
        );
        self.send_snapshot_embed(
            ctx.channel_id,
            Emoji::Check,
            "Playlist Added",
            &response,
            None,
        )
        .await
    }
}
