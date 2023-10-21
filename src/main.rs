use std::{env, error::Error, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;

mod administration;
mod antinuke;
mod antiraid;
mod api;
mod appeals;
mod automod;
mod commands;
mod config;
mod expiry;
mod handlers;
mod logging;
mod moderation;
mod mongo;
mod redis;
mod util;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let time_start = std::time::Instant::now();

    tracing_subscriber::fmt::init();

    let _g = tracing_axiom::builder()
        .with_dataset("black-mesa")
        .with_service_name(format!("Black Mesa v{}", VERSION))
        .with_token(env::var("AXIOM_TOKEN").unwrap_or_default())
        .init();

    tracing::info!("Starting Black Mesa v{}", VERSION);

    let token = env::var("DISCORD_TOKEN").unwrap_or_else(|_| {
        tracing::error!(
            "You must set the DISCORD_TOKEN environment variable before running the bot"
        );
        std::process::exit(1); // Exit the program with an error code.
    });

    let db = mongo::mongo::connect().await;

    let redis = redis::redis::connect().await;

    let intents = Intents::all();

    // BASIC SHARDING, THIS WILL BE REPLACED WITH A MORE ADVANCED SHARDING SYSTEM IN THE FUTURE

    let mut shards = Vec::new();

    let shard = Shard::new(ShardId::ONE, token.clone(), intents);

    shards.push(shard);

    let rest = Arc::new(HttpClient::new(token));

    let cache = Arc::new(
        InMemoryCache::builder()
            .resource_types(
                ResourceType::MESSAGE
                    | ResourceType::USER
                    | ResourceType::MEMBER
                    | ResourceType::GUILD
                    | ResourceType::USER_CURRENT,
            )
            .message_cache_size(1000)
            .build(),
    );

    let mut handler = handlers::Handler {
        db: db.clone(),
        redis: redis.clone(),
        rest: rest.clone(),
        cache: cache.clone(),
        arc: None,
    };

    // doing this is super ugly, i'll find a better way to do this later
    handler.arc = Some(Arc::new(handler.clone()));

    // Start background tasks to run concurrently with the bot
    tokio::spawn(expiry::action_expiry(db.clone(), rest.clone()));
    tokio::spawn(run_meter(redis.clone()));
    if std::env::var("INTERNAL_API_KEY").is_ok()
        && std::env::var("ENABLE_INTERNAL_API").unwrap_or("false".to_string()) == "true"
    {
        tokio::spawn(api::start_api(handler.arc.clone().unwrap()));
    }

    // for each shard, spawn a new task to handle events from that shard

    let mut thread_handles = Vec::new();

    for mut shard in shards {
        let handler = handler.arc.clone().unwrap();
        let thread = tokio::spawn(async move {
            let shard_id = shard.id().number();
            loop {
                let event = match shard.next_event().await {
                    Ok(event) => event,
                    Err(err) => {
                        tracing::warn!(?err, "error receiving event on shard {}", shard.id());

                        if err.is_fatal() {
                            break;
                        }

                        continue;
                    }
                };
                handler.cache.update(&event);

                tokio::spawn({
                    let handler = handler.clone();
                    async move {
                        if let Err(why) = handler.handle_event(shard_id, &event).await {
                            tracing::error!("Error handling event: {:?}", why);
                        }
                    }
                });
            }
        });
        thread_handles.push(thread);
    }

    tracing::info!(
        "Black Mesa is has finished initializing in {}ms",
        time_start.elapsed().as_millis()
    );

    tokio::join!(async {
        for handle in thread_handles {
            handle.await.unwrap();
        }
    });

    Ok(())
}

async fn run_meter(redis: redis::redis::Redis) {
    let mut meter = self_meter::Meter::new(std::time::Duration::from_secs(30)).unwrap();
    meter.track_current_thread("main");
    loop {
        let res = meter.scan();
        if let Err(err) = res {
            tracing::warn!("Meter Error: {}", err);
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
