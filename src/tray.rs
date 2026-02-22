// src/tray.rs
// Manages the system tray UI thread efficiently using event-driven updates.

use gtk::glib;
use std::cell::RefCell;
use std::sync::mpsc;
use std::thread;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem},
};

// Thread-local storage to hold UI handles safely within the GTK thread.
thread_local! {
    static TRAY_STATE: RefCell<Option<(TrayIcon, MenuItem, MenuItem)>> = const { RefCell::new(None) };
    static TRAY_RX: RefCell<Option<mpsc::Receiver<[u8; 3]>>> = const { RefCell::new(None) };
}

#[derive(Clone)]
pub struct TraySender {
    tx: mpsc::Sender<[u8; 3]>,
}

impl TraySender {
    pub fn send(&self, rgb: [u8; 3]) {
        if self.tx.send(rgb).is_ok() {
            // Schedule a ONE-OFF execution on the GTK thread.
            // This wakes the thread up, runs the closure once, then lets it sleep.
            glib::idle_add_once(move || {
                TRAY_RX.with(|rx_cell| {
                    if let Some(rx) = rx_cell.borrow().as_ref() {
                        let mut latest = None;
                        while let Ok(color) = rx.try_recv() {
                            latest = Some(color);
                        }

                        if let Some(color) = latest {
                            update_ui(color);
                        }
                    }
                });
            });
        }
    }
}

fn update_ui(rgb: [u8; 3]) {
    TRAY_STATE.with(|state_cell| {
        if let Some((tray, hex_item, rgb_item)) = state_cell.borrow_mut().as_mut() {
            let img = crate::color::create_color_icon(rgb);
            let buf = img.into_vec();
            if let Ok(rgba_icon) = tray_icon::Icon::from_rgba(buf, 16, 16) {
                let _ = tray.set_icon(Some(rgba_icon));
            }
            hex_item.set_text(&crate::color::format_hex_string(rgb));
            rgb_item.set_text(&crate::color::format_rgb_string(rgb));
        }
    });
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
        let _ = menu.append(&hex_item);
        let _ = menu.append(&rgb_item);

        let tray = TrayIconBuilder::new()
            .with_tooltip("Aura XDG-Accent Sync")
            .with_menu(Box::new(menu))
            .build()
            .expect("Failed to build tray icon");

        // Initialize thread-local state
        TRAY_STATE.with(|s| *s.borrow_mut() = Some((tray, hex_item, rgb_item)));
        TRAY_RX.with(|r| *r.borrow_mut() = Some(rx));

        gtk::main();
    });

    (tray_sender, handle)
}
