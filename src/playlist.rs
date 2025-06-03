use crate::config::AppConfig;
use crate::csvparse::TrackInfo;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Wrapper for playlist conversion with improved error handling and optional logging.
///
/// # Arguments
/// * `tracks` - List of tracks to process.
/// * `config` - Application configuration.
/// * `yt_dlp_path` - Path to yt-dlp executable.
/// * `ffmpeg_path` - Path to ffmpeg executable.
/// * `output_dir` - Directory to save output files.
/// * `progress_callback` - Callback for progress updates.
/// * `log` - Optional log for GUI or CLI output.
///
/// # Returns
/// * `Ok(())` on success, or `Err(String)` with a detailed error message.
pub fn convert_playlist(
    tracks: &[TrackInfo],
    config: &AppConfig,
    yt_dlp_path: &Path,
    ffmpeg_path: &Path,
    output_dir: &Path,
    progress_callback: impl Fn(usize, usize, &str),
    log: Option<&Arc<Mutex<Vec<String>>>>,
) -> Result<(), String> {
    match crate::spotify2media::convert_playlist(
        tracks,
        config,
        Some(yt_dlp_path),
        Some(ffmpeg_path),
        output_dir,
        |i, total, track| {
            progress_callback(i, total, track);
            if let Some(log) = log {
                log.lock().unwrap().push(format!("Progress: {}/{} - {}", i + 1, total, track));
            }
        },
    ) {
        Ok(_) => {
            if let Some(log) = log {
                log.lock().unwrap().push("Playlist conversion finished successfully.".to_string());
            }
            Ok(())
        }
        Err(e) => {
            if let Some(log) = log {
                log.lock().unwrap().push(format!("Error during playlist conversion: {e}"));
            }
            Err(format!("Playlist conversion failed: {e}"))
        }
    }
}