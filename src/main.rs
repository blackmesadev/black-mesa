use std::{env, error::Error, sync::Arc};
use expiry::action_expiry;
use futures_util::stream::StreamExt;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{cluster::{Cluster, ShardScheme}, Intents};
use twilight_http::Client as HttpClient;

mod handlers;
mod automod;
mod logging;
mod mongo;
mod util;
mod redis;
mod commands;
mod moderation;
mod expiry;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let token = env::var("DISCORD_TOKEN")?;

    let db = mongo::mongo::connect().await;

    let redis = redis::redis::connect().await;

    let scheme = ShardScheme::Range {
        from: 0,
        to: 0,
        total: 1,
    };

    let intents = Intents::all();

    let (cluster, mut events) = Cluster::builder(token.clone(), intents)
        .shard_scheme(scheme)
        .build()
        .await?;

    let cluster = Arc::new(cluster);
    let cluster_spawn = cluster.clone();
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let rest = Arc::new(HttpClient::new(token));

    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE | ResourceType::USER | ResourceType::MEMBER | ResourceType::GUILD | ResourceType::USER_CURRENT)
        .message_cache_size(10000)
        .build();

    let expiry_db = db.clone();
    let expiry_rest = rest.clone();

    tokio::spawn(async {
        action_expiry(expiry_db, expiry_rest).await
    });
    
    let handler = handlers::Handler {
        db,
        redis,
        rest,
        cache,
        last_process: std::time::SystemTime::now(),
    };

    // event listener
    while let Some((shard_id, event)) = events.next().await {
        match handler.handle_event(shard_id, &event).await {
            Ok(()) => {}
            Err(e) => println!("Error: {}", e),
        }
    }
    

    Ok(())
}
