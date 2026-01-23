use std::sync::mpsc;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};

/// Events from the tray icon that the main app should handle
#[derive(Debug, Clone)]
pub enum TrayEvent {
    ShowWindow,
    Quit,
}

pub struct Tray {
    _tray_icon: TrayIcon,
}

impl Tray {
    pub fn new(event_sender: mpsc::Sender<TrayEvent>) -> Result<Self, Box<dyn std::error::Error>> {
        // Create menu with Show and Quit options
        let menu = Menu::new();
        let show_item = MenuItem::new("Show", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        menu.append(&show_item)?;
        menu.append(&quit_item)?;

        let show_id = show_item.id().clone();
        let quit_id = quit_item.id().clone();

        // Create icon from embedded data
        let icon = create_default_icon()?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("System Settings")
            .with_icon(icon)
            .build()?;

        // Handle menu events in a separate thread
        std::thread::spawn(move || {
            loop {
                if let Ok(event) = MenuEvent::receiver().recv() {
                    if event.id == quit_id {
                        let _ = event_sender.send(TrayEvent::Quit);
                    } else if event.id == show_id {
                        let _ = event_sender.send(TrayEvent::ShowWindow);
                    }
                }
            }
        });

        Ok(Self {
            _tray_icon: tray_icon,
        })
    }
}

fn create_default_icon() -> Result<Icon, Box<dyn std::error::Error>> {
    // Create a simple 32x32 CPU-like icon
    let size = 32;
    let mut rgba = vec![0u8; size * size * 4];

    // Draw a simple CPU icon (square with pins)
    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;

            // Main CPU body (centered square)
            let in_body = (8..24).contains(&x) && (8..24).contains(&y);

            // Pins on sides
            let on_pin = ((10..22).contains(&y) && !(8..24).contains(&x) && (y % 3 != 0))
                || ((10..22).contains(&x) && !(8..24).contains(&y) && (x % 3 != 0));

            if in_body || on_pin {
                // Light blue color for the icon
                rgba[idx] = 100; // R
                rgba[idx + 1] = 149; // G
                rgba[idx + 2] = 237; // B
                rgba[idx + 3] = 255; // A
            } else {
                // Transparent
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 0;
            }
        }
    }

    Ok(Icon::from_rgba(rgba, size as u32, size as u32)?)
}
