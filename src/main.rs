mod automod;
mod commands;
mod config;
mod handler;
mod logging;
mod telemetry;
mod workers;

use std::sync::Arc;

use bm_lib::{
    cache::{Cache, RedisCache},
    clients::{MesastreamClient, MesastreamWsClient},
    db::Database,
    discord::{DiscordRestClient, Id, ShardConfig},
};
use config::Config;
use handler::EventHandler;

const SERVICE_NAME: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));

const AUTHOR_COLON_THREE: &str = "@dhopcs"; // hi :3
const GOAT_ID: Id = Id::new(206309860038410240); // hi :3

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let config = Config::from_env()?;

    let otel_provider = telemetry::init(
        SERVICE_NAME,
        &config.otlp_endpoint,
        config.otlp_auth.as_deref(),
        config.otlp_organization.as_deref(),
    );

    let rest = Arc::new(DiscordRestClient::new(&config.discord_token));

    let cache = Arc::new(Cache::new(
        RedisCache::new(config.redis_uri, config.redis_prefix).await?,
    ));
    let db = Arc::new(Database::connect(config.database_url).await?);

    db.migrate().await?;

    let worker = workers::Worker::new(20, Arc::clone(&db), Arc::clone(&rest));
    tokio::spawn(async move {
        worker.start_expiry().await;
    });

    let mesastream = Arc::new(MesastreamClient::new(
        config.mesastream_base_url.clone(),
        config.mesastream_token.clone(),
    ));

    let event_handler = Arc::new(EventHandler::new(
        rest.clone(),
        cache.clone(),
        db.clone(),
        mesastream.clone(),
    ));

    // Spawn the mesastream WebSocket event listener.
    // Converts the HTTP base URL to a WS URL: http(s)://host → ws(s)://host/ws
    let ws_url = config
        .mesastream_base_url
        .replace("https://", "wss://")
        .replace("http://", "ws://")
        + "/ws";
    let (ws_client, ws_rx) = MesastreamWsClient::new(ws_url);
    tokio::spawn(async move { ws_client.run().await });
    event_handler.spawn_mesastream_event_handler(ws_rx);

    let shard_config = ShardConfig::new(config.shard_id, config.num_shards);

    let listen_result = event_handler
        .listen(&config.discord_token, shard_config)
        .await;

    if let Err(e) = otel_provider.shutdown() {
        tracing::warn!("OpenTelemetry provider shutdown error: {}", e);
    }

    listen_result?;

    Ok(())
}
