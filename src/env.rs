// src/env.rs
// Validates system environment requirements and verifies asusctl availability on startup.

use anyhow::{Context, Result, anyhow};
use std::process::Command;

// Public API
pub fn validate_dependencies() -> Result<()> {
    validate_asusctl_connection().map_err(|e| {
        anyhow!(
            "Could not connect to asusctl: {}. \
            Ensure asusctl is installed and running.",
            e
        )
    })
}

// Private implementation details: helpers first, then orchestrator

// Test the asusctl connection by querying device info
fn test_asusctl_connection() -> Result<()> {
    let output = Command::new("asusctl")
        .arg("info")
        .output()
        .context("Failed to execute asusctl info command")?;

    if !output.status.success() {
        return Err(anyhow!(
            "asusctl info returned non-zero exit code: {:?}",
            output.status.code().unwrap_or(1)
        ));
    }

    log::info!(
        "asusctl info: {}",
        String::from_utf8_lossy(&output.stdout).trim()
    );
    Ok(())
}

// Validate asusctl connection
fn validate_asusctl_connection() -> Result<()> {
    test_asusctl_connection()
}
