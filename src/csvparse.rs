use std::path::Path;
use csv::StringRecord;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: Option<u32>,
}

pub fn parse_csv(csv_path: &Path) -> Result<Vec<TrackInfo>> {
    let mut rdr = csv::Reader::from_path(csv_path)?;
    let mut tracks = Vec::new();
    for result in rdr.records() {
        let record = result?;
        // Adjust field numbers to match your CSV headers
        let title = record.get(0).unwrap_or("").to_string();
        let artist = record.get(1).unwrap_or("").to_string();
        let album = record.get(2).unwrap_or("").to_string();
        let duration_ms = record.get(3).and_then(|s| s.parse().ok());
        tracks.push(TrackInfo { title, artist, album, duration_ms });
    }
    Ok(tracks)
}