use std::{borrow::Cow, sync::Arc, time::Instant};

use crate::discord::{
    error::{DiscordError, DiscordResult},
    model::{ConnectionProperties, Identify, Payload},
    Message as DiscordMessage,
};
use futures_util::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{sync::RwLock, time::Duration};
use tokio::{
    net::TcpStream,
    time::{sleep, timeout, Interval},
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use super::{DiscordRestClient, Guild, GuildMemberUpdate, Hello, Intents, Ready, ShardConfig};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub enum Event {
    Ready(Ready),
    MessageCreate(DiscordMessage),
    MessageUpdate(DiscordMessage),
    GuildCreate(Guild),
    GuildUpdate(Guild),
    GuildMemberUpdate(GuildMemberUpdate),
}

pub struct DiscordWebsocket {
    token: Cow<'static, str>,
    socket: WsStream,
    heartbeat_interval: Interval,
    sequence: u64,
    shard_config: ShardConfig,
    session_id: Option<String>,
    heartbeat_ack_received: bool,
    last_heartbeat: Option<Instant>,
    ping: Arc<RwLock<Duration>>
}

impl DiscordWebsocket {
    pub async fn connect(
        rest: Arc<DiscordRestClient>,
        token: Cow<'static, str>,
        shard_config: ShardConfig,
        ping: Arc<RwLock<Duration>>
    ) -> DiscordResult<Self> {
        tracing::info!(shard = ?shard_config.shard_id, "Starting WebSocket connection");

        let gateway = rest.get_gateway().await?;
        let (socket, _) = connect_async(&gateway.url)
            .await
            .map_err(|e| DiscordError::WebSocket(e))?;

        let discord = DiscordWebsocket {
            token: token.clone(),
            socket,
            heartbeat_interval: tokio::time::interval(Duration::from_secs(45)),
            sequence: 0,
            shard_config,
            session_id: None,
            heartbeat_ack_received: true,
            last_heartbeat: None,
            ping,
        };

        Ok(discord)
    }

    pub async fn handle_initial_connection(&mut self) -> DiscordResult<()> {
        tracing::debug!("Receiving initial hello payload");
        let hello: Payload<Hello> = {
            let msg = self
                .socket
                .next()
                .await
                .ok_or_else(|| DiscordError::NotConnected)?
                .map_err(|e| DiscordError::WebSocket(e))?;
            let text = msg.into_text().map_err(|e| DiscordError::WebSocket(e))?;
            serde_json::from_str(&text)?
        };

        self.heartbeat_interval = tokio::time::interval(Duration::from_millis(
            hello
                .d
                .as_ref()
                .ok_or_else(|| DiscordError::InvalidPayload("Hello missing d field".to_string()))?
                .heartbeat_interval,
        ));

        self.send_heartbeat().await?;

        self.socket
            .next()
            .await
            .ok_or_else(|| DiscordError::NotConnected)?
            .map_err(|e| DiscordError::WebSocket(e))?;

        if let Some(session_id) = &self.session_id {
            self.resume(session_id.clone()).await?;
        } else {
            self.identify().await?;
        }

        Ok(())
    }

    pub async fn next_event(&mut self) -> DiscordResult<Option<Event>> {
        loop {
            if let Some(last_heartbeat) = self.last_heartbeat {
                if !self.heartbeat_ack_received
                    && last_heartbeat.elapsed() > Duration::from_secs(10)
                {
                    return Err(DiscordError::NotConnected);
                }
            }

            tokio::select! {
                _ = self.heartbeat_interval.tick() => {
                    if !self.heartbeat_ack_received {
                        return Err(DiscordError::NotConnected);
                    }
                    self.heartbeat_ack_received = false;
                    self.send_heartbeat().await?;
                    self.last_heartbeat = Some(Instant::now());
                },
                msg = timeout(Duration::from_secs(60), self.socket.next()) => {
                    match msg {
                        Ok(Some(Ok(msg))) => {
                            if let Some(event) = self.handle_message(msg).await? {
                                return Ok(Some(event));
                            }
                        },
                        Ok(Some(Err(e))) => return Err(DiscordError::WebSocket(e)),
                        Ok(None) => return Ok(None),
                        Err(_) => return Err(DiscordError::NotConnected),
                    }
                },
            }
        }
    }

    async fn handle_message(&mut self, msg: Message) -> DiscordResult<Option<Event>> {
        tracing::debug!("Received WebSocket message");

        let text = match msg {
            Message::Text(text) => text,
            other => {
                tracing::debug!(message_type = ?other, "Ignoring non-text message");
                return Ok(None);
            }
        };

        let payload: Payload<serde_json::Value> = serde_json::from_str(&text)?;

        match payload.op {
            0 => {
                if let Some(seq) = payload.s {
                    self.sequence = seq;
                }

                match payload.t.as_deref() {
                    Some("READY") => {
                        if let Some(d) = payload.d {
                            let ready: Ready = serde_json::from_value(d)?;
                            self.session_id = ready.session_id.clone();
                            return Ok(Some(Event::Ready(ready)));
                        }
                    }
                    Some("GUILD_CREATE") => {
                        if let Some(d) = payload.d {
                            let guild = serde_json::from_value(d)?;
                            return Ok(Some(Event::GuildCreate(guild)));
                        }
                    }
                    Some("GUILD_UPDATE") => {
                        if let Some(d) = payload.d {
                            let guild_update = serde_json::from_value(d)?;
                            return Ok(Some(Event::GuildUpdate(guild_update)));
                        }
                    }
                    Some("GUILD_MEMBER_UPDATE") => {
                        if let Some(d) = payload.d {
                            let member_update = serde_json::from_value(d)?;
                            return Ok(Some(Event::GuildMemberUpdate(member_update)));
                        }
                    }
                    Some("MESSAGE_CREATE") => {
                        if let Some(d) = payload.d {
                            let message = serde_json::from_value(d)?;
                            return Ok(Some(Event::MessageCreate(message)));
                        }
                    }
                    _ => (),
                }
            }
            11 => {
                self.heartbeat_ack_received = true;
                if let Some(last_hb) = self.last_heartbeat {
                    let ping = last_hb.elapsed();
                    *self.ping.write().await = ping;
                }
            }
            7 => {
                return Err(DiscordError::NotConnected);
            }
            9 => {
                self.session_id = None;
                sleep(Duration::from_secs(5)).await;
                return Err(DiscordError::NotConnected);
            }
            _ => (),
        }

        Ok(None)
    }

    pub async fn send_payload<T: DeserializeOwned + Serialize>(
        &mut self,
        payload: Payload<T>,
    ) -> DiscordResult<()> {
        let payload = serde_json::to_string(&payload).map_err(|e| DiscordError::Json(e))?;

        self.socket
            .send(Message::Text(payload))
            .await
            .map_err(|e| DiscordError::WebSocket(e))?;

        Ok(())
    }

    pub async fn send_heartbeat(&mut self) -> DiscordResult<()> {
        self.send_payload(Payload {
            op: 1,
            d: Some(self.sequence),
            s: None,
            t: None,
        })
        .await
    }

    async fn identify(&mut self) -> DiscordResult<()> {
        let identify = Identify {
            token: self.token.clone(),
            properties: ConnectionProperties::new(),
            intents: Intents::all(),
            shard: self.shard_config.to_array(),
        };

        self.send_payload(Payload {
            op: 2,
            d: Some(identify),
            s: None,
            t: None,
        })
        .await
    }

    async fn resume(&mut self, session_id: String) -> DiscordResult<()> {
        let resume = serde_json::json!({
            "token": self.token,
            "session_id": session_id,
            "seq": self.sequence
        });

        self.send_payload(Payload {
            op: 6,
            d: Some(resume),
            s: None,
            t: None,
        })
        .await
    }

    async fn set_speaking(&mut self, speaking: bool) -> DiscordResult<()> {
        self.send_payload(Payload {
            op: 5,
            d: Some(serde_json::json!({
                "speaking": speaking,
                "delay": 0,
                "ssrc": 1
            })),
            s: None,
            t: None,
        })
        .await
    }
}
