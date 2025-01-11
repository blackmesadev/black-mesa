pub mod automod;
mod config;
mod infraction;

use std::str::FromStr;

pub use config::*;
pub use infraction::{Infraction, InfractionType};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnixTimestamp(u64);

impl UnixTimestamp {
    pub fn now() -> Self {
        Self(chrono::Utc::now().timestamp() as u64)
    }

    pub fn as_secs(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uuid(ObjectId);

impl Uuid {
    pub fn new() -> Self {
        Self(ObjectId::new())
    }

    pub fn to_string(&self) -> String {
        self.0.to_hex()
    }

    pub fn from_string(s: &str) -> Option<Self> {
        ObjectId::from_str(s).map(Self).ok()
    }

    pub fn inner(&self) -> &ObjectId {
        &self.0
    }
}

impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ObjectId> for Uuid {
    fn from(id: ObjectId) -> Self {
        Self(id)
    }
}

impl From<Uuid> for ObjectId {
    fn from(id: Uuid) -> Self {
        id.0
    }
}

impl Into<bson::Bson> for Uuid {
    fn into(self) -> bson::Bson {
        bson::Bson::ObjectId(self.0)
    }
}
