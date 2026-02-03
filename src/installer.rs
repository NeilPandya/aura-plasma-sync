use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const SERVICE_NAME: &str = "aura-plasma-sync.service";

fn get_service_path() -> Result<PathBuf> {
    let config_dir = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .ok_or_else(|| anyhow!("Could not determine user config directory"))?;

    Ok(config_dir.join("systemd/user").join(SERVICE_NAME))
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

pub fn install() -> Result<()> {
    let service_path = get_service_path()?;
    let current_exe = std::env::current_exe()?;

    if let Some(parent) = service_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = format!(
        "[Unit]\nDescription=Asus Aura Plasma Sync\nAfter=graphical-session.target\n\n\
        [Service]\nExecStart={}\nRestart=always\n\n\
        [Install]\nWantedBy=graphical-session.target",
        current_exe.display()
    );

    fs::write(&service_path, content)?;
    systemctl_user(&["daemon-reload"])?;
    systemctl_user(&["enable", "--now", SERVICE_NAME])?;

    log::info!("Service installed and started.");
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let _ = systemctl_user(&["stop", SERVICE_NAME]);
    let _ = systemctl_user(&["disable", SERVICE_NAME]);

    let path = get_service_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }

    systemctl_user(&["daemon-reload"])?;
    log::info!("Service removed.");
    Ok(())
}
