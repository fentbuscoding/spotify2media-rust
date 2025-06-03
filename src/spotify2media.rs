use crate::config::AppConfig;
use crate::audio::{set_mp3_tags, set_m4a_tags, is_valid_mp3};
use crate::csvparse::TrackInfo;
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use anyhow::{Result, Context};
use chrono::Utc;
use std::sync::{Arc, Mutex};

/// Improved CSV parser (if you don't use the one from csvparse.rs, otherwise remove this)
pub fn parse_csv(csv_path: &Path) -> Result<Vec<TrackInfo>> {
    let mut rdr = csv::Reader::from_path(csv_path)?;
    let mut tracks = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let title = record.get(0).unwrap_or("").trim().to_string();
        let artist = record.get(1).unwrap_or("").trim().to_string();
        let album = record.get(2).unwrap_or("").trim().to_string();
        if title.is_empty() && artist.is_empty() {
            continue;
        }
        tracks.push(TrackInfo { title, artist, album });
    }
    Ok(tracks)
}

/// Run yt-dlp and return the path to the downloaded audio file.
/// Logs all output and errors to the provided log (if any).
pub fn run_yt_dlp(
    yt_dlp_path: Option<&Path>,
    ffmpeg_path: Option<&Path>,
    query: &str,
    output_dir: &Path,
    as_mp3: bool,
    log: Option<&Arc<Mutex<Vec<String>>>>,
) -> Result<PathBuf> {
    let yt_dlp_path = yt_dlp_path.unwrap_or_else(|| Path::new("yt-dlp"));
    let ffmpeg_path = ffmpeg_path.unwrap_or_else(|| Path::new("ffmpeg"));
    let timestamp = Utc::now().timestamp_millis();
    let output_template = output_dir.join(format!("yt2media_{}_%(title)s.%(ext)s", timestamp));
    let output_template_str = output_template.to_string_lossy();

    let mut cmd = Command::new(yt_dlp_path);
    cmd.arg("-x")
        .arg("--audio-format")
        .arg(if as_mp3 { "mp3" } else { "m4a" })
        .arg("--audio-quality")
        .arg("0")
        .arg("--ffmpeg-location")
        .arg(ffmpeg_path)
        .arg("-o")
        .arg(&*output_template_str)
        .arg(format!("ytsearch1:{}", query));

    if let Some(log) = log {
        log.lock().unwrap().push(format!("Running: {:?}", cmd));
    }

    let output = cmd.output().context("Failed to start yt-dlp")?;
    if let Some(log) = log {
        log.lock().unwrap().push(format!("yt-dlp stdout: {}", String::from_utf8_lossy(&output.stdout)));
        log.lock().unwrap().push(format!("yt-dlp stderr: {}", String::from_utf8_lossy(&output.stderr)));
    }
    if !output.status.success() {
        return Err(anyhow::anyhow!("yt-dlp failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    // List all files in output_dir for debugging
    if let Some(log) = log {
        let files: Vec<_> = fs::read_dir(output_dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();
        log.lock().unwrap().push(format!("Files in output dir: {:?}", files));
    }

    // Find the downloaded file (accept any extension, prefer mp3/m4a)
    let mut found: Option<PathBuf> = None;
    let mut fallback: Option<PathBuf> = None;
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if fname.starts_with(&format!("yt2media_{}", timestamp)) {
            match path.extension().and_then(|e| e.to_str()) {
                Some("mp3") | Some("m4a") => {
                    found = Some(path.clone());
                    break;
                }
                Some(_) => {
                    fallback = Some(path.clone());
                }
                None => {}
            }
        }
    }
    found.or(fallback).ok_or_else(|| anyhow::anyhow!("yt-dlp did not produce an output file"))
}

/// Main playlist conversion logic
pub fn convert_playlist(
    tracks: &[TrackInfo],
    config: &AppConfig,
    yt_dlp_path: Option<&Path>,
    ffmpeg_path: Option<&Path>,
    output_dir: &Path,
    progress_cb: impl Fn(usize, usize, &str),
) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    for (i, track) in tracks.iter().enumerate() {
        // Prefer official artist topic channel, fallback to plain search if needed
        let queries = [
            format!("{} {} topic", track.title, track.artist),
            format!("{} {} official audio", track.title, track.artist),
            format!("{} {}", track.title, track.artist),
        ];
        let mut last_err = None;
        let mut out_file = None;
        for query in &queries {
            progress_cb(i, tracks.len(), &track.title);
            match run_yt_dlp(
                yt_dlp_path,
                ffmpeg_path,
                query,
                output_dir,
                config.transcode_mp3,
                None,
            ) {
                Ok(path) => {
                    out_file = Some(path);
                    break;
                }
                Err(e) => {
                    last_err = Some(e);
                }
            }
        }
        let out_file = out_file.ok_or_else(|| {
            anyhow::anyhow!(
                "yt-dlp failed for '{} - {}': {:?}",
                track.title,
                track.artist,
                last_err
            )
        })?;

        // Set tags if possible
        if config.transcode_mp3 && out_file.extension().unwrap_or_default() == "mp3" {
            if is_valid_mp3(&out_file) {
                set_mp3_tags(&out_file, &track.title, &track.artist, &track.album)
                    .with_context(|| format!("Failed to save tags to {:?}", out_file))?;
            } else {
                return Err(anyhow::anyhow!("Downloaded file is not a valid MP3: {:?}", out_file));
            }
        } else if out_file.extension().unwrap_or_default() == "m4a" {
            set_m4a_tags(&out_file, &track.title, &track.artist, &track.album)
                .with_context(|| format!("Failed to save tags to {:?}", out_file))?;
        }
    }
    Ok(())
}