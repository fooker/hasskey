use anyhow::Context;
use anyhow::Result;
use serde::Serialize;
use tracing::instrument;
use tracing::log::error;
use tracing::Level;
use url::Url;

use crate::config::HomeAssistantConfig;

pub struct HomeAssistantClient {
    client: reqwest::Client,

    url: Url,
    token: String,
}

impl HomeAssistantClient {
    const EVENT_TYPE: &'static str = "hasskey";

    pub async fn new(config: HomeAssistantConfig) -> Result<Self> {
        let url = config
            .url
            .join("api/events/")
            .context("Invalid URL")?
            .join(Self::EVENT_TYPE)
            .context("Invalid URL")?;

        let client = reqwest::Client::new();

        return Ok(Self {
            client,
            url,
            token: config.token.read().await?.to_string(),
        });
    }

    #[instrument(level = Level::DEBUG, skip(self))]
    pub async fn send_event(&self, event: EventData) {
        let result = self
            .client
            .post(self.url.clone())
            .bearer_auth(&self.token)
            .json(&event)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status);

        if let Err(err) = result {
            error!("Failed to send event: {}", err);
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
pub enum EventValue {
    UP,
    DOWN,
}

#[derive(Serialize, Debug)]
pub struct EventData {
    pub device: String,
    pub key: evdev::Key,
    pub value: EventValue,
}
