// src/tray.rs
use crate::color;
use gtk;
use image::{ImageBuffer, Rgba};
use std::sync::mpsc::Receiver;
use std::thread;
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuItem},
};

pub fn spawn_tray(color_rx: Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        // Initialize GTK in the tray thread
        if let Err(e) = gtk::init() {
            log::error!("Failed to initialize GTK for tray: {}", e);
            return;
        }

        // Create menu items
        let hex_item = MenuItem::new("HEX: #------", false, None);
        let rgb_item = MenuItem::new("RGB: ---,---,---", false, None);

        // Create menu
        let menu = Menu::new();
        if let Err(e) = menu.append(&hex_item) {
            log::warn!("Failed to append hex item to menu: {}", e);
        }
        if let Err(e) = menu.append(&rgb_item) {
            log::warn!("Failed to append rgb item to menu: {}", e);
        }

        // Build tray
        let tray_result = TrayIconBuilder::new()
            .with_tooltip("Aura XDG-Accent Sync")
            .with_menu(Box::new(menu))
            .build();

        match tray_result {
            Ok(mut tray) => {
                log::info!("System tray icon created successfully");

                loop {
                    // Pump GTK events
                    while gtk::events_pending() {
                        gtk::main_iteration();
                    }

                    // Handle color updates only
                    if let Ok(color) = color_rx.try_recv() {
                        if let Ok(normalized_hex) = color::normalize_hex(&color) {
                            update_tray_visuals(&mut tray, &hex_item, &rgb_item, &normalized_hex);
                        } else {
                            log::warn!("Invalid hex color received: {}", color);
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
            }

            Err(e) => {
                log::error!("Failed to create system tray icon: {}", e);
            }
        }
    })
}

fn update_tray_visuals(
    tray: &mut tray_icon::TrayIcon,
    hex_item: &MenuItem,
    rgb_item: &MenuItem,
    hex: &str,
) {
    update_tray_icon(tray, hex);
    update_color_display(hex_item, rgb_item, hex);
}

fn update_tray_icon(tray: &mut tray_icon::TrayIcon, hex: &str) {
    if let Ok(color_bytes) = hex::decode(hex) {
        if color_bytes.len() == 3 {
            let img = create_color_icon([color_bytes[0], color_bytes[1], color_bytes[2]]);
            let buf = img.into_vec();
            if let Ok(rgba_icon) = tray_icon::Icon::from_rgba(buf, 16, 16) {
                let _ = tray.set_icon(Some(rgba_icon));
            }
        }
    }
}

fn update_color_display(hex_item: &MenuItem, rgb_item: &MenuItem, hex_str: &str) {
    // Update HEX display
    hex_item.set_text(&format!("HEX: #{}", hex_str));

    // Update RGB display
    if let Ok(bytes) = hex::decode(hex_str) {
        if bytes.len() == 3 {
            rgb_item.set_text(&format!("RGB: {}, {}, {}", bytes[0], bytes[1], bytes[2]));
            return;
        }
    }
    rgb_item.set_text("RGB: Invalid");
}

fn create_color_icon(rgb: [u8; 3]) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(16, 16);
    let color = Rgba([rgb[0], rgb[1], rgb[2], 255]);
    for pixel in img.pixels_mut() {
        *pixel = color;
    }
    img
}
