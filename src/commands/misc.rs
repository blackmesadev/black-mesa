use std::str::FromStr;

use regex::Regex;
use twilight_model::{channel::{Message, embed::{Embed, EmbedField}}, id::Id};
use lazy_static::lazy_static;

use crate::{handlers::Handler, util::{permissions, snowflakes::snowflake_to_unix}, mongo::mongo::Config, VERSION};

impl Handler {
    pub async fn user_info_cmd(&self, conf: &Config, msg: &Message)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let author_id = msg.author.id.to_string();
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None
        };

        let content = &msg.content;
        lazy_static! {
            static ref RE: Regex = Regex::new(r"([0-9]{17,19})").unwrap();
        }
        let mut id_list: Vec<String> = RE.find_iter(content).map(|m| m.as_str().to_string()).collect();
        if id_list.len() == 0 {
            id_list.push(msg.author.id.to_string());
        }
        let id = &id_list[0];
        if id == "" {
            self.rest.create_message(msg.channel_id).content("No user id found")?.exec().await?;
        }

        let mut perm = permissions::PERMISSION_USERINFO;

        if id == &author_id {
            perm = permissions::PERMISSION_USERINFOSELF;
        }

        let ok = permissions::check_permission(conf, roles, &author_id, vec![perm]);
        if !ok {
            self.rest.create_message(msg.channel_id)
            .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", perm).as_str())?
            .exec()
            .await?;
            return Ok(());
        }

        let guild_id = match &msg.guild_id {
            Some(id) => id,
            None => return Ok(())
        };

        let guild = self.rest.guild(*guild_id).exec().await?.model().await?;

        let member = match self.rest.guild_member(*guild_id, Id::from_str(id)?).exec().await {
            Ok(member) => member.model().await?,
            Err(_) => {
                self.rest.create_message(msg.channel_id).content("<:mesaCross:832350526414127195> Member not found.")?.exec().await?;
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
                    format!("https://cdn.discordapp.com/avatars/{}/{}.gif", user_id, hash)
                } else {
                    format!("https://cdn.discordapp.com/avatars/{}/{}.png", user_id, hash)
                }
            }
            None => {
                format!("https://cdn.discordapp.com/embed/avatars/{}.png", member.user.discriminator % 5)
            }
        };

        let mut fields: Vec<EmbedField> = vec![];

        match &member.nick {
            Some(nick) => {
                fields.push(EmbedField{
                    name: "Nickname".to_string(),
                    value: nick.to_string(),
                    inline: true
                });
            }
            None => {}
        }

        fields.append(&mut vec![
            EmbedField{
                name: "ID".to_string(),
                value: format!("`{}`", user_id),
                inline: true
            },
            EmbedField{
                name: "Created".to_string(),
                value: format!("<t:{}:f>", (snowflake_to_unix(member.user.id)/1000)),
                inline: true
            },
            EmbedField{
                name: "Joined".to_string(),
                value: format!("<t:{}:f>", member.joined_at.as_secs()),
                inline: true
            },
            EmbedField{
                name: "Top Role".to_string(),
                value: format!("<@&{}>", top_role.id),
                inline: true
            }
        ]);

        let embeds = vec![Embed {
            title: Some(format!("{}#{:04}'s User Info", &member.user.name, &member.user.discriminator)),
            description: None,
            color: Some(0),
            footer: Some(twilight_model::channel::embed::EmbedFooter { 
                icon_url: None,
                proxy_icon_url: None,
                text: format!("Black Mesa v{} by Tyler#0911 written in Rust", VERSION)
            }),
            fields,
            kind: "rich".to_string(),
            author: None,
            image: None,
            provider: None,
            thumbnail: Some(twilight_model::channel::embed::EmbedThumbnail {
                height: None,
                proxy_url: None,
                url: icon_url,
                width: None
            }),
            timestamp: None,
            url: None,
            video: None
        }];

        self.rest.create_message(msg.channel_id).embeds(&embeds)?.exec().await?;

        Ok(())
    }

    pub async fn guild_info_cmd(&self, _conf: &Config, msg: &Message)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let guild = self.rest.guild(match msg.guild_id {
            Some(id) => id,
            None => return Ok(())
        }).exec().await?.model().await?;

        let mut fields = vec![
            EmbedField{
                name: "Name".to_string(),
                value: guild.name.to_string(),
                inline: true
            },
            EmbedField{
                name: "ID".to_string(),
                value: format!("`{}`", guild.id.to_string()),
                inline: true
            },
            EmbedField{
                name: "Created".to_string(),
                value: format!("<t:{}:f>", snowflake_to_unix(guild.id)/1000),
                inline: true
            },
            EmbedField{
                name: "Owner".to_string(),
                value: format!("<@{}>", guild.owner_id.to_string()),
                inline: true
            },
            EmbedField{
                name: "Role Count".to_string(),
                value:  guild.roles.len().to_string(),
                inline: true
            },
            EmbedField{
                name: "Emoji Count".to_string(),
                value:  guild.emojis.len().to_string(),
                inline: true
            },
        ];

        match &guild.max_members {
            Some(max) => {
                fields.push(EmbedField{
                    name: "Max Members".to_string(),
                    value:  max.to_string(),
                    inline: true
                });
            },
            None => {}
        }

        match &guild.member_count {
            Some(count) => {
                fields.push(EmbedField{
                    name: "Member Count".to_string(),
                    value: count.to_string(),
                    inline: true
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
            None => {
                "".to_string()
            }
        };

        let embeds = vec![Embed {
            title: Some(format!("{}'s Guild Info", &guild.name)),
            description: None,
            color: Some(0),
            footer: Some(twilight_model::channel::embed::EmbedFooter { 
                icon_url: None,
                proxy_icon_url: None,
                text: format!("Black Mesa v{} by Tyler#0911 written in Rust", VERSION)
            }),
            fields,
            kind: "rich".to_string(),
            author: None,
            image: None,
            provider: None,
            thumbnail: Some(twilight_model::channel::embed::EmbedThumbnail {
                height: None,
                proxy_url: None,
                url: icon_url,
                width: None
            }),
            timestamp: None,
            url: None,
            video: None
        }];

        self.rest.create_message(msg.channel_id).embeds(&embeds)?.exec().await?;

        Ok(())
    }
}