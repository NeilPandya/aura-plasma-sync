// src/installer.rs
// Automates the deployment and removal of the systemd user service for XDG-compliant session integration.

use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const SERVICE_NAME: &str = "aura-accent-sync.service";

// Service path construction
fn get_service_path() -> Result<PathBuf> {
    let config_dir = get_user_config_dir()?;
    Ok(config_dir.join("systemd/user").join(SERVICE_NAME))
}

fn get_user_config_dir() -> Result<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .ok_or_else(|| anyhow!("Could not determine user config directory"))
}

fn systemctl_user(args: &[&str]) -> Result<()> {
    let status = Command::new("systemctl")
        .arg("--user")
        .args(args)
        .status()
        .context("Failed to run systemctl")?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("systemctl {:?} failed", args))
    }
}

// Generate systemd service file content
fn generate_service_content(exec_start: &str) -> String {
    format!(
        "[Unit]\nDescription=Aura XDG Accent Color Sync\n\
        After=graphical-session.target\n\
        PartOf=graphical-session.target\n\n\
        [Service]\nExecStart={}\nRestart=always\nRestartSec=5\n\
        Environment=XDG_DATA_DIRS=%h/.local/share:/usr/local/share:/usr/share\n\n\
        [Install]\nWantedBy=graphical-session.target",
        exec_start
    )
}

// Helper: ensure parent directory exists
fn ensure_parent_dir_exists(path: &PathBuf) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }
    Ok(())
}

// Public API: install the service
pub fn install() -> Result<()> {
    let service_path = get_service_path()?;
    let current_exe = std::env::current_exe()?;

    let content = generate_service_content(&current_exe.display().to_string());

    ensure_parent_dir_exists(&service_path)?;
    fs::write(&service_path, content)?;

    systemctl_user(&["daemon-reload"])?;
    systemctl_user(&["enable", "--now", SERVICE_NAME])?;

    log::info!("Service installed.");
    Ok(())
}

// Public API: uninstall the service
pub fn uninstall() -> Result<()> {
    systemctl_user(&["stop", SERVICE_NAME])?;
    systemctl_user(&["disable", SERVICE_NAME])?;

    let path = get_service_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }

    systemctl_user(&["daemon-reload"])?;
    log::info!("Service removed.");
    Ok(())
}
