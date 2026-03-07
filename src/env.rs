// src/env.rs
// Validates system environment requirements and verifies asusctl availability on startup.

use anyhow::{Context, Result, bail};
use std::process::Command;

/// Validates that asusctl is installed and reachable
pub fn validate_dependencies() -> Result<()> {
    let output = Command::new("asusctl")
        .arg("info")
        .output()
        .context("Could not execute 'asusctl'. Ensure it is installed and in your PATH.")?;

    if !output.status.success() {
        bail!("'asusctl info' returned an error. Ensure the asusd daemon is running.");
    }

    log::info!(
        "asusctl connected: {}",
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or("(no output)")
    );
    Ok(())
}
