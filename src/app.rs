use anyhow::Result;
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
    tray_updater: Option<Sender<String>>, // For sending color updates to tray
}

impl AuraSyncApp {
    pub fn new(active: Arc<AtomicBool>, tray_updater: Option<Sender<String>>) -> Self {
        Self {
            last_hex: String::new(),
            active,
            tray_updater,
        }
    }

    pub fn start_sync_thread(
        mut self,
        control_rx: Receiver<ControlMsg>,
        control_tx: Sender<ControlMsg>,
    ) -> Result<()> {
        // Initial sync only via XDG portal
        if let Err(e) = self.sync_via_portal() {
            log::warn!("Initial sync failed: {}", e);
        }

        // Start DBus listener thread
        let dbus_tx = control_tx.clone();
        thread::spawn(move || {
            if let Err(e) = portal::listen(dbus_tx) {
                log::error!("Portal listener died: {}", e);
            }
        });

        // Start control message handler thread
        thread::spawn(move || {
            for msg in control_rx {
                match msg {
                    ControlMsg::TriggerSync => {
                        if let Err(e) = self.sync_via_portal() {
                            log::error!("Sync failed: {}", e);
                        }
                    }
                    ControlMsg::UpdateColor(hex) => {
                        if let Err(e) = self.update_hardware(&hex) {
                            log::error!("Hardware update failed: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    fn sync_via_portal(&mut self) -> Result<()> {
        if !self.active.load(Ordering::Relaxed) {
            return Ok(());
        }

        if let Some(hex) = portal::get_current_accent_color() {
            self.update_hardware(&hex)?;
        }
        Ok(())
    }

    fn update_hardware(&mut self, hex: &str) -> Result<()> {
        if hex != self.last_hex {
            executor::set_aura_color_with_brightness_preservation(hex)?;
            self.last_hex = hex.to_string();

            if let Some(ref updater) = self.tray_updater {
                let _ = updater.send(hex.to_string());
            }
        }
        Ok(())
    }
}
