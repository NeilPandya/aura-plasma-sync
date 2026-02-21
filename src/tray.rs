// src/tray.rs
// Manages the system tray UI thread to provide visual status and color information to the user.

use gtk::glib;
use std::sync::mpsc;
use std::thread;
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuItem},
};

/// Simple wrapper to send updates to the GTK thread.
#[derive(Clone)]
pub struct TraySender {
    tx: mpsc::Sender<[u8; 3]>,
}

impl TraySender {
    pub fn send(&self, rgb: [u8; 3]) {
        if self.tx.send(rgb).is_ok() {
            // Use the THREAD-SAFE idle_add_once.
            // This is safe to call from the control loop thread.
            glib::idle_add_once(move || {
                // This empty closure just "pokes" the main loop to wake up
                // and run the idle handler attached in spawn_tray.
            });
        }
    }
}

fn update_tray_visuals(
    tray: &mut tray_icon::TrayIcon,
    hex_item: &MenuItem,
    rgb_item: &MenuItem,
    rgb: [u8; 3],
) {
    let img = crate::color::create_color_icon(rgb);
    let buf = img.into_vec();
    if let Ok(rgba_icon) = tray_icon::Icon::from_rgba(buf, 16, 16) {
        let _ = tray.set_icon(Some(rgba_icon));
    }
    hex_item.set_text(&crate::color::format_hex_string(rgb));
    rgb_item.set_text(&crate::color::format_rgb_string(rgb));
}

pub fn spawn_tray() -> (TraySender, thread::JoinHandle<()>) {
    let (tx, rx) = mpsc::channel::<[u8; 3]>();
    let tray_sender = TraySender { tx };

    let handle = thread::spawn(move || {
        if let Err(e) = gtk::init() {
            log::error!("Failed to initialize GTK: {}", e);
            return;
        }

        let hex_item = MenuItem::new("HEX: #------", false, None);
        let rgb_item = MenuItem::new("RGB: ---,---,---", false, None);
        let menu = Menu::new();
        let _ = menu.append(&hex_item).unwrap();
        let _ = menu.append(&rgb_item).unwrap();

        let mut tray = TrayIconBuilder::new()
            .with_tooltip("Aura XDG-Accent Sync")
            .with_menu(Box::new(menu))
            .build()
            .expect("Failed to build tray icon");

        // Inside the GTK thread, we use idle_add_local to monitor the channel.
        // This is safe because this thread owns the MainContext.
        glib::idle_add_local(move || {
            let mut latest = None;
            while let Ok(color) = rx.try_recv() {
                latest = Some(color);
            }

            if let Some(color) = latest {
                update_tray_visuals(&mut tray, &hex_item, &rgb_item, color);
            }

            glib::ControlFlow::Continue
        });

        gtk::main();
    });

    (tray_sender, handle)
}
