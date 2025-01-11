use super::CacheBackend;
use async_trait::async_trait;
use dashmap::DashMap;
use redis::ToRedisArgs;
use serde::{de::DeserializeOwned, Serialize};
use serde_json;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    expires_at: Option<Instant>,
    zset: Option<BTreeMap<String, f64>>,
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryCacheError {
    Serialization(#[from] serde_json::Error),
}

impl std::fmt::Display for MemoryCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Default)]
pub struct MemoryCache {
    data: DashMap<String, CacheEntry>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self::default()
    }

    fn is_expired(entry: &CacheEntry) -> bool {
        entry
            .expires_at
            .map(|expires| expires <= Instant::now())
            .unwrap_or(false)
    }

    fn get_entry(&self, key: &str) -> Option<dashmap::mapref::one::Ref<'_, String, CacheEntry>> {
        let entry = self.data.get(key)?;
        if Self::is_expired(&entry) {
            let key = key.to_string();
            drop(entry);
            self.data.remove(&key);
            None
        } else {
            Some(entry)
        }
    }

    fn get_entry_mut(
        &self,
        key: &str,
    ) -> Option<dashmap::mapref::one::RefMut<'_, String, CacheEntry>> {
        let entry = self.data.get_mut(key)?;
        if Self::is_expired(&entry) {
            let key = key.to_string();
            drop(entry);
            self.data.remove(&key);
            None
        } else {
            Some(entry)
        }
    }

    fn key_to_string<K: ToRedisArgs>(key: &K) -> String {
        let mut args = vec![];
        key.write_redis_args(&mut args);
        String::from_utf8_lossy(&args.concat()).into_owned()
    }
}

#[async_trait]
impl CacheBackend for MemoryCache {
    type Error = MemoryCacheError;

    async fn get<K, V>(&self, key: &K) -> Result<Option<V>, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
        V: DeserializeOwned,
    {
        let key = Self::key_to_string(key);

        let result = match self.get_entry(&key) {
            Some(entry) => {
                tracing::debug!("Cache hit");
                let value: V = serde_json::from_slice(&entry.data)?;
                Ok(Some(value))
            }
            None => {
                tracing::debug!("Cache miss");
                Ok(None)
            }
        };

        result
    }

    async fn set<K, V>(&self, key: K, value: &V, ttl: Option<Duration>) -> Result<(), Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
        V: Serialize + Send + Sync,
    {
        let key = Self::key_to_string(&key);

        let data = match serde_json::to_vec(value) {
            Ok(d) => {
                tracing::debug!(value_size = d.len(), "Serializing value");
                d
            }
            Err(e) => {
                tracing::error!(error = ?e, "Failed to serialize value");
                return Err(e.into());
            }
        };

        let expires_at = ttl.map(|duration| Instant::now() + duration);
        self.data.insert(
            key.to_string(),
            CacheEntry {
                data,
                expires_at,
                zset: None,
            },
        );

        tracing::debug!("Operation successful");
        Ok(())
    }

    async fn incr<K>(&self, key: &K, ttl: Option<Duration>) -> Result<u64, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let key = Self::key_to_string(key);

        let value = match self.get_entry_mut(&key) {
            Some(mut entry) => {
                let value = match serde_json::from_slice::<u64>(&entry.data) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!(error = ?e, "Failed to deserialize value");
                        return Err(e.into());
                    }
                };

                let new_value = value + 1;
                entry.data = serde_json::to_vec(&new_value)?;
                entry.expires_at = ttl.map(|duration| Instant::now() + duration);

                new_value
            }
            None => {
                self.set(key, &1u64, ttl).await?;
                1
            }
        };

        tracing::debug!("Operation successful");

        Ok(value)
    }

    async fn incrby<K>(&self, key: &K, increment: u64, ttl: Option<Duration>) -> Result<u64, Self::Error>
        where
            K: ToRedisArgs + Send + Sync {
        let key = Self::key_to_string(key);

        let value = match self.get_entry_mut(&key) {
            Some(mut entry) => {
                let value = match serde_json::from_slice::<u64>(&entry.data) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!(error = ?e, "Failed to deserialize value");
                        return Err(e.into());
                    }
                };

                let new_value = value + increment;
                entry.data = serde_json::to_vec(&new_value)?;
                entry.expires_at = ttl.map(|duration| Instant::now() + duration);

                new_value
            }
            None => {
                self.set(key, &increment, ttl).await?;
                increment
            }
        };

        tracing::debug!("Operation successful");

        Ok(value)
    }

    async fn delete<K>(&self, key: &K) -> Result<(), Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let key = Self::key_to_string(key);
        self.data.remove(&key);
        Ok(())
    }

    async fn exists<K>(&self, key: &K) -> Result<bool, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let key = Self::key_to_string(key);

        if let Some(entry) = self.data.get(&key) {
            if Self::is_expired(&entry) {
                self.data.remove(&key);
                return Ok(false);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn zadd<K>(&self, key: &K, score: f64, member: &str) -> Result<bool, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let key = Self::key_to_string(key);
        let mut entry = self.data.entry(key).or_insert_with(|| CacheEntry {
            data: Vec::new(),
            expires_at: None,
            zset: Some(BTreeMap::new()),
        });

        let zset = entry.zset.get_or_insert_with(BTreeMap::new);
        let is_new = !zset.contains_key(member);
        zset.insert(member.to_string(), score);
        Ok(is_new)
    }

    async fn zremrangebyscore<K>(&self, key: &K, min: f64, max: f64) -> Result<u64, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let key = Self::key_to_string(key);
        if let Some(mut entry) = self.data.get_mut(&key) {
            if let Some(zset) = &mut entry.zset {
                let before_len = zset.len();
                zset.retain(|_, score| *score < min || *score > max);
                return Ok((before_len - zset.len()) as u64);
            }
        }
        Ok(0)
    }

    async fn zcard<K>(&self, key: &K) -> Result<u64, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let key = Self::key_to_string(key);
        if let Some(entry) = self.get_entry(&key) {
            if let Some(zset) = &entry.zset {
                return Ok(zset.len() as u64);
            }
        }
        Ok(0)
    }

    async fn expire<K>(&self, key: &K, ttl: Duration) -> Result<bool, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let key = Self::key_to_string(key);
        if let Some(mut entry) = self.data.get_mut(&key) {
            entry.expires_at = Some(Instant::now() + ttl);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
