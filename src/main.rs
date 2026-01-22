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

    // Run as a daemon - this won't exit when windows close
    iced::daemon(App::new, App::update, App::view)
        .title("SMT Toggle")
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}
