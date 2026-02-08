// src/executor.rs

use anyhow::{Context, Result, bail};
use std::process::Command;

/// Sets both aura color and preserves keyboard brightness
pub fn set_aura_color_with_brightness_preservation(hex: &str) -> Result<()> {
    let brightness_manager = KeyboardBrightnessManager::new()?;
    brightness_manager.preserve_during(|| execute_aura_static_effect(hex))?;
    log::info!("Hardware Updated: #{} (brightness preserved)", hex);
    Ok(())
}

/// Execute the aura static effect command
fn execute_aura_static_effect(hex: &str) -> Result<()> {
    let status = Command::new("asusctl")
        .args(["aura", "effect", "static", "-c", hex])
        .status()
        .context("Failed to execute asusctl color command")?;

    if !status.success() {
        bail!(
            "asusctl color command failed with exit code {:?}",
            status.code().unwrap_or(1)
        );
    }
    Ok(())
}

/// Manages keyboard brightness state
struct KeyboardBrightnessManager {
    level: u8,
}

impl KeyboardBrightnessManager {
    /// Create a new brightness manager by reading current brightness
    pub fn new() -> Result<Self> {
        let level = get_current_keyboard_brightness_level()?;
        Ok(Self { level })
    }

    /// Execute an operation while preserving the current brightness level
    pub fn preserve_during<F>(&self, operation: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        // Execute the operation
        operation()?;

        // Restore brightness
        set_keyboard_brightness_level(self.level)?;
        Ok(())
    }
}

/// Get current keyboard brightness level (0=off, 1=low, 2=med, 3=high)
fn get_current_keyboard_brightness_level() -> Result<u8> {
    let output = Command::new("asusctl")
        .args(["leds", "get"])
        .output()
        .context("Failed to execute asusctl leds get")?;

    if !output.status.success() {
        bail!("asusctl leds get failed");
    }

    let brightness_str = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_lowercase();

    // Extract just the brightness level from the full string
    // e.g., "current keyboard led brightness: low" -> "low"
    let brightness_level = if let Some(pos) = brightness_str.find(':') {
        brightness_str[pos + 2..].trim() // Skip ": " after the colon
    } else {
        &brightness_str // Fallback to whole string if no colon found
    };

    match brightness_level {
        "off" => Ok(0),
        "low" => Ok(1),
        "med" => Ok(2),
        "high" => Ok(3),
        _ => {
            log::warn!(
                "Unknown brightness level '{}' (parsed from '{}'), defaulting to medium",
                brightness_level,
                brightness_str
            );
            Ok(2) // Default to medium
        }
    }
}

/// Set keyboard brightness level (0=off, 1=low, 2=med, 3=high)
fn set_keyboard_brightness_level(level: u8) -> Result<()> {
    let level_str = match level {
        0 => "off",
        1 => "low",
        2 => "med",
        3 => "high",
        _ => {
            log::warn!("Invalid brightness level {}, defaulting to medium", level);
            "med"
        }
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
