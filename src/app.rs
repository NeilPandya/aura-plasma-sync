use anyhow::{Context, Result, anyhow, bail};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::executor;

pub enum ControlMsg {
    TriggerSync,
    UpdateColor(String),
}

#[zbus::proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Settings {
    #[zbus(signal)]
    fn setting_changed(
        &self,
        namespace: &str,
        key: &str,
        value: zbus::zvariant::Value<'_>,
    ) -> zbus::Result<()>;
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
        // Perform initial sync
        if let Err(e) = self.sync(None) {
            log::warn!("Initial sync failed: {}", e);
        }

        let dbus_tx = control_tx.clone();
        thread::spawn(move || {
            if let Err(e) = listen_for_dbus_changes(dbus_tx) {
                log::error!("DBus listener failed: {}", e);
            }
        });

        thread::spawn(move || {
            for msg in control_rx {
                let result = match msg {
                    ControlMsg::TriggerSync => self.sync(None),
                    ControlMsg::UpdateColor(hex) => self.sync(Some(hex)),
                };
                if let Err(e) = result {
                    log::error!("Sync operation failed: {}", e);
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
        // Lazy evaluation: only shell out for the fallback if the primary is missing
        let raw = self
            .read_config("General", "AccentColor")?
            .or_else(|| {
                self.read_config("Colors:Selection", "BackgroundNormal")
                    .ok()
                    .flatten()
            })
            .ok_or_else(|| anyhow!("Accent color not found in kdeglobals"))?;

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

fn listen_for_dbus_changes(tx: Sender<ControlMsg>) -> Result<()> {
    use zbus::zvariant::Value;

    let conn = zbus::blocking::Connection::session()?;
    let proxy = SettingsProxyBlocking::new(&conn)?;
    let signals = proxy.receive_setting_changed()?;

    for signal in signals {
        if let Ok(args) = signal.args() {
            let namespace = *args.namespace();
            let key = *args.key();

            if namespace == "org.freedesktop.appearance" {
                if key == "accent-color" {
                    // Peek into the Variant (Value::Value) to find the Tuple (Value::Structure)
                    if let Value::Value(inner) = args.value() {
                        if let Value::Structure(s) = &**inner {
                            let f = s.fields();
                            if f.len() == 3 {
                                // Match the three doubles (Value::F64)
                                if let (Value::F64(r), Value::F64(g), Value::F64(b)) =
                                    (&f[0], &f[1], &f[2])
                                {
                                    let r = (r * 255.0).round() as u8;
                                    let g = (g * 255.0).round() as u8;
                                    let b = (b * 255.0).round() as u8;
                                    let hex = format!("{:02x}{:02x}{:02x}", r, g, b);
                                    let _ = tx.send(ControlMsg::UpdateColor(hex));
                                    continue;
                                }
                            }
                        }
                    }
                }
                // Fallback: trigger a full re-read if the fast-path fails
                let _ = tx.send(ControlMsg::TriggerSync);
            }
        }
    }
    Ok(())
}
