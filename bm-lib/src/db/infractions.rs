use super::Database;

use futures::StreamExt;
use mongodb::bson::doc;
use tracing::instrument;

use crate::model::InfractionType;
use crate::{
    discord::Id,
    model::{Infraction, Uuid},
};

pub use mongodb;
pub use mongodb::error::Error as MongoError;

impl Database {
    #[instrument(
        name = "db_create_infraction",
        skip(self, infraction),
        fields(
            guild_id = %infraction.guild_id,
            infraction_id = %infraction.uuid,
            infraction_type = %infraction.infraction_type
        )
    )]
    pub async fn create_infraction(&self, infraction: &Infraction) -> Result<(), MongoError> {
        self.infractions().insert_one(infraction).await.map(|_| ())
    }

    #[instrument(
        name = "db_get_infraction",
        skip(self),
        fields(
            guild_id = %guild_id,
            infraction_id = %id
        )
    )]
    pub async fn get_infraction(
        &self,
        guild_id: &Id,
        id: &Uuid,
    ) -> Result<Option<Infraction>, MongoError> {
        self.infractions()
            .find_one(doc! { "_id": id, "guild_id": guild_id.to_string() })
            .await
    }

    #[instrument(
        name = "db_delete_infraction",
        skip(self),
        fields(
            guild_id = %guild_id,
            infraction_id = %id
        )
    )]
    pub async fn delete_infraction(
        &self,
        guild_id: &Id,
        id: &Uuid,
    ) -> Result<Option<Infraction>, MongoError> {
        self.infractions()
            .find_one_and_delete(doc! { "_id": id.inner(), "guild_id": guild_id.to_string() })
            .await
    }

    #[instrument(
        name = "db_get_active_infractions",
        skip(self),
        fields(
            guild_id = %guild_id,
            user_id = %user_id,
            infraction_type = ?typ
        )
    )]
    pub async fn get_active_infractions(
        &self,
        guild_id: &Id,
        user_id: &Id,
        typ: Option<InfractionType>,
    ) -> Result<Vec<Infraction>, MongoError> {
        let mut filter = doc! {
            "guild_id": guild_id.to_string(),
            "user_id": user_id.to_string(),
            "active": true
        };

        if let Some(typ) = typ {
            filter.insert("infraction_type", typ.to_str());
        }

        let mut infractions = Vec::new();
        let mut cursor = self.infractions().find(filter).await?;

        while let Some(result) = cursor.next().await {
            if let Ok(infraction) = result {
                infractions.push(infraction);
            }
        }

        Ok(infractions)
    }

    #[instrument(
        name = "db_deactivate_infraction",
        skip(self, id),
        fields(
            infraction_id = %id
        )
    )]
    pub async fn deactivate_infraction(&self, id: &Uuid) -> Result<bool, MongoError> {
        let result = self
            .infractions()
            .update_one(doc! { "_id": id }, doc! { "$set": { "active": false } })
            .await?;

        Ok(result.modified_count > 0)
    }

    #[instrument(name = "db_get_expired_infractions", skip(self))]
    pub async fn get_expired_infractions(&self) -> Result<Vec<Infraction>, MongoError> {
        let now = chrono::Utc::now().timestamp();
        let filter = doc! {
            "expires_at": { "$lte": now },
            "active": true
        };

        let mut infractions = Vec::new();
        let mut cursor = self.infractions().find(filter).await?;

        while let Some(result) = cursor.next().await {
            if let Ok(infraction) = result {
                infractions.push(infraction);
            }
        }

        Ok(infractions)
    }
}
