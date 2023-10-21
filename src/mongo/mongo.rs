use std::env;

use bson::Document;
use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::options::FindOneOptions;
use mongodb::results::{DeleteResult, InsertOneResult, UpdateResult};
use mongodb::{bson::doc, options::ClientOptions, Client, Collection};

use crate::appeals::{Appeal, AppealStatus};
use crate::config::{Config, Guild};
use crate::moderation::moderation::{Punishment, PunishmentType};

#[derive(Clone, Debug)]
pub struct Database {
    pub client: Client,
}

impl Database {
    pub async fn status(&self) -> bool {
        let db = self.client.database("black-mesa");

        match db
            .run_command(
                doc! {
                    "ping": 1
                },
                None,
            )
            .await
        {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_guild(
        &self,
        guild_id: &String,
    ) -> Result<Option<Config>, mongodb::error::Error> {
        let guilds: Collection<Guild> = self.client.database("black-mesa").collection("guilds");

        let res = guilds.find_one(doc! { "guild_id": guild_id }, None).await?;

        match res {
            Some(guild) => Ok(Some(guild.config)),
            None => Ok(None),
        }
    }

    pub async fn set_guild(
        &self,
        guild_id: &String,
        config: &Config,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let guilds: Collection<Guild> = self.client.database("black-mesa").collection("guilds");

        let res = guilds
            .update_one(
                doc! { "guild_id": guild_id },
                doc! { "$set": { "config": config } },
                None,
            )
            .await?;

        Ok(res)
    }

    pub async fn create_guild(
        &self,
        guild: &Guild,
    ) -> Result<InsertOneResult, mongodb::error::Error> {
        let guilds: Collection<Guild> = self.client.database("black-mesa").collection("guilds");

        let res = guilds.insert_one(guild, None).await?;

        Ok(res)
    }

    pub async fn get_punishments(
        &self,
        guild_id: &String,
        user_id: &String,
    ) -> Result<Vec<Punishment>, mongodb::error::Error> {
        let mut punishments_vec: Vec<Punishment> = Vec::new();
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let mut cur = punishments
            .find(doc! { "guild_id": guild_id, "user_id": user_id }, None)
            .await?;

        loop {
            let next_cur = cur.try_next().await;
            match next_cur {
                Ok(Some(punishment)) => punishments_vec.push(punishment),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(punishments_vec)
    }

    pub async fn get_mute(
        &self,
        guild_id: &String,
        user_id: &String,
    ) -> Result<Option<Punishment>, mongodb::error::Error> {
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let res = punishments.find_one(doc!
            { "guild_id": guild_id, "user_id": user_id, "type": PunishmentType::Mute.to_string() },
            None)
            .await?;

        Ok(res)
    }

    pub async fn delete_mute(
        &self,
        guild_id: &String,
        user_id: &String,
    ) -> Result<DeleteResult, mongodb::error::Error> {
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let res = punishments.delete_one(doc!
            { "guild_id": guild_id, "user_id": user_id, "type": PunishmentType::Mute.to_string() },
            None)
            .await?;

        Ok(res)
    }

    pub async fn delete_ban(
        &self,
        guild_id: &String,
        user_id: &String,
    ) -> Result<DeleteResult, mongodb::error::Error> {
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let res = punishments.delete_one(doc!
            { "guild_id": guild_id, "user_id": user_id, "type": PunishmentType::Ban.to_string() },
            None)
            .await?;

        Ok(res)
    }

    pub async fn delete_by_uuid(
        &self,
        guild_id: &String,
        uuid: &String,
    ) -> Result<DeleteResult, mongodb::error::Error> {
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let res = punishments
            .delete_one(doc! { "guild_id": guild_id, "uuid": uuid }, None)
            .await?;

        Ok(res)
    }

    pub async fn delete_many_by_uuid(
        &self,
        guild_id: &String,
        uuids: &Vec<String>,
    ) -> Result<DeleteResult, mongodb::error::Error> {
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let res = punishments
            .delete_many(
                doc! { "guild_id": guild_id, "uuid": { "$in": uuids } },
                None,
            )
            .await?;

        Ok(res)
    }

    pub async fn expire_actions(
        &self,
        guild_id: Option<String>,
        uuids: &Vec<String>,
        reason: Option<&String>,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let mut query = Document::new();

        if let Some(guild_id) = guild_id {
            query.insert("guild_id", guild_id);
        }

        query.insert("uuid", doc! { "$in": uuids });

        let mut update = doc! {
            "$set": {
                "expired": true,
                "expires": chrono::Utc::now().timestamp()
            }
        };

        if let Some(reason) = reason {
            update.insert("$set", doc! { "reason": reason });
        }

        let res = punishments.update_many(query, update, None).await?;

        Ok(res)
    }

    pub async fn expire_action(
        &self,
        guild_id: Option<String>,
        uuid: &String,
        reason: Option<&String>,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let mut query = Document::new();

        if let Some(guild_id) = guild_id {
            query.insert("guild_id", guild_id);
        }

        query.insert("uuid", uuid);

        let mut update = doc! {
            "$set": {
                "expired": true,
                "expires": chrono::Utc::now().timestamp()
            }
        };

        if let Some(reason) = reason {
            update.insert("$set", doc! { "reason": reason });
        }

        let res = punishments.update_one(query, update, None).await?;

        Ok(res)
    }

    pub async fn update_punishment(
        &self,
        uuid: &String,
        guild_id: &String,
        reason: Option<String>,
        duration: Option<i64>,
        appeal: Option<AppealStatus>,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let mut update = Document::new();

        if let Some(reason) = reason {
            update.insert("reason", bson::Bson::String(reason));
        }

        if let Some(duration) = duration {
            update.insert("expires", bson::Bson::Int64(duration));
        }

        if let Some(appeal) = appeal {
            update.insert("appeal_status", bson::Bson::String(appeal.to_string()));
        }

        let res = punishments
            .update_one(
                doc! { "uuid": uuid, "guild_id": guild_id },
                doc! { "$set": update },
                None,
            )
            .await?;

        Ok(res)
    }

    pub async fn get_strikes(
        &self,
        guild_id: &String,
        user_id: &String,
    ) -> Result<Vec<Punishment>, mongodb::error::Error> {
        let mut punishments_vec: Vec<Punishment> = Vec::new();
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let mut cur = punishments.find(doc!
            { "guild_id": guild_id, "user_id": user_id, "type": PunishmentType::Strike.to_string(), "expired": false },
            None)
            .await?;

        loop {
            let next_cur = cur.try_next().await;
            match next_cur {
                Ok(Some(punishment)) => punishments_vec.push(punishment),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(punishments_vec)
    }

    pub async fn get_action_by_escalation_uuid(
        &self,
        escalation_id: &String,
    ) -> Result<Option<Punishment>, mongodb::error::Error> {
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let mut cur = punishments
            .find(
                doc! { "escalation_id": escalation_id, "expired": false },
                None,
            )
            .await?;

        let res = cur.try_next().await?;

        Ok(res)
    }

    pub async fn get_last_action(
        &self,
        guild_id: &String,
        issuer_id: &String,
    ) -> Result<Option<Punishment>, mongodb::error::Error> {
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let res = punishments
            .find_one(
                doc! { "guild_id": guild_id, "issuer": issuer_id },
                FindOneOptions::builder()
                    .sort(doc! {"$natural": -1})
                    .build(),
            )
            .await?;

        Ok(res)
    }

    pub async fn get_action_by_uuid(
        &self,
        guild_id: &String,
        uuid: &String,
    ) -> Result<Option<Punishment>, mongodb::error::Error> {
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let res = punishments
            .find_one(doc! { "guild_id": guild_id, "uuid": uuid }, None)
            .await?;

        Ok(res)
    }

    pub async fn get_action_by_uuid_opt(
        &self,
        guild_id: Option<&String>,
        user_id: Option<&String>,
        uuid: &String,
    ) -> Result<Option<Punishment>, mongodb::error::Error> {
        if guild_id.is_none() && user_id.is_none() {
            return Err(mongodb::error::Error::custom(
                "guild_id or user_id must be specified",
            ));
        }

        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let mut query = doc! { "uuid": uuid };

        if let Some(guild_id) = guild_id {
            query.insert("guild_id", guild_id);
        }

        if let Some(user_id) = user_id {
            query.insert("user_id", user_id);
        }

        let res = punishments.find_one(query, None).await?;

        Ok(res)
    }

    pub async fn get_actions_by_uuid(
        &self,
        guild_id: &String,
        uuids: Vec<String>,
    ) -> Result<Vec<Punishment>, mongodb::error::Error> {
        let mut punishments_vec: Vec<Punishment> = Vec::new();
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let mut cur = punishments
            .find(
                doc! { "guild_id": guild_id, "uuid": { "$in": uuids } },
                None,
            )
            .await?;

        loop {
            let next_cur = cur.try_next().await;
            match next_cur {
                Ok(Some(punishment)) => punishments_vec.push(punishment),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(punishments_vec)
    }

    pub async fn add_punishment(
        &self,
        punishment: &Punishment,
    ) -> Result<(), mongodb::error::Error> {
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        match punishments.insert_one(punishment, None).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn get_expired(&self) -> Result<Vec<Punishment>, mongodb::error::Error> {
        let mut punishments_vec: Vec<Punishment> = Vec::new();
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let mut cur = punishments
            .find(
                doc! {
                    "expires": {
                        "$lte": Utc::now().timestamp()
                    },

                    "$or": [
                        {
                            "expired": false
                        },
                        {
                            "expired": { "$exists": false }
                        }
                    ]
                },
                None,
            )
            .await?;

        loop {
            let next_cur = cur.try_next().await;
            match next_cur {
                Ok(Some(punishment)) => punishments_vec.push(punishment),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(punishments_vec)
    }

    pub async fn create_appeal(&self, appeal: &Appeal) -> Result<(), mongodb::error::Error> {
        let appeals: Collection<Appeal> = self.client.database("black-mesa").collection("appeals");

        match appeals.insert_one(appeal, None).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn update_appeal_status(
        &self,
        uuid: &String,
        status: AppealStatus,
        reason: Option<String>,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let appeals: Collection<Appeal> = self.client.database("black-mesa").collection("appeals");

        let mut update = Document::new();

        if let Some(reason) = reason {
            update.insert("reason", bson::Bson::String(reason));
        }

        update.insert("status", status.to_string());

        let res = appeals
            .update_one(doc! { "uuid": uuid }, update, None)
            .await?;

        Ok(res)
    }

    pub async fn get_action_by_notif_id(
        &self,
        notif_id: &String,
    ) -> Result<Option<Punishment>, mongodb::error::Error> {
        let punishments: Collection<Punishment> =
            self.client.database("black-mesa").collection("actions");

        let res = punishments
            .find_one(doc! { "notif_id": notif_id }, None)
            .await?;

        Ok(res)
    }
}

pub async fn connect() -> Database {
    let uri = env::var("MONGO_URI").expect("Expected a MongoDB URI in the environment");
    let mut options = ClientOptions::parse(&uri)
        .await
        .expect("Error creating client options");

    options.app_name = Some("Black Mesa".to_string());

    let client = Client::with_options(options).expect("Error creating client");

    Database { client }
}
