use twilight_model::channel::Message;

use crate::{handlers::Handler, mongo::mongo::Config, util::permissions};

impl Handler {
    pub async fn process_cmd(&self, conf: &Config, msg: &Message)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get the prefix from config, TODO: if it's not set, use the bot mention

        let prefix = match &conf.prefix {
            Some(prefix) => prefix,
            None => return Ok(())
        };

        if !msg.content.starts_with(prefix) {
            return Ok(());
        }

        // Get the command name from the msg content from whats between the prefix and the next space
        let content = msg.content.trim_start_matches(prefix);

        let cmd_name = match content.split_whitespace().next() {
            Some(cmd_name) => cmd_name,
            None => return Ok(())
        };

        match cmd_name {
            "ping" => {
                let msg_time = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("?").as_micros() as i64 - msg.timestamp.as_micros()) / 1000;
                let ping = format!("Ping: `{}ms`", msg_time.to_string());
                self.rest.create_message(msg.channel_id).content(ping.as_str())?.exec().await?;
                Ok(())
            },

            "getlvl" => {
                let author_id = msg.author.id.to_string();
                let roles = match &msg.member {
                    Some(member) => Some(&member.roles),
                    None => None
                };
                let lvl = permissions::get_user_level(conf, roles, &author_id);
                if lvl == 0 {
                    self.rest.create_message(msg.channel_id).content(format!("you're retarded, fuck off").as_str())?.exec().await?;
                }
                self.rest.create_message(msg.channel_id).content(format!("Level: `{}`", lvl).as_str())?.exec().await?;
                Ok(())
            },

            "getcmdlvl" => {
                let cmd_name = match content.split_whitespace().nth(1) {
                    Some(cmd_name) => cmd_name,
                    None => return Ok(())
                };
                let lvl = permissions::get_permission(conf, cmd_name)?;
                self.rest.create_message(msg.channel_id).content(format!("Cmd Level: `{}`", lvl).as_str())?.exec().await?;
                Ok(())
            },

            "userinfo" => self.user_info_cmd(conf, msg).await,
            "guildinfo" => self.guild_info_cmd(conf, msg).await,

            "search" => self.search_user_cmd(conf, msg, false).await,
            "deepsearch" => self.search_user_cmd(conf, msg, true).await,
            "reason" => self.update_reason_cmd(conf, msg).await,
            "duration" => self.update_duration_cmd(&conf, msg).await,
            "remove" => self.remove_punishment_cmd(conf, msg).await,

            "strike" => self.strike_user_cmd(&conf, msg).await,
            "kick" => self.kick_user_cmd(&conf, msg).await,
            "ban" => self.ban_user_cmd(&conf, msg).await,
            "unban" => self.unban_user_cmd(&conf, msg).await,
            "mute" => self.mute_user_cmd(&conf, msg).await,
            "unmute" => self.unmute_user_cmd(&conf, msg).await,

            _ => Ok(())
        }
    }

}