// src/main.rs

mod app;
mod color;
mod env_check;
mod executor;
mod installer;
mod portal;
mod tray;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::mpsc;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Install,
    Uninstall,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Install) => installer::install(),
        Some(Commands::Uninstall) => installer::uninstall(),
        None => run_application(),
    }
}

fn run_application() -> Result<()> {
    env_check::validate_dependencies()?;

    let (control_tx, control_rx) = mpsc::channel();
    let (tray_update_tx, tray_update_rx) = mpsc::channel();

    let tray_handle = tray::spawn_tray(tray_update_rx);

    let sync_app = app::AuraSyncApp::new(Some(tray_update_tx));

    sync_app
        .start_sync_thread(control_rx, control_tx)
        .map_err(|e| anyhow::anyhow!("Failed to start sync thread: {e}"))?;

    log::info!("Aura Accent Sync is running. Waiting for XDG portal events...");

    let _ = tray_handle.join();
    Ok(())
}
