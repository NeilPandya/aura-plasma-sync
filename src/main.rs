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

    // Set up high-level communication
    let (control_tx, control_rx) = mpsc::channel::<app::ControlMsg>();

    // Spawn tray and get the custom TraySender
    let (tray_sender, tray_handle) = tray::spawn_tray();

    // Create the application state
    let sync_app = std::sync::Arc::new(app::AuraSync::new(Some(tray_sender)));

    // Spawn the Portal Listener
    let portal_tx = control_tx.clone();
    std::thread::spawn(move || {
        if let Err(e) = portal::listen(portal_tx) {
            log::error!("Portal listener died: {}", e);
        }
    });

    // Spawn the Control Loop
    let app_instance = sync_app.clone();
    std::thread::spawn(move || {
        for msg in control_rx {
            match msg {
                app::ControlMsg::TriggerSync => {
                    if let Some(rgb) = portal::get_current_accent_color() {
                        app_instance.update(rgb);
                    }
                }
                app::ControlMsg::UpdateColor(rgb) => {
                    app_instance.update(rgb);
                }
            }
        }
    });

    // Trigger initial sync
    let _ = control_tx.send(app::ControlMsg::TriggerSync);

    log::info!("Aura Accent Sync is running. Waiting for XDG portal events...");

    // Application runs until terminated by systemd or tray closed
    let _ = tray_handle.join();
    log::info!("Aura Accent Sync stopped");
    Ok(())
}
