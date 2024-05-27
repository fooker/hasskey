use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use url::Url;

#[derive(Deserialize)]
pub struct HomeAssistantConfig {
    pub url: Url,
    pub token: String,
}

#[serde_as]
#[derive(Deserialize)]
pub struct DeviceConfig {
    pub name: String,
    #[serde_as(as = "HashMap<_, Option<DisplayFromStr>>")]
    pub filter: HashMap<String, Option<regex::bytes::Regex>>,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(alias = "home-assistant", alias = "hass")]
    pub home_assistant: HomeAssistantConfig,

    pub devices: Vec<DeviceConfig>,
}

impl Config {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let config = tokio::fs::read(path)
            .await
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config = serde_yaml::from_slice(config.as_slice())
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        return Ok(config);
    }
}
