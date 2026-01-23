mod app;
mod smt;
mod tray;

use std::sync::mpsc;

use app::App;
use gtk::prelude::GtkSettingsExt;
use tray::Tray;

fn get_system_font() -> iced::Font {
    let Some(settings) = gtk::Settings::default() else {
        return iced::Font::DEFAULT;
    };
    let Some(font_name) = settings.gtk_font_name() else {
        return iced::Font::DEFAULT;
    };
    // font_name is like "Cantarell 11" - extract just the family name
    let family: String = font_name
        .split_whitespace()
        .take_while(|s: &&str| s.parse::<f32>().is_err())
        .collect::<Vec<_>>()
        .join(" ");

    if family.is_empty() {
        return iced::Font::DEFAULT;
    }

    // Font::with_name requires 'static str, so we leak the allocation
    let leaked: &'static str = Box::leak(family.into_boxed_str());
    iced::Font::with_name(leaked)
}

fn main() -> iced::Result {
    // Initialize GTK (required for tray icon menu)
    gtk::init().expect("Failed to initialize GTK");

    // Get the system font from GTK settings
    let system_font = get_system_font();

    // Create channel for tray events
    let (tray_sender, tray_receiver) = mpsc::channel();

    // Set up the tray event receiver in the app module
    app::set_tray_receiver(tray_receiver);

    // Initialize the tray icon with the event sender
    let _tray = Tray::new(tray_sender).expect("Failed to create tray icon");

    // Run as a daemon - this won't exit when windows close
    iced::daemon(App::new, App::update, App::view)
        .title("System Settings")
        .subscription(App::subscription)
        .theme(App::theme)
        .settings(iced::Settings {
            default_font: system_font,
            ..Default::default()
        })
        .run()
}
