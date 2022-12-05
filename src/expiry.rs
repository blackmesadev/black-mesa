use std::str::FromStr;
use std::sync::Arc;

use tracing::{warn, error};
use twilight_http::request::AuditLogReason;
use twilight_model::id::Id;

use crate::mongo::mongo::{Database, PunishmentType};
use crate::HttpClient;

// this function is responsible for all action expiry, this is written to not care about any errors and just skip over them if presented,
// as this can not return under any circumstances

pub async fn action_expiry(db: Database, rest: Arc<HttpClient>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        match db.get_expired().await {
            Ok(ref exp) => {
                for punishment in exp {
                    let guild_id = match Id::from_str(&punishment.guild_id) {
                        Ok(id) => id,
                        Err(_) => continue
                    };

                    let user_id = match Id::from_str(&punishment.user_id) {
                            Ok(id) => id,
                            Err(_) => continue
                    };

                    match punishment.typ {
                        PunishmentType::Mute => {
                            match rest.remove_guild_member_role(
                                guild_id,
                                user_id,
                                match Id::from_str(match punishment.role_id{
                                    Some(ref role_id) => role_id,
                                    None => continue // this should never happen, but just in case
                                    }){
                                    Ok(id) => id,
                                    Err(_) => continue
                                },
                            )
                                .reason("Punishment expired.") {
                                Ok(e) => {
                                    match e.await {
                                        Ok(_) => {},
                                        Err(_) => continue
                                    }
                                },
                                Err(e) => {
                                    warn!("Failed to remove role from user: {}", e);
                                    continue;
                                }
                            }
                        }
                        PunishmentType::Ban => {
                            match rest.delete_ban(
                                guild_id,
                                user_id
                            )
                                .reason("Punishment expired.") {
                                Ok(e) => {
                                    match e.await {
                                        Ok(_) => {},
                                        Err(_) => continue
                                    }
                                },
                                Err(e) => {
                                    warn!("Failed to remove role from user: {}", e);
                                    continue;
                                }
                            }
                        }
                        _ => {}
                    }
                    let update_uuids = exp.iter().map(|x| x.uuid.clone()).collect::<Vec<String>>();
                    match db.expire_actions(None, &update_uuids).await {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Error updating actions: {}", e);
                        }
                    }
                }
            }

            Err(e) => {
                error!("Error getting expired actions: {}", e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}