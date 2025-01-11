use super::Database;

use mongodb::bson::doc;
use tracing::instrument;

use crate::{discord::Id, model::Config};

pub use mongodb;
pub use mongodb::error::Error as MongoError;

impl Database {
    #[instrument(
        name = "db_get_config",
        skip(self),
        fields(
            guild_id = %guild_id
        )
    )]
    pub async fn get_config(&self, guild_id: &Id) -> Result<Option<Config>, MongoError> {
        self.configs()
            .find_one(doc! { "id": guild_id.to_string() })
            .await
    }

    #[instrument(
        name = "db_update_config",
        skip(self, config),
        fields(
            guild_id = %config.id
        )
    )]
    pub async fn update_config(&self, guild_id: &Id, config: &Config) -> Result<(), MongoError> {
        self.configs()
            .replace_one(doc! { "id": guild_id.to_string() }, config)
            .await
            .map(|_| ())
    }

    #[instrument(
        name = "db_create_config",
        skip(self, config),
        fields(
            guild_id = %config.id
        )
    )]
    pub async fn create_config(&self, config: &Config) -> Result<(), MongoError> {
        self.configs().insert_one(config).await.map(|_| ())
    }
}
