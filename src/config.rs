use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub discord_token: String,
    pub redis_uri: String,
    pub redis_prefix: String,
    pub database_url: String,
    pub mesastream_base_url: String,
    pub mesastream_token: String,
    pub otlp_endpoint: String,
    pub otlp_auth: Option<String>,
    pub otlp_organization: Option<String>,
    pub shard_id: u32,
    pub num_shards: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Config {
            discord_token: env::var("DISCORD_TOKEN").map_err(|_| "DISCORD_TOKEN not found")?,
            redis_uri: env::var("REDIS_URI").map_err(|_| "REDIS_URI not found")?,
            redis_prefix: env::var("REDIS_PREFIX").unwrap_or_else(|_| "black-mesa".into()),
            database_url: env::var("DATABASE_URL").map_err(|_| "DATABASE_URL not found")?,
            mesastream_base_url: env::var("MESASTREAM_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".into()),
            mesastream_token: env::var("MESASTREAM_TOKEN")
                .map_err(|_| "MESASTREAM_TOKEN not found")?,
            otlp_endpoint: env::var("OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:5081".into()),
            otlp_auth: env::var("OTLP_AUTH").ok(),
            otlp_organization: env::var("OTLP_ORGANIZATION").ok(),
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
