use crate::config::AppConfig;
use crate::csvparse::{parse_csv, TrackInfo};
use crate::playlist::convert_playlist;
use eframe::{egui, App};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use rfd::FileDialog;

pub struct Spotify2MediaApp {
    csv_path: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    config: AppConfig,
    status: Arc<Mutex<String>>,
    is_running: bool,
    tracks: Vec<TrackInfo>,
    progress: Arc<Mutex<(usize, usize)>>,
    last_error: Option<String>,
    show_about: bool,
    theme_is_dark: bool,
    confirm_dialog_open: bool,
}

impl Default for Spotify2MediaApp {
    fn default() -> Self {
        let config_path = PathBuf::from("config.json");
        let config = AppConfig::load(&config_path);
        Self {
            csv_path: None,
            output_dir: None,
            config,
            status: Arc::new(Mutex::new("Waiting...".into())),
            is_running: false,
            tracks: vec![],
            progress: Arc::new(Mutex::new((0, 1))),
            last_error: None,
            show_about: false,
            theme_is_dark: true,
            confirm_dialog_open: false,
        }
    }
}

impl App for Spotify2MediaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Spotify2Media (Rust Port)");
                if ui.button("About").on_hover_text("More info about this app").clicked() {
                    self.show_about = true;
                }
                if ui.button("Toggle Theme").on_hover_text("Switch between dark and light mode").clicked() {
                    self.theme_is_dark = !self.theme_is_dark;
                    if self.theme_is_dark {
                        ctx.set_visuals(egui::Visuals::dark());
                    } else {
                        ctx.set_visuals(egui::Visuals::light());
                    }
                }
                if ui.button("Clear errors").on_hover_text("Clear error and status messages").clicked() {
                    self.last_error = None;
                    *self.status.lock().unwrap() = String::from("Ready.");
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Error banner
            if let Some(err) = &self.last_error {
                ui.colored_label(egui::Color32::RED, format!("Error: {err}"));
            }

            ui.horizontal(|ui| {
                if ui.add_enabled(!self.is_running, egui::Button::new("Select CSV")).clicked() {
                    if let Some(path) = FileDialog::new().add_filter("CSV", &["csv"]).pick_file() {
                        if path.extension().map(|e| e != "csv").unwrap_or(true) {
                            self.last_error = Some("Please select a CSV file.".into());
                        } else {
                            self.csv_path = Some(path.clone());
                            *self.status.lock().unwrap() = "CSV loaded.".into();
                            match parse_csv(&path) {
                                Ok(tracks) => self.tracks = tracks,
                                Err(e) => self.last_error = Some(format!("CSV error: {e}")),
                            }
                        }
                    }
                }
                if let Some(path) = &self.csv_path {
                    ui.label(format!("CSV: {}", path.display()));
                }
            });

            ui.horizontal(|ui| {
                if ui.add_enabled(!self.is_running, egui::Button::new("Select Output Folder")).clicked() {
                    if let Some(dir) = FileDialog::new().pick_folder() {
                        self.output_dir = Some(dir.clone());
                        *self.status.lock().unwrap() = "Output folder selected.".into();
                    }
                }
                if let Some(dir) = &self.output_dir {
                    ui.label(format!("Output: {}", dir.display()));
                }
            });

            if !self.tracks.is_empty() {
                ui.collapsing("CSV Preview (first 5 rows)", |ui| {
                    for t in self.tracks.iter().take(5) {
                        ui.label(format!("{} — {} [{}]", t.title, t.artist, t.album));
                    }
                });
            }

            ui.separator();

            ui.checkbox(&mut self.config.transcode_mp3, "Transcode to MP3")
                .on_hover_text("Convert downloaded files to MP3 for maximum compatibility.");
            ui.checkbox(&mut self.config.generate_m3u, "Generate M3U")
                .on_hover_text("Create an M3U playlist file with the downloaded tracks.");
            ui.checkbox(&mut self.config.exclude_instrumentals, "Exclude instrumentals")
                .on_hover_text("Try to skip instrumental versions in your playlist.");

            ui.separator();

            // Progress Bar
            let (curr, total) = *self.progress.lock().unwrap();
            if self.is_running {
                ui.add(egui::ProgressBar::new(curr as f32 / total.max(1) as f32)
                    .show_percentage()
                    .desired_width(400.0)
                    .text(format!("{} / {}", curr, total)));
            }

            if ui.add_enabled(!self.is_running, egui::Button::new("Convert Playlist"))
                .on_hover_text("Start downloading and tagging tracks.").clicked() 
            {
                // Confirm dialog before starting
                self.confirm_dialog_open = true;
            }

            if self.confirm_dialog_open {
                egui::Window::new("Confirm Conversion")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label("Are you sure you want to start playlist conversion?");
                        if ui.button("Yes, start").clicked() {
                            self.confirm_dialog_open = false;
                            // Start conversion logic
                            if let (Some(_csv), Some(out_dir)) = (&self.csv_path, &self.output_dir) {
                                let config = self.config.clone();
                                let out_dir = out_dir.clone();
                                let tracks = self.tracks.clone();
                                let status = Arc::clone(&self.status);
                                let progress = Arc::clone(&self.progress);

                                // You must set the correct path to yt-dlp and ffmpeg on your system!
                                let yt_dlp_path = PathBuf::from("yt-dlp");
                                let ffmpeg_path = PathBuf::from("ffmpeg");

                                self.is_running = true;
                                self.last_error = None;
                                *status.lock().unwrap() = "Starting conversion...".into();
                                *progress.lock().unwrap() = (0, tracks.len().max(1));

                                thread::spawn(move || {
                                    let cb = |i: usize, total: usize, track: &str| {
                                        *progress.lock().unwrap() = (i + 1, total.max(1));
                                        *status.lock().unwrap() = format!("Downloading: {} ({}/{})", track, i + 1, total);
                                    };
                                    let result = convert_playlist(&tracks, &config, &yt_dlp_path, &ffmpeg_path, &out_dir, cb);
                                    match result {
                                        Ok(_) => *status.lock().unwrap() = format!(
                                            "Conversion finished! Downloaded {}/{} tracks.", tracks.len(), tracks.len()),
                                        Err(e) => *status.lock().unwrap() = format!("Error: {e}"),
                                    }
                                });
                            } else {
                                self.last_error = Some("Please select a CSV and output folder.".into());
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_dialog_open = false;
                        }
                    });
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Save Settings").on_hover_text("Save your configuration for next time.").clicked() {
                    if let Err(e) = self.config.save(Path::new("config.json")) {
                        self.last_error = Some(format!("Failed to save config: {e}"));
                    } else {
                        *self.status.lock().unwrap() = "Settings saved.".into();
                    }
                }
            });

            ui.separator();

            ui.label(format!("Status: {}", self.status.lock().unwrap()));

            if self.is_running {
                let s = self.status.lock().unwrap();
                if s.starts_with("Conversion finished!") || s.starts_with("Error:") {
                    self.is_running = false;
                }
            }

            ctx.request_repaint_after(std::time::Duration::from_millis(150));
        });

        // About dialog fix: use local variable for open state!
        if self.show_about {
            let mut show_about_open = true;
            egui::Window::new("About Spotify2Media Rust Port")
                .open(&mut show_about_open)
                .show(ctx, |ui| {
                    ui.label("Spotify2Media Rust Port");
                    ui.label("By fentbuscoding.");
                    ui.label("Powered by eframe/egui.");
                    ui.label("Downloads and tags music from Spotify playlists using yt-dlp & ffmpeg.");
                    ui.label("Project: github.com/fentbuscoding/spotify2media-rust");
                    if ui.button("Close").clicked() {
                        // just let egui close the window, handled below
                    }
                });
            self.show_about = show_about_open;
        }
    }
}