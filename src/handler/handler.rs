use std::time::Duration;
use tracing::{field, Span};

use bm_lib::discord::{
    commands::{Args, Ctx}, DiscordResult, DiscordWebsocket, Event, Guild, GuildMemberUpdate, Message, Ready, ShardConfig
};
use tracing::instrument;

use super::EventHandler;

impl EventHandler {
    pub async fn handle_event(&self, event: &Event) -> DiscordResult<()> {
        match event {
            Event::Ready(ready) => self.on_ready(ready).await?,
            Event::MessageCreate(message) => self.on_message_create(message).await?,
            Event::GuildCreate(guild) => self.on_guild_create(guild).await?,
            Event::GuildUpdate(guild) => self.on_guild_update(guild).await?,
            Event::GuildMemberUpdate(member_update) => {
                self.on_member_update(member_update).await?
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn listen(
        &self,
        token: &str,
        shard_config: ShardConfig,
    ) -> DiscordResult<()> {
        let mut reconnect_attempts = 0;
        const BASE_RECONNECT_DELAY: Duration = Duration::from_secs(1);
        const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(15);

        loop {
            tracing::info!(
                shard_id = shard_config.shard_id,
                attempt = reconnect_attempts + 1,
                "Establishing WebSocket connection"
            );

            match DiscordWebsocket::connect(self.rest.clone(), token, shard_config, self.ping_nanos.clone()).await {
                Ok(mut ws) => {
                    if let Err(e) = ws.handle_initial_connection().await {
                        tracing::error!(error = ?e, "Failed initial connection setup");
                        reconnect_attempts += 1;
                        let delay = std::cmp::min(
                            BASE_RECONNECT_DELAY * reconnect_attempts,
                            MAX_RECONNECT_DELAY
                        );
                        tracing::warn!("Retrying connection in {:?}", delay);
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    reconnect_attempts = 0;
                    tracing::info!("WebSocket connection established successfully");

                    loop {
                        match ws.next_event().await {
                            Ok(Some(event)) => {
                                if let Err(e) = self.handle_event(&event).await {
                                    tracing::error!(error = ?e, "Failed to handle event");
                                }
                            }
                            Ok(None) => {
                                tracing::debug!("Received empty event, continuing");
                                continue;
                            }
                            Err(e) => {
                                tracing::warn!(error = ?e, "WebSocket connection lost");
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Failed to establish WebSocket connection");
                    reconnect_attempts += 1;
                }
            }

            let delay = std::cmp::min(
                BASE_RECONNECT_DELAY * (2_u32.pow(reconnect_attempts.min(6)) as u32),
                MAX_RECONNECT_DELAY
            );
            tracing::warn!(
                "Reconnecting in {:?} (attempt {})",
                delay,
                reconnect_attempts + 1
            );
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

        tracing::info!("Connected as {} ({})", username, id);

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
        let start_time = std::time::Instant::now();
        let guild_id = match message.guild_id {
            Some(guild_id) => guild_id,
            None => return Ok(()),
        };

        let author = match message.author.as_ref() {
            Some(author) => author,
            None => return Ok(()),
        };

        if author.bot {
            return Ok(());
        }

        let roles = self.get_member_roles(&guild_id, &author.id).await?;

        self.set_user(&author.id, author).await?;

        let ctx = match Ctx::new(message, &roles) {
            Some(ctx) => ctx,
            None => return Ok(()),
        };

        let result = self.handle_message(&ctx).await;
        let elapsed = start_time.elapsed();

        match result {
            Ok(_) => {
                tracing::info!("Command completed in {:?}", elapsed);
                if elapsed > Duration::from_secs(5) {
                    tracing::warn!("Command took unusually long to execute: {:?}", elapsed);
                }
            }
            Err(e) => {
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
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, guild), fields(guild_id = %guild.id))]
    async fn on_guild_create(&self, guild: &Guild) -> DiscordResult<()> {
        self.set_guild(guild).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self, guild), fields(guild_id = %guild.id))]
    async fn on_guild_update(&self, guild: &Guild) -> DiscordResult<()> {
        self.set_guild(guild).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self, member_update), fields(guild_id = %member_update.guild_id))]
    async fn on_member_update(&self, member_update: &GuildMemberUpdate) -> DiscordResult<()> {
        self.set_member_roles(
            &member_update.guild_id,
            &member_update.user.id,
            &member_update.roles,
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self, ctx), fields(guild_id = %ctx.guild_id, user_id = %ctx.user.id, message_id = %ctx.message.id, channel_id = %ctx.channel_id))]
    pub async fn handle_message(&self, ctx: &Ctx<'_>) -> DiscordResult<()> {
        let mut config = self.get_config(ctx.guild_id).await?;

        if let Err(e) = self.handle_automod(&config, &ctx).await {
            tracing::error!(error = ?e, "Failed to handle automod");
            return Err(e);
        }

        if !ctx.message.content.starts_with(&config.prefix) {
            return Ok(());
        }

        let content = ctx.message.content.split_whitespace().collect::<Vec<_>>();
        if content.is_empty() {
            return Ok(());
        }

        let command = content[0].trim_start_matches(&config.prefix);
        let raw_args = &content[1..];

        let parsed_args = bm_lib::discord::commands::parse_args(raw_args.iter().copied()).await;

        let mut args = Args::new(&parsed_args, raw_args);

        if let Err(e) = self.handle_command(&mut config, &ctx, command, &mut args).await {
            tracing::error!(error = ?e, "Failed to handle command");
            return Err(e);
        }

        Ok(())
    }
}
