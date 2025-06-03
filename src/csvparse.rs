use std::path::Path;

#[derive(Clone, Debug)]
pub struct TrackInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    // pub duration_ms: Option<u32>, // Uncomment if you ever use it.
}

pub fn parse_csv(path: &Path) -> Result<Vec<TrackInfo>, String> {
    let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for rec in rdr.records() {
        let rec = rec.map_err(|e| e.to_string())?;
        if rec.len() < 3 {
            continue;
        }
        result.push(TrackInfo {
            title: rec.get(0).unwrap_or("").to_string(),
            artist: rec.get(1).unwrap_or("").to_string(),
            album: rec.get(2).unwrap_or("").to_string(),
            // duration_ms: None,
        });
    }
    Ok(result)
}