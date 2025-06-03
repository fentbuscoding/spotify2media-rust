use crate::csvparse::TrackInfo;
use crate::audio::{set_mp3_tags, set_m4a_tags};
use crate::subprocess::run_yt_dlp;
use crate::config::AppConfig;
use std::path::{Path, PathBuf};
use anyhow::Result;

pub fn convert_playlist(
    tracks: &[TrackInfo],
    config: &AppConfig,
    yt_dlp_path: &Path,
    ffmpeg_path: &Path,
    output_dir: &Path,
    progress_cb: impl Fn(usize, usize, &str) + Send + Sync,
) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;
    for (i, track) in tracks.iter().enumerate() {
        let query = format!("{} {}", track.title, track.artist);
        progress_cb(i, tracks.len(), &track.title);
        let out_file = run_yt_dlp(
            yt_dlp_path,
            ffmpeg_path,
            &query,
            output_dir,
            config.transcode_mp3,
        )?;
        // Set tags
        if config.transcode_mp3 && out_file.extension().unwrap_or_default() == "mp3" {
            set_mp3_tags(&out_file, &track.title, &track.artist, &track.album)?;
        } else if out_file.extension().unwrap_or_default() == "m4a" {
            set_m4a_tags(&out_file, &track.title, &track.artist, &track.album)?;
        }
    }
    Ok(())
}