use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, Context};
use chrono::Utc;

/// Call yt-dlp to download audio for a track query, saving to output_dir.
/// Returns the path to the downloaded file.
pub fn run_yt_dlp(
    yt_dlp_path: &Path,
    ffmpeg_path: &Path,
    query: &str,
    output_dir: &Path,
    as_mp3: bool,
) -> Result<PathBuf> {
    // Use a unique output template to avoid collisions
    let timestamp = Utc::now().timestamp_millis();
    let output_template = output_dir.join(format!("yt2media_{}_%(title)s.%(ext)s", timestamp));
    let output_template_str = output_template.to_string_lossy();

    let mut cmd = Command::new(yt_dlp_path);
    cmd.arg("-x")
        .arg("--audio-format")
        .arg(if as_mp3 { "mp3" } else { "m4a" })
        .arg("--ffmpeg-location")
        .arg(ffmpeg_path)
        .arg("-o")
        .arg(&output_template_str)
        .arg(format!("ytsearch1:{}", query));

    println!("Running command: {:?}", cmd);

    let output = cmd.output().context("Failed to start yt-dlp")?;
    if !output.status.success() {
        eprintln!("yt-dlp failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("yt-dlp failed to download: {}", String::from_utf8_lossy(&output.stderr)));
    }

    // Find the downloaded file by matching the unique prefix and extension
    let mut found = None;
    for entry in std::fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        println!("Found file: {:?}", path); // Add this line
        if path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with(&format!("yt2media_{}", timestamp)))
            .unwrap_or(false)
            && path.extension().map(|e| e == "mp3" || e == "m4a").unwrap_or(false)
        {
            found = Some(path);
            break;
        }
    }

    found.ok_or_else(|| anyhow::anyhow!("yt-dlp did not produce an output file"))
}