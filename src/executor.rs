// src/executor.rs

use anyhow::{Context, Result, bail};
use std::process::Command;

/// Interfaces with Asus hardware via asusctl
pub fn set_aura_color(hex: &str) -> Result<()> {
    let status = Command::new("asusctl")
        .args(["aura", "effect", "static", "-c", hex])
        .status()
        .context("Failed to execute asusctl command")?;

    if !status.success() {
        bail!(
            "asusctl failed with exit code {:?}",
            status.code().unwrap_or(1)
        );
    }

    log::info!("Hardware Updated: #{}", hex);
    Ok(())
}
