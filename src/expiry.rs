use std::str::FromStr;
use std::sync::Arc;

use twilight_http::request::AuditLogReason;
use twilight_model::id::Id;

use crate::moderation::moderation::PunishmentType;
use crate::mongo::mongo::Database;
use crate::HttpClient;

pub async fn action_expiry(
    db: Database,
    rest: Arc<HttpClient>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        if let Ok(exp) = db.get_expired().await {
            for punishment in &exp {
                if let (Ok(guild_id), Ok(user_id)) = (
                    Id::from_str(&punishment.guild_id),
                    Id::from_str(&punishment.user_id),
                ) {
                    let reason = punishment
                        .expired_reason
                        .as_deref()
                        .unwrap_or("Punishment expired.");

                    match punishment.typ {
                        PunishmentType::Mute => {
                            if punishment.role_id.is_none() {
                                tracing::warn!(
                                    "No role ID found for mute UUID {}, skipping",
                                    punishment.uuid
                                );
                                continue;
                            }

                            let role_id = match Id::from_str(punishment.role_id.as_ref().unwrap()) {
                                Ok(id) => id,
                                Err(e) => {
                                    tracing::warn!(
                                        "Invalid role ID found for mute UUID {}, skipping: {}",
                                        punishment.uuid,
                                        e
                                    );
                                    continue;
                                }
                            };

                            let remove_role = rest
                                .remove_guild_member_role(guild_id, user_id, role_id)
                                .reason(reason);

                            match remove_role {
                                Ok(remove_role) => {
                                    if let Err(e) = remove_role.await {
                                        tracing::error!(
                                            "Failed expire mute for {}, error: {}",
                                            punishment.uuid,
                                            e
                                        );
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to apply audit log reason to {}, sending without reason applied: {}", punishment.uuid, e);
                                    if let Err(e) = rest
                                        .remove_guild_member_role(guild_id, user_id, role_id)
                                        .await
                                    {
                                        tracing::error!(
                                            "Failed expire mute for {}, error: {}",
                                            punishment.uuid,
                                            e
                                        );
                                    }
                                }
                            }
                        }
                        PunishmentType::Ban => {
                            let unban = rest.delete_ban(guild_id, user_id).reason(reason);

                            match unban {
                                Ok(unban) => {
                                    if let Err(e) = unban.await {
                                        tracing::error!(
                                            "Failed expire ban for {}, error: {}",
                                            punishment.uuid,
                                            e
                                        );
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to apply audit log reason to {}, sending without reason applied: {}", punishment.uuid, e);
                                    if let Err(e) = rest.delete_ban(guild_id, user_id).await {
                                        tracing::error!(
                                            "Failed expire ban for {}, error: {}",
                                            punishment.uuid,
                                            e
                                        );
                                    }
                                }
                            }
                        }
                        _ => {}
                    };
                }
            }

            let update_uuids = exp.iter().map(|x| x.uuid.clone()).collect::<Vec<_>>();
            if let Err(e) = db.expire_actions(None, &update_uuids, None).await {
                tracing::error!("Error updating actions: {}", e);
            }
        } else {
            tracing::error!("Error getting expired actions");
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
