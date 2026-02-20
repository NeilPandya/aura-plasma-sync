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

/// Represents an XDG Portal RGB color value structure
#[derive(Debug)]
struct XdgPortalColor {
    r: f64,
    g: f64,
    b: f64,
}

impl TryFrom<&Value<'_>> for XdgPortalColor {
    type Error = &'static str;

    fn try_from(value: &Value<'_>) -> Result<Self, Self::Error> {
        // Unwrap Value::Value wrappers to get to the actual structure
        let mut value = value;
        while let Value::Value(inner) = value {
            value = inner;
        }

        // Extract structure fields
        if let Value::Structure(s) = value {
            let fields = s.fields();
            if fields.len() == 3 {
                if let (Value::F64(r), Value::F64(g), Value::F64(b)) =
                    (&fields[0], &fields[1], &fields[2])
                {
                    return Ok(XdgPortalColor {
                        r: *r,
                        g: *g,
                        b: *b,
                    });
                }
            }
        }

        Err("Invalid XDG Portal color structure")
    }
}

impl XdgPortalColor {
    /// Converts the normalized color to RGB bytes
    pub fn to_rgb_bytes(&self) -> [u8; 3] {
        crate::color::from_f64_rgb(self.r, self.g, self.b)
    }

    /// Validates that the color values are within expected range (0.0-1.0)
    pub fn is_valid(&self) -> bool {
        (0.0..=1.0).contains(&self.r)
            && (0.0..=1.0).contains(&self.g)
            && (0.0..=1.0).contains(&self.b)
    }
}

/// Global connection cache for DBus session
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

fn parse_rgb_value(value: &Value) -> Option<[u8; 3]> {
    XdgPortalColor::try_from(value)
        .ok()
        .filter(XdgPortalColor::is_valid)
        .map(|color| color.to_rgb_bytes())
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
