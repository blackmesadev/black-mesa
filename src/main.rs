use expiry::action_expiry;
use futures_util::stream::StreamExt;
use std::{env, error::Error, sync::Arc};
use tracing::{error, info, warn};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Intents,
};
use twilight_http::Client as HttpClient;

mod automod;
mod commands;
mod expiry;
mod handlers;
mod logging;
mod misc;
mod moderation;
mod mongo;
mod redis;
mod util;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    tracing_subscriber::fmt::init();

    info!("Starting Black Mesa v{}", VERSION);

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
        .resource_types(
            ResourceType::MESSAGE
                | ResourceType::USER
                | ResourceType::MEMBER
                | ResourceType::GUILD
                | ResourceType::USER_CURRENT,
        )
        .message_cache_size(10000)
        .build();

    let expiry_db = db.clone();
    let expiry_rest = rest.clone();

    tokio::spawn(async { action_expiry(expiry_db, expiry_rest).await });

    let meter_redis = redis.clone();
    tokio::spawn(async { run_meter(meter_redis).await });

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
            Err(e) => error!("Error: {}", e),
        }
    }

    Ok(())
}

async fn run_meter(redis: redis::redis::Redis) {
    let mut meter = self_meter::Meter::new(std::time::Duration::from_secs(30)).unwrap();
    meter.track_current_thread("main");
    loop {
        let res = meter.scan();
        if let Err(err) = res {
            warn!("Meter Error: {}", err);
        }

        match meter.report() {
            Some(report) => {
                redis.set_memory_usage(report.memory_rss as i64).await;
            }
            None => {}
        }

        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}
