mod app;
mod env_check;
mod executor;
mod installer;
mod portal;
mod tray;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
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

    let active = Arc::new(AtomicBool::new(true));
    let (control_tx, control_rx) = mpsc::channel();

    let tray = tray::AuraTray::new(Arc::clone(&active), control_tx.clone());
    let service = ksni::TrayService::new(tray);

    let sync_app = app::AuraSyncApp::new(active);

    sync_app
        .start_sync_thread(control_rx, control_tx)
        .map_err(|e| anyhow::anyhow!("Failed to start sync thread: {e}"))?;

    service
        .run()
        .map_err(|e| anyhow::anyhow!("Tray service failed: {e}"))
}
