// src/portal.rs
// Acts as the event source by monitoring XDG Desktop Portal DBus signals for accent color changes.

use crate::app::ControlMsg;
use anyhow::{Context, Result};
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

// Parses an XDG Portal color value and converts it to RGB bytes.
// Returns `None` if the value is invalid or out of range.
fn parse_rgb_value(value: &Value) -> Option<[u8; 3]> {
    let mut inner = value;
    while let Value::Value(v) = inner {
        inner = v;
    }

    if let Value::Structure(s) = inner {
        let f = s.fields();
        if f.len() >= 3 {
            if let (Value::F64(r), Value::F64(g), Value::F64(b)) = (&f[0], &f[1], &f[2]) {
                return crate::color::from_f64_rgb(*r, *g, *b);
            }
        }
    }
    None
}

// Global connection cache for DBus session
static DBUS_CONNECTION: OnceLock<zbus::blocking::Connection> = OnceLock::new();

// Connection management
fn get_dbus_connection() -> Result<&'static zbus::blocking::Connection> {
    if let Some(conn) = DBUS_CONNECTION.get() {
        return Ok(conn);
    }

    let conn = zbus::blocking::Connection::session()
        .context("Failed to establish DBus session connection")?;

    // Use set() which returns an Err if another thread won the race
    let _ = DBUS_CONNECTION.set(conn);

    // Return the value that is now guaranteed to be in the lock
    Ok(DBUS_CONNECTION.get().expect("OnceLock must be set"))
}

// Private helpers
fn create_settings_proxy(conn: &zbus::blocking::Connection) -> Result<SettingsProxyBlocking<'_>> {
    SettingsProxyBlocking::new(conn).with_context(|| "Failed to create Settings proxy")
}

// Public API: get current accent color
pub fn get_current_accent_color() -> Option<[u8; 3]> {
    let conn = get_dbus_connection().ok()?;

    let proxy = create_settings_proxy(conn).ok()?;

    let val = proxy
        .read("org.freedesktop.appearance", "accent-color")
        .ok()?;

    parse_rgb_value(&val)
}

// Public API: listen for accent color changes
pub fn listen(tx: Sender<ControlMsg>) -> Result<()> {
    let conn = get_dbus_connection()?;
    let proxy = create_settings_proxy(conn)?;

    let signals = proxy.receive_setting_changed()?;

    for signal in signals {
        let Ok(args) = signal.args() else { continue };

        if *args.namespace() == "org.freedesktop.appearance" && *args.key() == "accent-color" {
            if let Some(rgb) = parse_rgb_value(args.value()) {
                let _ = tx.send(ControlMsg::UpdateColor(rgb));
            } else {
                let _ = tx.send(ControlMsg::TriggerSync);
            }
        }
    }
    Ok(())
}
