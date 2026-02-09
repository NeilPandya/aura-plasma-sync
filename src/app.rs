// src/app.rs
use anyhow::Result;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::{executor, portal};

pub enum ControlMsg {
    TriggerSync,
    UpdateColor(String),
}

pub struct AuraSyncApp {
    tray_updater: Option<Sender<String>>,
    last_hex: Mutex<String>,
    host: String,
    port: u16,
}

impl AuraSyncApp {
    pub fn new(tray_updater: Option<Sender<String>>, host: String, port: u16) -> Self {
        Self {
            tray_updater,
            last_hex: Mutex::new(String::new()),
            host,
            port,
        }
    }

    pub fn start_sync_thread(
        self,
        control_rx: Receiver<ControlMsg>,
        control_tx: Sender<ControlMsg>,
    ) -> Result<()> {
        // DBus listener thread
        let dbus_tx = control_tx.clone();
        thread::spawn(move || {
            if let Err(e) = portal::listen(dbus_tx) {
                log::error!("Portal listener died: {}", e);
            }
        });

        // Main control loop
        thread::spawn(move || {
            for msg in control_rx {
                match msg {
                    ControlMsg::TriggerSync => {
                        if let Some(hex) = portal::get_current_accent_color() {
                            self.perform_update(hex);
                        }
                    }
                    ControlMsg::UpdateColor(hex) => {
                        self.perform_update(hex);
                    }
                }
            }
        });

        let _ = control_tx.send(ControlMsg::TriggerSync);
        Ok(())
    }

    fn perform_update(&self, hex: String) {
        {
            let mut last = self.last_hex.lock().unwrap();
            if *last == hex {
                log::debug!("Color unchanged, skipping update");
                return;
            }
            *last = hex.clone();
        }

        log::info!("Updating hardware to color: #{}", hex);
        if let Err(e) = executor::sync_colors(&hex, &self.host, self.port) {
            log::error!("Hardware update failed: {}", e);
        }

        if let Some(ref updater) = self.tray_updater {
            let _ = updater.send(hex);
        }
    }
}
