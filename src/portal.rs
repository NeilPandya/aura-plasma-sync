use crate::app::ControlMsg;
use anyhow::Result;
use std::sync::mpsc::Sender;
use zbus::zvariant::Value;

#[zbus::proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Settings {
    #[zbus(signal)]
    fn setting_changed(&self, namespace: &str, key: &str, value: Value<'_>) -> zbus::Result<()>;
}

pub fn listen(tx: Sender<ControlMsg>) -> Result<()> {
    let conn = zbus::blocking::Connection::session()?;
    let proxy = SettingsProxyBlocking::new(&conn)?;
    let signals = proxy.receive_setting_changed()?;

    for signal in signals {
        let Ok(args) = signal.args() else { continue };
        let (namespace, key) = (*args.namespace(), *args.key());

        if namespace == "org.freedesktop.appearance" {
            if key == "accent-color" {
                if let Some(hex) = try_parse_rgb(args.value()) {
                    let _ = tx.send(ControlMsg::UpdateColor(hex));
                    continue;
                }
            }
            let _ = tx.send(ControlMsg::TriggerSync);
        }
    }
    Ok(())
}

fn try_parse_rgb(value: &Value) -> Option<String> {
    if let Value::Value(inner) = value {
        if let Value::Structure(s) = &**inner {
            let f = s.fields();
            if f.len() == 3 {
                if let (Value::F64(r), Value::F64(g), Value::F64(b)) = (&f[0], &f[1], &f[2]) {
                    return Some(format!(
                        "{:02x}{:02x}{:02x}",
                        (r * 255.0).round() as u8,
                        (g * 255.0).round() as u8,
                        (b * 255.0).round() as u8
                    ));
                }
            }
        }
    }
    None
}
