mod gui;
mod config;
mod csvparse;
mod playlist;
mod audio;
mod spotify2media;

use eframe::NativeOptions;
use gui::Spotify2MediaApp;

fn main() {
    // Print version info at startup
    println!("Spotify2Media (Rust Port) v{}", env!("CARGO_PKG_VERSION"));

    // Set a panic hook for better diagnostics
    std::panic::set_hook(Box::new(|info| {
        eprintln!("Application panicked: {info}");
        #[cfg(target_os = "windows")]
        {
            use std::io::Write;
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("panic.log")
                .and_then(|mut f| writeln!(f, "Panic: {info}"));
        }
    }));

    let options = NativeOptions::default();

    // Try to run the GUI app, show error in stderr and a message box if it fails
    if let Err(e) = eframe::run_native(
        "Spotify2Media (Rust Port)",
        options,
        Box::new(|_cc| Box::new(Spotify2MediaApp::default())),
    ) {
        eprintln!("Application error: {e}");
        #[cfg(target_os = "windows")]
        {
            use rfd::MessageDialog;
            MessageDialog::new()
                .set_title("Spotify2Media Error")
                .set_description(&format!("Application error: {e}"))
                .show();
        }
    }
}
