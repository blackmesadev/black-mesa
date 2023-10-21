use std::{collections::HashMap, str::FromStr};

use mongodb::results::{DeleteResult, UpdateResult};
use twilight_model::{channel::Message, id::Id};

use crate::{
    config::Config,
    handlers::Handler,
    util::{self, permissions},
};

use super::purge::{self, add_purge, Purge, PurgeType};

impl Handler {
    pub async fn remove_punishment_cmd(
        &self,
        conf: &Config,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = &msg.content;

        let uuid_list: Vec<String> = util::regex::UUID
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect();

        let guild_id = match &msg.guild_id {
            Some(id) => id.to_string(),
            None => return Ok(()),
        };

        let author_id = msg.author.id.to_string();
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None,
        };

        let punishment_list = self.db.get_actions_by_uuid(&guild_id, uuid_list).await?;
        for punishment in &punishment_list {
            if &punishment.issuer == &author_id {
                let ok = permissions::check_permission(
                    conf,
                    roles,
                    msg.author.id,
                    permissions::PERMISSION_REMOVEACTIONSELF,
                );
                if !ok {
                    self.rest.create_message(msg.channel_id)
                            .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", permissions::PERMISSION_REMOVEACTIONSELF).as_str())?
                            .await?;

                    return Ok(());
                }
            } else {
                let ok = permissions::check_permission(
                    conf,
                    roles,
                    msg.author.id,
                    permissions::PERMISSION_REMOVEACTION,
                );
                if !ok {
                    self.rest.create_message(msg.channel_id)
                        .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", permissions::PERMISSION_REMOVEACTION).as_str())?
                        .await?;

                    return Ok(());
                }

                let guild_id_marker = msg.guild_id.unwrap();
                let original_issuer_id = Id::from_str(punishment.issuer.as_str())?;
                let original_issuer_roles =
                    match self.cache.member(guild_id_marker, original_issuer_id) {
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
        }

        let uuid_list = &punishment_list.iter().map(|p| p.uuid.clone()).collect();
        let res = self.db.delete_many_by_uuid(&guild_id, &uuid_list).await?;

        match res {
            DeleteResult { deleted_count, .. } => {
                if deleted_count < 1 {
                    self.rest
                        .create_message(msg.channel_id)
                        .content(
                            format!(
                                "<:mesaCross:832350526414127195> Unable to delete `{}` punishments",
                                punishment_list.len()
                            )
                            .as_str(),
                        )?
                        .await?;
                } else {
                    self.rest
                        .create_message(msg.channel_id)
                        .content(
                            format!(
                                "<:mesaUnstrike:869664457788358716> Deleted `{}` punishments",
                                deleted_count
                            )
                            .as_str(),
                        )?
                        .await?;
                }
            }
        }

        Ok(())
    }

    pub async fn expire_punishment_cmd(
        &self,
        conf: &Config,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = &msg.content;

        let uuid_list: Vec<String> = util::regex::UUID
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect();

        let guild_id = match &msg.guild_id {
            Some(id) => id.to_string(),
            None => return Ok(()),
        };

        let author_id = msg.author.id.to_string();
        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None,
        };

        let punishment_list = self.db.get_actions_by_uuid(&guild_id, uuid_list).await?;
        for punishment in &punishment_list {
            if &punishment.issuer == &author_id {
                let ok = permissions::check_permission(
                    conf,
                    roles,
                    msg.author.id,
                    permissions::PERMISSION_UPDATE,
                );
                if !ok {
                    self.rest.create_message(msg.channel_id)
                            .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", permissions::PERMISSION_UPDATESELF).as_str())?
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
                    self.rest.create_message(msg.channel_id)
                        .content(format!("<:mesaCross:832350526414127195> You do not have permission to `{}`", permissions::PERMISSION_UPDATESELF).as_str())?
                        .await?;
                    return Ok(());
                }

                let guild_id_marker = msg.guild_id.unwrap();
                let original_issuer_id = Id::from_str(punishment.issuer.as_str())?;
                let original_issuer_roles =
                    match self.cache.member(guild_id_marker, original_issuer_id) {
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

                let author_roles = match &msg.member {
                    Some(member) => member.roles.to_vec(),
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
        }

        let uuid_list = &punishment_list.iter().map(|p| p.uuid.clone()).collect();
        let res = self
            .db
            .expire_actions(Some(guild_id), &uuid_list, None)
            .await?;

        match res {
            UpdateResult { modified_count, .. } => {
                if modified_count < 1 {
                    self.rest
                        .create_message(msg.channel_id)
                        .content(
                            format!(
                                "<:mesaCross:832350526414127195> Unable to expire `{}` punishments",
                                punishment_list.len()
                            )
                            .as_str(),
                        )?
                        .await?;
                } else {
                    self.rest
                        .create_message(msg.channel_id)
                        .content(
                            format!(
                                "<:mesaUnstrike:869664457788358716> Expired `{}` punishments",
                                modified_count
                            )
                            .as_str(),
                        )?
                        .await?;
                }
            }
        }

        Ok(())
    }

    pub async fn purge_cmd(
        &self,
        conf: &Config,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let guild_id = match &msg.guild_id {
            Some(id) => id.to_string(),
            None => return Ok(()),
        };

        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None,
        };
        let ok = permissions::check_permission(
            conf,
            roles,
            msg.author.id,
            permissions::PERMISSION_PURGE,
        );
        if !ok {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    format!(
                        "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                        permissions::PERMISSION_PURGE
                    )
                    .as_str(),
                )?
                .await?;

            return Ok(());
        }

        let mut args = msg.content.split_whitespace();

        let typ = args.nth(1);

        let purge_type = PurgeType::from(typ.unwrap_or("all"));

        let mut filter: Option<String> = None;

        if purge_type == PurgeType::String {
            filter = Some(args.nth(1).unwrap_or("").to_string());
        }

        let mut user_ids: Option<Vec<String>> = None;

        if purge_type == PurgeType::User {
            user_ids = Some(
                util::regex::SNOWFLAKE
                    .find_iter(&msg.content)
                    .map(|m| m.as_str().to_string())
                    .collect(),
            );
        }

        let limit = args.nth(1).unwrap_or("0").parse::<u16>().unwrap_or(0);

        if limit == 0 {
            self.rest
                .create_message(msg.channel_id)
                .content("<:mesaCross:832350526414127195> Invalid number of messages to purge")?
                .await?;
            return Ok(());
        }

        let purge = Purge {
            typ: purge_type,
            initiated_by: msg.author.clone(),
            guild_id,
            channel_id: msg.channel_id.to_string(),
            limit,
            user_ids,
            filter,
        };

        match &self.arc {
            Some(handler) => add_purge(handler.to_owned(), purge).await?,
            None => return Ok(()),
        }

        Ok(())
    }

    pub async fn stop_purge_cmd(
        &self,
        conf: &Config,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let guild_id = match &msg.guild_id {
            Some(id) => id.to_string(),
            None => return Ok(()),
        };

        let roles = match &msg.member {
            Some(member) => Some(&member.roles),
            None => None,
        };
        let ok = permissions::check_permission(
            conf,
            roles,
            msg.author.id,
            permissions::PERMISSION_PURGE,
        );
        if !ok {
            self.rest
                .create_message(msg.channel_id)
                .content(
                    format!(
                        "<:mesaCross:832350526414127195> You do not have permission to `{}`",
                        permissions::PERMISSION_PURGE
                    )
                    .as_str(),
                )?
                .await?;

            return Ok(());
        }

        purge::stop_purge(guild_id).await?;

        Ok(())
    }
}
