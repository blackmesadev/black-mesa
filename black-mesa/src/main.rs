mod automod;
mod commands;
mod handler;
mod telemetry;
mod workers;

use std::{borrow::Cow, sync::Arc};

use bm_lib::{
    cache::{Cache, RedisCache},
    db::Database,
    discord::{DiscordRestClient, Id, ShardConfig},
};
use handler::EventHandler;

const SERVICE_NAME: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));

const AUTHOR_COLON_THREE: &str = "@dhopcs"; // hi :3
const GOAT_ID: Id = Id::new(206309860038410240); // hi :3

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let openobserve_endpoint = std::env::var("OPENOBSERVE_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:5080/api/black-mesa/v1/traces".to_string());

    let openobserve_email =
        std::env::var("OPENOBSERVE_EMAIL").expect("OPENOBSERVE_EMAIL not found");
    let openobserve_password =
        std::env::var("OPENOBSERVE_PASSWORD").expect("OPENOBSERVE_PASSWORD not found");

    telemetry::init_telemetry(
        &openobserve_endpoint,
        &openobserve_email,
        &openobserve_password,
    )?;

    let token: Cow<'static, str> =
        Cow::Owned(std::env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN not found"));

    let rest = Arc::new(DiscordRestClient::new(token.clone()));

    //let cache = MemoryCache::new();
    let cache = RedisCache::new(std::env::var("REDIS_URI").expect("REDIS_URI not found")).await?;

    let cache = Arc::new(Cache::new(cache));

    let db = Arc::new(Database::connect(std::env::var("MONGO_URI").expect("MONGO_URI not found"), "black-mesa").await?);

    let event_handler = EventHandler::new(
        rest.clone(),
        cache.clone(),
        db.clone(),
    );

    let shard_id: u32 = std::env::var("SHARD_ID")
        .unwrap_or("0".to_string())
        .parse()
        .expect("SHARD_ID is not a number");

    let num_shards: u32 = std::env::var("NUM_SHARDS")
        .unwrap_or("1".to_string())
        .parse()
        .expect("NUM_SHARDS is not a number");

    let shard_config = ShardConfig::new(shard_id, num_shards);

    let worker = workers::Worker::new(20, Arc::clone(&db), Arc::clone(&rest));
    tokio::spawn(async move {
        worker.start_expiry().await;
    });

    tokio::select! {
        res = event_handler.listen(token, shard_config) => {
            if let Err(e) = res {
                tracing::error!("Fatal error in main websocket connection: {}", e);
            }
        },
        _ = tokio::signal::ctrl_c() => {}
    };

    Ok(())
}
