// src/tray.rs
// Manages the system tray UI thread using event-driven updates.

use gtk::gdk_pixbuf::{self, Pixbuf};
use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::sync::Mutex;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem},
};

const TRAY_ICON_SIZE: i32 = 64;

// Thread-local storage to hold UI handles and the pre-scaled icon template safely within the GTK thread.
thread_local! {
    static TRAY_STATE: RefCell<Option<(TrayIcon, MenuItem, MenuItem)>> = const { RefCell::new(None) };
    static ICON_TEMPLATE: RefCell<Option<Pixbuf>> = const { RefCell::new(None) };
}

// Shared color state for cross-thread communication.
static TRAY_COLOR: Mutex<Option<[u8; 3]>> = Mutex::new(None);

#[derive(Clone)]
pub struct TraySender;

impl TraySender {
    pub fn send(&self, rgb: [u8; 3]) {
        if let Ok(mut color) = TRAY_COLOR.lock() {
            *color = Some(rgb);
        }

        glib::idle_add_once(|| {
            if let Ok(color) = TRAY_COLOR.lock() {
                if let Some(rgb) = *color {
                    update_ui(rgb);
                }
            }
        });
    }
}

fn update_ui(rgb: [u8; 3]) {
    TRAY_STATE.with(|state_cell| {
        if let Some((tray, hex_item, rgb_item)) = state_cell.borrow_mut().as_mut() {
            ICON_TEMPLATE.with(|template_cell| {
                if let Some(template) = template_cell.borrow().as_ref() {
                    let buf = create_tinted_buffer(template, rgb);
                    let res = TRAY_ICON_SIZE as u32;

                    if let Ok(icon) = tray_icon::Icon::from_rgba(buf, res, res) {
                        let _ = tray.set_icon(Some(icon));
                    }
                }
            });

            hex_item.set_text(&crate::color::format_hex_string(rgb));
            rgb_item.set_text(&crate::color::format_rgb_string(rgb));
        }
    });
}

fn get_scaled_icon_template() -> Pixbuf {
    let theme = gtk::IconTheme::default().expect("GTK not initialized");
    let icon_names = [
        "preferences-color",
        "preferences-theme",
        "colormanagement",
        "color-profile",
    ];

    let pixbuf = icon_names
        .iter()
        .find_map(|name| {
            theme.lookup_icon(name, TRAY_ICON_SIZE, gtk::IconLookupFlags::FORCE_SYMBOLIC)
        })
        .and_then(|info| info.load_icon().ok())
        .unwrap_or_else(|| {
            let pb = Pixbuf::new(
                gdk_pixbuf::Colorspace::Rgb,
                true,
                8,
                TRAY_ICON_SIZE,
                TRAY_ICON_SIZE,
            )
            .unwrap();
            pb.fill(0xffffffff);
            pb
        });

    // Scale once at startup if necessary
    if pixbuf.width() != TRAY_ICON_SIZE || pixbuf.height() != TRAY_ICON_SIZE {
        pixbuf
            .scale_simple(
                TRAY_ICON_SIZE,
                TRAY_ICON_SIZE,
                gdk_pixbuf::InterpType::Hyper,
            )
            .unwrap()
    } else {
        pixbuf
    }
}

fn create_tinted_buffer(template: &Pixbuf, rgb: [u8; 3]) -> Vec<u8> {
    let pixels = template.read_pixel_bytes();
    let pixels_ref = pixels.as_ref();
    let mut tinted_data = Vec::with_capacity(pixels_ref.len());

    for chunk in pixels_ref.chunks_exact(4) {
        // Apply RGB tint while preserving the template's alpha channel
        tinted_data.extend_from_slice(&[rgb[0], rgb[1], rgb[2], chunk[3]]);
    }

    tinted_data
}

pub fn spawn_tray() -> (TraySender, std::thread::JoinHandle<()>) {
    let handle = std::thread::spawn(move || {
        if gtk::init().is_err() {
            return;
        }

        // Cache the PRE-SCALED template
        ICON_TEMPLATE.with(|t| *t.borrow_mut() = Some(get_scaled_icon_template()));

        let hex_item = MenuItem::new("HEX: #------", false, None);
        let rgb_item = MenuItem::new("RGB: ---,---,---", false, None);
        let menu = Menu::new();
        let _ = menu.append(&MenuItem::new("Aura Accent Sync", false, None));
        let _ = menu.append(&tray_icon::menu::PredefinedMenuItem::separator());
        let _ = menu.append(&hex_item);
        let _ = menu.append(&rgb_item);

        let tray = TrayIconBuilder::new()
            .with_tooltip("Aura Accent Sync")
            .with_menu(Box::new(menu))
            .build()
            .expect("Failed to build tray icon");

        TRAY_STATE.with(|s| *s.borrow_mut() = Some((tray, hex_item, rgb_item)));
        gtk::main();
    });

    (TraySender, handle)
}
