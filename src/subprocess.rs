use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::Result;

/// Call yt-dlp to download audio for a track query, saving to output_dir.
/// Returns the path to the downloaded file.
pub fn run_yt_dlp(
    yt_dlp_path: &Path,
    ffmpeg_path: &Path,
    query: &str,
    output_dir: &Path,
    as_mp3: bool,
) -> Result<PathBuf> {
    let output_template = output_dir.join("%(title)s.%(ext)s");
    let mut cmd = Command::new(yt_dlp_path);
    cmd.arg("-x")
        .arg("--audio-format")
        .arg(if as_mp3 { "mp3" } else { "m4a" })
        .arg("--ffmpeg-location")
        .arg(ffmpeg_path)
        .arg("-o")
        .arg(output_template)
        .arg(format!("ytsearch1:{}", query));
    let status = cmd.status()?;
    if status.success() {
        // Return first .mp3 or .m4a found in output_dir
        for entry in std::fs::read_dir(output_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "mp3" || e == "m4a").unwrap_or(false) {
                return Ok(path);
            }
        }
    }
    Err(anyhow::anyhow!("yt-dlp failed to download"))
}