use std::collections::HashMap;

use twilight_model::{channel::Message, guild};

use crate::{handlers::Handler, util::permissions};

use super::{Group, Guild};

impl Handler {
    pub async fn setup_cmd(
        &self,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let guild_id = match &msg.guild_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let member = match &msg.member {
            Some(member) => member,
            None => return Ok(()),
        };

        let roles = self.rest.roles(*guild_id).await?.model().await?;

        let mut ok = false;
        for role in roles {
            if member.roles.contains(&role.id) {
                if role.permissions.contains(guild::Permissions::ADMINISTRATOR) {
                    ok = true;
                    break;
                }
            }
        }

        if !ok {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    format!(
                        "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                        permissions::PERMISSION_SETUP
                    )
                    .as_str(),
                )?
                .await?;

            return Ok(());
        }

        let guild = self.db.get_guild(&guild_id.to_string()).await?;

        if guild.is_some() {
            self.rest
                .create_message(msg.channel_id)
                .content("This guild is already setup!")?
                .await?;

            return Ok(());
        }

        let mut conf = Guild::new(guild_id.to_string());

        conf.config.prefix = Some("!".to_string());

        let mut default_groups = HashMap::new();
        default_groups.insert(
            "default".to_string(),
            Group {
                permissions: vec!["guild".to_string(), "music.play".to_string()],
                inherit: vec![],
                priority: 0,
            },
        );
        default_groups.insert(
            "dj".to_string(),
            Group {
                permissions: vec!["music".to_string()],
                inherit: vec!["default".to_string()],
                priority: 10,
            },
        );
        default_groups.insert(
            "moderator".to_string(),
            Group {
                permissions: vec!["moderation".to_string()],
                inherit: vec!["dj".to_string()],
                priority: 50,
            },
        );
        default_groups.insert(
            "admin".to_string(),
            Group {
                permissions: vec!["admin".to_string()],
                inherit: vec!["moderator".to_string()],
                priority: 100,
            },
        );

        conf.config.groups = Some(default_groups);

        self.db.create_guild(&conf).await?;

        self.rest
            .create_message(msg.channel_id)
            .content("This guild has been setup!")?
            .await?;

        Ok(())
    }
}
