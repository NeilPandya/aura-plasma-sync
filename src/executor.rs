// src/executor.rs
// Manages synchronous hardware-level LED updates via asusctl.

use anyhow::{Context, Result, bail};
use std::process::Command;

/// Sync colors to Aura devices
pub fn sync_colors(rgb: [u8; 3]) -> Result<()> {
    let hex = crate::color::to_hex_string(rgb);

    // 1. Capture current brightness (default to "med" if reading fails)
    let level = get_current_brightness_level().unwrap_or_else(|_| "med".to_string());

    // 2. Apply the color effect
    execute_aura_static_effect(&hex)?;

    // 3. Restore brightness level
    // Aura effect commands often reset the LED state to 'on' or 'max'
    if let Err(e) = set_brightness_level(&level) {
        log::warn!("Could not restore keyboard brightness: {}", e);
    }

    log::info!("Hardware Updated: #{} (brightness level: {})", hex, level);
    Ok(())
}

/// Execute the aura static effect command
fn execute_aura_static_effect(hex: &str) -> Result<()> {
    let output = Command::new("asusctl")
        .args(["aura", "effect", "static", "-c", hex])
        .output()
        .context("Failed to execute asusctl color command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "asusctl color command failed ({:?}): {}",
            output.status.code().unwrap_or(1),
            stderr.trim()
        );
    }
    Ok(())
}

/// Get current keyboard brightness level (off, low, med, high)
fn get_current_brightness_level() -> Result<String> {
    let output = Command::new("asusctl")
        .args(["leds", "get"])
        .output()
        .context("Failed to execute asusctl leds get")?;

    if !output.status.success() {
        bail!("asusctl leds get failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let brightness_str = stdout.trim();

    // Parse level from output (e.g., "Keyboard brightness: Med")
    let level = brightness_str
        .split_once(':')
        .map(|(_, val)| val.trim())
        .unwrap_or(brightness_str)
        .to_lowercase();

    // Validate against known asusctl levels
    match level.as_str() {
        "off" | "low" | "med" | "high" => Ok(level),
        _ => {
            log::warn!("Unknown brightness level '{}', defaulting to 'med'", level);
            Ok("med".to_string())
        }
    }
}

/// Set keyboard brightness level (off, low, med, high)
fn set_brightness_level(level: &str) -> Result<()> {
    let status = Command::new("asusctl")
        .args(["leds", "set", level])
        .status()
        .context("Failed to execute asusctl leds set")?;

    if !status.success() {
        bail!("asusctl leds set failed for level: {}", level);
    }

    Ok(())
}
