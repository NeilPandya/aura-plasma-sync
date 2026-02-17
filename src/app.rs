// src/app.rs
// Orchestrates application state and coordinates message flow between the portal listener and hardware executor.

use anyhow::Result;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::{executor, portal};

pub enum ControlMsg {
    TriggerSync,
    UpdateColor([u8; 3]),
}

pub struct AuraSync {
    tray_updater: Option<Sender<[u8; 3]>>,
    last_color: Mutex<Option<[u8; 3]>>,
}

impl AuraSync {
    /// Constructs a new AuraSync with the given configuration.
    pub fn new(tray_updater: Option<Sender<[u8; 3]>>) -> Self {
        Self {
            tray_updater,
            last_color: Mutex::new(None),
        }
    }

    /// Core update logic: check for color changes, sync hardware, and notify UI.
    fn perform_update(
        rgb: [u8; 3],
        last_color: &Mutex<Option<[u8; 3]>>,
        tray_updater: &Option<Sender<[u8; 3]>>,
    ) {
        let mut last = last_color.lock().unwrap();
        if Some(rgb) == *last {
            log::debug!("Color unchanged, skipping update");
            return;
        }

        log::info!(
            "Updating hardware to color: #{}",
            crate::color::to_hex_string(rgb)
        );

        if let Err(e) = executor::sync_colors(rgb, "127.0.0.1", 6742) {
            log::error!("Hardware update failed: {}", e);
        }

        if let Some(updater) = tray_updater {
            let _ = updater.send(rgb);
        }

        *last = Some(rgb);
    }

    /// Spawns the DBus listener thread to monitor accent color changes.
    fn spawn_portal_listener(control_tx: Sender<ControlMsg>) {
        let dbus_tx = control_tx.clone();
        thread::spawn(move || {
            if let Err(e) = portal::listen(dbus_tx) {
                log::error!("Portal listener died: {}", e);
            }
        });
    }

    /// Spawns the main control loop that processes color update messages.
    fn spawn_control_loop(self, control_rx: Receiver<ControlMsg>) {
        let app_last_color = self.last_color;
        let app_tray_updater = self.tray_updater;

        thread::spawn(move || {
            for msg in control_rx {
                match msg {
                    ControlMsg::TriggerSync => {
                        if let Some(rgb) = portal::get_current_accent_color() {
                            Self::perform_update(rgb, &app_last_color, &app_tray_updater);
                        }
                    }
                    ControlMsg::UpdateColor(rgb) => {
                        Self::perform_update(rgb, &app_last_color, &app_tray_updater);
                    }
                }
            }
        });
    }

    /// Starts the synchronization threads (portal listener and control loop).
    pub fn start_sync_thread(
        self,
        control_rx: Receiver<ControlMsg>,
        control_tx: Sender<ControlMsg>,
    ) -> Result<()> {
        Self::spawn_portal_listener(control_tx.clone());
        self.spawn_control_loop(control_rx);
        let _ = control_tx.send(ControlMsg::TriggerSync);
        Ok(())
    }
}
