use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub discord_token: String,
    pub redis_uri: String,
    pub mongo_uri: String,
    pub otlp_endpoint: String,
    pub otlp_auth: Option<String>,
    pub shard_id: u32,
    pub num_shards: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Config {
            discord_token: env::var("DISCORD_TOKEN")
                .map_err(|_| "DISCORD_TOKEN not found")?,
            redis_uri: env::var("REDIS_URI").map_err(|_| "REDIS_URI not found")?,
            mongo_uri: env::var("MONGO_URI").map_err(|_| "MONGO_URI not found")?,
            otlp_endpoint: env::var("OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:5080/api/black-mesa/v1/traces".into()),
            otlp_auth: env::var("OTLP_AUTH").ok(),
            shard_id: env::var("SHARD_ID")
                .unwrap_or_else(|_| "0".into())
                .parse()
                .map_err(|_| "SHARD_ID is not a valid number")?,
            num_shards: env::var("NUM_SHARDS")
                .unwrap_or_else(|_| "1".into())
                .parse()
                .map_err(|_| "NUM_SHARDS is not a valid number")?,
        })
    }
}
