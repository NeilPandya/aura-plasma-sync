// src/app.rs
// Orchestrates application state and coordinates message flow between the portal listener and hardware executor.

use crate::executor;
use crate::tray::TraySender;
use std::sync::Mutex;

pub enum ControlMsg {
    TriggerSync,
    UpdateColor([u8; 3]),
}

pub struct AuraSync {
    tray_updater: Option<TraySender>,
    last_color: Mutex<Option<[u8; 3]>>,
}

impl AuraSync {
    pub fn new(tray_updater: Option<TraySender>) -> Self {
        Self {
            tray_updater,
            last_color: Mutex::new(None),
        }
    }

    /// Core update logic: check for color changes and sync hardware.
    pub fn update(&self, rgb: [u8; 3]) {
        let mut last = self.last_color.lock().unwrap();
        if Some(rgb) == *last {
            return;
        }

        if let Err(e) = executor::sync_colors(rgb) {
            log::error!("Hardware update failed: {}", e);
        }

        if let Some(ref updater) = self.tray_updater {
            let _ = updater.send(rgb);
        }

        *last = Some(rgb);
    }
}
