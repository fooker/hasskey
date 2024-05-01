use anyhow::{Context, Result};
use evdev::BusType;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Deserialize)]
pub struct HomeAssistant {
    pub url: Url,
    pub token: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceFilter {
    Path(PathBuf),
    Input(String),
    Device {
        bus_type: Option<BusType>,
        vendor: Option<u16>,
        product: Option<u16>,
        version: Option<u16>,
    },
}

#[derive(Deserialize)]
pub struct Device {
    pub name: String,

    #[serde(flatten)]
    pub filter: DeviceFilter,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(alias = "home-assistant", alias = "hass")]
    pub home_assistant: HomeAssistant,

    pub devices: Vec<Device>,
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
