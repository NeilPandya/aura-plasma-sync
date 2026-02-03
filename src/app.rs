use anyhow::{Context, Result, anyhow, bail};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::{thread, time::Duration};

use crate::executor;

const KDEG_FILE: &str = "kdeglobals";
const SETTLE_MS: u64 = 250;

pub enum ControlMsg {
    TriggerSync,
}

pub struct AuraSyncApp {
    last_hex: String,
    active: Arc<AtomicBool>,
}

impl AuraSyncApp {
    pub fn new(active: Arc<AtomicBool>) -> Self {
        Self {
            last_hex: String::new(),
            active,
        }
    }

    pub fn start_sync_thread(mut self, control_rx: Receiver<ControlMsg>) -> Result<()> {
        let config_dir = get_config_dir()?;
        if let Err(e) = self.sync() {
            log::warn!("Initial sync failed: {}", e);
        }

        thread::spawn(move || {
            let (tx, rx) = std::sync::mpsc::channel();

            match RecommendedWatcher::new(tx, Config::default()) {
                Ok(mut watcher) => {
                    if let Err(e) = watcher.watch(&config_dir, RecursiveMode::NonRecursive) {
                        log::error!("Failed to watch config directory: {}", e);
                        return;
                    }

                    // Event processing loop
                    self.event_loop(control_rx, rx);
                }
                Err(e) => {
                    log::error!("Failed to create file watcher: {}", e);
                }
            }
        });

        Ok(())
    }

    fn event_loop(
        mut self,
        control_rx: Receiver<ControlMsg>,
        file_rx: std::sync::mpsc::Receiver<notify::Result<notify::Event>>,
    ) {
        loop {
            // Handle control messages
            while let Ok(ControlMsg::TriggerSync) = control_rx.try_recv() {
                if let Err(e) = self.sync() {
                    log::error!("Sync triggered by control message failed: {}", e);
                }
            }

            // Handle filesystem events with timeout
            match file_rx.recv_timeout(Duration::from_millis(500)) {
                Ok(Ok(event)) => {
                    let is_target = event
                        .paths
                        .iter()
                        .any(|p| p.file_name().map_or(false, |n| n == KDEG_FILE));

                    if is_target && (event.kind.is_modify() || event.kind.is_create()) {
                        thread::sleep(Duration::from_millis(SETTLE_MS));
                        while file_rx.try_recv().is_ok() {}
                        if let Err(e) = self.sync() {
                            log::error!("File event sync failed: {}", e);
                        }
                    }
                }
                // If the watcher channel is disconnected, exit the thread
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                _ => {}
            }
        }
    }

    fn sync(&mut self) -> Result<()> {
        if !self.active.load(Ordering::Relaxed) {
            return Ok(());
        }

        let hex = self.fetch_kde_accent()?;
        if hex != self.last_hex {
            executor::set_aura_color(&hex)?;
            self.last_hex = hex;
        }
        Ok(())
    }

    fn fetch_kde_accent(&self) -> Result<String> {
        let raw = self
            .read_config("General", "AccentColor")?
            .or(self.read_config("Colors:Selection", "BackgroundNormal")?)
            .ok_or_else(|| anyhow!("Accent color not found"))?;

        let rgb: Vec<u8> = raw
            .split(',')
            .take(3)
            .map(|s| s.parse::<u8>().context("Invalid RGB component"))
            .collect::<Result<_>>()?;

        if rgb.len() < 3 {
            bail!("Invalid color format: {raw}");
        }
        Ok(format!("{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]))
    }

    fn read_config(&self, group: &str, key: &str) -> Result<Option<String>> {
        let output = Command::new("kreadconfig6")
            .args(["--group", group, "--key", key])
            .output()?;

        let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(if val.is_empty() { None } else { Some(val) })
    }
}

fn get_config_dir() -> Result<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .context("Missing HOME/XDG_CONFIG_HOME")
}
