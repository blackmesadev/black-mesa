use twilight_model::channel::Message;

use crate::{config::Config, handlers::Handler};

impl Handler {
    pub async fn appeal_cmd(
        &self,
        conf: &Config,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = &msg.content;

        let args = content.split_whitespace().collect::<Vec<&str>>();

        let action = match args.get(1) {
            Some(uuid) => {
                match self
                    .db
                    .get_action_by_uuid_opt(
                        None,
                        Some(&msg.author.id.to_string()),
                        &uuid.to_string(),
                    )
                    .await?
                {
                    Some(action) => action,
                    None => return Ok(()),
                }
            }
            None => {
                if let Some(msg_ref) = &msg.reference {
                    let msg_ref_id = match msg_ref.message_id {
                        Some(id) => id.to_string(),
                        None => return Ok(()),
                    };

                    match self.db.get_action_by_notif_id(&msg_ref_id).await? {
                        Some(p) => p,
                        None => return Ok(()),
                    };
                }

                return Ok(());
            }
        };

        self.create_appeal(&conf, action.guild_id, action.user_id, action.uuid, vec![])
            .await?;

        Ok(())
    }
}
