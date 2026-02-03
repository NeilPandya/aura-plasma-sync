use anyhow::{Result, bail};
use std::process::Command;

/// Interfaces with the hardware via asusctl
pub fn set_aura_color(hex: &str) -> Result<()> {
    // Correct command for modern asusctl
    let status = Command::new("asusctl")
        .args(["aura", "effect", "static", "-c", hex])
        .status()?;

    if !status.success() {
        bail!(
            "asusctl failed with exit code {:?}",
            status.code().unwrap_or(1)
        );
    }

    log::info!("Hardware Updated: #{}", hex);
    Ok(())
}
