use bm_lib::{
    discord::{DiscordError, DiscordResult, EmbedBuilder, Id},
    emojis::Emoji,
    model::Infraction,
    permissions::Permission,
};
use tracing::instrument;

use crate::{handler::EventHandler, AUTHOR_COLON_THREE, SERVICE_NAME};

use super::ZWSP;

impl EventHandler {
    pub async fn send_error(&self, channel_id: &Id, error: DiscordError) -> DiscordResult<()> {
        let embed = EmbedBuilder::new()
            .title("Error")
            .description("An error occurred while processing your command")
            .color(0xFF0000)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
            .field("Error", error.to_string().as_str(), false)
            .field("Please report this to the developers", ZWSP, false)
            .build();

        self.rest
            .create_message_with_embed(&channel_id, &vec![embed])
            .await?;

        Ok(())
    }

    #[instrument(skip(self, infractions))]
    pub async fn send_infraction_channel_embed(
        &self,
        channel_id: &Id,
        infractions: &[Infraction],
    ) -> DiscordResult<()> {
        if infractions.is_empty() {
            tracing::warn!("send_infraction_channel called with no infractions");
            return Ok(());
        }

        let typ = infractions[0].infraction_type.to_past_tense();

        let mut embed = EmbedBuilder::new()
            .title(format!("Successfully {} {} users", typ, infractions.len()))
            .description(format!("The following users have been {}:", typ))
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None);

        for infraction in infractions {
            let dur_str = match infraction.expires_at {
                Some(expires) => format!("<t:{}:R>", expires),
                None => "`Never`".to_string(),
            };

            let mut reason = infraction
                .reason
                .as_deref()
                .map(|s| s.chars().take(61).collect::<String>())
                .unwrap_or_else(|| String::from("No reason provided"));

            if reason.len() == 61 {
                reason.push_str("...");
            }

            embed = embed.field(
                format!("<@{}>", infraction.user_id),
                format!("**Reason:** {}\n**Expires:** {}", reason, dur_str).as_str(),
                true,
            );
        }

        let embed = embed.build();

        self.rest
            .create_message_with_embed(&channel_id, &vec![embed])
            .await?;

        Ok(())
    }

    #[instrument(skip(self, infractions))]
    pub async fn send_infraction_channel_text(
        &self,
        channel_id: &Id,
        infractions: &[Infraction],
    ) -> DiscordResult<()> {
        if infractions.is_empty() {
            tracing::warn!("send_infraction_channel_text called with no infractions");
            return Ok(());
        }

        let typ = infractions[0].infraction_type.to_past_tense();
        let reason = infractions[0]
            .reason
            .as_deref()
            .unwrap_or_else(|| "No reason provided");

        let expires = match infractions[0].expires_at {
            Some(expires) => format!("<t:{}:R>", expires),
            None => "`Never`".to_string(),
        };

        let mut content = format!("{} Successfully {} ", Emoji::Check, typ);

        if infractions.len() > 20 {
            content.push_str(&format!("{} users", infractions.len()));
        } else {
            content.push_str(&format!(
                "{}",
                infractions
                    .iter()
                    .map(|infraction| format!("<@{}>", infraction.user_id))
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
        }

        content.push_str(&format!(
            " for the reason: {}. Expires: {}.",
            reason, expires
        ));

        self.rest
            .create_message_no_ping(&channel_id, &content)
            .await?;

        Ok(())
    }

    pub async fn send_infraction_channel(
        &self,
        channel_id: &Id,
        infractions: &[Infraction],
        prefer_embeds: bool,
    ) -> DiscordResult<()> {
        if prefer_embeds {
            self.send_infraction_channel_embed(channel_id, infractions)
                .await
        } else {
            self.send_infraction_channel_text(channel_id, infractions)
                .await
        }
    }

    #[instrument(skip(self, infraction), fields(user_id = infraction.user_id.get()))]
    pub async fn send_infraction_dm(&self, infraction: &Infraction) -> DiscordResult<()> {
        let channel_id = match self.get_user_dm_channel(&infraction.user_id).await {
            Ok(channel_id) => channel_id,
            Err(e) => {
                tracing::warn!("Failed to get DM channel for user: {}", e);
                return Ok(());
            }
        };

        let guild_name = match self.get_guild(&infraction.guild_id).await {
            Ok(guild) => guild.name,
            Err(e) => {
                tracing::warn!("Failed to get guild for infraction DM: {}", e);
                infraction
                    .guild_id
                    .to_string()
                    .parse()
                    .map_err(|_| DiscordError::Other("Failed to parse guild ID".to_string()))?
            }
        };

        let mut embed = EmbedBuilder::new()
            .title(
                format!(
                    "You have recieved a {} from {}",
                    infraction.infraction_type.to_noun(),
                    guild_name
                )
                .as_str(),
            )
            .description(
                format!(
                    "You have been {} in {}",
                    infraction.infraction_type.to_past_tense(),
                    guild_name
                )
                .as_str(),
            )
            .field(
                "Moderator",
                format!("<@{}>", infraction.moderator_id).as_str(),
                true,
            )
            .field(
                "Reason",
                infraction
                    .reason
                    .as_deref()
                    .unwrap_or_else(|| "No reason provided"),
                true,
            )
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None);

        if let Some(expires) = infraction.expires_at {
            embed = embed.field("Expires", format!("<t:{}:R>", expires).as_str(), true);
        }

        let embed = embed.build();

        self.rest
            .create_message_with_embed_and_forget(&channel_id, &vec![embed])
            .await;
        Ok(())
    }

    #[instrument(skip(self, infraction), fields(user_id = infraction.user_id.get()))]
    pub async fn send_infraction_remove_dm(&self, infraction: &Infraction) -> DiscordResult<()> {
        let channel_id = match self.get_user_dm_channel(&infraction.user_id).await {
            Ok(channel_id) => channel_id,
            Err(e) => {
                tracing::warn!("Failed to get DM channel for user in remove: {}", e);
                return Ok(());
            }
        };

        let embed = EmbedBuilder::new()
            .title("Infraction Removed")
            .description(
                format!(
                    "Your {} from {} has been removed",
                    infraction.infraction_type.to_noun(),
                    infraction.guild_id
                )
                .as_str(),
            )
            .field(
                "Moderator",
                format!("<@{}>", infraction.moderator_id).as_str(),
                true,
            )
            .field(
                "Reason",
                infraction
                    .reason
                    .as_deref()
                    .unwrap_or_else(|| "No reason provided"),
                true,
            )
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
            .build();

        self.rest
            .create_message_with_embed(&channel_id, &vec![embed])
            .await?;

        Ok(())
    }

    #[instrument(skip(self, infraction))]
    pub async fn send_infraction_remove_channel(
        &self,
        channel_id: &Id,
        infraction: &Infraction,
    ) -> DiscordResult<()> {
        let embed = EmbedBuilder::new()
            .title("Infraction Removed")
            .description(
                format!(
                    "User <@{}>'s {} has been removed",
                    infraction.user_id,
                    infraction.infraction_type.to_noun()
                )
                .as_str(),
            )
            .field(
                "Moderator",
                format!("<@{}>", infraction.moderator_id).as_str(),
                true,
            )
            .field(
                "Reason",
                infraction
                    .reason
                    .as_deref()
                    .unwrap_or_else(|| "No reason provided"),
                true,
            )
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
            .build();

        self.rest
            .create_message_with_embed(&channel_id, &vec![embed])
            .await?;

        Ok(())
    }

    #[instrument(skip(self, permission))]
    pub async fn send_permission_denied(
        &self,
        channel_id: &Id,
        permission: Permission,
    ) -> DiscordResult<()> {
        let embed = EmbedBuilder::new()
            .title("Permission Denied")
            .description("You do not have permission to use this command")
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
            .field(ZWSP, format!("**Required:** `{}`", permission), false)
            .build();

        self.rest
            .create_message_with_embed(&channel_id, &vec![embed])
            .await?;

        Ok(())
    }

    #[instrument(skip(self, permission))]
    pub async fn send_permission_denied_text(
        &self,
        channel_id: &Id,
        permission: Permission,
    ) -> DiscordResult<()> {
        self.rest
            .create_message(
                &channel_id,
                &format!(
                    "{} You do not have permission to use this command. Required `{}`",
                    Emoji::Cross,
                    permission
                ),
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn send_cant_target_user(&self, channel_id: &Id) -> DiscordResult<()> {
        let embed = EmbedBuilder::new()
            .title("Can't Target User")
            .description("You can't target that user")
            .color(0xFF8C00)
            .footer(format!("{SERVICE_NAME} by {AUTHOR_COLON_THREE}"), None)
            .build();

        self.rest
            .create_message_with_embed(&channel_id, &vec![embed])
            .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn send_cant_target_user_text(&self, channel_id: &Id) -> DiscordResult<()> {
        self.rest
            .create_message(
                &channel_id,
                format!("{} You can't target that user", Emoji::Cross).as_str(),
            )
            .await?;

        Ok(())
    }
}
