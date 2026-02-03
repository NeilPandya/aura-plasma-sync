use anyhow::{Context, Result, anyhow};
use which::which;

pub fn validate_dependencies() -> Result<()> {
    let deps = [
        ("asusctl", "asusctl not found - please install asusctl"),
        (
            "kreadconfig6",
            "kreadconfig6 not found - requires KDE Plasma 6",
        ),
    ];

    for (cmd, msg) in &deps {
        which(cmd)
            .map_err(|_| anyhow!("{}", msg))
            .context(format!("Dependency check failed for '{}'", cmd))?;
    }

    // Additional validation for asusctl version (make it non-fatal)
    if let Err(e) = validate_asusctl_version() {
        log::warn!("Could not verify asusctl version: {}", e);
    }

    Ok(())
}

fn validate_asusctl_version() -> Result<()> {
    use std::process::Command;

    let output = Command::new("asusctl")
        .arg("info")
        .output()
        .context("Failed to execute asusctl info")?;

    if !output.status.success() {
        return Err(anyhow!("asusctl info returned non-zero exit code"));
    }

    let version_output = String::from_utf8_lossy(&output.stdout);
    log::info!("asusctl info: {}", version_output.trim());

    Ok(())
}
