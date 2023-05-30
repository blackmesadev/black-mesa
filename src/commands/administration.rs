use std::str::FromStr;

use lazy_static::lazy_static;
use mongodb::results::{DeleteResult, UpdateResult};
use regex::Regex;
use twilight_model::{channel::Message, id::Id};

use crate::{handlers::Handler, mongo::mongo::Config, util::permissions};

impl Handler {
    pub async fn remove_punishment_cmd(
        &self,
        conf: &Config,
        msg: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = &msg.content;
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b")
                    .unwrap();
        }

        let uuid_list: Vec<String> = RE
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
                    &author_id,
                    vec![permissions::PERMISSION_REMOVEACTIONSELF],
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
                    &author_id,
                    vec![permissions::PERMISSION_REMOVEACTION],
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

                let original_issuer_level = permissions::get_user_level(
                    conf,
                    Some(&original_issuer_roles),
                    &original_issuer_id.to_string(),
                );

                let author_level = permissions::get_user_level(conf, roles, &author_id);

                if original_issuer_level >= author_level
                    && !conf.modules.moderation.update_higher_level_action
                {
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
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b")
                    .unwrap();
        }

        let uuid_list: Vec<String> = RE
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
                    &author_id,
                    vec![permissions::PERMISSION_UPDATE],
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
                    &author_id,
                    vec![permissions::PERMISSION_UPDATE],
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

                let original_issuer_level = permissions::get_user_level(
                    conf,
                    Some(&original_issuer_roles),
                    &original_issuer_id.to_string(),
                );

                let author_level = permissions::get_user_level(conf, roles, &author_id);

                if original_issuer_level >= author_level
                    && !conf.modules.moderation.update_higher_level_action
                {
                    self.rest.create_message(msg.channel_id)
                        .content("<:mesaCross:832350526414127195> You do not have permission to update this punishment as it is of a user of equal or higher level")?
                        .await?;
                    return Ok(());
                }
            }
        }

        let uuid_list = &punishment_list.iter().map(|p| p.uuid.clone()).collect();
        let res = self.db.expire_actions(Some(guild_id), &uuid_list).await?;

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
}
