use core::fmt;
use std::str::FromStr;

use bson::Bson;
use serde_derive::{Deserialize, Serialize};
use twilight_model::{
    channel::message::{
        component::{self, ActionRow},
        Component,
    },
    id::Id,
};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use crate::{config::Config, handlers::Handler, moderation::moderation::PunishmentType};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Appeals {
    pub enabled: bool,
    pub channel_id: Option<String>,
    pub appeal_questions: Option<Vec<AppealContent>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppealContent {
    typ: AppealContentType,
    question: Vec<String>,
    answers: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum AppealContentType {
    #[default]
    WrittenResponse,
    MultipleChoice,
    SingleChoice,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Appeal {
    pub guild_id: String,
    pub user_id: String,
    pub punishment_uuid: String,
    pub content: Vec<AppealContent>,
    pub status: AppealStatus,
    pub status_reason: Option<String>,
}

impl Into<Bson> for Appeal {
    fn into(self) -> Bson {
        bson::to_bson(&self).unwrap()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppealStatus {
    Pending,
    Approved,
    Denied,
}

impl fmt::Display for AppealStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppealStatus::Pending => write!(f, "pending"),
            AppealStatus::Approved => write!(f, "approved"),
            AppealStatus::Denied => write!(f, "denied"),
        }
    }
}

impl Appeal {
    pub fn new(
        guild_id: String,
        user_id: String,
        punishment_uuid: String,
        content: Vec<AppealContent>,
    ) -> Self {
        Self {
            guild_id,
            user_id,
            punishment_uuid,
            content,
            status: AppealStatus::Pending,
            status_reason: None,
        }
    }
}

impl Handler {
    pub async fn create_appeal(
        &self,
        conf: &Config,
        guild_id: String,
        user_id: String,
        punishment_uuid: String,
        content: Vec<AppealContent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let appeal = Appeal::new(guild_id, user_id, punishment_uuid, content);

        self.db.create_appeal(&appeal).await?;

        self.db
            .update_punishment(
                &appeal.punishment_uuid,
                &appeal.guild_id,
                None,
                None,
                Some(AppealStatus::Pending),
            )
            .await?;

        let mut appeal_text = String::new();

        if let Some(modules) = &conf.modules {
            if let Some(appeals) = &modules.appeals {
                let channel_id = match &appeals.channel_id {
                    Some(channel_id) => Id::from_str(&channel_id)?,
                    None => return Ok(()),
                };

                for c in appeal.content {
                    if c.typ == AppealContentType::WrittenResponse {
                        if let Some(answers) = c.answers {
                            appeal_text = answers.join("\n");
                        }
                    }
                }

                let appeal_embed = EmbedBuilder::new()
                    .title("New Appeal")
                    .description(format!(
                        "A new appeal has been submitted for punishment {}",
                        appeal.punishment_uuid
                    ))
                    .field(EmbedFieldBuilder::new("Status", appeal.status.to_string()))
                    .field(EmbedFieldBuilder::new(
                        "User",
                        format!("<@{}>", appeal.user_id),
                    ))
                    .field(EmbedFieldBuilder::new("Appeal Text", appeal_text))
                    .field(EmbedFieldBuilder::new(
                        "Appeal UUID",
                        &appeal.punishment_uuid,
                    ))
                    .color(0xff0000)
                    .build();

                let buttons = vec![
                    component::Button {
                        custom_id: Some(
                            format!("appeal_approve_{}", &appeal.punishment_uuid).to_string(),
                        ),
                        disabled: false,
                        emoji: None,
                        label: Some("Approve".to_string()),
                        style: component::ButtonStyle::Success,
                        url: None,
                    },
                    component::Button {
                        custom_id: Some(
                            format!("appeal_deny_{}", &appeal.punishment_uuid).to_string(),
                        ),
                        disabled: false,
                        emoji: None,
                        label: Some("Deny".to_string()),
                        style: component::ButtonStyle::Danger,
                        url: None,
                    },
                ];

                let buttons = buttons
                    .into_iter()
                    .map(|b| Component::Button(b))
                    .collect::<Vec<Component>>();

                let components = vec![Component::ActionRow(ActionRow {
                    components: buttons,
                })];

                self.rest
                    .create_message(channel_id)
                    .embeds(&vec![appeal_embed])?
                    .components(&components)?
                    .await?;
            };
        };

        Ok(())
    }

    pub async fn deny_appeal(
        &self,
        punishment_uuid: String,
        reason: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.db
            .update_appeal_status(&punishment_uuid, AppealStatus::Denied, reason)
            .await?;

        Ok(())
    }

    pub async fn grant_appeal(
        &self,
        conf: Option<&Config>,
        guild_id: String,
        punishment_uuid: String,
        reason: Option<String>,
        expire: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.db
            .update_appeal_status(&punishment_uuid, AppealStatus::Approved, reason.clone())
            .await?;

        let reason = match reason {
            Some(reason) => format!("Appeal granted: {}", reason),
            None => String::from("Appeal granted"),
        };

        if let Some(action) = self
            .db
            .get_action_by_uuid(&guild_id, &punishment_uuid)
            .await?
        {
            if expire {
                self.db
                    .expire_action(Some(guild_id), &punishment_uuid, Some(&reason))
                    .await?;
                return Ok(());
            }

            match action.typ {
                PunishmentType::Strike => {
                    self.db.delete_by_uuid(&guild_id, &punishment_uuid).await?;

                    let esc_action = self.db.get_action_by_escalation_uuid(&action.uuid).await?;

                    if let Some(esc_action) = esc_action {
                        self.db
                            .expire_action(Some(guild_id), &esc_action.uuid, Some(&reason))
                            .await?;
                    }
                }
                PunishmentType::Mute => {
                    if conf.is_some() {
                        self.unmute_user(
                            conf,
                            None,
                            &guild_id,
                            &action.user_id,
                            &action.issuer,
                            Some(&reason),
                        )
                        .await?;
                    } else {
                        self.db.delete_by_uuid(&guild_id, &punishment_uuid).await?;
                    }
                }
                PunishmentType::Kick => {
                    self.db.delete_by_uuid(&guild_id, &punishment_uuid).await?;
                }
                PunishmentType::Ban => {
                    self.db.delete_by_uuid(&guild_id, &punishment_uuid).await?;
                    self.unban_user(&guild_id, &action.user_id, &action.issuer, Some(&reason))
                        .await?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
