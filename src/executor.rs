// src/executor.rs
use anyhow::{Context, Result, bail};
use openrgb2::{Color, OpenRgbClient};
use std::net::SocketAddr;
use std::sync::OnceLock;

const OPENRGB_PROTOCOL_VERSION: u32 = 4;

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub fn get_runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime")
    })
}

/// Sets the color for all OpenRGB devices
pub fn sync_colors(hex: &str, host: &str, port: u16) -> Result<()> {
    let color = parse_hex_to_color(hex)?;
    let rt = get_runtime();

    let addr_str = format!("{}:{}", host, port);
    let addr: SocketAddr = addr_str
        .parse()
        .with_context(|| format!("Invalid address: {}", addr_str))?;

    rt.block_on(async {
        let client = OpenRgbClient::connect_to(addr, OPENRGB_PROTOCOL_VERSION)
            .await
            .with_context(|| format!("Failed to connect to OpenRGB SDK server at {}:{} - is OpenRGB running with SDK server enabled?", host, port))?;

        // Get all controllers
        let controllers = client.get_all_controllers().await
            .context("Failed to retrieve controllers from OpenRGB")?;

        // Early exit if no controllers found
        if controllers.is_empty() {
            log::warn!("No OpenRGB controllers found - nothing to update");
            return Ok(());
        }

        log::debug!("Found {} controllers, preparing updates", controllers.len());

        // Create a command group for bulk updates
        let mut command_group = controllers.cmd();

        let mut update_count = 0;
        // Update all LEDs on all controllers
        for controller in controllers.controllers() {
            let led_count = controller.num_leds();
            if led_count > 0 {
                let colors = vec![color; led_count];
                command_group.set_controller_leds(controller.id(), colors)
                    .with_context(|| format!("Failed to queue LED update for controller '{}' (ID: {})",
                                           controller.name(), controller.id()))?;
                update_count += 1;
            } else {
                log::debug!("Controller '{}' (ID: {}) has 0 LEDs, skipping",
                           controller.name(), controller.id());
            }
        }

        if update_count == 0 {
            log::warn!("No controllers had LEDs to update");
            return Ok(());
        }

        log::debug!("Queued updates for {} controllers, executing...", update_count);

        // Execute all commands in a single network call
        command_group.execute()
            .await
            .context("Failed to execute LED updates - check OpenRGB SDK server connection")?;

        log::debug!("Successfully updated {} controllers", update_count);
        Ok::<(), anyhow::Error>(())
    })?;

    log::info!(
        "Hardware Updated: #{} (connected to {}:{} protocol v{})",
        hex,
        host,
        port,
        OPENRGB_PROTOCOL_VERSION
    );
    Ok(())
}

fn parse_hex_to_color(hex_str: &str) -> Result<Color> {
    let clean_hex = hex_str.trim_start_matches('#');
    let bytes = hex::decode(clean_hex)
        .with_context(|| format!("Invalid hex color format: '{}'", hex_str))?;

    if bytes.len() != 3 {
        bail!(
            "Expected 3 bytes for RGB (RRGGBB), got {} bytes",
            bytes.len()
        );
    }

    let color = Color::new(bytes[0], bytes[1], bytes[2]);
    log::debug!(
        "Parsed color: #{:02x}{:02x}{:02x}",
        color.r,
        color.g,
        color.b
    );
    Ok(color)
}
