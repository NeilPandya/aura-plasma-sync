// src/tray.rs
// Manages the system tray UI thread efficiently using event-driven updates.

use gtk::gdk_pixbuf::{self, Pixbuf};
use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::sync::mpsc;
use std::thread;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem},
};

const TRAY_ICON_SIZE: i32 = 64;

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

/// Orchestrates the UI updates within the GTK thread.
fn update_ui(rgb: [u8; 3]) {
    TRAY_STATE.with(|state_cell| {
        if let Some((tray, hex_item, rgb_item)) = state_cell.borrow_mut().as_mut() {
            let buf = create_themed_icon_buffer(rgb);

            // We cast to u32 here as tray-icon's API expects u32 dimensions
            let icon_res = TRAY_ICON_SIZE as u32;

            if let Ok(rgba_icon) = tray_icon::Icon::from_rgba(buf, icon_res, icon_res) {
                let _ = tray.set_icon(Some(rgba_icon));
            }

            hex_item.set_text(&crate::color::format_hex_string(rgb));
            rgb_item.set_text(&crate::color::format_rgb_string(rgb));
        }
    });
}

fn create_themed_icon_buffer(rgb: [u8; 3]) -> Vec<u8> {
    let theme = gtk::IconTheme::default().expect("GTK not initialized");

    // We try to find the best symbolic icon available in the system theme
    let icon_names = [
        "preferences-color-symbolic",
        "preferences-desktop-color-symbolic",
        "preferences-color",
    ];

    let mut icon_info = None;
    for name in icon_names {
        // We request the icon at our high-res constant.
        // GTK will handle the SVG -> Raster conversion at this resolution.
        if let Some(info) =
            theme.lookup_icon(name, TRAY_ICON_SIZE, gtk::IconLookupFlags::FORCE_SYMBOLIC)
        {
            icon_info = Some(info);
            break;
        }
    }

    let pixbuf = icon_info
        .and_then(|info| info.load_icon().ok())
        .unwrap_or_else(|| {
            // Fallback: Transparent square if no icon found
            Pixbuf::new(
                gdk_pixbuf::Colorspace::Rgb,
                true,
                8,
                TRAY_ICON_SIZE,
                TRAY_ICON_SIZE,
            )
            .unwrap()
        });

    // Final safety check: if the theme returned a different size than requested, scale it.
    let scaled = if pixbuf.width() != TRAY_ICON_SIZE || pixbuf.height() != TRAY_ICON_SIZE {
        pixbuf
            .scale_simple(
                TRAY_ICON_SIZE,
                TRAY_ICON_SIZE,
                gdk_pixbuf::InterpType::Bilinear,
            )
            .unwrap()
    } else {
        pixbuf
    };

    let pixels = scaled.read_pixel_bytes();
    let pixels_ref = pixels.as_ref();

    // Optimization: Pre-allocate the exact size needed (Width * Height * RGBA)
    let mut tinted_data = Vec::with_capacity(pixels_ref.len());

    for chunk in pixels_ref.chunks_exact(4) {
        // We replace the RGB channels with our accent color,
        // but we MUST keep the Alpha channel from the symbolic icon.
        tinted_data.extend_from_slice(&[
            rgb[0], rgb[1], rgb[2], chunk[3], // The "shape" of the icon
        ]);
    }

    tinted_data
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
            .with_tooltip("Aura Accent Sync")
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
