use crate::config::AppConfig;
use crate::csvparse::TrackInfo;
use std::path::Path;

pub fn convert_playlist(
    tracks: &[TrackInfo],
    _config: &AppConfig,
    _yt_dlp_path: &Path,
    _ffmpeg_path: &Path,
    _output_dir: &Path,
    progress_callback: impl Fn(usize, usize, &str) + Send + Sync + 'static,
) -> Result<(), String> {
    // Dummy logic: just call the progress callback for each track
    for (i, track) in tracks.iter().enumerate() {
        progress_callback(i, tracks.len(), &track.title);
        // Here, you'd spawn yt-dlp/ffmpeg processes, etc.
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}