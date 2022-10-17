use std::{env, fmt};
use std::collections::HashMap;
use std::str::FromStr;
use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use mongodb::options::FindOneOptions;
use mongodb::results::{DeleteResult, UpdateResult};
use mongodb::{bson::doc, options::ClientOptions, Client, Collection};
use futures_util::TryStreamExt;
use serde_derive::{Serialize, Deserialize};
use bson::{serde_helpers::*, Document};
use serde_aux::prelude::*;

use crate::automod::*;
use crate::logging::*;
use crate::moderation::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum PunishmentType {
    #[default]
    Unknown,
    None,
    Strike,
    Mute,
    Kick,
    Ban,
}
impl FromStr for PunishmentType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "strike" => Ok(PunishmentType::Strike),
            "mute" => Ok(PunishmentType::Mute),
            "kick" => Ok(PunishmentType::Kick),
            "ban" => Ok(PunishmentType::Ban),
            _ => Ok(PunishmentType::Unknown),
        }
    }
}

impl fmt::Display for PunishmentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PunishmentType::Strike => write!(f, "strike"),
            PunishmentType::Mute => write!(f, "mute"),
            PunishmentType::Kick => write!(f, "kick"),
            PunishmentType::Ban => write!(f, "ban"),
            _ => write!(f, "unknown"),
        }
    }
}

impl PunishmentType {
    pub fn pretty_string(&self) -> String {
        match self {
            PunishmentType::Unknown => "Unknown",
            PunishmentType::None => "None",
            PunishmentType::Strike => "Strike",
            PunishmentType::Mute => "Mute",
            PunishmentType::Kick => "Kick",
            PunishmentType::Ban => "Ban",
        }
        .to_string()
    }

    pub fn past_tense_string(&self) -> String {
        match self {
            PunishmentType::Unknown => "Unknown",
            PunishmentType::None => "None",
            PunishmentType::Strike => "Striked",
            PunishmentType::Mute => "Muted",
            PunishmentType::Kick => "Kicked",
            PunishmentType::Ban => "Banned",
        }
        .to_string()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Modules {
    pub automod: automod::Automod,
    pub logging: logging::Logging,
    pub moderation: moderation::Moderation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub prefix: Option<String>,
    pub permissions: HashMap<String, i64>,
    pub levels: HashMap<String, i64>,
    pub modules: Modules
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Guild {
    pub config: Config,
    #[serde(rename = "guildID")]
    pub guild_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Punishment {
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
    #[serde(rename = "_id")]
    pub oid: ObjectId,
    #[serde(rename = "guildID")]
    pub guild_id: String,
    #[serde(rename = "userID")]
    pub user_id: String,
    pub issuer: String,
    #[serde(rename = "type")]
    pub typ : PunishmentType,
    pub expires: Option<i64>,
    #[serde(rename = "roleID")]
    pub role_id: Option<String>,
    pub weight: Option<i64>,
    pub reason: Option<String>,
    pub uuid: String,
    #[serde(default = "bool_true")]
    pub expired: bool,
}

impl Into<String> for Punishment {
    fn into(self) -> String {
        self.uuid
    }
}

#[derive(Clone, Debug)]
pub struct Database {
    pub client: Client,
}

impl Database {
    pub async fn get_guild(&self, guild_id: &String) -> Result<Option<Config>, mongodb::error::Error> {
        let guilds: Collection<Guild> = self.client.database("black-mesa").collection("guilds");
        
        let res = guilds.find_one(doc!
            { "guildID": guild_id },
            None)
            .await?;
        
        match res {
            Some(guild) => Ok(Some(guild.config)),
            None => Ok(None)
        }
    }

    pub async fn get_punishments(&self, guild_id: &String, user_id: &String) -> Result<Vec<Punishment>, mongodb::error::Error>{
        let mut punishments_vec: Vec<Punishment> = Vec::new();
        let punishments: Collection<Punishment> = self.client.database("black-mesa").collection("actions");
    
        let mut cur = punishments.find(doc!
            { "guildID": guild_id, "userID": user_id },
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

    pub async fn get_mute(&self, guild_id: &String, user_id: &String) -> Result<Option<Punishment>, mongodb::error::Error> {
        let punishments: Collection<Punishment> = self.client.database("black-mesa").collection("actions");
        
        let res = punishments.find_one(doc!
            { "guildID": guild_id, "userID": user_id, "type": PunishmentType::Mute.to_string() },
            None)
            .await?;

        Ok(res)
    }

    pub async fn delete_mute(&self, guild_id: &String, user_id: &String) -> Result<DeleteResult, mongodb::error::Error> {
        let punishments: Collection<Punishment> = self.client.database("black-mesa").collection("actions");
        
        let res = punishments.delete_one(doc!
            { "guildID": guild_id, "userID": user_id, "type": PunishmentType::Mute.to_string() },
            None)
            .await?;

        Ok(res)
    }

    pub async fn delete_ban(&self, guild_id: &String, user_id: &String) -> Result<DeleteResult, mongodb::error::Error> {
        let punishments: Collection<Punishment> = self.client.database("black-mesa").collection("actions");
        
        let res = punishments.delete_one(doc!
            { "guildID": guild_id, "userID": user_id, "type": PunishmentType::Ban.to_string() },
            None)
            .await?;

        Ok(res)
    }

    pub async fn delete_many_by_uuid(&self, guild_id: &String, uuids: &Vec<String>) -> Result<DeleteResult, mongodb::error::Error> {
        let punishments: Collection<Punishment> = self.client.database("black-mesa").collection("actions");

        let res = punishments.delete_many(doc!
            { "guildID": guild_id, "uuid": { "$in": uuids } },
            None)
            .await?;

        Ok(res)
    }

    pub async fn expire_actions(&self, guild_id: Option<String>, uuids: &Vec<String>) -> Result<UpdateResult, mongodb::error::Error> {
        let punishments: Collection<Punishment> = self.client.database("black-mesa").collection("actions");
        
        let mut query = Document::new();

        if let Some(guild_id) = guild_id {
            query.insert("guildID", guild_id);
        }

        query.insert("uuid", doc! { "$in": uuids });

        let res = punishments.update_many(query,
            doc! { "$set": { "expired": true, "expires": chrono::Utc::now().timestamp() } },
            None)
            .await?;

        Ok(res)
    }

    pub async fn update_punishment(&self,
        uuid: &String,
        guild_id: &String,
        user_id: &String,
        issuer: &String,
        reason: Option<String>,
        duration: Option<i64>,
    ) -> Result<UpdateResult, mongodb::error::Error> {
        let punishments: Collection<Punishment> = self.client.database("black-mesa").collection("actions");

        let mut update = Document::new();

        if let Some(reason) = reason {
            update.insert("reason", bson::Bson::String(reason));
        }

        if let Some(duration) = duration {
            update.insert("expires", bson::Bson::Int64(duration));
        }

        let res = punishments.update_one(
            doc! { "uuid": uuid, "guildID": guild_id, "issuer": issuer, "userID": user_id },
            doc! { "$set": update },
            None)
            .await?;

        Ok(res)
    }

    pub async fn get_strikes(&self, guild_id: &String, user_id: &String) -> Result<Vec<Punishment>, mongodb::error::Error> {
        let mut punishments_vec: Vec<Punishment> = Vec::new();
        let punishments: Collection<Punishment> = self.client.database("black-mesa").collection("actions");

        let mut cur = punishments.find(doc!
            { "guildID": guild_id, "userID": user_id, "type": PunishmentType::Strike.to_string() },
            None)
            .await?;

        loop {
            let next_cur = cur.try_next().await;
            match next_cur {
                Ok(Some(punishment)) => punishments_vec.push(punishment),
                Ok(None) => break,
                Err(e) => return Err(e)
            }
        }

        Ok(punishments_vec)
    }

    pub async fn get_last_action(&self, guild_id: &String, issuer_id: &String) -> Result<Option<Punishment>, mongodb::error::Error> {
        let punishments: Collection<Punishment> = self.client.database("black-mesa").collection("actions");
        
        let res = punishments.find_one(doc!
            { "guildID": guild_id, "issuer": issuer_id },
            FindOneOptions::builder().sort(doc! {"$natural": -1}).build())
            .await?;

        Ok(res)
    }

    pub async fn get_action_by_uuid(&self, guild_id: &String, uuid: &String) -> Result<Option<Punishment>, mongodb::error::Error> {
        let punishments: Collection<Punishment> = self.client.database("black-mesa").collection("actions");
        
        let res = punishments.find_one(doc!
            { "guildID": guild_id, "uuid": uuid },
            None)
            .await?;

        Ok(res)
    }

    pub async fn get_actions_by_uuid(&self, guild_id: &String, uuids: Vec<String>) -> Result<Vec<Punishment>, mongodb::error::Error> {
        let mut punishments_vec: Vec<Punishment> = Vec::new();
        let punishments: Collection<Punishment> = self.client.database("black-mesa").collection("actions");
    
        let mut cur = punishments.find(doc!
            { "guildID": guild_id, "uuid": { "$in": uuids } },
            None)
            .await?;

        loop {
            let next_cur = cur.try_next().await;
            match next_cur {
                Ok(Some(punishment)) => punishments_vec.push(punishment),
                Ok(None) => break,
                Err(e) => return Err(e)
            }
        }

        Ok(punishments_vec)
    }

    pub async fn add_punishment(&self, punishment: &Punishment) -> Result<(), mongodb::error::Error> {
        let punishments: Collection<Punishment> = self.client.database("black-mesa").collection("actions");
        
        match punishments.insert_one(punishment, None).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }
    }

    pub async fn get_expired(&self) -> Result<Vec<Punishment>, mongodb::error::Error> {
        let mut punishments_vec: Vec<Punishment> = Vec::new();
        let punishments: Collection<Punishment> = self.client.database("black-mesa").collection("actions");
    
        let mut cur = punishments.find(doc!
            {
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
            None)
            .await?;

        loop {
            let next_cur = cur.try_next().await;
            match next_cur {
                Ok(Some(punishment)) => punishments_vec.push(punishment),
                Ok(None) => break,
                Err(e) => return Err(e)
            }
        }

        Ok(punishments_vec)
    }
}

pub async fn connect() -> Database {
    let uri = env::var("MONGO_URI").expect("Expected a MongoDB URI in the environment");
    let mut options = ClientOptions::parse(&uri).await.expect("Error creating client options");

    options.app_name = Some("Black Mesa".to_string());

    let client = Client::with_options(options).expect("Error creating client");

    Database { client }
}