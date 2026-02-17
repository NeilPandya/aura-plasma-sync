// src/main.rs
// Entry point for Aura Accent Sync, handling CLI parsing, environment initialization, and application lifecycle management.

mod app;
mod color;
mod env;
mod executor;
mod installer;
mod portal;
mod tray;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::mpsc;

/// Command-line interface definition for Aura Accent Sync
#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Available subcommands for Aura Accent Sync
#[derive(Subcommand)]
enum Commands {
    /// Install as systemd service
    Install,
    /// Uninstall systemd service
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

/// Runs the main application after environment validation
fn run_application() -> Result<()> {
    // Validate environment dependencies before starting
    env::validate_dependencies()?;

    // Set up communication channels
    let (control_tx, control_rx) = mpsc::channel();
    let (tray_update_tx, tray_update_rx) = mpsc::channel::<[u8; 3]>();

    // Spawn system tray UI
    let tray_handle = tray::spawn_tray(tray_update_rx);

    // Create and start the application
    let sync_app = app::AuraSync::new(Some(tray_update_tx));
    sync_app
        .start_sync_thread(control_rx, control_tx)
        .map_err(|e| anyhow::anyhow!("Failed to start sync thread: {e}"))?;

    log::info!("Aura Accent Sync is running. Waiting for XDG portal events...");

    // Application runs until terminated by systemd
    let _ = tray_handle.join();
    log::info!("Aura Accent Sync stopped by systemd");
    Ok(())
}
