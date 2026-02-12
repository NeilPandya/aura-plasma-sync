// src/executor.rs

use anyhow::{Context, Result, bail};
use std::process::Command;

/// Sets both aura color and preserves keyboard brightness
pub fn set_aura_color_with_brightness_preservation(hex: &str) -> Result<()> {
    let manager = KeyboardBrightnessManager::new()?;
    manager.preserve_during(|| execute_aura_static_effect(hex))?;

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

/// This ensures the brightness is set back even if the main operation fails.
struct BrightnessGuard {
    level: u8,
}

impl BrightnessGuard {
    fn new(level: u8) -> Self {
        Self { level }
    }
}

impl Drop for BrightnessGuard {
    fn drop(&mut self) {
        if let Err(e) = set_keyboard_brightness_level(self.level) {
            log::error!(
                "Critical failure: could not restore brightness in Drop guard: {}",
                e
            );
        }
    }
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
    pub fn preserve_during<F>(&self, operation: F) -> Result<()>
    where
        F: FnOnce() -> Result<()>,
    {
        let _guard = BrightnessGuard::new(self.level);
        operation()
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    let brightness_str = stdout.trim();

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
                "Unknown brightness level '{}' (full output: '{}'), defaulting to medium",
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
