// src/tray.rs
// Manages the system tray UI thread to provide visual status and color information to the user.

use gtk;
use std::sync::mpsc::Receiver;
use std::thread;
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuItem},
};

// Private helpers (ordered by dependency - low-level first)

/// Updates the tray icon with a solid color preview
fn update_tray_icon(tray: &mut tray_icon::TrayIcon, rgb: [u8; 3]) {
    let img = crate::color::create_color_icon(rgb);
    let buf = img.into_vec();
    if let Ok(rgba_icon) = tray_icon::Icon::from_rgba(buf, 16, 16) {
        let _ = tray.set_icon(Some(rgba_icon));
    }
}

/// Updates the menu text with color information
fn update_color_display(hex_item: &MenuItem, rgb_item: &MenuItem, rgb: [u8; 3]) {
    hex_item.set_text(&crate::color::format_hex_string(rgb));
    rgb_item.set_text(&crate::color::format_rgb_string(rgb));
}

/// Orchestrates tray UI updates for a new color
fn update_tray_visuals(
    tray: &mut tray_icon::TrayIcon,
    hex_item: &MenuItem,
    rgb_item: &MenuItem,
    rgb: [u8; 3],
) {
    update_tray_icon(tray, rgb);
    update_color_display(hex_item, rgb_item, rgb);
}

/// Spawns the system tray UI thread with the provided color receiver
pub fn spawn_tray(color_rx: Receiver<[u8; 3]>) -> thread::JoinHandle<()> {
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
            log::warn!("Failed to append rgb item in menu: {}", e);
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

                    // Update visuals for the LATEST color received
                    let mut latest_color = None;
                    while let Ok(color) = color_rx.try_recv() {
                        latest_color = Some(color);
                    }

                    if let Some(color) = latest_color {
                        update_tray_visuals(&mut tray, &hex_item, &rgb_item, color);
                    }

                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }

            Err(e) => {
                log::error!("Failed to create system tray icon: {}", e);
                // Document expected behavior if GTK initialization fails
                log::warn!("Continuing to run without system tray icon");
            }
        }
    })
}
