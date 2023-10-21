use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use twilight_http::request::AuditLogReason;
use twilight_model::channel::Message;
use twilight_model::id::marker::MessageMarker;
use twilight_model::id::Id;
use twilight_model::user::User;

use crate::handlers::Handler;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub completed: u16,
    pub total: u16,
}

struct PurgeTask {
    task: JoinHandle<Result<(), Error>>,
    //info: Arc<Mutex<TaskInfo>>,
}

impl PurgeTask {
    fn new(task: JoinHandle<Result<(), Error>>, _task_info: Arc<Mutex<TaskInfo>>) -> Self {
        Self {
            task,
            //info: task_info,
        }
    }
}

lazy_static! {
    static ref PURGE_TASKS: Arc<Mutex<HashMap<String, PurgeTask>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum PurgeType {
    #[default]
    All,
    Attachments,
    Bot,
    Images,
    String,
    User,
    Videos,
}

impl From<&str> for PurgeType {
    fn from(value: &str) -> Self {
        match value {
            "attachments" => Self::Attachments,
            "bot" => Self::Bot,
            "images" => Self::Images,
            "string" => Self::String,
            "user" => Self::User,
            "users" => Self::User,
            "videos" => Self::Videos,
            _ => Self::All,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Purge {
    pub typ: PurgeType,
    pub initiated_by: User,
    pub guild_id: String,
    pub channel_id: String,
    pub limit: u16,
    pub user_ids: Option<Vec<String>>,
    pub filter: Option<String>,
}

pub async fn stop_purge(guild_id: String) -> Result<(), Error> {
    let mut tasks = PURGE_TASKS.lock().unwrap();
    if let Some(task) = tasks.remove(&guild_id) {
        task.task.abort();
    }
    Ok(())
}

pub async fn add_purge(handler: Arc<Handler>, purge: Purge) -> Result<(), Error> {
    let task_info = Arc::new(Mutex::new(TaskInfo {
        completed: 0,
        total: purge.limit,
    }));

    let mut tasks = PURGE_TASKS.lock().unwrap();
    let task_purge = purge.clone();
    let task_info_clone = task_info.clone();

    let task = tokio::spawn(async move {
        handler.purge(&task_purge, task_info_clone.clone()).await?;

        Ok(())
    });
    tasks.insert(purge.guild_id.clone(), PurgeTask::new(task, task_info));

    Ok(())
}

impl Handler {
    pub async fn purge(&self, purge: &Purge, task_info: Arc<Mutex<TaskInfo>>) -> Result<(), Error> {
        let mut completed = 0u16;

        let channel_id = Id::from_str(&purge.channel_id)?;
        let progress_msg: twilight_model::channel::Message = self
            .rest
            .create_message(channel_id)
            .content(
                format!(
                    "Purging messages... [{}/{}]",
                    completed,
                    task_info.lock().unwrap().total
                )
                .as_str(),
            )?
            .await?
            .model()
            .await?;

        let mut last_id: Id<MessageMarker> = progress_msg.id;

        let iterations = purge.limit / 100;

        for _ in 0..iterations {
            let mut delete_count = if purge.limit - completed < 100 {
                purge.limit - completed
            } else {
                100
            };

            let messages: Vec<Message> = self
                .rest
                .channel_messages(channel_id)
                .before(last_id)
                .limit(delete_count)?
                .await?
                .models()
                .await?;

            last_id = messages.last().unwrap().id;

            let message_ids = messages
                .iter()
                .filter(move |m| match &purge.typ {
                    PurgeType::All => true,
                    PurgeType::Attachments => m.attachments.len() > 0,
                    PurgeType::Bot => m.author.bot,
                    PurgeType::Images => m.attachments.iter().any(|a| a.width.is_some()),
                    PurgeType::String => m
                        .content
                        .contains(&purge.filter.clone().unwrap_or("".to_string())),
                    PurgeType::User => purge
                        .user_ids
                        .clone()
                        .unwrap_or_default()
                        .contains(&m.author.id.to_string()),
                    PurgeType::Videos => m.attachments.iter().any(|a| a.width.is_some()),
                })
                .map(|m| m.id)
                .collect::<Vec<Id<MessageMarker>>>();

            delete_count = message_ids.len() as u16;

            completed += delete_count;

            self.rest
                .delete_messages(channel_id, &message_ids)?
                .reason(
                    format!(
                        "Purge completed [{}/{}] initiated by {} ({})",
                        completed,
                        purge.limit,
                        purge.initiated_by.name,
                        purge.initiated_by.id.to_string()
                    )
                    .as_str(),
                )?
                .await?;

            self.rest
                .update_message(channel_id, progress_msg.id)
                .content(Some(
                    format!(
                        "Purging messages... [{}/{}]",
                        completed,
                        task_info.lock().unwrap().total
                    )
                    .as_str(),
                ))?
                .await?;

            task_info.lock().unwrap().completed += delete_count;
        }

        self.rest
            .update_message(channel_id, progress_msg.id)
            .content(Some(
                format!(
                    "Successfully purged {} messages.",
                    task_info.lock().unwrap().total
                )
                .as_str(),
            ))?
            .await?;

        Ok(())
    }
}
