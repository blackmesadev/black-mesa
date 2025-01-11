pub mod data;
pub mod groups;
pub mod handler;
pub mod help;
pub mod macros;
pub mod messages;
pub mod moderation;
pub mod permissions;

use bm_lib::{
    cache::{Cache, RedisCache},
    db::Database,
    discord::DiscordRestClient,
};
use tokio::sync::RwLock;

use std::{sync::Arc, time::Duration};

pub const ZWSP: &str = "\u{200B}";

pub struct EventHandler {
    pub rest: Arc<DiscordRestClient>,
    pub cache: Arc<Cache<RedisCache>>,
    pub db: Arc<Database>,

    pub start_time: std::time::Instant,
    pub ping: Arc<RwLock<Duration>>
}

impl EventHandler {
    pub fn new(
        rest: Arc<DiscordRestClient>,
        cache: Arc<Cache<RedisCache>>,
        db: Arc<Database>,
    ) -> Self {
        Self {
            rest,
            cache,
            db,
            start_time: std::time::Instant::now(),
            ping: Arc::new(RwLock::new(Duration::from_secs(0))),
        }
    }
}
