//src/color.rs

use anyhow::{Result, anyhow};

/// Converts RGB values (each in the range [0.0, 1.0]) to a hex string
pub fn rgb_to_hex(r: f64, g: f64, b: f64) -> String {
    // Clamp values to prevent out-of-range errors
    let r = r.clamp(0.0, 1.0);
    let g = g.clamp(0.0, 1.0);
    let b = b.clamp(0.0, 1.0);

    format!(
        "{:02x}{:02x}{:02x}",
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8
    )
}

/// Parses a DBus value containing RGB color information and returns a hex string
pub fn parse_rgb_value(value: &zbus::zvariant::Value) -> Option<String> {
    // Directly match the expected structure without while-loop unwrapping
    if let zbus::zvariant::Value::Structure(s) = value {
        let f = s.fields();
        if f.len() == 3 {
            if let (
                zbus::zvariant::Value::F64(r),
                zbus::zvariant::Value::F64(g),
                zbus::zvariant::Value::F64(b),
            ) = (&f[0], &f[1], &f[2])
            {
                return Some(rgb_to_hex(*r, *g, *b));
            }
        }
    }
    None
}

/// Validates a hex color string and returns it in a consistent format
pub fn normalize_hex(hex: &str) -> Result<String> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return Err(anyhow!("Hex color must be 6 characters long"));
    }

    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow!("Hex color contains invalid characters"));
    }

    Ok(hex.to_lowercase())
}
