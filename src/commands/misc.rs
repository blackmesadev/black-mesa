use std::str::FromStr;

use twilight_model::{
    channel::{
        message::{
            embed::{EmbedField, EmbedFooter, EmbedThumbnail},
            Embed,
        },
        Message,
    },
    http::attachment::Attachment,
    id::Id,
};

use crate::{config::Config, handlers::Handler, util, util::permissions, VERSION};

impl Handler {
    pub async fn user_info_cmd(
        &self,
        conf: &Config,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let author_id = msg.author.id.to_string();
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None,
        };

        let content = &msg.content;

        let mut id_list: Vec<String> = util::regex::SNOWFLAKE
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect();
        if id_list.len() == 0 {
            id_list.push(msg.author.id.to_string());
        }
        let id = &id_list[0];
        if id == "" {
            self.rest
                .create_message(msg.channel_id)
                .content("No user id found")?
                .await?;
        }

        let perm = if id == &author_id {
            permissions::PERMISSION_USERINFOSELF
        } else {
            permissions::PERMISSION_USERINFO
        };

        let ok = permissions::check_permission(conf, roles, msg.author.id, perm);
        if !ok {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    format!(
                        "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                        perm
                    )
                    .as_str(),
                )?
                .await?;
            return Ok(());
        }

        let guild_id = match &msg.guild_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let guild = self.rest.guild(*guild_id).await?.model().await?;

        let member = match self.rest.guild_member(*guild_id, Id::from_str(id)?).await {
            Ok(member) => member.model().await?,
            Err(_) => {
                self.rest
                    .create_message(msg.channel_id)
                    .content("<:mesaCross:832350526414127195> Member not found.")?
                    .await?;
                return Ok(());
            }
        };

        let user_id = member.user.id.to_string();

        let mut top_role = &guild.roles[0];
        let mut highest = 0u8; // theres only a max of 250 roles so a u8 is enough, yes im that pedantic
        for role in &guild.roles {
            for member_role in &member.roles {
                if role.id == *member_role && highest < role.position as u8 {
                    top_role = role;
                    highest = role.position as u8; // fuck you. im using my u8.
                    break;
                }
            }
        }

        let icon_url = match &member.user.avatar {
            Some(hash) => {
                if hash.is_animated() {
                    format!(
                        "https://cdn.discordapp.com/avatars/{}/{}.gif",
                        user_id, hash
                    )
                } else {
                    format!(
                        "https://cdn.discordapp.com/avatars/{}/{}.png",
                        user_id, hash
                    )
                }
            }
            None => {
                format!(
                    "https://cdn.discordapp.com/embed/avatars/{}.png",
                    member.user.discriminator % 5
                )
            }
        };

        let mut fields: Vec<EmbedField> = vec![];

        match &member.nick {
            Some(nick) => {
                fields.push(EmbedField {
                    name: "Nickname".to_string(),
                    value: nick.to_string(),
                    inline: true,
                });
            }
            None => {}
        }

        fields.append(&mut vec![
            EmbedField {
                name: "ID".to_string(),
                value: format!("`{}`", user_id),
                inline: true,
            },
            EmbedField {
                name: "Created".to_string(),
                value: format!(
                    "<t:{}:f>",
                    (util::snowflakes::snowflake_to_unix(member.user.id) / 1000)
                ),
                inline: true,
            },
            EmbedField {
                name: "Joined".to_string(),
                value: format!("<t:{}:f>", member.joined_at.as_secs()),
                inline: true,
            },
            EmbedField {
                name: "Top Role".to_string(),
                value: format!("<@&{}>", top_role.id),
                inline: true,
            },
        ]);

        let embeds = vec![Embed {
            title: Some(format!(
                "{}'s User Info",
                util::format_username(&member.user.name, member.user.discriminator)
            )),
            description: None,
            color: Some(0),
            footer: Some(EmbedFooter {
                icon_url: None,
                proxy_icon_url: None,
                text: format!("Black Mesa v{}", VERSION),
            }),
            fields,
            kind: "rich".to_string(),
            author: None,
            image: None,
            provider: None,
            thumbnail: Some(EmbedThumbnail {
                height: None,
                proxy_url: None,
                url: icon_url,
                width: None,
            }),
            timestamp: None,
            url: None,
            video: None,
        }];

        self.rest
            .create_message(msg.channel_id)
            .embeds(&embeds)?
            .await?;

        Ok(())
    }

    pub async fn guild_info_cmd(
        &self,
        conf: &Config,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None,
        };

        let ok = permissions::check_permission(
            conf,
            roles,
            msg.author.id,
            permissions::PERMISSION_GUILDINFO,
        );
        if !ok {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    format!(
                        "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                        permissions::PERMISSION_GUILDINFO
                    )
                    .as_str(),
                )?
                .await?;
            return Ok(());
        }

        let guild = self
            .rest
            .guild(match msg.guild_id {
                Some(id) => id,
                None => return Ok(()),
            })
            .await?
            .model()
            .await?;

        let mut fields = vec![
            EmbedField {
                name: "Name".to_string(),
                value: guild.name.to_string(),
                inline: true,
            },
            EmbedField {
                name: "ID".to_string(),
                value: format!("`{}`", guild.id.to_string()),
                inline: true,
            },
            EmbedField {
                name: "Created".to_string(),
                value: format!(
                    "<t:{}:f>",
                    util::snowflakes::snowflake_to_unix(guild.id) / 1000
                ),
                inline: true,
            },
            EmbedField {
                name: "Owner".to_string(),
                value: format!("<@{}>", guild.owner_id.to_string()),
                inline: true,
            },
            EmbedField {
                name: "Role Count".to_string(),
                value: guild.roles.len().to_string(),
                inline: true,
            },
            EmbedField {
                name: "Emoji Count".to_string(),
                value: guild.emojis.len().to_string(),
                inline: true,
            },
        ];

        match &guild.max_members {
            Some(max) => {
                fields.push(EmbedField {
                    name: "Max Members".to_string(),
                    value: max.to_string(),
                    inline: true,
                });
            }
            None => {}
        }

        match &guild.member_count {
            Some(count) => {
                fields.push(EmbedField {
                    name: "Member Count".to_string(),
                    value: count.to_string(),
                    inline: true,
                });
            }
            None => {}
        }

        let icon_url = match &guild.icon {
            Some(hash) => {
                if hash.is_animated() {
                    format!("https://cdn.discordapp.com/icons/{}/{}.gif", guild.id, hash)
                } else {
                    format!("https://cdn.discordapp.com/icons/{}/{}.png", guild.id, hash)
                }
            }
            None => "".to_string(),
        };

        let embeds = vec![Embed {
            title: Some(format!("{}'s Guild Info", &guild.name)),
            description: None,
            color: Some(0),
            footer: Some(EmbedFooter {
                icon_url: None,
                proxy_icon_url: None,
                text: format!("Black Mesa v{}", VERSION),
            }),
            fields,
            kind: "rich".to_string(),
            author: None,
            image: None,
            provider: None,
            thumbnail: Some(EmbedThumbnail {
                height: None,
                proxy_url: None,
                url: icon_url,
                width: None,
            }),
            timestamp: None,
            url: None,
            video: None,
        }];

        self.rest
            .create_message(msg.channel_id)
            .embeds(&embeds)?
            .await?;

        Ok(())
    }

    pub async fn bot_info(
        &self,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut fields = vec![];

        let thumbnail = match self.cache.current_user() {
            Some(user) => {
                fields.push(EmbedField {
                    name: "Bot Name".to_string(),
                    value: format!("{}", util::format_username(&user.name, user.discriminator)),
                    inline: true,
                });
                fields.push(EmbedField {
                    name: "Bot ID".to_string(),
                    value: format!("`{}`", user.id.to_string()),
                    inline: true,
                });
                fields.push(EmbedField {
                    name: "Bot Created".to_string(),
                    value: format!(
                        "<t:{}:f>",
                        util::snowflakes::snowflake_to_unix(user.id) / 1000
                    ),
                    inline: true,
                });

                let icon_url = match &user.avatar {
                    Some(hash) => {
                        if hash.is_animated() {
                            format!(
                                "https://cdn.discordapp.com/avatars/{}/{}.gif",
                                user.id.to_string(),
                                hash
                            )
                        } else {
                            format!(
                                "https://cdn.discordapp.com/avatars/{}/{}.png",
                                user.id.to_string(),
                                hash
                            )
                        }
                    }
                    None => {
                        format!(
                            "https://cdn.discordapp.com/embed/avatars/{}.png",
                            user.discriminator % 5
                        )
                    }
                };

                Some(EmbedThumbnail {
                    height: None,
                    proxy_url: None,
                    url: icon_url,
                    width: None,
                })
            }
            None => None,
        };

        fields.append(&mut vec![
            EmbedField {
                name: "Total Guilds".to_string(),
                value: format!(
                    "{}",
                    self.cache.stats().guilds() + self.cache.stats().unavailable_guilds()
                ),
                inline: true,
            },
            EmbedField {
                name: "Version".to_string(),
                value: format!("v{}", VERSION),
                inline: true,
            },
            EmbedField {
                name: "Memory Usage".to_string(),
                value: format!(
                    "`{:.3} MB`",
                    match self.redis.get_memory_usage().await {
                        Ok(usage) => usage as f64 / 1024.0 / 1024.0,
                        Err(e) => {
                            tracing::warn!("Failed to get memory usage: {}", e);
                            0.0
                        }
                    }
                ),
                inline: true,
            },
        ]);

        let embeds = vec![Embed {
            title: Some("Black Mesa Info".to_string()),
            description: None,
            color: Some(0),
            footer: Some(EmbedFooter {
                icon_url: None,
                proxy_icon_url: None,
                text: format!("Black Mesa v{}", VERSION),
            }),
            fields,
            kind: "rich".to_string(),
            author: None,
            image: None,
            provider: None,
            thumbnail,
            timestamp: None,
            url: None,
            video: None,
        }];

        self.rest
            .create_message(msg.channel_id)
            .embeds(&embeds)?
            .await?;

        Ok(())
    }

    pub async fn yaml_cmd(
        &self,
        conf: &Config,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let yaml_out = if let Ok(yaml) = conf.to_yaml() {
            yaml
        } else {
            "`Failed to get config as YAML`".to_string()
        };

        // upload string as a file

        let attachment =
            Attachment::from_bytes(String::from("config.yaml"), yaml_out.as_bytes().to_vec(), 1);

        self.rest
            .create_message(msg.channel_id)
            .content("Here is the current config as YAML")?
            .attachments(&vec![attachment])?
            .await?;

        Ok(())
    }
}
