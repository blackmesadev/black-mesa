pub mod data;
pub mod groups;
pub mod handler;
pub mod help;
pub mod macros;
pub mod mesastream;
pub mod messages;
pub mod moderation;
pub mod permissions;
pub mod voice;

use std::collections::HashSet;
use std::sync::{atomic::AtomicU64, Arc, OnceLock};

use bm_lib::{
    cache::{Cache, RedisCache},
    clients::MesastreamClient,
    db::Database,
    discord::{DiscordRestClient, GatewaySender, Id},
};
use tokio::sync::Mutex;

pub const ZWSP: &str = "\u{200B}";

pub struct EventHandler {
    pub rest: Arc<DiscordRestClient>,
    pub cache: Arc<Cache<RedisCache>>,
    pub db: Arc<Database>,
    pub mesastream: Arc<MesastreamClient>,
    pub gateway: Arc<Mutex<Option<GatewaySender>>>,

    pub start_time: std::time::Instant,
    pub ping_nanos: Arc<AtomicU64>,

    pub bot_mention: Arc<OnceLock<String>>,
    pub bot_id: Arc<OnceLock<Id>>,

    /// Guild IDs where the bot is currently in a voice channel.
    /// Used to recreate mesastream players after mesastream restarts.
    pub voice_guilds: Arc<Mutex<HashSet<Id>>>,
}

impl EventHandler {
    pub fn new(
        rest: Arc<DiscordRestClient>,
        cache: Arc<Cache<RedisCache>>,
        db: Arc<Database>,
        mesastream: Arc<MesastreamClient>,
    ) -> Self {
        Self {
            rest,
            cache,
            db,
            mesastream,
            start_time: std::time::Instant::now(),
            ping_nanos: Arc::new(AtomicU64::new(0)),
            bot_mention: Arc::new(OnceLock::new()),
            bot_id: Arc::new(OnceLock::new()),
            gateway: Arc::new(Mutex::new(None)),
            voice_guilds: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}
