use std::path::{Path, PathBuf};
use std::pin::Pin;

use anyhow::{Context, Result};
use clap::Parser;
use evdev::InputEventKind;
use futures::stream::SelectAll;
use futures::{Stream, StreamExt, TryStreamExt};
use tokio_udev as udev;
use tracing::{debug, error, info, trace, Level};

use crate::config::DeviceConfig;
use crate::hass::{EventData, EventValue, HomeAssistantClient};

pub mod config;
pub mod hass;

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
    let config = config::Config::load(&config)
        .await
        .with_context(|| format!("Failed to load config: {}", config.display()))?;

    let hass = HomeAssistantClient::new(config.home_assistant)
        .await
        .context("Failed to initialize home-assistant client")?;

    info!("🥳 Go Go Go");

    let mut devices = SelectAll::new();

    let mut enumerator = udev::Enumerator::new().context("Failed to create enumerator")?;
    enumerator
        .match_subsystem("input")
        .context("Failed to match input subsystem")?;

    for device in enumerator
        .scan_devices()
        .context("Failed to enumerate devices")?
    {
        if let Some(devnode) = device.devnode() {
            if let Some(name) = match_device(&device, &config.devices) {
                debug!(
                    "Matched device {name}: {devnode}",
                    devnode = devnode.display()
                );

                let device = handle_device(name, devnode)?;
                devices.push(device);
            }
        }
    }

    let monitor = udev::MonitorBuilder::new()?
        .match_subsystem("input")?
        .listen()?;
    let mut monitor = udev::AsyncMonitorSocket::new(monitor)?;

    loop {
        tokio::select! {
             Some(Ok(event)) = monitor.next() => {
                 debug!("Received udev event: {:?}", event);
                 match event.event_type() {
                     udev::EventType::Add => {
                         let device = event.device();
                         if let Some(devnode) = device.devnode() {
                             if let Some(name) = match_device(&device, &config.devices) {
                                 debug!("Matched device {name}: {devnode}", devnode=devnode.display());

                                 let device = handle_device(name, devnode)?;
                                 devices.push(device);
                             }
                         }
                     }

                     udev::EventType::Remove => {

                     }

                     _ => {}
                 }
             }

             Some(event) = devices.next() => {
                 hass.send_event(event).await;
             }
        }
    }
}

fn match_device(device: &udev::Device, config: &[DeviceConfig]) -> Option<String> {
    fn match_key(device: &udev::Device, key: &str, filter: Option<&regex::bytes::Regex>) -> bool {
        if let Some(value) = device.property_value(key) {
            return filter.map_or(false, |filter| filter.is_match(value.as_encoded_bytes()));
        }

        if let Some(parent) = device.parent() {
            return match_key(&parent, key, filter);
        }

        return false;
    }

    return config
        .iter()
        .find(|config| {
            config
                .filter
                .iter()
                .all(|(key, filter)| match_key(&device, key, filter.as_ref()))
        })
        .map(|config| config.name.clone());
}

fn handle_device(
    name: String,
    devnode: impl AsRef<Path>,
) -> Result<Pin<Box<dyn Stream<Item = EventData> + Send>>> {
    let devnode = devnode.as_ref();

    let device = evdev::Device::open(&devnode).with_context(|| {
        format!(
            "Failed to open input device {}: {}",
            name,
            devnode.display()
        )
    })?;

    let device = device
        .into_event_stream()
        .with_context(|| {
            format!(
                "Failed to stream input device {}: {}",
                name,
                devnode.display()
            )
        })?
        .err_into::<anyhow::Error>()
        .inspect(|event| trace!("Got evdev event: {event:?}"));

    let device = device.map_ok(move |event| match event.kind() {
        InputEventKind::Key(key) => Some(EventData {
            device: name.clone(),
            key,
            value: match event.value() {
                0 => EventValue::UP,
                1 => EventValue::DOWN,
                _ => unreachable!("Unsupported evdev event value"),
            },
        }),

        _ => None,
    });

    let device = device.filter_map(|event| async {
        match event {
            Ok(event) => event, // Flatten the Option<_> using the filter part
            Err(err) => {
                error!("Error reading event: {}", err);
                return None;
            }
        }
    });

    return Ok(Box::pin(device));
}
