mod config;
mod csvparse;
mod gui;
mod audio;
mod subprocess;
mod playlist;

use gui::Spotify2MediaApp;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Spotify2Media (Rust Port)",
        options,
        Box::new(|_cc| Box::new(Spotify2MediaApp::default())),
    );
}