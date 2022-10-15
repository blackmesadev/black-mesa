use std::env;
use redis::AsyncCommands;

#[derive(Clone, Debug)]
pub struct Redis {
    pub client: redis::Client,
}

pub async fn connect() -> Redis {
    let client = redis::Client::open(env::var("REDIS_URI")
    .expect("Expected a Redis URI in the environment"))
    .expect("Error creating client");

    Redis { client }
}

impl Redis {
    pub async fn incr_max_messages(&self, guild_id: u64, user_id: u64, exp: usize) -> Option<i64> {
        let key = format!("max_messages:{}:{}", guild_id, user_id);
        match self.client.get_async_connection().await {
            Ok(mut connection) => {
                match connection.incr(&key, 1).await {
                    Ok(value) => {
                        match connection.expire(&key, exp).await {
                            Ok(()) => (),
                            Err(e) => {
                                println!("Error setting expire: {}", e);
                                return None;
                            }
                        }
                        Some(value)
                    }
                    Err(_) => None,
                }
            },
            Err(e) => {
                println!("Error incrementing max messages: {}", e);
                return None;
            }
        }
    }

    pub async fn set_max_messages(&self, guild_id: u64, user_id: u64, value: i64, exp: usize) -> Option<i64> {
        let key = format!("max_messages:{}:{}", guild_id, user_id);
        match self.client.get_async_connection().await {
            Ok(mut connection) => {
                match connection.set_ex(key, value, exp).await {
                    Ok(value) => Some(value),
                    Err(e) => {
                        println!("Error setting max messages: {}", e);
                        return None;
                    }
                }
            },
            Err(e) => {
                println!("Error setting max messages: {}", e);
                return None;
            }
        }
    }

    pub async fn set_memory_usage(&self, value: i64) -> Option<redis::Value> {
        let key = "memory_usage";
        match self.client.get_async_connection().await {
            Ok(mut connection) => {
                match connection.set(key, value).await {
                    Ok(value) => Some(value),
                    Err(e) => {
                        println!("Error setting memory usage: {}", e);
                        return None;
                    }
                }
            },
            Err(e) => {
                println!("Error setting memory usage: {}", e);
                return None;
            }
        }
    }

    pub async fn get_memory_usage(&self) -> Result<i64, redis::RedisError> {
        let key = "memory_usage";
        match self.client.get_async_connection().await {
            Ok(mut connection) => {
                let value: String = connection.get(key).await?;
                Ok(value.parse::<i64>().unwrap_or(0))
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}