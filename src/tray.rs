// src/tray.rs

use gtk;
use image::{ImageBuffer, Rgba};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::thread;
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

pub struct AuraTrayHandle; // Zero-sized type to keep the tray thread alive

pub fn spawn_tray(
    active: Arc<AtomicBool>,
    control_tx: std::sync::mpsc::Sender<crate::app::ControlMsg>,
    color_rx: Receiver<String>,
) -> AuraTrayHandle {
    thread::spawn(move || {
        // Initialize GTK in the tray thread where it's needed
        if let Err(e) = gtk::init() {
            log::warn!("Failed to initialize GTK for tray: {}", e);
            // Continue anyway - maybe tray-icon has fallbacks
        }

        // Create menu items first
        let toggle_item = MenuItem::new("Toggle Sync", true, None);
        let separator = PredefinedMenuItem::separator();
        let exit_item = MenuItem::new("Exit", true, None);

        // Create menu
        let menu = Menu::new();
        if let Err(e) = menu.append(&toggle_item) {
            log::warn!("Failed to append toggle item to menu: {}", e);
        }
        if let Err(e) = menu.append(&separator) {
            log::warn!("Failed to append separator to menu: {}", e);
        }
        if let Err(e) = menu.append(&exit_item) {
            log::warn!("Failed to append exit item to menu: {}", e);
        }

        // Build tray - this might still fail but we'll handle it gracefully
        let tray_result = TrayIconBuilder::new()
            .with_tooltip("Aura XDG-Accent Sync")
            .with_menu(Box::new(menu))
            .build();

        match tray_result {
            Ok(mut tray) => {
                log::info!("System tray icon created successfully");
                let menu_channel = MenuEvent::receiver();
                loop {
                    // Pump GTK events so the tray icon actually appears/updates
                    while gtk::events_pending() {
                        gtk::main_iteration();
                    }

                    if menu_channel
                        .recv_timeout(std::time::Duration::from_millis(10))
                        .is_ok()
                    {
                        let new_state = !active.load(std::sync::atomic::Ordering::Relaxed);
                        active.store(new_state, std::sync::atomic::Ordering::Relaxed);
                        if new_state {
                            let _ = control_tx.send(crate::app::ControlMsg::TriggerSync);
                        }
                    }

                    if let Ok(color) = color_rx.try_recv() {
                        update_tray_icon(&mut tray, &color);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
            }

            Err(e) => {
                log::warn!(
                    "Failed to create system tray icon: {}. Continuing without tray.",
                    e
                );

                // Event loop without tray - just handle color updates
                loop {
                    if let Ok(color) = color_rx.try_recv() {
                        log::debug!("Color update received: {}", color);
                        // Process color updates even without tray
                        let _ = control_tx.send(crate::app::ControlMsg::UpdateColor(color));
                    }

                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    });

    AuraTrayHandle
}

fn update_tray_icon(tray: &mut tray_icon::TrayIcon, hex: &str) {
    if let Ok(color) = hex::decode(hex) {
        if color.len() == 3 {
            let img = create_color_icon([color[0], color[1], color[2]]);
            let buf = img.into_vec();
            if let Ok(rgba_icon) = tray_icon::Icon::from_rgba(buf, 16, 16) {
                let _ = tray.set_icon(Some(rgba_icon));
            }
        }
    }
}

fn create_color_icon(rgb: [u8; 3]) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(16, 16);
    let color = Rgba([rgb[0], rgb[1], rgb[2], 255]);
    for pixel in img.pixels_mut() {
        *pixel = color;
    }
    img
}
