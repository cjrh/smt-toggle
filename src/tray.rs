use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

pub struct Tray {
    _tray_icon: TrayIcon,
}

impl Tray {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create menu
        let menu = Menu::new();
        let quit_item = MenuItem::new("Quit", true, None);

        menu.append(&quit_item)?;

        let quit_id = quit_item.id().clone();

        // Create icon from embedded data
        let icon = create_default_icon()?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("SMT Toggle")
            .with_icon(icon)
            .build()?;

        // Handle menu events
        std::thread::spawn(move || {
            loop {
                if let Ok(event) = MenuEvent::receiver().recv() {
                    if event.id == quit_id {
                        std::process::exit(0);
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
            let in_body = x >= 8 && x < 24 && y >= 8 && y < 24;

            // Pins on sides
            let on_pin = (y >= 10 && y < 22 && (x < 8 || x >= 24) && (y % 3 != 0))
                || (x >= 10 && x < 22 && (y < 8 || y >= 24) && (x % 3 != 0));

            if in_body || on_pin {
                // Light blue color for the icon
                rgba[idx] = 100;     // R
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
