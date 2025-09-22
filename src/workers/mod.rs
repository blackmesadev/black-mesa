use std::sync::Arc;

use bm_lib::{db::Database, discord::DiscordRestClient};

mod expiry;

pub struct Worker {
    pub interval: u64,
    pub db: Arc<Database>,
    pub rest: Arc<DiscordRestClient>,
}

impl Worker {
    pub fn new(interval: u64, db: Arc<Database>, rest: Arc<DiscordRestClient>) -> Self {
        Self { interval, db, rest }
    }
}
