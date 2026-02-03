use ksni::menu::StandardItem;
use ksni::{MenuItem, Tray};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;

use crate::app::ControlMsg;

const ICON_ON: &str = "preferences-desktop-color";
const ICON_OFF: &str = "preferences-desktop-color-disabled";

pub struct AuraTray {
    pub active: Arc<AtomicBool>,
    pub control_tx: Sender<ControlMsg>,
}

impl AuraTray {
    pub fn new(active: Arc<AtomicBool>, control_tx: Sender<ControlMsg>) -> Self {
        Self { active, control_tx }
    }
}

impl Tray for AuraTray {
    fn icon_name(&self) -> String {
        if self.active.load(Ordering::Relaxed) {
            ICON_ON.into()
        } else {
            ICON_OFF.into()
        }
    }

    fn title(&self) -> String {
        "Aura Plasma Sync".into()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let is_active = self.active.load(Ordering::Relaxed);
        let label = if is_active {
            "Sync Enabled"
        } else {
            "Sync Disabled"
        };
        let icon = if is_active { "emblem-ok" } else { "" };

        vec![MenuItem::Standard(StandardItem {
            label: label.into(),
            icon_name: icon.into(),
            activate: Box::new(move |this| {
                let new_state = !this.active.load(Ordering::Relaxed);
                this.active.store(new_state, Ordering::Relaxed);

                if new_state {
                    let _ = this.control_tx.send(ControlMsg::TriggerSync);
                }
            }),
            ..Default::default()
        })]
    }
}
