// src/executor.rs
// Manages synchronous hardware-level LED updates via asusctl.

use anyhow::{Context, Result, bail};
use std::process::Command;

// Public API: sync colors to Aura devices
pub fn sync_colors(rgb: [u8; 3]) -> Result<()> {
    let hex = crate::color::to_hex_string(rgb);

    // Capture current brightness (default to 'med' if reading fails)
    let level = get_current_keyboard_brightness_level().unwrap_or(2);

    // Apply the color effect
    execute_aura_static_effect(&hex)?;

    // Restore brightness level
    // Aura effect commands often reset the LED state to 'on' or 'max'
    if let Err(e) = set_keyboard_brightness_level(level) {
        log::warn!("Could not restore keyboard brightness: {}", e);
    }

    log::info!("Hardware Updated: #{} (brightness level: {})", hex, level);
    Ok(())
}

// Execute the aura static effect command
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

// Get current keyboard brightness level (0=off, 1=low, 2=med, 3=high)
fn get_current_keyboard_brightness_level() -> Result<u8> {
    let output = Command::new("asusctl")
        .args(["leds", "get"])
        .output()
        .context("Failed to execute asusctl leds get")?;

    if !output.status.success() {
        bail!("asusctl leds get failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let brightness_str = stdout.trim();

    // Parse the level from output (e.g., "Keyboard brightness: Med")
    let brightness_level = brightness_str
        .split_once(':')
        .map(|(_, val)| val.trim())
        .unwrap_or(brightness_str);

    match brightness_level.to_lowercase().as_str() {
        "off" => Ok(0),
        "low" => Ok(1),
        "med" => Ok(2),
        "high" => Ok(3),
        _ => {
            log::warn!(
                "Unknown brightness level '{}', defaulting to medium",
                brightness_level
            );
            Ok(2)
        }
    }
}

// Set keyboard brightness level (0=off, 1=low, 2=med, 3=high)
fn set_keyboard_brightness_level(level: u8) -> Result<()> {
    let level_str = match level {
        0 => "off",
        1 => "low",
        2 => "med",
        3 => "high",
        _ => "med",
    };

    let status = Command::new("asusctl")
        .args(["leds", "set", level_str])
        .status()
        .context("Failed to execute asusctl leds set")?;

    if !status.success() {
        bail!("asusctl leds set failed");
    }

    Ok(())
}
