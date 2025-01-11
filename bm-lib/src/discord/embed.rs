use super::model::{Embed, EmbedAuthor, EmbedField, EmbedFooter, EmbedImage, EmbedThumbnail};
use std::time::SystemTime;

#[derive(Default)]
pub struct EmbedBuilder {
    embed: Embed,
}

impl EmbedBuilder {
    pub fn new() -> Self {
        Self {
            embed: Embed::new(),
        }
    }

    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.embed.title = Some(title.into());
        self
    }

    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.embed.description = Some(description.into());
        self
    }

    pub fn url<S: Into<String>>(mut self, url: S) -> Self {
        self.embed.url = Some(url.into());
        self
    }

    pub fn timestamp(mut self) -> Self {
        if let Ok(time) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            self.embed.timestamp = Some(time.as_secs().to_string());
        }
        self
    }

    pub fn color(mut self, color: u32) -> Self {
        self.embed.color = Some(color);
        self
    }

    pub fn footer(mut self, text: impl Into<String>, icon_url: Option<String>) -> Self {
        self.embed.footer = Some(EmbedFooter {
            text: text.into(),
            icon_url,
            proxy_icon_url: None,
        });
        self
    }

    pub fn image(mut self, url: impl Into<String>) -> Self {
        self.embed.image = Some(EmbedImage {
            url: url.into(),
            proxy_url: None,
            height: None,
            width: None,
        });
        self
    }

    pub fn thumbnail(mut self, url: impl Into<String>) -> Self {
        self.embed.thumbnail = Some(EmbedThumbnail {
            url: url.into(),
            proxy_url: None,
            height: None,
            width: None,
        });
        self
    }

    pub fn author(
        mut self,
        name: impl Into<String>,
        url: Option<String>,
        icon_url: Option<String>,
    ) -> Self {
        self.embed.author = Some(EmbedAuthor {
            name: name.into(),
            url,
            icon_url,
            proxy_icon_url: None,
        });
        self
    }

    pub fn field(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
        inline: bool,
    ) -> Self {
        let field = EmbedField {
            name: name.into(),
            value: value.into(),
            inline: Some(inline),
        };

        if let Some(ref mut fields) = self.embed.fields {
            fields.push(field);
        } else {
            self.embed.fields = Some(vec![field]);
        }
        self
    }

    pub fn build(self) -> Embed {
        self.embed
    }
}
