use crate::config::AppConfig;
use crate::audio::{set_mp3_tags, set_m4a_tags};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use std::collections::HashMap;
use csv::Reader;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: Option<u32>,
}

pub fn parse_csv(csv_path: &Path) -> anyhow::Result<Vec<TrackInfo>> {
    let mut rdr = Reader::from_path(csv_path)?;
    let mut tracks = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let title = record.get(0).unwrap_or("").to_string();
        let artist = record.get(1).unwrap_or("").to_string();
        let album = record.get(2).unwrap_or("").to_string();
        let duration_ms = record.get(3).and_then(|s| s.parse().ok());
        tracks.push(TrackInfo { title, artist, album, duration_ms });
    }
    Ok(tracks)
}

pub fn run_yt_dlp(
    yt_dlp_path: &Path,
    ffmpeg_path: &Path,
    query: &str,
    output_dir: &Path,
    as_mp3: bool,
) -> anyhow::Result<PathBuf> {
    let mut cmd = Command::new(yt_dlp_path);
    let output_template = format!("{}/%(title)s.%(ext)s", output_dir.display());
    cmd.arg("-x")
        .arg("--audio-format")
        .arg(if as_mp3 { "mp3" } else { "m4a" })
        .arg("--ffmpeg-location")
        .arg(ffmpeg_path)
        .arg("-o")
        .arg(&output_template)
        .arg(format!("ytsearch1:{}", query));
    let status = cmd.status()?;
    if status.success() {
        // Find the output file
        for entry in fs::read_dir(output_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "mp3" || e == "m4a").unwrap_or(false) {
                return Ok(path);
            }
        }
    }
    Err(anyhow::anyhow!("yt-dlp failed to download"))
}

pub fn convert_playlist(
    tracks: Vec<TrackInfo>,
    config: &AppConfig,
    yt_dlp_path: &Path,
    ffmpeg_path: &Path,
    output_dir: &Path,
    progress_cb: impl Fn(usize, usize, &str),
) -> anyhow::Result<()> {
    fs::create_dir_all(output_dir)?;
    for (i, track) in tracks.iter().enumerate() {
        let query = format!("{} {}", track.title, track.artist);
        progress_cb(i, tracks.len(), &track.title);
        let out_file = run_yt_dlp(
            yt_dlp_path,
            ffmpeg_path,
            &query,
            output_dir,
            config.transcode_mp3
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