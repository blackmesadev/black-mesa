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
            tokio::time::sleep(std::time::Duration::from_secs(self.interval)).await;
            let rest = Arc::clone(&self.rest);
            let db = Arc::clone(&self.db);
            tokio::spawn(async move {
                match expiry_job(rest, db).await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Error in expiry job: {:?}", e);
                    }
                }
            });
        }
    }
}

#[instrument(skip(rest, db))]
async fn expiry_job(rest: Arc<DiscordRestClient>, db: Arc<Database>) -> DiscordResult<()> {
    let infractions = db.get_expired_infractions().await?;

    for infraction in infractions {
        if let Some(role_id) = infraction.mute_role_id {
            rest.remove_role(
                &infraction.guild_id,
                &infraction.user_id,
                &role_id,
                Some(Cow::Borrowed(REASON)),
            )
            .await?;
            tracing::debug!(
                "Removed mute role for user {} in guild {}",
                infraction.user_id,
                infraction.guild_id
            );
        }

        if infraction.infraction_type == InfractionType::Ban {
            rest.unban_member(
                &infraction.guild_id,
                &infraction.user_id,
                Some(Cow::Borrowed(REASON)),
            )
            .await?;
            tracing::debug!(
                "Unbanned user {} in guild {}",
                infraction.user_id,
                infraction.guild_id
            );
        }

        db.deactivate_infraction(&infraction.uuid).await?;
        tracing::debug!("Deactivated infraction {}", infraction.uuid);
    }

    Ok(())
}