use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Antiraid {
    pub enabled: Option<bool>,
}
