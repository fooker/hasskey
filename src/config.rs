use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use url::Url;

#[derive(Debug, Deserialize)]
pub enum Secret {
    #[serde(rename = "path")]
    Reference(PathBuf),

    #[serde(untagged)]
    Literal(String),
}

impl Secret {
    pub async fn read(&self) -> Result<Cow<str>> {
        return match self {
            Secret::Literal(secret) => Ok(Cow::Borrowed(secret)),
            Secret::Reference(path) => Ok(Cow::Owned(
                tokio::fs::read_to_string(&path)
                    .await
                    .with_context(|| format!("Failed to read secret: {}", path.display()))?,
            )),
        };
    }
}

#[derive(Deserialize)]
pub struct HomeAssistantConfig {
    pub url: Url,
    pub token: Secret,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GrabConfig {
    Exclusive,
    Shared,
}

impl Default for GrabConfig {
    fn default() -> Self {
        return Self::Exclusive;
    }
}

#[serde_as]
#[derive(Deserialize)]
pub struct DeviceConfig {
    pub name: String,
    #[serde_as(as = "HashMap<_, Option<DisplayFromStr>>")]
    pub filter: HashMap<String, Option<regex::bytes::Regex>>,

    #[serde(default)]
    pub grab: GrabConfig,
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
