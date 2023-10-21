use twilight_model::{
    channel::Message,
    id::{marker::UserMarker, Id},
};

use crate::{config::Config, handlers::Handler};

impl Handler {
    #[tracing::instrument(skip(self))]
    pub async fn process_cmd(
        &self,
        conf: Option<&Config>,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get the prefix from config
        let mut prefix = "";
        let mut mention = String::new();

        if let Some(conf) = conf {
            if let Some(p) = &conf.prefix {
                prefix = p;
            } else {
                let self_id = match self.get_self_id().await {
                    Some(id) => id,
                    None => return Ok(()),
                };
                mention = format!("<@{}> ", self_id);
            }
        } else {
            let self_id = match self.get_self_id().await {
                Some(id) => id,
                None => return Ok(()),
            };
            mention = format!("<@{}> ", self_id);
        }

        prefix = if prefix.is_empty() { &mention } else { prefix };

        if !msg.content.starts_with(prefix) {
            return Ok(());
        }

        // Get the command name from the msg content from what's between the prefix and the next space
        let content = msg.content.trim_start_matches(prefix);
        let cmd_name = content.split_whitespace().next().unwrap_or_default();

        // config required commands
        if let Some(conf) = conf {
            match match cmd_name {
                "userinfo" => self.user_info_cmd(conf, msg).await,
                "guildinfo" => self.guild_info_cmd(conf, msg).await,

                "search" => self.search_user_cmd(conf, msg, false).await,
                "deepsearch" => self.search_user_cmd(conf, msg, true).await,
                "reason" => self.update_reason_cmd(conf, msg).await,
                "duration" => self.update_duration_cmd(conf, msg).await,
                "remove" => self.remove_punishment_cmd(conf, msg).await,
                "expire" => self.expire_punishment_cmd(conf, msg).await,
                "purge" => self.purge_cmd(conf, msg).await,
                "stoppurge" => self.stop_purge_cmd(conf, msg).await,

                "strike" => self.strike_user_cmd(conf, msg).await,
                "kick" => self.kick_user_cmd(conf, msg).await,
                "ban" => self.ban_user_cmd(conf, msg).await,
                "softban" => self.softban_user_cmd(conf, msg).await,
                "unban" => self.unban_user_cmd(conf, msg).await,
                "mute" => self.mute_user_cmd(conf, msg).await,
                "unmute" => self.unmute_user_cmd(conf, msg).await,

                "appeal" => self.appeal_cmd(conf, msg).await,

                "yaml" => self.yaml_cmd(conf, msg).await,
                _ => Ok(()),
            } {
                Ok(_) => {}
                Err(e) => {
                    let msg_json = serde_json::to_value(msg).unwrap_or_default();
                    let args = content.split_whitespace().collect::<Vec<&str>>();
                    let span = tracing::error_span!("Command error", error = e, msg = ?msg_json, command = cmd_name, args = ?args);
                    span.in_scope(|| {
                        tracing::error!("");
                    });
                }
            }
        }

        // config not required commands
        match cmd_name {
            "ping" => {
                let msg_time = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("?")
                    .as_micros() as i64
                    - msg.timestamp.as_micros())
                    / 1000;
                let ping = format!("Ping: `{}ms`", msg_time.to_string());
                self.rest
                    .create_message(msg.channel_id)
                    .content(ping.as_str())?
                    .await?;
                Ok(())
            }

            "botinfo" => self.bot_info(msg).await,
            "setup" => self.setup_cmd(msg).await,

            _ => Ok(()),
        }
    }

    async fn get_self_id(&self) -> Option<Id<UserMarker>> {
        if let Some(user) = self.cache.current_user() {
            return Some(user.id);
        }

        if let Ok(user) = self.rest.current_user().await {
            if let Ok(user) = user.model().await {
                return Some(user.id);
            }
        }

        None
    }
}
