use super::Guild;

impl Guild {
    pub fn icon_url(&self) -> Option<String> {
        if let Some(icon) = &self.icon {
            Some(format!(
                "https://cdn.discordapp.com/icons/{}/{}.png",
                self.id, icon
            ))
        } else {
            None
        }
    }
}
