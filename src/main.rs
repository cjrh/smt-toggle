mod app;
mod smt;
mod tray;

use std::sync::mpsc;

use app::App;
use tray::Tray;

fn main() -> iced::Result {
    // Initialize GTK (required for tray icon menu)
    gtk::init().expect("Failed to initialize GTK");

    // Create channel for tray events
    let (tray_sender, tray_receiver) = mpsc::channel();

    // Set up the tray event receiver in the app module
    app::set_tray_receiver(tray_receiver);

    // Initialize the tray icon with the event sender
    let _tray = Tray::new(tray_sender).expect("Failed to create tray icon");

    // Run the Iced application
    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .window(App::window_settings())
        .exit_on_close_request(false)
        .run_with(App::new)
}
