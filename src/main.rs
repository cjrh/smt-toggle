mod app;
mod smt;
mod tray;

use app::App;
use tray::Tray;

fn main() -> iced::Result {
    // Initialize GTK (required for tray icon menu)
    gtk::init().expect("Failed to initialize GTK");

    // Initialize the tray icon
    let _tray = Tray::new().expect("Failed to create tray icon");

    // Run the Iced application
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .window(App::window_settings())
        .run_with(App::new)
}
