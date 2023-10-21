use std::{collections::HashMap, str::FromStr};

use mongodb::results::UpdateResult;
use twilight_mention::Mention;
use twilight_model::{
    channel::{
        message::{
            embed::{EmbedField, EmbedFooter},
            Embed,
        },
        Message,
    },
    id::Id,
};

use crate::{
    config::Config,
    handlers::Handler,
    moderation::moderation::Punishment,
    util,
    util::{duration::Duration, permissions},
    VERSION,
};

impl Handler {
    // UTILITY COMMANDS

    pub async fn search_user_cmd(
        &self,
        conf: &Config,
        msg: &Message,
        deep: bool,
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

        let mut perms = if id == &author_id {
            vec![permissions::PERMISSION_SEARCHSELF]
        } else {
            vec![permissions::PERMISSION_SEARCH]
        };

        if deep {
            perms.push(permissions::PERMISSION_DEEPSEARCH);
        }

        let perm_str = perms.join("`, `");

        let ok = permissions::check_permission_many(conf, roles, msg.author.id, perms);
        if !ok {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    format!(
                        "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                        perm_str
                    )
                    .as_str(),
                )?
                .await?;
            return Ok(());
        }

        let punishments = match self
            .db
            .get_punishments(
                &match &msg.guild_id {
                    Some(id) => id.to_string(),
                    None => return Ok(()),
                },
                &id,
            )
            .await
        {
            Ok(p) => p,
            Err(e) => {
                self.rest
                    .create_message(msg.channel_id)
                    .content("Error getting punishments")?
                    .await?;
                // Return Ok here because an error here shouldn't cause further issue, this can be manually investigated.
                tracing::warn!("Error getting punishments: {:?}", e);
                return Ok(());
            }
        };

        let embed_footer = EmbedFooter {
            text: format!("Black Mesa v{}", VERSION),
            icon_url: None,
            proxy_icon_url: None,
        };

        let mut fields: Vec<EmbedField> = Vec::new();

        let mut total_punishments = punishments.len();

        for punishment in &punishments {
            if punishment.expired && !deep {
                total_punishments -= 1;
                continue;
            }

            let expiry: String;
            let exp = punishment.expires.unwrap_or(0);
            if exp == 0 {
                expiry = "Never".to_string();
            } else {
                expiry = format!("<t:{}:f> (<t:{}:R>)", exp, exp);
            }
            let mut value = format!("**Reason:** `{}`\n**Issued By:** <@{}>\n**Expires:** {}\n**Created:** <t:{}:f>\n**UUID:** `{}`",
                punishment.reason.as_ref().unwrap_or(&"None".to_string()),
                punishment.issuer,
                expiry,
                punishment.oid.timestamp().timestamp_millis()/1000,
                punishment.uuid);

            if deep {
                value += &format!("\n**Active:** `{}`", !punishment.expired);
            }

            fields.push(EmbedField {
                name: punishment.typ.pretty_string(),
                value,
                inline: true,
            });
        }

        let user = match self.cache.user(Id::from_str(id)?) {
            Some(user) => user.clone(),
            None => self.rest.user(Id::from_str(&id)?).await?.model().await?,
        };

        let embeds = vec![Embed {
            title: Some(format!(
                "{}'s Infraction log.",
                util::format_username(&user.name, user.discriminator)
            )),
            description: Some(format!(
                "{} has {} infractions.",
                user.mention(),
                total_punishments
            )),
            color: Some(0),
            footer: Some(embed_footer),
            fields: fields,
            kind: "rich".to_string(),
            author: None,
            image: None,
            provider: None,
            thumbnail: None,
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

    pub async fn update_reason_cmd(
        &self,
        conf: &Config,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = &msg.content;

        let uuid_list: Vec<String> = util::regex::UUID
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect();

        let mut punishment_list: Vec<Punishment> = Vec::new();

        let guild_id = match &msg.guild_id {
            Some(id) => id.to_string(),
            None => return Ok(()),
        };

        let author_id = msg.author.id.to_string();
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None,
        };

        let splitby = &match uuid_list.len() {
            0 => {
                let punishment = match self
                    .db
                    .get_last_action(&guild_id.to_string(), &msg.author.id.to_string())
                    .await
                {
                    Ok(p) => match p {
                        Some(p) => p,
                        None => {
                            self.rest
                                .create_message(msg.channel_id)
                                .content(
                                    "<:mesaCross:832350526414127195> Unable to get last action",
                                )?
                                .await?;
                            return Ok(());
                        }
                    },
                    Err(_) => {
                        self.rest
                            .create_message(msg.channel_id)
                            .content("<:mesaCross:832350526414127195> Unable to get last action")?
                            .await?;
                        return Ok(());
                    }
                };
                punishment_list.push(punishment);
                match content.split_once(" ") {
                    Some(s) => s.0.to_string() + " ",
                    None => {
                        self.rest.create_message(msg.channel_id).content("<:mesaCommand:832350527131746344> `reason [uuid:uuid] [reason:string...]`")?.await?;
                        return Ok(());
                    }
                }
            }
            1 => {
                let uuid = uuid_list.last().unwrap().to_string();
                let punishment = match self
                    .db
                    .get_action_by_uuid(&guild_id.to_string(), &uuid)
                    .await
                {
                    Ok(a) => {
                        match a {
                            Some(a) => a,
                            None => {
                                self.rest.create_message(msg.channel_id)
                                    .content(format!("<:mesaCross:832350526414127195> Unable to get action {}", uuid)
                                    .as_str())?
                                    .await?;

                                return Ok(());
                            }
                        }
                    }
                    Err(_) => {
                        self.rest
                            .create_message(msg.channel_id)
                            .content(
                                format!(
                                    "<:mesaCross:832350526414127195> Unable to get action {}",
                                    uuid
                                )
                                .as_str(),
                            )?
                            .await?;

                        return Ok(());
                    }
                };
                punishment_list.push(punishment);
                content.split(" ").collect::<Vec<&str>>()[1].to_string() + " "
            }
            _ => {
                self.rest
                    .create_message(msg.channel_id)
                    .content(
                        "<:mesaCommand:832350527131746344> `reason [uuid:uuid] [reason:string...]`",
                    )?
                    .await?;
                return Ok(());
            }
        };

        let reason = match content.to_string().split(splitby).collect::<Vec<&str>>() {
            mut vec if vec.len() > 1 => {
                vec.remove(0);
                vec.join("").trim().to_string()
            }
            _ => {
                self.rest
                    .create_message(msg.channel_id)
                    .content(
                        "<:mesaCommand:832350527131746344> `reason [uuid:uuid] [reason:string...]`",
                    )?
                    .await?;
                return Ok(());
            }
        };

        let punishment = &punishment_list[0]; // This is safe because we check the length of the list above.

        if &punishment.issuer == &author_id {
            let ok = permissions::check_permission(
                conf,
                roles,
                msg.author.id,
                permissions::PERMISSION_UPDATESELF,
            );
            if !ok {
                self.rest
                    .create_message(msg.channel_id)
                    .content(
                        format!(
                            "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                            permissions::PERMISSION_UPDATESELF
                        )
                        .as_str(),
                    )?
                    .await?;
                return Ok(());
            }
        } else {
            let ok = permissions::check_permission(
                conf,
                roles,
                msg.author.id,
                permissions::PERMISSION_UPDATE,
            );
            if !ok {
                self.rest
                    .create_message(msg.channel_id)
                    .content(
                        format!(
                            "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                            permissions::PERMISSION_UPDATE
                        )
                        .as_str(),
                    )?
                    .await?;
                return Ok(());
            }

            // now check if the user is trying to remove a punishment of someone with a higher level

            let guild_id_marker = msg.guild_id.unwrap();
            let original_issuer_id = Id::from_str(punishment.issuer.as_str())?;
            let original_issuer_roles = match self.cache.member(guild_id_marker, original_issuer_id)
            {
                Some(member) => member.to_owned().roles().to_vec(),
                None => {
                    self.rest
                        .guild_member(guild_id_marker, original_issuer_id)
                        .await?
                        .model()
                        .await?
                        .roles
                }
            };

            let author_roles = match self.cache.member(guild_id_marker, msg.author.id) {
                Some(member) => member.to_owned().roles().to_vec(),
                None => {
                    self.rest
                        .guild_member(guild_id_marker, msg.author.id)
                        .await?
                        .model()
                        .await?
                        .roles
                }
            };

            let user_groups = match permissions::get_user_groups(
                conf,
                original_issuer_id,
                Some(&original_issuer_roles),
            ) {
                Ok(groups) => groups,
                Err(_) => HashMap::new(),
            };

            let original_issuer_level = permissions::get_user_priority(&user_groups);

            let author_groups =
                match permissions::get_user_groups(conf, msg.author.id, Some(&author_roles)) {
                    Ok(groups) => groups,
                    Err(_) => HashMap::new(),
                };

            let author_level = permissions::get_user_priority(&author_groups);

            let update_higher_level_action = match &conf.modules {
                Some(modules) => match &modules.moderation {
                    Some(m) => m.update_higher_level_action,
                    None => false,
                },
                None => false,
            };

            if original_issuer_level >= author_level && !update_higher_level_action {
                self.rest.create_message(msg.channel_id)
                    .content("<:mesaCross:832350526414127195> You do not have permission to update this punishment as it is of a user of equal or higher level")?
                    .await?;

                return Ok(());
            }
        }

        let res = self
            .db
            .update_punishment(
                &punishment.uuid,
                &guild_id,
                Some(reason.clone()),
                None,
                None,
            )
            .await?;

        match res {
            UpdateResult {
                matched_count: 1,
                modified_count: 1,
                ..
            } => {
                self.rest
                    .create_message(msg.channel_id)
                    .content(
                        format!(
                            "<:mesaCheck:832350526729224243> Updated punishment `{}`",
                            punishment.uuid
                        )
                        .as_str(),
                    )?
                    .await?;
            }
            UpdateResult { .. } => {
                self.rest
                    .create_message(msg.channel_id)
                    .content(
                        format!(
                            "<:mesaCross:832350526414127195> Unable to update punishment `{}`",
                            punishment.uuid
                        )
                        .as_str(),
                    )?
                    .await?;
            }
        }

        if let Some(modules) = &conf.modules {
            if let Some(logging) = &modules.logging {
                self.log_update_action(
                    &logging,
                    &msg.author.id.to_string(),
                    punishment,
                    None,
                    Some(&reason),
                )
                .await;
            }
        }

        Ok(())
    }

    pub async fn update_duration_cmd(
        &self,
        conf: &Config,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = &msg.content;

        let uuid_list: Vec<String> = util::regex::UUID
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect();

        let mut punishment_list: Vec<Punishment> = Vec::new();

        let guild_id = match &msg.guild_id {
            Some(id) => id.to_string(),
            None => return Ok(()),
        };

        let author_id = msg.author.id.to_string();
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None,
        };

        let splitby = &match uuid_list.len() {
            0 => {
                let punishment = match self
                    .db
                    .get_last_action(&guild_id.to_string(), &msg.author.id.to_string())
                    .await
                {
                    Ok(p) => match p {
                        Some(p) => p,
                        None => {
                            self.rest
                                .create_message(msg.channel_id)
                                .content(
                                    "<:mesaCross:832350526414127195> Unable to get last action",
                                )?
                                .await?;
                            return Ok(());
                        }
                    },
                    Err(_) => {
                        self.rest
                            .create_message(msg.channel_id)
                            .content("<:mesaCross:832350526414127195> Unable to get last action")?
                            .await?;
                        return Ok(());
                    }
                };
                punishment_list.push(punishment);
                match content.split_once(" ") {
                    Some(s) => s.0.to_string() + " ",
                    None => {
                        self.rest.create_message(msg.channel_id).content("<:mesaCommand:832350527131746344> `duration [uuid:uuid] [time:duration]`")?.await?;
                        return Ok(());
                    }
                }
            }
            1 => {
                let uuid = uuid_list.last().unwrap().to_string();
                let punishment = match self
                    .db
                    .get_action_by_uuid(&guild_id.to_string(), &uuid)
                    .await
                {
                    Ok(a) => {
                        match a {
                            Some(a) => a,
                            None => {
                                self.rest.create_message(msg.channel_id)
                                    .content(format!("<:mesaCross:832350526414127195> Unable to get action {}", uuid)
                                    .as_str())?
                                    .await?;

                                return Ok(());
                            }
                        }
                    }
                    Err(_) => {
                        self.rest
                            .create_message(msg.channel_id)
                            .content(
                                format!(
                                    "<:mesaCross:832350526414127195> Unable to get action {}",
                                    uuid
                                )
                                .as_str(),
                            )?
                            .await?;

                        return Ok(());
                    }
                };
                punishment_list.push(punishment);
                content.split(" ").collect::<Vec<&str>>()[1].to_string() + " "
            }
            _ => {
                self.rest
                    .create_message(msg.channel_id)
                    .content(
                        "<:mesaCommand:832350527131746344> `duration [uuid:uuid] [time:duration]`",
                    )?
                    .await?;
                return Ok(());
            }
        };

        let dur_split = match content.to_string().split(splitby).collect::<Vec<&str>>() {
            mut vec if vec.len() > 1 => {
                vec.remove(0);
                vec.join("").trim().to_string()
            }
            _ => {
                self.rest
                    .create_message(msg.channel_id)
                    .content(
                        "<:mesaCommand:832350527131746344> `duration [uuid:uuid] [time:duration]`",
                    )?
                    .await?;

                return Ok(());
            }
        };

        let duration = Duration::new(dur_split);

        let punishment = &punishment_list[0]; // This is safe because we check the length of the list above.

        if &punishment.issuer == &author_id {
            let ok = permissions::check_permission(
                conf,
                roles,
                msg.author.id,
                permissions::PERMISSION_UPDATESELF,
            );
            if !ok {
                self.rest
                    .create_message(msg.channel_id)
                    .content(
                        format!(
                            "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                            permissions::PERMISSION_UPDATESELF
                        )
                        .as_str(),
                    )?
                    .await?;

                return Ok(());
            }
        } else {
            let ok = permissions::check_permission(
                conf,
                roles,
                msg.author.id,
                permissions::PERMISSION_UPDATE,
            );
            if !ok {
                self.rest
                    .create_message(msg.channel_id)
                    .content(
                        format!(
                            "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                            permissions::PERMISSION_UPDATE
                        )
                        .as_str(),
                    )?
                    .await?;

                return Ok(());
            }

            // now check if the user is trying to update a punishment of someone with a higher level
            let guild_id_marker = msg.guild_id.unwrap();
            let original_issuer_id = Id::from_str(punishment.issuer.as_str())?;
            let original_issuer_roles = match self.cache.member(guild_id_marker, original_issuer_id)
            {
                Some(member) => member.to_owned().roles().to_vec(),
                None => {
                    self.rest
                        .guild_member(guild_id_marker, original_issuer_id)
                        .await?
                        .model()
                        .await?
                        .roles
                }
            };

            let author_roles = match self.cache.member(guild_id_marker, msg.author.id) {
                Some(member) => member.to_owned().roles().to_vec(),
                None => {
                    self.rest
                        .guild_member(guild_id_marker, msg.author.id)
                        .await?
                        .model()
                        .await?
                        .roles
                }
            };

            let user_groups = match permissions::get_user_groups(
                conf,
                original_issuer_id,
                Some(&original_issuer_roles),
            ) {
                Ok(groups) => groups,
                Err(_) => HashMap::new(),
            };

            let original_issuer_level = permissions::get_user_priority(&user_groups);

            let author_groups =
                match permissions::get_user_groups(conf, msg.author.id, Some(&author_roles)) {
                    Ok(groups) => groups,
                    Err(_) => HashMap::new(),
                };

            let author_level = permissions::get_user_priority(&author_groups);

            let update_higher_level_action = match &conf.modules {
                Some(modules) => match modules.moderation {
                    Some(ref mod_conf) => mod_conf.update_higher_level_action,
                    None => false,
                },
                None => false,
            };

            if original_issuer_level >= author_level && !update_higher_level_action {
                self.rest.create_message(msg.channel_id)
                    .content("<:mesaCross:832350526414127195> You do not have permission to update this punishment as it is of a user of equal or higher level")?
                    .await?;

                return Ok(());
            }
        }

        let res = self
            .db
            .update_punishment(
                &punishment.uuid,
                &guild_id,
                None,
                duration.to_unix_expiry(),
                None,
            )
            .await?;

        match res {
            UpdateResult {
                matched_count: 1,
                modified_count: 1,
                ..
            } => {
                self.rest
                    .create_message(msg.channel_id)
                    .content(
                        format!(
                            "<:mesaCheck:832350526729224243> Updated punishment `{}`",
                            punishment.uuid
                        )
                        .as_str(),
                    )?
                    .await?;
            }
            UpdateResult { .. } => {
                self.rest
                    .create_message(msg.channel_id)
                    .content(
                        format!(
                            "<:mesaCross:832350526414127195> Unable to update punishment `{}`",
                            punishment.uuid
                        )
                        .as_str(),
                    )?
                    .await?;
            }
        }

        if let Some(modules) = &conf.modules {
            if let Some(logging) = &modules.logging {
                self.log_update_action(
                    &logging,
                    &msg.author.id.to_string(),
                    punishment,
                    Some(&duration),
                    None,
                )
                .await;
            }
        }

        Ok(())
    }

    // MODERATION COMMANDS

    pub async fn strike_user_cmd(
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
            permissions::PERMISSION_STRIKE,
        );
        if !ok {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    format!(
                        "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                        permissions::PERMISSION_STRIKE
                    )
                    .as_str(),
                )?
                .await?;

            return Ok(());
        }

        let mut content = msg.content.clone();

        util::regex::EMOJI
            .captures_iter(&msg.content)
            .for_each(|cap| {
                let full_str = cap.get(0).unwrap().as_str();
                let emoji_name = cap.get(1).unwrap().as_str();
                content = content.replace(full_str, format!(":{}:", emoji_name).as_str());
            });

        let mut last_id = "";
        let id_list: Vec<String> = util::regex::SNOWFLAKE
            .captures_iter(&content)
            .map(|cap| {
                last_id = cap.get(0).unwrap().as_str();
                cap.get(1).unwrap().as_str().to_string()
            })
            .collect();

        if id_list.len() == 0 {
            self.rest.create_message(msg.channel_id)
                .content("<:mesaCommand:832350527131746344> `strike <target:user[]> [time:duration] [reason:string...]`")?
                .await?;

            return Ok(());
        }

        let mut duration = Duration::new(content.to_string());

        // check the string rather than calling .is_permenant() because if the user wishes to strike permanently, they should be able to do so.
        if duration.full_string.is_empty() {
            duration = Duration::new_no_str(match &conf.modules {
                Some(modules) => match &modules.moderation {
                    Some(m) => match &m.default_strike_duration {
                        Some(dur) => dur.to_string(),
                        None => "30d".to_string(),
                    },
                    None => "30d".to_string(),
                },
                None => "30d".to_string(),
            });
        }

        let mut splitby = last_id;

        if !duration.full_string.is_empty() {
            splitby = &duration.full_string;
        }

        let reason = match content.to_string().split(splitby).collect::<Vec<&str>>() {
            mut vec if vec.len() > 1 => {
                vec.remove(0);
                let r = vec.join("").trim().to_string();
                if r.is_empty() {
                    None
                } else {
                    Some(r)
                }
            }
            _ => None,
        };

        let duration_str = if duration.is_permenant() {
            "`Never`".to_string()
        } else {
            format!(
                "{} ({})",
                duration.to_discord_timestamp(),
                duration.to_discord_relative_timestamp()
            )
        };

        let resp = match reason {
            Some(ref reason) => {
                format!("<:mesaStrike:869663336843845752> Successfully striked {} expiring {} for reason `{}`", 
                    util::mentions::mentions_from_id_str_vec(&id_list), duration_str, reason)
            }
            None => format!(
                "<:mesaStrike:869663336843845752> Successfully striked {} expiring {}",
                util::mentions::mentions_from_id_str_vec(&id_list),
                duration_str
            ),
        };
        self.rest
            .create_message(msg.channel_id)
            .content(resp.as_str())?
            .await?;

        let mut uuids: Vec<String> = Vec::new();

        let reason = reason.as_ref();

        match msg.guild_id {
            Some(guild_id) => {
                for id in &id_list {
                    let punishment = self
                        .issue_strike(
                            conf,
                            &guild_id.to_string(),
                            id,
                            &msg.author.id.to_string(),
                            reason,
                            &duration,
                        )
                        .await?;
                    uuids.push(punishment.uuid);
                }
            }

            None => {}
        }

        if let Some(modules) = &conf.modules {
            if let Some(logging) = &modules.logging {
                for (i, id) in id_list.iter().enumerate() {
                    self.log_strike(
                        logging,
                        &msg.author.id.to_string(),
                        id,
                        reason,
                        &duration,
                        &match uuids.get(i) {
                            Some(uuid) => uuid.to_string(),
                            None => "".to_string(),
                        },
                    )
                    .await;
                }
            }
        }

        Ok(())
    }

    pub async fn kick_user_cmd(
        &self,
        conf: &Config,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None,
        };
        let ok =
            permissions::check_permission(conf, roles, msg.author.id, permissions::PERMISSION_KICK);
        if !ok {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    format!(
                        "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                        permissions::PERMISSION_KICK
                    )
                    .as_str(),
                )?
                .await?;

            return Ok(());
        }

        let mut content = msg.content.clone();

        util::regex::EMOJI
            .captures_iter(&msg.content)
            .for_each(|cap| {
                let full_str = cap.get(0).unwrap().as_str();
                let emoji_name = cap.get(1).unwrap().as_str();
                content = content.replace(full_str, format!(":{}:", emoji_name).as_str());
            });

        let mut last_id = "";
        let id_list: Vec<String> = util::regex::SNOWFLAKE
            .captures_iter(&content)
            .map(|cap| {
                last_id = cap.get(0).unwrap().as_str();
                cap.get(1).unwrap().as_str().to_string()
            })
            .collect();

        if id_list.len() == 0 {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    "<:mesaCommand:832350527131746344> `kick <target:user[]> [reason:string...]`",
                )?
                .await?;

            return Ok(());
        }

        let splitby = last_id;

        let reason = match content.to_string().split(splitby).collect::<Vec<&str>>() {
            mut vec if vec.len() > 1 => {
                vec.remove(0);
                let r = vec.join("").trim().to_string();
                if r.is_empty() {
                    None
                } else {
                    Some(r)
                }
            }
            _ => None,
        };

        let resp = match reason {
            Some(ref reason) => {
                format!(
                    "<:mesaKick:869665034312253460> Successfully kicked {} for reason `{}`",
                    util::mentions::mentions_from_id_str_vec(&id_list),
                    reason
                )
            }
            None => format!(
                "<:mesaKick:869665034312253460> Successfully kicked {}",
                util::mentions::mentions_from_id_str_vec(&id_list)
            ),
        };
        self.rest
            .create_message(msg.channel_id)
            .content(resp.as_str())?
            .await?;

        let mut uuids: Vec<String> = Vec::new();

        let reason = reason.as_ref();

        let appealable = (*conf)
            .modules
            .as_ref()
            .and_then(|modules| modules.appeals.as_ref())
            .map(|appeals| appeals.enabled)
            .unwrap_or(false);

        match msg.guild_id {
            Some(guild_id) => {
                for id in &id_list {
                    let punishment = self
                        .kick_user(
                            &guild_id.to_string(),
                            id,
                            &msg.author.id.to_string(),
                            reason,
                            None,
                            appealable,
                        )
                        .await?;
                    uuids.push(punishment.uuid);
                }
            }

            None => {}
        }
        if let Some(modules) = &conf.modules {
            if let Some(logging) = &modules.logging {
                for (i, id) in id_list.iter().enumerate() {
                    self.log_kick(
                        logging,
                        &msg.author.id.to_string(),
                        id,
                        reason.clone(),
                        &match uuids.get(i) {
                            Some(uuid) => uuid.to_string(),
                            None => "".to_string(),
                        },
                    )
                    .await;
                }
            }
        }

        Ok(())
    }

    pub async fn softban_user_cmd(
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
            permissions::PERMISSION_SOFTBAN,
        );
        if !ok {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    format!(
                        "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                        permissions::PERMISSION_SOFTBAN
                    )
                    .as_str(),
                )?
                .await?;

            return Ok(());
        }

        let mut content = msg.content.clone();

        util::regex::EMOJI
            .captures_iter(&msg.content)
            .for_each(|cap| {
                let full_str = cap.get(0).unwrap().as_str();
                let emoji_name = cap.get(1).unwrap().as_str();
                content = content.replace(full_str, format!(":{}:", emoji_name).as_str());
            });

        let mut last_id = "";
        let id_list: Vec<String> = util::regex::SNOWFLAKE
            .captures_iter(&content)
            .map(|cap| {
                last_id = cap.get(0).unwrap().as_str();
                cap.get(1).unwrap().as_str().to_string()
            })
            .collect();

        if id_list.len() == 0 {
            self.rest.create_message(msg.channel_id)
                .content("<:mesaCommand:832350527131746344> `softban <target:user[]> [delete:duration] [reason:string...]`")?
                .await?;

            return Ok(());
        }

        let duration = Duration::new(content.to_string());
        if duration.to_seconds() > 604800 {
            self.rest
                .create_message(msg.channel_id)
                .content("<:mesaCross:832350526414127195> Duration must be less than 7 days")?
                .await?;

            return Ok(());
        }

        let mut splitby = last_id;

        if !duration.full_string.is_empty() {
            splitby = &duration.full_string;
        }

        let reason = match content.to_string().split(splitby).collect::<Vec<&str>>() {
            mut vec if vec.len() > 1 => {
                vec.remove(0);
                let r = vec.join("").trim().to_string();
                if r.is_empty() {
                    None
                } else {
                    Some(r)
                }
            }
            _ => None,
        };

        let duration_str = format!(
            "{} ({})",
            duration.to_discord_timestamp(),
            duration.to_discord_relative_timestamp()
        );

        let resp = match reason {
            Some(ref reason) => {
                format!("<:mesaBan:869663336625733634> Successfully softbanned {} deleting `{}` worth of messages for reason `{}`", 
                    util::mentions::mentions_from_id_str_vec(&id_list), duration_str, reason)
            }
            None => format!(
                "<:mesaBan:869663336625733634> Successfully softbanned {} deleting `{}` worth of messages",
                util::mentions::mentions_from_id_str_vec(&id_list),
                duration_str
            ),
        };
        self.rest
            .create_message(msg.channel_id)
            .content(resp.as_str())?
            .await?;

        let mut uuids: Vec<String> = Vec::new();

        let reason = reason.as_ref();

        let appealable = (*conf)
            .modules
            .as_ref()
            .and_then(|modules| modules.appeals.as_ref())
            .map(|appeals| appeals.enabled)
            .unwrap_or(false);

        match msg.guild_id {
            Some(guild_id) => {
                for id in &id_list {
                    let punishment = self
                        .softban_user(
                            &guild_id.to_string(),
                            id,
                            &msg.author.id.to_string(),
                            &duration,
                            reason,
                            appealable,
                        )
                        .await?;
                    uuids.push(punishment.uuid);
                }
            }

            None => {}
        }

        if let Some(modules) = &conf.modules {
            if let Some(logging) = &modules.logging {
                for (i, id) in id_list.iter().enumerate() {
                    self.log_ban(
                        logging,
                        &msg.author.id.to_string(),
                        id,
                        reason,
                        &duration,
                        &match uuids.get(i) {
                            Some(uuid) => uuid.to_string(),
                            None => "".to_string(),
                        },
                    )
                    .await;
                }
            }
        }

        Ok(())
    }

    pub async fn ban_user_cmd(
        &self,
        conf: &Config,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None,
        };
        let ok =
            permissions::check_permission(conf, roles, msg.author.id, permissions::PERMISSION_BAN);
        if !ok {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    format!(
                        "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                        permissions::PERMISSION_BAN
                    )
                    .as_str(),
                )?
                .await?;

            return Ok(());
        }

        let mut content = msg.content.clone();

        util::regex::EMOJI
            .captures_iter(&msg.content)
            .for_each(|cap| {
                let full_str = cap.get(0).unwrap().as_str();
                let emoji_name = cap.get(1).unwrap().as_str();
                content = content.replace(full_str, format!(":{}:", emoji_name).as_str());
            });

        let mut last_id = "";
        let id_list: Vec<String> = util::regex::SNOWFLAKE
            .captures_iter(&content)
            .map(|cap| {
                last_id = cap.get(0).unwrap().as_str();
                cap.get(1).unwrap().as_str().to_string()
            })
            .collect();

        if id_list.len() == 0 {
            self.rest.create_message(msg.channel_id)
                .content("<:mesaCommand:832350527131746344> `ban <target:user[]> [time:duration] [reason:string...]`")?
                .await?;

            return Ok(());
        }

        let duration = Duration::new(content.to_string());

        let mut splitby = last_id;

        if !duration.full_string.is_empty() {
            splitby = &duration.full_string;
        }

        let reason = match content.to_string().split(splitby).collect::<Vec<&str>>() {
            mut vec if vec.len() > 1 => {
                vec.remove(0);
                let r = vec.join("").trim().to_string();
                if r.is_empty() {
                    None
                } else {
                    Some(r)
                }
            }
            _ => None,
        };

        let duration_str = if duration.is_permenant() {
            "`Never`".to_string()
        } else {
            format!(
                "{} ({})",
                duration.to_discord_timestamp(),
                duration.to_discord_relative_timestamp()
            )
        };

        let resp = match reason {
            Some(ref reason) => {
                format!("<:mesaBan:869663336625733634> Successfully banned {} expiring {} for reason `{}`", 
                    util::mentions::mentions_from_id_str_vec(&id_list), duration_str, reason)
            }
            None => format!(
                "<:mesaBan:869663336625733634> Successfully banned {} expiring {}",
                util::mentions::mentions_from_id_str_vec(&id_list),
                duration_str
            ),
        };
        self.rest
            .create_message(msg.channel_id)
            .content(resp.as_str())?
            .await?;

        let mut uuids: Vec<String> = Vec::new();

        let reason = reason.as_ref();

        let appealable = (*conf)
            .modules
            .as_ref()
            .and_then(|modules| modules.appeals.as_ref())
            .map(|appeals| appeals.enabled)
            .unwrap_or(false);

        match msg.guild_id {
            Some(guild_id) => {
                for id in &id_list {
                    let punishment = self
                        .ban_user(
                            &guild_id.to_string(),
                            id,
                            &msg.author.id.to_string(),
                            &duration,
                            reason,
                            None,
                            appealable,
                        )
                        .await?;
                    uuids.push(punishment.uuid);
                }
            }

            None => {}
        }

        if let Some(modules) = &conf.modules {
            if let Some(logging) = &modules.logging {
                for (i, id) in id_list.iter().enumerate() {
                    self.log_ban(
                        logging,
                        &msg.author.id.to_string(),
                        id,
                        reason,
                        &duration,
                        &match uuids.get(i) {
                            Some(uuid) => uuid.to_string(),
                            None => "".to_string(),
                        },
                    )
                    .await;
                }
            }
        }

        Ok(())
    }

    pub async fn unban_user_cmd(
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
            permissions::PERMISSION_UNBAN,
        );
        if !ok {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    format!(
                        "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                        permissions::PERMISSION_UNBAN
                    )
                    .as_str(),
                )?
                .await?;
            return Ok(());
        }

        let mut content = msg.content.clone();

        util::regex::EMOJI
            .captures_iter(&msg.content)
            .for_each(|cap| {
                let full_str = cap.get(0).unwrap().as_str();
                let emoji_name = cap.get(1).unwrap().as_str();
                content = content.replace(full_str, format!(":{}:", emoji_name).as_str());
            });

        let mut last_id = "";
        let id_list: Vec<String> = util::regex::SNOWFLAKE
            .captures_iter(&content)
            .map(|cap| {
                last_id = cap.get(0).unwrap().as_str();
                cap.get(1).unwrap().as_str().to_string()
            })
            .collect();

        if id_list.len() == 0 {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    "<:mesaCommand:832350527131746344> `unban <target:user[]> [reason:string...]`",
                )?
                .await?;

            return Ok(());
        }

        let splitby = last_id;

        let reason = match content.to_string().split(splitby).collect::<Vec<&str>>() {
            mut vec if vec.len() > 1 => {
                vec.remove(0);
                let r = vec.join("").trim().to_string();
                if r.is_empty() {
                    None
                } else {
                    Some(r)
                }
            }
            _ => None,
        };

        let resp = match reason {
            Some(ref reason) => {
                format!(
                    "<:mesaUnban:869663336697069619> Successfully unbanned {} for reason `{}`",
                    util::mentions::mentions_from_id_str_vec(&id_list),
                    reason
                )
            }
            None => format!(
                "<:mesaUnban:869663336697069619> Successfully unbanned {}",
                util::mentions::mentions_from_id_str_vec(&id_list)
            ),
        };
        self.rest
            .create_message(msg.channel_id)
            .content(resp.as_str())?
            .await?;

        let reason = reason.as_ref();

        match msg.guild_id {
            Some(guild_id) => {
                for id in &id_list {
                    self.unban_user(
                        &guild_id.to_string(),
                        id,
                        &msg.author.id.to_string(),
                        reason,
                    )
                    .await?;
                }
            }

            None => {}
        }

        if let Some(modules) = &conf.modules {
            if let Some(logging) = &modules.logging {
                for id in id_list {
                    self.log_unban(logging, &msg.author.id.to_string(), &id, reason)
                        .await;
                }
            }
        }

        Ok(())
    }

    pub async fn mute_user_cmd(
        &self,
        conf: &Config,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None,
        };
        let ok =
            permissions::check_permission(conf, roles, msg.author.id, permissions::PERMISSION_MUTE);
        if !ok {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    format!(
                        "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                        permissions::PERMISSION_MUTE
                    )
                    .as_str(),
                )?
                .await?;

            return Ok(());
        }

        let mut content = msg.content.clone();

        util::regex::EMOJI
            .captures_iter(&msg.content)
            .for_each(|cap| {
                let full_str = cap.get(0).unwrap().as_str();
                let emoji_name = cap.get(1).unwrap().as_str();
                content = content.replace(full_str, format!(":{}:", emoji_name).as_str());
            });

        let mut last_id = "";
        let id_list: Vec<String> = util::regex::SNOWFLAKE
            .captures_iter(&content)
            .map(|cap| {
                last_id = cap.get(0).unwrap().as_str();
                cap.get(1).unwrap().as_str().to_string()
            })
            .collect();

        if id_list.len() == 0 {
            self.rest.create_message(msg.channel_id)
                .content("<:mesaCommand:832350527131746344> `mute <target:user[]> [time:duration] [reason:string...]`")?
                .await?;

            return Ok(());
        }

        let duration = Duration::new(content.to_string());

        let mut splitby = last_id;

        if !duration.full_string.is_empty() {
            splitby = &duration.full_string;
        }

        let reason = match content.to_string().split(splitby).collect::<Vec<&str>>() {
            mut vec if vec.len() > 1 => {
                vec.remove(0);
                let r = vec.join("").trim().to_string();
                if r.is_empty() {
                    None
                } else {
                    Some(r)
                }
            }
            _ => None,
        };

        let duration_str = if duration.is_permenant() {
            "`Never`".to_string()
        } else {
            format!(
                "{} ({})",
                duration.to_discord_timestamp(),
                duration.to_discord_relative_timestamp()
            )
        };

        let resp = match reason {
            Some(ref reason) => {
                format!("<:mesaMemberMute:869663336814497832> Successfully muted {} expiring {} for reason `{}`", 
                    util::mentions::mentions_from_id_str_vec(&id_list), duration_str, reason)
            }
            None => format!(
                "<:mesaMemberMute:869663336814497832> Successfully muted {} expiring {}",
                util::mentions::mentions_from_id_str_vec(&id_list),
                duration_str
            ),
        };
        self.rest
            .create_message(msg.channel_id)
            .content(resp.as_str())?
            .await?;

        let mut uuids: Vec<String> = Vec::new();

        let reason = reason.as_ref();

        let appealable = (*conf)
            .modules
            .as_ref()
            .and_then(|modules| modules.appeals.as_ref())
            .map(|appeals| appeals.enabled)
            .unwrap_or(false);

        match msg.guild_id {
            Some(guild_id) => {
                for id in &id_list {
                    let punishment = self
                        .mute_user(
                            conf,
                            &guild_id.to_string(),
                            id,
                            &msg.author.id.to_string(),
                            &duration,
                            reason,
                            None,
                            appealable,
                        )
                        .await?;

                    uuids.push(punishment.uuid);
                }
            }

            None => {}
        }

        if let Some(modules) = &conf.modules {
            if let Some(logging) = &modules.logging {
                for (i, id) in id_list.iter().enumerate() {
                    self.log_mute(
                        logging,
                        &msg.author.id.to_string(),
                        id,
                        reason,
                        &duration,
                        &match uuids.get(i) {
                            Some(uuid) => uuid.to_string(),
                            None => "".to_string(),
                        },
                    )
                    .await;
                }
            }
        }

        Ok(())
    }

    pub async fn unmute_user_cmd(
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
            permissions::PERMISSION_UNMUTE,
        );
        if !ok {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    format!(
                        "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                        permissions::PERMISSION_UNMUTE
                    )
                    .as_str(),
                )?
                .await?;

            return Ok(());
        }

        let mut content = msg.content.clone();

        util::regex::EMOJI
            .captures_iter(&msg.content)
            .for_each(|cap| {
                let full_str = cap.get(0).unwrap().as_str();
                let emoji_name = cap.get(1).unwrap().as_str();
                content = content.replace(full_str, format!(":{}:", emoji_name).as_str());
            });

        let mut last_id = "";
        let id_list: Vec<String> = util::regex::SNOWFLAKE
            .captures_iter(&content)
            .map(|cap| {
                last_id = cap.get(0).unwrap().as_str();
                cap.get(1).unwrap().as_str().to_string()
            })
            .collect();

        if id_list.len() == 0 {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    "<:mesaCommand:832350527131746344> `unmute <target:user[]> [reason:string...]`",
                )?
                .await?;

            return Ok(());
        }

        let splitby = last_id;

        let reason = match content.to_string().split(splitby).collect::<Vec<&str>>() {
            mut vec if vec.len() > 1 => {
                vec.remove(0);
                let r = vec.join("").trim().to_string();
                if r.is_empty() {
                    None
                } else {
                    Some(r)
                }
            }
            _ => None,
        };

        let resp = match reason {
            Some(ref reason) => {
                format!("<:mesaMemberUnmute:869663336583802982> Successfully unmuted {} for reason `{}`", 
                    util::mentions::mentions_from_id_str_vec(&id_list), reason)
            }
            None => format!(
                "<:mesaMemberUnmute:869663336583802982> Successfully unmuted {}",
                util::mentions::mentions_from_id_str_vec(&id_list)
            ),
        };
        self.rest
            .create_message(msg.channel_id)
            .content(resp.as_str())?
            .await?;

        let reason = reason.as_ref();

        match msg.guild_id {
            Some(guild_id) => {
                for id in &id_list {
                    self.unmute_user(
                        Some(conf),
                        None,
                        &guild_id.to_string(),
                        id,
                        &msg.author.id.to_string(),
                        reason,
                    )
                    .await?;
                }
            }

            None => {}
        }

        if let Some(modules) = &conf.modules {
            if let Some(logging) = &modules.logging {
                for id in id_list {
                    self.log_unmute(logging, &msg.author.id.to_string(), &id, reason)
                        .await;
                }
            }
        }

        Ok(())
    }
}
