mod automod;
mod commands;
mod config;
mod handler;
mod telemetry;
mod workers;

use std::{borrow::Cow, sync::Arc};

use bm_lib::{
    cache::{Cache, RedisCache},
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

    telemetry::init_telemetry(&config.otlp_endpoint, config.otlp_auth)?;

    let rest = Arc::new(DiscordRestClient::new(&config.discord_token));

    //let cache = MemoryCache::new();
    let cache = RedisCache::new(config.redis_uri).await?;

    let cache = Arc::new(Cache::new(cache));

    let db = Arc::new(Database::connect(config.mongo_uri, "black-mesa").await?);

    let event_handler = EventHandler::new(rest.clone(), cache.clone(), db.clone());

    let shard_config = ShardConfig::new(config.shard_id, config.num_shards);

    let worker = workers::Worker::new(20, Arc::clone(&db), Arc::clone(&rest));
    tokio::spawn(async move {
        worker.start_expiry().await;
    });

    event_handler.listen(&config.discord_token, shard_config).await?;

    Ok(())
}
