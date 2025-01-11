pub(crate) mod memory;
pub(crate) mod redis;

use ::redis::ToRedisArgs;
use async_trait::async_trait;
pub use memory::MemoryCache;
pub use redis::RedisCache;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::instrument;

pub use memory::MemoryCacheError;
pub use redis::RedisCacheError;

#[async_trait]
pub trait CacheBackend: Send + Sync {
    type Error;

    async fn get<K, V>(&self, key: &K) -> Result<Option<V>, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
        V: DeserializeOwned;

    async fn set<K, V>(&self, key: K, value: &V, ttl: Option<Duration>) -> Result<(), Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
        V: Serialize + Send + Sync;

    async fn incr<K>(&self, key: &K, ttl: Option<Duration>) -> Result<u64, Self::Error>
    where
        K: ToRedisArgs + Send + Sync;

    async fn incrby<K>(&self, key: &K, increment: u64, ttl: Option<Duration>) -> Result<u64, Self::Error>
    where
        K: ToRedisArgs + Send + Sync;

    async fn delete<K>(&self, key: &K) -> Result<(), Self::Error>
    where
        K: ToRedisArgs + Send + Sync;

    async fn exists<K>(&self, key: &K) -> Result<bool, Self::Error>
    where
        K: ToRedisArgs + Send + Sync;

    async fn zadd<K>(&self, key: &K, score: f64, member: &str) -> Result<bool, Self::Error>
    where
        K: ToRedisArgs + Send + Sync;

    async fn zremrangebyscore<K>(&self, key: &K, min: f64, max: f64) -> Result<u64, Self::Error>
    where
        K: ToRedisArgs + Send + Sync;

    async fn zcard<K>(&self, key: &K) -> Result<u64, Self::Error>
    where
        K: ToRedisArgs + Send + Sync;

    async fn expire<K>(&self, key: &K, ttl: Duration) -> Result<bool, Self::Error>
    where
        K: ToRedisArgs + Send + Sync;
}

#[derive(Debug)]
pub struct Cache<B: CacheBackend> {
    backend: B,
}

impl<B: CacheBackend> Cache<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    #[instrument(skip(self), fields(key = key.to_string()))]
    pub async fn get<K, V>(&self, key: &K) -> Result<Option<V>, B::Error>
    where
        K: ToString + ToRedisArgs + Send + Sync,
        V: DeserializeOwned,
    {
        self.backend.get(key).await
    }

    #[instrument(skip(self, value, ttl), fields(key = key.to_string()))]
    pub async fn set<K, V>(&self, key: K, value: &V, ttl: Option<Duration>) -> Result<(), B::Error>
    where
        K: ToString + ToRedisArgs + Send + Sync,
        V: Serialize + Send + Sync,
    {
        self.backend.set(key, value, ttl).await
    }

    #[instrument(skip(self), fields(key = key.to_string()))]
    pub async fn incr<K>(&self, key: &K, ttl: Option<Duration>) -> Result<u64, B::Error>
    where
        K: ToString + ToRedisArgs + Send + Sync,
    {
        self.backend.incr(key, ttl).await
    }

    #[instrument(skip(self), fields(key = key.to_string()))]
    pub async fn delete<K>(&self, key: &K) -> Result<(), B::Error>
    where
        K: ToString + ToRedisArgs + Send + Sync,
    {
        self.backend.delete(key).await
    }

    #[instrument(skip(self), fields(key = key.to_string()))]
    pub async fn exists<K>(&self, key: &K) -> Result<bool, B::Error>
    where
        K: ToString + ToRedisArgs + Send + Sync,
    {
        self.backend.exists(key).await
    }

    #[instrument(skip(self), fields(key = key.to_string()))]
    pub async fn zadd<K>(&self, key: &K, score: f64, member: &str) -> Result<bool, B::Error>
    where
        K: ToString + ToRedisArgs + Send + Sync,
    {
        self.backend.zadd(key, score, member).await
    }

    #[instrument(skip(self), fields(key = key.to_string()))]
    pub async fn zremrangebyscore<K>(&self, key: &K, min: f64, max: f64) -> Result<u64, B::Error>
    where
        K: ToString + ToRedisArgs + Send + Sync,
    {
        self.backend.zremrangebyscore(key, min, max).await
    }

    #[instrument(skip(self), fields(key = key.to_string()))]
    pub async fn zcard<K>(&self, key: &K) -> Result<u64, B::Error>
    where
        K: ToString + ToRedisArgs + Send + Sync,
    {
        self.backend.zcard(key).await
    }

    #[instrument(skip(self), fields(key = key.to_string()))]
    pub async fn expire<K>(&self, key: &K, ttl: Duration) -> Result<bool, B::Error>
    where
        K: ToString + ToRedisArgs + Send + Sync,
    {
        self.backend.expire(key, ttl).await
    }
}
