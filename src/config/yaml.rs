use std::error::Error;

use super::Config;

impl Config {
    pub fn to_yaml(&self) -> Result<String, Box<dyn Error>> {
        Ok(serde_yaml::to_string(&self)?)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, Box<dyn Error>> {
        Ok(serde_yaml::from_str(yaml)?)
    }
}
