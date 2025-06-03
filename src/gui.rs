use crate::config::AppConfig;
use crate::csvparse::{parse_csv, TrackInfo};
use crate::playlist::convert_playlist;
use eframe::{egui, App};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use rfd::FileDialog;
use egui::{RichText, Color32};
use std::time::{Instant, Duration};

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
    yt_dlp_path: PathBuf,
    ffmpeg_path: PathBuf,
    log: Arc<Mutex<Vec<String>>>,
    start_time: Option<Instant>,
    cancel_requested: Arc<Mutex<bool>>,
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
            yt_dlp_path: PathBuf::from("yt-dlp"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            log: Arc::new(Mutex::new(Vec::new())),
            start_time: None,
            cancel_requested: Arc::new(Mutex::new(false)),
        }
    }
}

impl App for Spotify2MediaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Spotify2Media (Rust Port)");
                if ui.button("About").clicked() { self.show_about = true; }
                if ui.button("Toggle Theme").clicked() {
                    self.theme_is_dark = !self.theme_is_dark;
                    ctx.set_visuals(if self.theme_is_dark { egui::Visuals::dark() } else { egui::Visuals::light() });
                }
                if ui.button("Reset").on_hover_text("Reset all fields").clicked() {
                    *self = Self::default();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Error/status area
            if let Some(err) = &self.last_error {
                ui.horizontal(|ui| {
                    ui.colored_label(Color32::RED, format!("❌ Error: {err}"));
                    if ui.button("Copy").on_hover_text("Copy error to clipboard").clicked() {
                        ui.output_mut(|o| o.copied_text = err.clone());
                    }
                });
            } else {
                ui.colored_label(Color32::LIGHT_GREEN, format!("Status: {}", self.status.lock().unwrap()));
            }

            ui.separator();
            ui.heading("Step 1: Select Files");

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
                    if ui.link(path.display().to_string()).clicked() {
                        // Optionally open in explorer
                    }
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
                    if ui.link(dir.display().to_string()).clicked() {
                        // Optionally open in explorer
                    }
                }
            });

            // yt-dlp/ffmpeg path settings
            ui.separator();
            ui.collapsing("Advanced: yt-dlp/ffmpeg paths", |ui| {
                ui.horizontal(|ui| {
                    ui.label("yt-dlp path:");
                    let mut path_str = self.yt_dlp_path.display().to_string();
                    if ui.text_edit_singleline(&mut path_str).changed() {
                        self.yt_dlp_path = PathBuf::from(path_str.clone());
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("ffmpeg path:");
                    let mut path_str = self.ffmpeg_path.display().to_string();
                    if ui.text_edit_singleline(&mut path_str).changed() {
                        self.ffmpeg_path = PathBuf::from(path_str.clone());
                    }
                });
                if ui.button("Test yt-dlp/ffmpeg").clicked() {
                    let yt_dlp = self.yt_dlp_path.clone();
                    let ffmpeg = self.ffmpeg_path.clone();
                    let log = Arc::clone(&self.log);
                    thread::spawn(move || {
                        let yt_dlp_ok = std::process::Command::new(&yt_dlp).arg("--version").output().is_ok();
                        let ffmpeg_ok = std::process::Command::new(&ffmpeg).arg("-version").output().is_ok();
                        let mut log = log.lock().unwrap();
                        log.push(format!("yt-dlp: {}", if yt_dlp_ok { "OK" } else { "NOT FOUND" }));
                        log.push(format!("ffmpeg: {}", if ffmpeg_ok { "OK" } else { "NOT FOUND" }));
                    });
                }
            });

            if !self.tracks.is_empty() {
                ui.separator();
                ui.collapsing(format!("CSV Preview (showing first 5 of {} tracks)", self.tracks.len()), |ui| {
                    for t in self.tracks.iter().take(5) {
                        ui.label(format!("{} — {} [{}]", t.title, t.artist, t.album));
                    }
                });
                ui.label(format!("Total tracks loaded: {}", self.tracks.len()));
            }

            ui.separator();
            ui.heading("Step 2: Settings");

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.config.transcode_mp3, "Transcode to MP3")
                    .on_hover_text("Convert downloaded files to MP3 for maximum compatibility.");
                ui.checkbox(&mut self.config.generate_m3u, "Generate M3U")
                    .on_hover_text("Create an M3U playlist file with the downloaded tracks.");
                ui.checkbox(&mut self.config.exclude_instrumentals, "Exclude instrumentals")
                    .on_hover_text("Try to skip instrumental versions in your playlist.");
            });

            ui.collapsing("Show current settings", |ui| {
                ui.monospace(format!("{:#?}", self.config));
            });

            ui.separator();
            ui.heading("Step 3: Convert");

            let can_convert = !self.is_running && self.csv_path.is_some() && self.output_dir.is_some() && !self.tracks.is_empty();

            let convert_btn = ui.add_enabled(
                can_convert,
                egui::Button::new("Convert Playlist")
                    .fill(if can_convert { egui::Color32::DARK_GREEN } else { egui::Color32::GRAY })
            ).on_hover_text("Start downloading and tagging tracks.");

            if convert_btn.clicked() {
                self.confirm_dialog_open = true;
            }

            if !can_convert {
                ui.label(egui::RichText::new("Please select a CSV and output folder to enable conversion.").color(egui::Color32::YELLOW));
            }

            // Progress and cancel
            let (curr, total) = *self.progress.lock().unwrap();
            if self.is_running {
                let elapsed = self.start_time.map(|t| t.elapsed()).unwrap_or(Duration::ZERO);
                let percent = curr as f32 / total.max(1) as f32;
                let eta = if curr > 0 {
                    let avg = elapsed / curr as u32;
                    avg * (total as u32 - curr as u32)
                } else {
                    Duration::ZERO
                };
                ui.add(egui::ProgressBar::new(percent)
                    .show_percentage()
                    .desired_width(400.0)
                    .text(format!("{} / {} (ETA: {:?})", curr, total, eta)));
                if ui.button("Cancel").clicked() {
                    *self.cancel_requested.lock().unwrap() = true;
                }
                ui.spinner();
            }

            // Log window
            ui.separator();
            ui.collapsing("Log", |ui| {
                let log = self.log.lock().unwrap();
                for line in log.iter().rev().take(30) {
                    ui.label(line);
                }
            });

            // Confirm dialog logic
            if self.confirm_dialog_open {
                let mut dialog_open = true;
                let mut should_close_dialog = false;
                egui::Window::new("Confirm Conversion")
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut dialog_open)
                    .show(ctx, |ui| {
                        ui.label("Are you sure you want to start playlist conversion?");
                        if ui.button("Yes, start").clicked() {
                            should_close_dialog = true;
                            if let (Some(_csv), Some(out_dir)) = (&self.csv_path, &self.output_dir) {
                                let config = self.config.clone();
                                let out_dir = out_dir.clone();
                                let tracks = self.tracks.clone();
                                let status_main = Arc::clone(&self.status);
                                let progress_main = Arc::clone(&self.progress);
                                let yt_dlp_path = self.yt_dlp_path.clone();
                                let ffmpeg_path = self.ffmpeg_path.clone();
                                let log = Arc::clone(&self.log);
                                let cancel_requested = Arc::clone(&self.cancel_requested);

                                self.is_running = true;
                                self.last_error = None;
                                *status_main.lock().unwrap() = "Starting conversion...".into();
                                *progress_main.lock().unwrap() = (0, tracks.len().max(1));
                                self.start_time = Some(Instant::now());

                                thread::spawn(move || {
                                    let status_cb = Arc::clone(&status_main);
                                    let progress_cb = Arc::clone(&progress_main);
                                    let log_cb = Arc::clone(&log);
                                    let cancel_cb = Arc::clone(&cancel_requested);
                                    let cb = move |i: usize, total: usize, track: &str| {
                                        *progress_cb.lock().unwrap() = (i + 1, total.max(1));
                                        *status_cb.lock().unwrap() = format!(
                                            "Downloading: {} ({}/{})", track, i + 1, total
                                        );
                                    };
                                    let result = convert_playlist(
                                        &tracks, &config, &yt_dlp_path, &ffmpeg_path, &out_dir, cb
                                    );
                                    match result {
                                        Ok(_) => *status_main.lock().unwrap() = format!(
                                            "Conversion finished! Downloaded {}/{} tracks.", tracks.len(), tracks.len()
                                        ),
                                        Err(e) => {
                                            *status_main.lock().unwrap() = format!("Error: {e}");
                                            log_cb.lock().unwrap().push(format!("Error: {e}"));
                                        }
                                    }
                                });
                            } else {
                                self.last_error = Some("Please select a CSV and output folder.".into());
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            should_close_dialog = true;
                        }
                    });
                if should_close_dialog || !dialog_open {
                    self.confirm_dialog_open = false;
                }
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

            if self.is_running {
                let s = self.status.lock().unwrap();
                if s.starts_with("Conversion finished!") || s.starts_with("Error:") {
                    self.is_running = false;
                }
            }

            ctx.request_repaint_after(std::time::Duration::from_millis(150));
        });

        // About dialog
        if self.show_about {
            let mut show_about_open = true;
            egui::Window::new("About Spotify2Media Rust Port")
                .open(&mut show_about_open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Spotify2Media Rust Port");
                    ui.label("By fentbuscoding.");
                    ui.label("Powered by eframe/egui.");
                    ui.label("Downloads and tags music from Spotify playlists using yt-dlp & ffmpeg.");
                    ui.label("Project: github.com/fentbuscoding/spotify2media-rust");
                });
            self.show_about = show_about_open;
        }
    }
}