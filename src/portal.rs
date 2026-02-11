// src/portal.rs

use crate::app::ControlMsg;
use crate::color;
use anyhow::Result;
use std::sync::{OnceLock, mpsc::Sender};
use zbus::zvariant::Value;

#[zbus::proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Settings {
    #[zbus(signal)]
    fn setting_changed(&self, namespace: &str, key: &str, value: Value<'_>) -> zbus::Result<()>;

    fn read(&self, namespace: &str, key: &str) -> zbus::Result<zbus::zvariant::OwnedValue>;
}

/// Global connection cache
static DBUS_CONNECTION: OnceLock<zbus::blocking::Connection> = OnceLock::new();

fn get_dbus_connection() -> Result<&'static zbus::blocking::Connection> {
    Ok(DBUS_CONNECTION.get_or_init(|| {
        zbus::blocking::Connection::session().unwrap_or_else(|e| {
            log::error!("Failed to create DBus connection: {}", e);
            panic!("Critical: Could not establish DBus connection")
        })
    }))
}

pub fn get_current_accent_color() -> Option<String> {
    let conn = get_dbus_connection().ok()?;
    let proxy = SettingsProxyBlocking::new(conn).ok()?;
    let val = proxy
        .read("org.freedesktop.appearance", "accent-color")
        .ok()?;
    color::parse_rgb_value(&val)
}

pub fn listen(tx: Sender<ControlMsg>) -> Result<()> {
    let conn = get_dbus_connection()?;
    let proxy = SettingsProxyBlocking::new(conn)?;
    let signals = proxy.receive_setting_changed()?;

    for signal in signals {
        let Ok(args) = signal.args() else { continue };
        if *args.namespace() == "org.freedesktop.appearance" && *args.key() == "accent-color" {
            if let Some(hex) = color::parse_rgb_value(args.value()) {
                let _ = tx.send(ControlMsg::UpdateColor(hex));
            } else {
                let _ = tx.send(ControlMsg::TriggerSync);
            }
        }
    }
    Ok(())
}
