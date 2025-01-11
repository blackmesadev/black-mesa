mod config;
mod infractions;

use mongodb::{options::ClientOptions, Client, Collection};
use tracing::instrument;

use crate::model::{Config, Infraction};

pub use mongodb;
pub use mongodb::error::Error as MongoError;

pub struct Database {
    db: mongodb::Database,
}

impl Database {
    #[instrument(
        name = "db_connect",
        skip(connection_string),
        fields(
            database = %database_name
        )
    )]
    pub async fn connect(connection_string: String, database_name: &str) -> Result<Self, MongoError> {
        let options = ClientOptions::parse(connection_string).await?;
        let client = Client::with_options(options)?;
        let db = client.database(database_name);

        Ok(Self { db })
    }

    fn infractions(&self) -> Collection<Infraction> {
        self.db.collection("infractions")
    }

    fn configs(&self) -> Collection<Config> {
        self.db.collection("configs")
    }
}
