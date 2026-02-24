// src/color.rs
// Provides a centralized library of color parsing and data transformations.

/// Converts normalized F64 RGB values (0.0-1.0) from XDG Portal to u8 bytes
pub fn from_f64_rgb(r: f64, g: f64, b: f64) -> [u8; 3] {
    [
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    ]
}

/// Formats RGB array to hex string (without #)
pub fn to_hex_string(rgb: [u8; 3]) -> String {
    format!("{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])
}

/// Formats RGB values for display in tray menu
pub fn format_rgb_string(rgb: [u8; 3]) -> String {
    format!("RGB: {}, {}, {}", rgb[0], rgb[1], rgb[2])
}

/// Formats hex string for display in tray menu
pub fn format_hex_string(rgb: [u8; 3]) -> String {
    format!("HEX: #{}", to_hex_string(rgb))
}
