// src/main.rs

mod app;
mod env_check;
mod executor;
mod installer;
mod portal;
mod tray;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

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

    let active = Arc::new(AtomicBool::new(true));
    let active_clone = Arc::clone(&active);
    let (control_tx, control_rx) = mpsc::channel();

    // Create tray with a channel for color updates
    let (tray_update_tx, tray_update_rx) = mpsc::channel();
    let _tray_handle = tray::spawn_tray(Arc::clone(&active), control_tx.clone(), tray_update_rx);

    let sync_app = app::AuraSyncApp::new(active_clone, Some(tray_update_tx));

    sync_app
        .start_sync_thread(control_rx, control_tx)
        .map_err(|e| anyhow::anyhow!("Failed to start sync thread: {e}"))?;

    // Keep the main thread alive indefinitely
    log::info!("Aura Accent Sync is running. Waiting for XDG portal events...");
    loop {
        // Check if we should shut down
        if !active.load(Ordering::Relaxed) {
            log::info!("Shutdown requested, exiting...");
            break;
        }

        // Sleep for a reasonable amount of time
        std::thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}
