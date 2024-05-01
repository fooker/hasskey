use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use futures::{future, TryStreamExt};
use serde::Serialize;
use tokio_stream::{Stream, StreamExt};
use tracing::{debug, error, info, Level};

pub mod config;

const EVENT_TYPE: &'static str = "hasskey";

#[derive(Serialize, Debug)]
pub struct EventData {
    device: String,
    key: evdev::Key,
}

#[derive(Parser)]
#[command(version, about)]
struct CLI {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CLI::parse();

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(match cli.verbose {
            0 => Level::WARN,
            1 => Level::INFO,
            2 => Level::DEBUG,
            _ => Level::TRACE,
        })
        .init();

    let config = cli.config.unwrap_or("./config.yaml".into());
    let config = config::Config::load(&config).await
        .with_context(|| format!("Failed to load config: {}", config.display()))?;

    let mut threads = Vec::new();

    let client = reqwest::Client::new();

    for device in config.devices {
        debug!("Processing device: {}", device.name);

        let client = client.clone();

        let home_assistant = &config.home_assistant;

        threads.push(async move {
            let mut events = handle_device(device)?;
            while let Some(event) = events.next().await {
                let event = event?;

                debug!("Got event: {:?}", event);

                let url = home_assistant.url
                    .join("api/events/").expect("Valid URL")
                    .join(EVENT_TYPE).expect("Valid URL");

                let result = client.post(url.clone())
                    .bearer_auth(&home_assistant.token)
                    .json(&event)
                    .send()
                    .await
                    .and_then(|response| response.error_for_status());

                if let Err(err) = result {
                    error!("Failed to send event: {}", err);
                }

                debug!("Event delivered");
            }

            return anyhow::Ok(());
        });
    }

    future::try_join_all(threads).await?;

    return Ok(());
}

fn handle_device(config: config::Device) -> Result<impl Stream<Item=Result<EventData>>> {
    let device = match &config.filter {
        config::DeviceFilter::Path(path) => evdev::Device::open(&path)
            .with_context(|| format!("Failed to open input device: {}: {}", config.name, path.display()))?,

        config::DeviceFilter::Input(name) => evdev::enumerate()
            .map(|(_, device)| device)
            .find(|device| device.name().map_or(false, |device| device == name))
            .with_context(|| format!("No device found with name: {}: {}", config.name, name))?,

        config::DeviceFilter::Device {
            bus_type,
            vendor,
            product,
            version,
        } => evdev::enumerate()
            .map(|(_, device)| device)
            .find(|device| {
                let id = device.input_id();
                if !bus_type.map_or(true, |bus_type| id.bus_type() == bus_type) {
                    return false;
                }
                if !vendor.map_or(true, |vendor| id.vendor() == vendor) {
                    return false;
                }
                if !product.map_or(true, |product| id.product() == product) {
                    return false;
                }
                if !version.map_or(true, |version| id.version() == version) {
                    return false;
                }
                return true;
            })
            .with_context(|| format!("No device found: {} (bus={:?}, vendor={:?}, product={:?}, version={:?})", config.name, bus_type, vendor, product, version))?,
    };

    info!("Found device: {}: {:?}", config.name, device.input_id());

    let events = device.into_event_stream()
        .with_context(|| format!("Failed to open event stream for device: {}", config.name))?;

    let events = events
        .map_err(anyhow::Error::from)
        .try_filter_map(move |event| {
            debug!("Got event: {:?}", event);

            let event = match event.kind() {
                evdev::InputEventKind::Key(key) => EventData {
                    device: config.name.clone(),
                    key,
                },

                _ => {
                    return future::ready(Ok(None));
                }
            };

            return future::ready(Ok(Some(event)));
        });

    return Ok(events);
}
