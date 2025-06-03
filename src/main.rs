mod gui;
mod config;
mod csvparse;
mod playlist;
mod audio;

use eframe::NativeOptions;
use gui::Spotify2MediaApp;

fn main() {
    let options = NativeOptions::default();
    if let Err(e) = eframe::run_native(
        "Spotify2Media (Rust Port)",
        options,
        Box::new(|_cc| Box::new(Spotify2MediaApp::default())),
    ) {
        eprintln!("Application error: {e}");
    }
}