use std::str::FromStr;

use mongodb::results::UpdateResult;
use twilight_mention::Mention;
use twilight_model::{channel::{Message, embed::*, message::AllowedMentions}, id::Id};
use lazy_static::lazy_static;
use regex::Regex;

use crate::{handlers::Handler, util::{duration::{self, Duration}, mentions::mentions_from_id_str_vec, permissions}, mongo::mongo::{Config, Punishment}};

impl Handler {

    // UTILITY COMMANDS

    pub async fn search_user_cmd(&self, conf: &Config, msg: &Message, deep: bool)
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

        let mut perm = permissions::PERMISSION_SEARCH;

        if id == &author_id {
            perm = permissions::PERMISSION_SEARCHSELF;
        }

        let ok = permissions::check_permission(conf, roles, &author_id, vec![perm]);
        if !ok {
            self.rest.create_message(msg.channel_id)
            .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", perm).as_str())?
            .exec()
            .await?;
            return Ok(());
        }

        let punishments = match self.db.get_punishments(
            &match &msg.guild_id {
                Some(id) => id.to_string(),
                None => return Ok(())
            },
            &id)
            .await {
                Ok(p) => p,
                Err(e) => {
                    self.rest.create_message(msg.channel_id).content("Error getting punishments")?.exec().await?;
                    // Return Ok here because an error here shouldn't cause further issue, this can be manually investigated.
                    println!("Error getting punishments: {:?}", e);
                    return Ok(());
                }
            };
        
        let embed_footer = EmbedFooter{
            text: "Black Mesa Rust Beta by Tyler#0911 running on rustc 1.63.0".to_string(),
            icon_url: None,
            proxy_icon_url: None
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

            fields.push(
                EmbedField{
                    name: punishment.typ.pretty_string(),
                    value,
                    inline: true
                }
            );
        }
        
        let user = match self.cache.user(Id::from_str(id)?) {
            Some(user) => user.clone(),
            None => {
                self.rest.user(Id::from_str(&id)?).exec().await?.model().await?
            }
        };

        let embeds = vec![Embed {
            title: Some(format!("{}#{:04}'s Infraction log.", user.name, user.discriminator)),
            description: Some(format!("{} has {} infractions.", user.mention(), total_punishments)),
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
            video: None

        }];

        self.rest.create_message(msg.channel_id).embeds(&embeds)?.exec().await?;

        Ok(())
    }

    pub async fn update_reason_cmd(&self, conf: &Config, msg: &Message)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = &msg.content;
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b").unwrap();
        }

        let uuid_list: Vec<String> = RE.find_iter(content).map(|m| m.as_str().to_string()).collect();

        let mut punishment_list: Vec<Punishment> = Vec::new();

        let guild_id = match &msg.guild_id {
            Some(id) => id.to_string(),
            None => return Ok(())
        };

        let author_id = msg.author.id.to_string();
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None
        };

        let splitby = &match uuid_list.len() {
            0 => {
                let punishment = match self.db.get_last_action(&guild_id.to_string(), &msg.author.id.to_string()).await {
                    Ok(p) => {
                        match p {
                            Some(p) => p,
                            None => {
                                self.rest.create_message(msg.channel_id).content("<:mesaCross:832350526414127195> Unable to get last action")?.exec().await?;
                                return Ok(());
                            }
                        }
                    }
                    Err(_) => {
                        self.rest.create_message(msg.channel_id).content("<:mesaCross:832350526414127195> Unable to get last action")?.exec().await?;
                        return Ok(());
                    }
                };
                punishment_list.push(punishment);
                match content.split_once(" ") {
                    Some(s) => s.0.to_string() + " ",
                    None => {
                        self.rest.create_message(msg.channel_id).content("<:mesaCommand:832350527131746344> `reason [uuid:uuid] [reason:string...]`")?.exec().await?;
                        return Ok(());
                    }
                }
            },
            1 => {
                let uuid = uuid_list.last().unwrap().to_string();
                let punishment = match self.db.get_action_by_uuid(&guild_id.to_string(), &uuid).await {
                    Ok(a) => {
                        match a {
                            Some(a) => a,
                            None => {
                                self.rest.create_message(msg.channel_id)
                                .content(format!("<:mesaCross:832350526414127195> Unable to get action {}", uuid)
                                .as_str())?
                                .exec()
                                .await?;
                                return Ok(());
                            }
                        }
                    },
                    Err(_) => {
                        self.rest.create_message(msg.channel_id)
                        .content(format!("<:mesaCross:832350526414127195> Unable to get action {}", uuid)
                        .as_str())?
                        .exec()
                        .await?;
        
                        return Ok(());
                    }
                };
                punishment_list.push(punishment);
                match content.split_once(" ") {
                    Some(s) => s.1.to_string() + " ",
                    None => {
                        self.rest.create_message(msg.channel_id).content("<:mesaCommand:832350527131746344> `reason [uuid:uuid] [reason:string...]`")?.exec().await?;
                        return Ok(());
                    }
                }
            },
            _ => {
                self.rest.create_message(msg.channel_id).content("<:mesaCommand:832350527131746344> `reason [uuid:uuid] [reason:string...]`")?.exec().await?;
                return Ok(());
            }
        };

        let reason = match content.to_string().split(splitby).collect::<Vec<&str>>() {
            mut vec if vec.len() > 1 => {
                vec.remove(0);
                vec.join("").trim().to_string()
            }
            _ => {
                self.rest.create_message(msg.channel_id)
                .content("<:mesaCommand:832350527131746344> `reason [uuid:uuid] [reason:string...]`")?
                .exec()
                .await?;
                return Ok(());
            }
        };

        let punishment = &punishment_list[0]; // This is safe because we check the length of the list above.

        if &punishment.issuer == &author_id {
            let ok = permissions::check_permission(conf, roles, &author_id, vec![permissions::PERMISSION_UPDATESELF]);
                if !ok {
                    self.rest.create_message(msg.channel_id)
                    .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", permissions::PERMISSION_UPDATESELF).as_str())?
                    .exec()
                    .await?;
                    return Ok(());
                }
        } else {
            let ok = permissions::check_permission(conf, roles, &author_id, vec![permissions::PERMISSION_UPDATE]);
            if !ok {
                self.rest.create_message(msg.channel_id)
                .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", permissions::PERMISSION_UPDATE).as_str())?
                .exec()
                .await?;
                return Ok(());
            }
        
            // now check if the user is trying to remove a punishment of someone with a higher level
            
            let guild_id_marker = msg.guild_id.unwrap();
            let original_issuer_id = Id::from_str(punishment.issuer.as_str())?;
            let original_issuer_roles = match self.cache.member(guild_id_marker, original_issuer_id) {
                Some(member) => member.to_owned().roles().to_vec(),
                None => {
                    self.rest.guild_member(guild_id_marker, original_issuer_id).exec().await?.model().await?.roles
                }
            };

            let original_issuer_level = permissions::get_user_level(conf, Some(&original_issuer_roles), &original_issuer_id.to_string());
        
            let author_level = permissions::get_user_level(conf, roles, &author_id);

            if original_issuer_level >= author_level && !conf.modules.moderation.update_higher_level_reason {
                self.rest.create_message(msg.channel_id)
                .content("<:mesaCross:832350526414127195> You do not have permission to update this punishment as it is of a user of equal or higher level")?
                .exec()
                .await?;
                return Ok(());
            }
        }

        let res = self.db.update_punishment(&punishment.uuid,
            &guild_id,
            &punishment.user_id,
            &punishment.issuer,
            Some(reason.clone()),
            None)
            .await?;
            
        match res{
            UpdateResult{matched_count: 1, modified_count: 1, ..} => {
                self.rest.create_message(msg.channel_id)
                .content(format!("<:mesaCheck:832350526729224243> Updated punishment `{}`", punishment.uuid).as_str())?
                .exec()
                .await?;
            },
            UpdateResult{..} => {
                self.rest.create_message(msg.channel_id)
                .content(format!("<:mesaCross:832350526414127195> Unable to update punishment `{}`", punishment.uuid
                ).as_str())?
                .exec()
                .await?;
            }
        }

        let allowed_mentions = AllowedMentions::builder().build();

        match conf.modules.logging.log_update_action(msg.author.id.to_string(), punishment, None, Some(reason)) {
            Some(log) => {
            self.rest.create_message(match &conf.modules.logging.channel_id {
                Some(id) => Id::from_str(id.as_str())?,
                None => {return Ok(())}
                })
                .content(log.as_str())?
                .allowed_mentions(Some(&allowed_mentions))
                .exec()
                .await?;
            }
            None => {}
        }

        Ok(())
    }
    
    pub async fn update_duration_cmd(&self, conf: &Config, msg: &Message)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = &msg.content;
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b").unwrap();
        }

        let uuid_list: Vec<String> = RE.find_iter(content).map(|m| m.as_str().to_string()).collect();

        let mut punishment_list: Vec<Punishment> = Vec::new();

        let guild_id = match &msg.guild_id {
            Some(id) => id.to_string(),
            None => return Ok(())
        };

        let author_id = msg.author.id.to_string();
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None
        };

        let splitby = &match uuid_list.len() {
            0 => {
                let punishment = match self.db.get_last_action(&guild_id.to_string(), &msg.author.id.to_string()).await {
                    Ok(p) => {
                        match p {
                            Some(p) => p,
                            None => {
                                self.rest.create_message(msg.channel_id).content("<:mesaCross:832350526414127195> Unable to get last action")?.exec().await?;
                                return Ok(());
                            }
                        }
                    }
                    Err(_) => {
                        self.rest.create_message(msg.channel_id).content("<:mesaCross:832350526414127195> Unable to get last action")?.exec().await?;
                        return Ok(());
                    }
                };
                punishment_list.push(punishment);
                match content.split_once(" ") {
                    Some(s) => s.0.to_string() + " ",
                    None => {
                        self.rest.create_message(msg.channel_id).content("<:mesaCommand:832350527131746344> `duration [uuid:uuid] [time:duration]`")?.exec().await?;
                        return Ok(());
                    }
                }
            },
            1 => {
                let uuid = uuid_list.last().unwrap().to_string();
                let punishment = match self.db.get_action_by_uuid(&guild_id.to_string(), &uuid).await {
                    Ok(a) => {
                        match a {
                            Some(a) => a,
                            None => {
                                self.rest.create_message(msg.channel_id)
                                .content(format!("<:mesaCross:832350526414127195> Unable to get action {}", uuid)
                                .as_str())?
                                .exec()
                                .await?;
                                return Ok(());
                            }
                        }
                    },
                    Err(_) => {
                        self.rest.create_message(msg.channel_id)
                        .content(format!("<:mesaCross:832350526414127195> Unable to get action {}", uuid)
                        .as_str())?
                        .exec()
                        .await?;
        
                        return Ok(());
                    }
                };
                punishment_list.push(punishment);
                match content.split_once(" ") {
                    Some(s) => s.1.to_string() + " ",
                    None => {
                        self.rest.create_message(msg.channel_id).content("<:mesaCommand:832350527131746344> `duration [uuid:uuid] [time:duration]`")?.exec().await?;
                        return Ok(());
                    }
                }
            },
            _ => {
                self.rest.create_message(msg.channel_id).content("<:mesaCommand:832350527131746344> `duration [uuid:uuid] [time:duration]`")?.exec().await?;
                return Ok(());
            }
        };

        let dur_split = match content.to_string().split(splitby).collect::<Vec<&str>>() {
            mut vec if vec.len() > 1 => {
                vec.remove(0);
                vec.join("").trim().to_string()
            }
            _ => {
                self.rest.create_message(msg.channel_id)
                .content("<:mesaCommand:832350527131746344> `duration [uuid:uuid] [time:duration]`")?
                .exec()
                .await?;
                return Ok(());
            }
        };

        let duration = Duration::new(dur_split);

        let punishment = &punishment_list[0]; // This is safe because we check the length of the list above.

        if &punishment.issuer == &author_id {
            let ok = permissions::check_permission(conf, roles, &author_id, vec![permissions::PERMISSION_UPDATESELF]);
                if !ok {
                    self.rest.create_message(msg.channel_id)
                    .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", permissions::PERMISSION_UPDATESELF).as_str())?
                    .exec()
                    .await?;
                    return Ok(());
                }
        } else {
            let ok = permissions::check_permission(conf, roles, &author_id, vec![permissions::PERMISSION_UPDATE]);
            if !ok {
                self.rest.create_message(msg.channel_id)
                .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", permissions::PERMISSION_UPDATE).as_str())?
                .exec()
                .await?;
                return Ok(());
            }
        
            // now check if the user is trying to update a punishment of someone with a higher level
            let guild_id_marker = msg.guild_id.unwrap();
            let original_issuer_id = Id::from_str(punishment.issuer.as_str())?;
            let original_issuer_roles = match self.cache.member(guild_id_marker, original_issuer_id) {
                Some(member) => member.to_owned().roles().to_vec(),
                None => {
                    self.rest.guild_member(guild_id_marker, original_issuer_id).exec().await?.model().await?.roles
                }
            };

            let original_issuer_level = permissions::get_user_level(conf, Some(&original_issuer_roles), &original_issuer_id.to_string());
        
            let author_level = permissions::get_user_level(conf, roles, &author_id);

            if original_issuer_level >= author_level && !conf.modules.moderation.update_higher_level_reason {
                self.rest.create_message(msg.channel_id)
                .content("<:mesaCross:832350526414127195> You do not have permission to update this punishment as it is of a user of equal or higher level")?
                .exec()
                .await?;
                return Ok(());
            }
        }

        let res = self.db.update_punishment(&punishment.uuid,
            &guild_id,
            &punishment.user_id,
            &punishment.issuer,
            None,
            duration.to_unix_expiry())
            .await?;
            
        match res{
            UpdateResult{matched_count: 1, modified_count: 1, ..} => {
                self.rest.create_message(msg.channel_id)
                .content(format!("<:mesaCheck:832350526729224243> Updated punishment `{}`", punishment.uuid).as_str())?
                .exec()
                .await?;
            },
            UpdateResult{..} => {
                self.rest.create_message(msg.channel_id)
                .content(format!("<:mesaCross:832350526414127195> Unable to update punishment `{}`", punishment.uuid
                ).as_str())?
                .exec()
                .await?;
            }
        }

        let allowed_mentions = AllowedMentions::builder().build();

        match conf.modules.logging.log_update_action(msg.author.id.to_string(), punishment, Some(duration), None) {
            Some(log) => {
            self.rest.create_message(match &conf.modules.logging.channel_id {
                Some(id) => Id::from_str(id.as_str())?,
                None => {return Ok(())}
                })
                .content(log.as_str())?
                .allowed_mentions(Some(&allowed_mentions))
                .exec()
                .await?;
            }
            None => {}
        }

        Ok(())
    }

    // MODERATION COMMANDS

    pub async fn strike_user_cmd(&self, conf: &Config, msg: &Message)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let author_id = msg.author.id.to_string();
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None
        };
        let ok = permissions::check_permission(conf, roles, &author_id, vec![permissions::PERMISSION_STRIKE]);
        if !ok {
            self.rest.create_message(msg.channel_id)
            .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", permissions::PERMISSION_STRIKE).as_str())?
            .exec()
            .await?;
            return Ok(());
        }

        let content = &msg.content;
        lazy_static! {
            static ref RE: Regex = Regex::new(r"([0-9]{17,19})").unwrap();
        }
        let id_list: Vec<String> = RE.find_iter(content).map(|m| m.as_str().to_string()).collect();

        if id_list.len() == 0 {
            self.rest.create_message(msg.channel_id)
            .content("<:mesaCommand:832350527131746344> `strike <target:user[]> [time:duration] [reason:string...]`")?
            .exec()
            .await?;
            return Ok(());
        }

        let mut duration = duration::Duration::new(content.to_string());
            
        // check the string rather than calling .is_permenant() because if the user wishes to strike permanently, they should be able to do so.
        if duration.full_string.is_empty() {
            duration = duration::Duration::new_no_str(match &conf.modules.moderation.default_strike_duration {
                Some(dur) => dur.to_string(),
                None => "30d".to_string()
            });
        }
        

        let mut splitby = id_list.last().unwrap();

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
            _ => None
        };

        let mut uuids: Vec<String> = Vec::new();

        match msg.guild_id {
            Some(guild_id) => {
                for id in &id_list {
                    let punishment = self.issue_strike(conf,
                        &guild_id.to_string(),
                        id, &msg.author.id.to_string(),
                        &reason,
                        &duration)
                        .await?;
                    uuids.push(punishment.uuid);
                }
            }

            None => {} // dont care.
        }

        let duration_str = if duration.is_permenant() {
            "`Never`".to_string()
        } else {
            format!("{} ({})", duration.to_discord_timestamp(), duration.to_discord_relative_timestamp())
        };
        

        let resp = match reason {
            Some(ref reason) => {
                format!("<:mesaStrike:869663336843845752> Successfully striked {} expiring {} for reason `{}`", 
                    mentions_from_id_str_vec(&id_list), duration_str, reason)
            },
            None => format!("<:mesaStrike:869663336843845752> Successfully striked {} expiring {}", 
                mentions_from_id_str_vec(&id_list), duration_str)
        };
        self.rest.create_message(msg.channel_id).content(resp.as_str())?.exec().await?;

        let allowed_mentions = AllowedMentions::builder().build();

        for (i, id) in id_list.iter().enumerate() {
            match conf.modules.logging.log_strike(msg.author.id.to_string(),
            id.to_string(),
            reason.clone(),
            duration.clone(),
            match uuids.get(i) {
                Some(uuid) => uuid.to_string(),
                None => "".to_string()
            }){
                Some(log) => {
                self.rest.create_message(match &conf.modules.logging.channel_id {
                    Some(id) => Id::from_str(id.as_str())?,
                    None => {return Ok(())}
                    })
                    .content(log.as_str())?
                    .allowed_mentions(Some(&allowed_mentions))
                    .exec()
                    .await?;
                }
                None => {}
            }
        }

        Ok(())
    }

    pub async fn kick_user_cmd(&self, conf: &Config, msg: &Message)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let author_id = msg.author.id.to_string();
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None
        };
        let ok = permissions::check_permission(conf, roles, &author_id, vec![permissions::PERMISSION_KICK]);
        if !ok {
            self.rest.create_message(msg.channel_id)
            .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", permissions::PERMISSION_KICK).as_str())?
            .exec()
            .await?;
            return Ok(());
        }

        let content = &msg.content;
        lazy_static! {
            static ref RE: Regex = Regex::new(r"([0-9]{17,19})").unwrap();
        }
        let id_list: Vec<String> = RE.find_iter(content).map(|m| m.as_str().to_string()).collect();

        if id_list.len() == 0 {
            self.rest.create_message(msg.channel_id)
            .content("<:mesaCommand:832350527131746344> `kick <target:user[]> [reason:string...]`")?
            .exec()
            .await?;
            return Ok(());
        }

        let splitby = id_list.last().unwrap();

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
            _ => None
        };


        let mut uuids: Vec<String> = Vec::new();

        match msg.guild_id {
            Some(guild_id) => {
                for id in &id_list {
                    let punishment = self.kick_user(&guild_id.to_string(),
                        id,
                        &msg.author.id.to_string(),
                        &reason)
                        .await?;
                    uuids.push(punishment.uuid);
                }
            }

            None => {} // dont care.
        }
        

        let resp = match reason {
            Some(ref reason) => {
                format!("<:mesaKick:869665034312253460> Successfully kicked {} for reason `{}`", 
                    mentions_from_id_str_vec(&id_list), reason)
            },
            None => format!("<:mesaKick:869665034312253460> Successfully kicked {}", 
                mentions_from_id_str_vec(&id_list))
        };
        self.rest.create_message(msg.channel_id).content(resp.as_str())?.exec().await?;

        let allowed_mentions = AllowedMentions::builder().build();

        for (i, id) in id_list.iter().enumerate() {
            match conf.modules.logging.log_kick(msg.author.id.to_string(),
            id.to_string(),
            reason.clone(),
            match uuids.get(i) {
                Some(uuid) => uuid.to_string(),
                None => "".to_string()
            }){
                Some(log) => {
                self.rest.create_message(match &conf.modules.logging.channel_id {
                    Some(id) => Id::from_str(id.as_str())?,
                    None => {return Ok(())}
                    })
                    .content(log.as_str())?
                    .allowed_mentions(Some(&allowed_mentions))
                    .exec()
                    .await?;
                }
                None => {}
            }
        }

        Ok(())
    }

    pub async fn ban_user_cmd(&self, conf: &Config, msg: &Message)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let author_id = msg.author.id.to_string();
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None
        };
        let ok = permissions::check_permission(conf, roles, &author_id, vec![permissions::PERMISSION_BAN]);
        if !ok {
            self.rest.create_message(msg.channel_id)
            .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", permissions::PERMISSION_BAN).as_str())?
            .exec()
            .await?;
            return Ok(());
        }

        let content = &msg.content;
        lazy_static! {
            static ref RE: Regex = Regex::new(r"([0-9]{17,19})").unwrap();
        }
        let id_list: Vec<String> = RE.find_iter(content).map(|m| m.as_str().to_string()).collect();

        if id_list.len() == 0 {
            self.rest.create_message(msg.channel_id)
            .content("<:mesaCommand:832350527131746344> `ban <target:user[]> [time:duration] [reason:string...]`")?
            .exec()
            .await?;
            return Ok(());
        }

        let duration = duration::Duration::new(content.to_string());

        let mut splitby = id_list.last().unwrap();

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
            _ => None
        };

        let mut uuids: Vec<String> = Vec::new();

        match msg.guild_id {
            Some(guild_id) => {
                for id in &id_list {
                    let punishment = self.ban_user(&guild_id.to_string(),
                        id,
                        &msg.author.id.to_string(),
                        &duration,
                        &reason)
                        .await?;
                    uuids.push(punishment.uuid);
                }
            }

            None => {} // dont care.
        }

        let duration_str = if duration.is_permenant() {
            "`Never`".to_string()
        } else {
            format!("{} ({})", duration.to_discord_timestamp(), duration.to_discord_relative_timestamp())
        };
        

        let resp = match reason {
            Some(ref reason) => {
                format!("<:mesaBan:869663336625733634> Successfully banned {} expiring {} for reason `{}`", 
                    mentions_from_id_str_vec(&id_list), duration_str, reason)
            },
            None => format!("<:mesaBan:869663336625733634> Successfully banned {} expiring {}", 
                mentions_from_id_str_vec(&id_list), duration_str)
        };
        self.rest.create_message(msg.channel_id).content(resp.as_str())?.exec().await?;

        let allowed_mentions = AllowedMentions::builder().build();

        for (i, id) in id_list.iter().enumerate() {
            match conf.modules.logging.log_ban(msg.author.id.to_string(),
            id.to_string(),
            reason.clone(),
            duration.clone(),
            match uuids.get(i) {
                Some(uuid) => uuid.to_string(),
                None => "".to_string()
            }){
                Some(log) => {
                self.rest.create_message(match &conf.modules.logging.channel_id {
                    Some(id) => Id::from_str(id.as_str())?,
                    None => {return Ok(())}
                    })
                    .content(log.as_str())?
                    .allowed_mentions(Some(&allowed_mentions))
                    .exec()
                    .await?;
                }
                None => {}
            }
        }

        Ok(())
    }

    pub async fn unban_user_cmd(&self, conf: &Config, msg: &Message)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let author_id = msg.author.id.to_string();
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None
        };
        let ok = permissions::check_permission(conf, roles, &author_id, vec![permissions::PERMISSION_UNBAN]);
        if !ok {
            self.rest.create_message(msg.channel_id)
            .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", permissions::PERMISSION_UNBAN).as_str())?
            .exec()
            .await?;
            return Ok(());
        }

        let content = &msg.content;
        lazy_static! {
            static ref RE: Regex = Regex::new(r"([0-9]{17,19})").unwrap();
        }
        let id_list: Vec<String> = RE.find_iter(content).map(|m| m.as_str().to_string()).collect();

        if id_list.len() == 0 {
            self.rest.create_message(msg.channel_id)
            .content("<:mesaCommand:832350527131746344> `unban <target:user[]> [reason:string...]`")?
            .exec()
            .await?;
            return Ok(());
        }

        let splitby = id_list.last().unwrap();

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
            _ => None
        };
        match msg.guild_id {
            Some(guild_id) => {
                for id in &id_list {
                    self.unban_user(&guild_id.to_string(),
                        id,
                        &msg.author.id.to_string(),
                        &reason)
                        .await?;
                }
            }

            None => {} // dont care.
        }

        let resp = match reason {
            Some(ref reason) => {
                format!("<:mesaUnban:869663336697069619> Successfully unbanned {} for reason `{}`", 
                    mentions_from_id_str_vec(&id_list), reason)
            },
            None => format!("<:mesaUnban:869663336697069619> Successfully unbanned {}", 
                mentions_from_id_str_vec(&id_list))
        };
        self.rest.create_message(msg.channel_id).content(resp.as_str())?.exec().await?;

        let allowed_mentions = AllowedMentions::builder().build();

        for id in id_list {
            match conf.modules.logging.log_unban(msg.author.id.to_string(),
            id.to_string(),
            reason.clone()
            ){
                Some(log) => {
                self.rest.create_message(match &conf.modules.logging.channel_id {
                    Some(id) => Id::from_str(id.as_str())?,
                    None => {return Ok(())}
                    })
                    .content(log.as_str())?
                    .allowed_mentions(Some(&allowed_mentions))
                    .exec()
                    .await?;
                }
                None => {}
            }
        }

        Ok(())
    }

    pub async fn mute_user_cmd(&self, conf: &Config, msg: &Message)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let author_id = msg.author.id.to_string();
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None
        };
        let ok = permissions::check_permission(conf, roles, &author_id, vec![permissions::PERMISSION_MUTE]);
        if !ok {
            self.rest.create_message(msg.channel_id)
            .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", permissions::PERMISSION_MUTE).as_str())?
            .exec()
            .await?;
            return Ok(());
        }

        let content = &msg.content;
        lazy_static! {
            static ref RE: Regex = Regex::new(r"([0-9]{17,19})").unwrap();
        }
        let id_list: Vec<String> = RE.find_iter(content).map(|m| m.as_str().to_string()).collect();

        if id_list.len() == 0 {
            self.rest.create_message(msg.channel_id)
            .content("<:mesaCommand:832350527131746344> `mute <target:user[]> [time:duration] [reason:string...]`")?
            .exec()
            .await?;
            return Ok(());
        }

        let duration = duration::Duration::new(content.to_string());

        let mut splitby = id_list.last().unwrap();

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
            _ => None
        };

        let mut uuids: Vec<String> = Vec::new();

        match msg.guild_id {
            Some(guild_id) => {
                for id in &id_list {
                    let punishment = self.mute_user(conf,
                        &guild_id.to_string(),
                        id,
                        &msg.author.id.to_string(),
                        &duration,
                        &reason)
                        .await?;

                    uuids.push(punishment.uuid);
                }
            }

            None => {} // dont care.
        }

        let duration_str = if duration.is_permenant() {
            "`Never`".to_string()
        } else {
            format!("{} ({})", duration.to_discord_timestamp(), duration.to_discord_relative_timestamp())
        };
        

        let resp = match reason {
            Some(ref reason) => {
                format!("<:mesaMemberMute:869663336814497832> Successfully muted {} expiring {} for reason `{}`", 
                    mentions_from_id_str_vec(&id_list), duration_str, reason)
            },
            None => format!("<:mesaMemberMute:869663336814497832> Successfully muted {} expiring {}", 
                mentions_from_id_str_vec(&id_list), duration_str)
        };
        self.rest.create_message(msg.channel_id).content(resp.as_str())?.exec().await?;

        let allowed_mentions = AllowedMentions::builder().build();

        for (i, id) in id_list.iter().enumerate() {
            match conf.modules.logging.log_mute(msg.author.id.to_string(),
            id.to_string(),
            reason.clone(),
            duration.clone(),
            match uuids.get(i) {
                Some(uuid) => uuid.to_string(),
                None => "".to_string()
            }){
                Some(log) => {
                self.rest.create_message(match &conf.modules.logging.channel_id {
                    Some(id) => Id::from_str(id.as_str())?,
                    None => {return Ok(())}
                    })
                    .content(log.as_str())?
                    .allowed_mentions(Some(&allowed_mentions))
                    .exec()
                    .await?;
                }
                None => {}
            }
        }

        Ok(())
    }

    pub async fn unmute_user_cmd(&self, conf: &Config, msg: &Message)
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let author_id = msg.author.id.to_string();
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None
        };
        let ok = permissions::check_permission(conf, roles, &author_id, vec![permissions::PERMISSION_UNMUTE]);
        if !ok {
            self.rest.create_message(msg.channel_id)
            .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", permissions::PERMISSION_UNMUTE).as_str())?
            .exec()
            .await?;
            return Ok(());
        }

        let content = &msg.content;
        lazy_static! {
            static ref RE: Regex = Regex::new(r"([0-9]{17,19})").unwrap();
        }
        let id_list: Vec<String> = RE.find_iter(content).map(|m| m.as_str().to_string()).collect();

        if id_list.len() == 0 {
            self.rest.create_message(msg.channel_id)
            .content("<:mesaCommand:832350527131746344> `unmute <target:user[]> [reason:string...]`")?
            .exec()
            .await?;
            return Ok(());
        }

        let splitby = id_list.last().unwrap();

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
            _ => None
        };
        match msg.guild_id {
            Some(guild_id) => {
                for id in &id_list {
                    self.unmute_user(Some(conf),
                        None,
                        &guild_id.to_string(),
                        id,
                        &msg.author.id.to_string(),
                        &reason)
                        .await?;
                }
            }

            None => {} // dont care.
        }

        let resp = match reason {
            Some(ref reason) => {
                format!("<:mesaMemberUnmute:869663336583802982> Successfully unmuted {} for reason `{}`", 
                    mentions_from_id_str_vec(&id_list), reason)
            },
            None => format!("<:mesaMemberUnmute:869663336583802982> Successfully unmuted {}", 
                mentions_from_id_str_vec(&id_list))
        };
        self.rest.create_message(msg.channel_id).content(resp.as_str())?.exec().await?;

        let allowed_mentions = AllowedMentions::builder().build();

        for id in id_list {
            match conf.modules.logging.log_unmute(msg.author.id.to_string(),
            id.to_string(),
            reason.clone()
            ){
                Some(log) => {
                self.rest.create_message(match &conf.modules.logging.channel_id {
                    Some(id) => Id::from_str(id.as_str())?,
                    None => {return Ok(())}
                    })
                    .content(log.as_str())?
                    .allowed_mentions(Some(&allowed_mentions))
                    .exec()
                    .await?;
                }
                None => {}
            }
        }

        Ok(())
    }

}