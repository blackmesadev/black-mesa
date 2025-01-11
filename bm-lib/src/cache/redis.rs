use super::CacheBackend;
use async_trait::async_trait;
use redis::{aio::MultiplexedConnection, Client, ToRedisArgs};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum RedisCacheError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug)]
pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    pub async fn new(url: String) -> Result<Self, RedisCacheError> {
        let client = Client::open(url)?;
        Ok(Self { client })
    }

    async fn get_conn(&self) -> Result<MultiplexedConnection, RedisCacheError> {
        Ok(self.client.get_multiplexed_tokio_connection().await?)
    }
}

#[async_trait]
impl CacheBackend for RedisCache {
    type Error = RedisCacheError;

    async fn set<K, V>(&self, key: K, value: &V, ttl: Option<Duration>) -> Result<(), Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
        V: Serialize + Send + Sync,
    {
        let start = std::time::Instant::now();
        let value = match serde_json::to_string(value) {
            Ok(v) => {
                tracing::debug!(value_size = v.len(), "Serializing value");
                v
            }
            Err(e) => {
                tracing::error!(error = ?e, "Failed to serialize value");
                return Err(e.into());
            }
        };

        let result = if let Some(ttl) = ttl {
            redis::cmd("SETEX")
                .arg(key)
                .arg(ttl.as_secs() as usize)
                .arg(value)
                .exec_async(&mut self.get_conn().await?)
                .await
        } else {
            redis::cmd("SET")
                .arg(key)
                .arg(value)
                .exec_async(&mut self.get_conn().await?)
                .await
        };

        match result {
            Ok(_) => {
                tracing::debug!(
                    duration_ms = start.elapsed().as_millis(),
                    "Operation successful"
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(error = ?e, "Operation failed");
                Err(e.into())
            }
        }
    }

    async fn get<K, V>(&self, key: &K) -> Result<Option<V>, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
        V: DeserializeOwned,
    {
        let start = std::time::Instant::now();

        let result: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut self.get_conn().await?)
            .await?;

        match result {
            Some(v) => {
                tracing::debug!(
                    duration_ms = start.elapsed().as_millis(),
                    value_size = v.len(),
                    "Cache hit"
                );
                Ok(Some(serde_json::from_str(&v)?))
            }
            None => {
                tracing::debug!(duration_ms = start.elapsed().as_millis(), "Cache miss");
                Ok(None)
            }
        }
    }

    async fn incr<K>(&self, key: &K, ttl: Option<Duration>) -> Result<u64, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_conn().await?;

        let result = if let Some(ttl) = ttl {
            redis::cmd("INCR")
                .arg(key)
                .arg("EX")
                .arg(ttl.as_secs() as usize)
                .query_async(&mut conn)
                .await
        } else {
            redis::cmd("INCR").arg(key).query_async(&mut conn).await
        };

        if let Err(e) = &result {
            tracing::error!(error = ?e, "Redis operation failed");
        }

        result.map_err(Into::into)
    }

    async fn incrby<K>(&self, key: &K, value: u64, ttl: Option<Duration>) -> Result<u64, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_conn().await?;

        let result = if let Some(ttl) = ttl {
            redis::cmd("INCRBY")
                .arg(key)
                .arg(value)
                .arg("EX")
                .arg(ttl.as_secs() as usize)
                .query_async(&mut conn)
                .await
        } else {
            redis::cmd("INCRBY").arg(key).arg(value).query_async(&mut conn).await
        };

        if let Err(e) = &result {
            tracing::error!(error = ?e, "Redis operation failed");
        }

        result.map_err(Into::into)
    }

    async fn delete<K>(&self, key: &K) -> Result<(), Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_conn().await?;

        redis::cmd("DEL").arg(key).exec_async(&mut conn).await?;

        Ok(())
    }

    async fn exists<K>(&self, key: &K) -> Result<bool, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_conn().await?;

        let exists: bool = redis::cmd("EXISTS").arg(key).query_async(&mut conn).await?;

        Ok(exists)
    }

    async fn zadd<K>(&self, key: &K, score: f64, member: &str) -> Result<bool, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_conn().await?;

        let added: bool = redis::cmd("ZADD")
            .arg(key)
            .arg(score)
            .arg(member)
            .query_async(&mut conn)
            .await?;

        Ok(added)
    }

    async fn zremrangebyscore<K>(&self, key: &K, min: f64, max: f64) -> Result<u64, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_conn().await?;

        let removed: u64 = redis::cmd("ZREMRANGEBYSCORE")
            .arg(key)
            .arg(min)
            .arg(max)
            .query_async(&mut conn)
            .await?;

        Ok(removed)
    }

    async fn zcard<K>(&self, key: &K) -> Result<u64, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_conn().await?;

        let count: u64 = redis::cmd("ZCARD").arg(key).query_async(&mut conn).await?;

        Ok(count)
    }

    async fn expire<K>(&self, key: &K, ttl: Duration) -> Result<bool, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_conn().await?;

        let set: bool = redis::cmd("EXPIRE")
            .arg(key)
            .arg(ttl.as_secs() as usize)
            .query_async(&mut conn)
            .await?;

        Ok(set)
    }
}
