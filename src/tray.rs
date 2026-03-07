// src/tray.rs
// Manages the system tray UI thread using event-driven updates.

use gtk::gdk_pixbuf::{self, Pixbuf};
use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::sync::Mutex;
use std::thread;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem},
};

const TRAY_ICON_SIZE: i32 = 64;

// Thread-local storage to hold UI handles and the icon template safely within the GTK thread.
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
        // Update the shared color state
        if let Ok(mut color) = TRAY_COLOR.lock() {
            *color = Some(rgb);
        }

        // Schedule a one-off execution on the GTK thread to render the UI.
        glib::idle_add_once(|| {
            if let Ok(color) = TRAY_COLOR.lock() {
                if let Some(rgb) = *color {
                    update_ui(rgb);
                }
            }
        });
    }
}

// Orchestrate the UI updates within the GTK thread.
fn update_ui(rgb: [u8; 3]) {
    TRAY_STATE.with(|state_cell| {
        if let Some((tray, hex_item, rgb_item)) = state_cell.borrow_mut().as_mut() {
            ICON_TEMPLATE.with(|template_cell| {
                if let Some(template) = template_cell.borrow().as_ref() {
                    let buf = create_themed_icon_buffer(template, rgb);
                    let icon_res = TRAY_ICON_SIZE as u32;

                    /*
                     * Providing a 64x64 buffer allows the system to render cleanly,
                     * preventing fuzziness with smaller sizes.
                     */
                    if let Ok(rgba_icon) = tray_icon::Icon::from_rgba(buf, icon_res, icon_res) {
                        let _ = tray.set_icon(Some(rgba_icon));
                    }
                }
            });

            hex_item.set_text(&crate::color::format_hex_string(rgb));
            rgb_item.set_text(&crate::color::format_rgb_string(rgb));
        }
    });
}

fn get_cached_icon_template() -> Pixbuf {
    let theme = gtk::IconTheme::default().expect("GTK not initialized");
    let icon_names = [
        "preferences-color",
        "preferences-theme",
        "colormanagement",
        "color-profile",
        "preferences-desktop-color",
    ];

    let mut icon_info = None;
    for name in icon_names {
        if let Some(info) =
            theme.lookup_icon(name, TRAY_ICON_SIZE, gtk::IconLookupFlags::FORCE_SYMBOLIC)
        {
            icon_info = Some(info);
            break;
        }
    }

    icon_info
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
        })
}

fn create_themed_icon_buffer(template: &Pixbuf, rgb: [u8; 3]) -> Vec<u8> {
    let scaled = if template.width() != TRAY_ICON_SIZE || template.height() != TRAY_ICON_SIZE {
        template
            .scale_simple(
                TRAY_ICON_SIZE,
                TRAY_ICON_SIZE,
                gdk_pixbuf::InterpType::Hyper,
            )
            .unwrap()
    } else {
        template.clone()
    };

    let pixels = scaled.read_pixel_bytes();
    let pixels_ref = pixels.as_ref();
    let mut tinted_data = Vec::with_capacity(pixels_ref.len());

    for chunk in pixels_ref.chunks_exact(4) {
        tinted_data.extend_from_slice(&[rgb[0], rgb[1], rgb[2], chunk[3]]);
    }

    tinted_data
}

pub fn spawn_tray() -> (TraySender, thread::JoinHandle<()>) {
    let tray_sender = TraySender;

    let handle = thread::spawn(move || {
        if let Err(e) = gtk::init() {
            log::error!("Failed to initialize GTK: {}", e);
            return;
        }

        // Cache the icon template once at startup
        let template = get_cached_icon_template();
        ICON_TEMPLATE.with(|t| *t.borrow_mut() = Some(template));

        // Create the title item (disabled so it isn't clickable)
        let title_item = MenuItem::new("Aura Accent Sync", false, None);
        // Create a separator for visual clarity
        let separator = tray_icon::menu::PredefinedMenuItem::separator();

        let hex_item = MenuItem::new("HEX: #------", false, None);
        let rgb_item = MenuItem::new("RGB: ---,---,---", false, None);

        let menu = Menu::new();
        let _ = menu.append(&title_item);
        let _ = menu.append(&separator);
        let _ = menu.append(&hex_item);
        let _ = menu.append(&rgb_item);

        let tray = TrayIconBuilder::new()
            .with_tooltip("Aura Accent Sync")
            .with_menu(Box::new(menu))
            .build()
            .expect("Failed to build tray icon");

        // Initialize thread-local state
        TRAY_STATE.with(|s| *s.borrow_mut() = Some((tray, hex_item, rgb_item)));

        gtk::main();
    });

    (tray_sender, handle)
}
