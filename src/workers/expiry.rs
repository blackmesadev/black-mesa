use std::{borrow::Cow, sync::Arc};

use bm_lib::{
    db::Database,
    discord::{DiscordRestClient, DiscordResult},
    model::InfractionType,
};
use tracing::instrument;

use super::Worker;

const REASON: &str = "Infraction expired";

impl Worker {
    pub async fn start_expiry(&self) {
        tracing::info!("Starting expiry worker");
        loop {
            if let Err(e) = expiry_job(Arc::clone(&self.rest), Arc::clone(&self.db)).await {
                tracing::error!("Error in expiry job: {:?}", e);
            }
            tokio::time::sleep(std::time::Duration::from_secs(self.interval)).await;
        }
    }
}

#[instrument(skip(rest, db))]
async fn expiry_job(rest: Arc<DiscordRestClient>, db: Arc<Database>) -> DiscordResult<()> {
    let infractions = db.get_expired_infractions().await?;

    for infraction in infractions {
        if let Some(role_id) = infraction.mute_role_id {
            if let Err(e) = rest
                .remove_role(
                    &infraction.guild_id,
                    &infraction.user_id,
                    &role_id,
                    Some(Cow::Borrowed(REASON)),
                )
                .await
            {
                tracing::warn!(
                    infraction_id = %infraction.uuid,
                    error = ?e,
                    "remove_role failed, deactivating infraction anyway"
                );
            } else {
                tracing::info!(
                    "Removed mute role for user {} in guild {}",
                    infraction.user_id,
                    infraction.guild_id
                );
            }
        }

        if infraction.infraction_type == InfractionType::Ban {
            if let Err(e) = rest
                .unban_member(
                    &infraction.guild_id,
                    &infraction.user_id,
                    Some(Cow::Borrowed(REASON)),
                )
                .await
            {
                tracing::warn!(
                    infraction_id = %infraction.uuid,
                    error = ?e,
                    "unban_member failed, deactivating infraction anyway"
                );
            } else {
                tracing::info!(
                    "Unbanned user {} in guild {}",
                    infraction.user_id,
                    infraction.guild_id
                );
            }
        }

        db.deactivate_infraction(&infraction.uuid).await?;
        tracing::info!("Deactivated infraction {}", infraction.uuid);
    }

    Ok(())
}
