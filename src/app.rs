use anyhow::{Context, Result, anyhow};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::{executor, portal};

pub enum ControlMsg {
    TriggerSync,
    UpdateColor(String),
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

    pub fn start_sync_thread(
        mut self,
        control_rx: Receiver<ControlMsg>,
        control_tx: Sender<ControlMsg>,
    ) -> Result<()> {
        let _ = self.sync(None);

        let dbus_tx = control_tx.clone();
        thread::spawn(move || {
            if let Err(e) = portal::listen(dbus_tx) {
                log::error!("Portal listener died: {}", e);
            }
        });

        thread::spawn(move || {
            for msg in control_rx {
                let res = match msg {
                    ControlMsg::TriggerSync => self.sync(None),
                    ControlMsg::UpdateColor(hex) => self.sync(Some(hex)),
                };
                if let Err(e) = res {
                    log::error!("Sync failed: {}", e);
                }
            }
        });

        Ok(())
    }

    fn sync(&mut self, direct_hex: Option<String>) -> Result<()> {
        if !self.active.load(Ordering::Relaxed) {
            return Ok(());
        }

        let hex = match direct_hex {
            Some(h) => h,
            None => self.fetch_kde_accent()?,
        };

        if hex != self.last_hex {
            executor::set_aura_color(&hex)?;
            self.last_hex = hex;
        }
        Ok(())
    }

    fn fetch_kde_accent(&self) -> Result<String> {
        let raw = self
            .read_config("General", "AccentColor")?
            .or_else(|| {
                self.read_config("Colors:Selection", "BackgroundNormal")
                    .ok()
                    .flatten()
            })
            .ok_or_else(|| anyhow!("No accent color found"))?;

        let rgb: Vec<u8> = raw
            .split(',')
            .take(3)
            .map(|s| s.parse::<u8>().context("RGB parse error"))
            .collect::<Result<Vec<_>>>()?;

        if rgb.len() < 3 {
            anyhow::bail!("Incomplete RGB data");
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
