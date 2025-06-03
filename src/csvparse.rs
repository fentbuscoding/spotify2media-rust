use std::path::Path;

#[derive(Clone, Debug)]
pub struct TrackInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
}

pub fn parse_csv(path: &Path) -> Result<Vec<TrackInfo>, String> {
    let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
    let has_header = headers.iter().any(|h| h.eq_ignore_ascii_case("title") || h.eq_ignore_ascii_case("track"));

    let mut result = Vec::new();
    for (i, rec) in rdr.records().enumerate() {
        let rec = rec.map_err(|e| format!("CSV parse error on row {}: {}", i + 2, e))?;
        // If header present, use header names, else use index
        let (title, artist, album) = if has_header {
            (
                rec.get(headers.iter().position(|h| h.eq_ignore_ascii_case("title") || h.eq_ignore_ascii_case("track")).unwrap_or(0)).unwrap_or(""),
                rec.get(headers.iter().position(|h| h.eq_ignore_ascii_case("artist")).unwrap_or(1)).unwrap_or(""),
                rec.get(headers.iter().position(|h| h.eq_ignore_ascii_case("album")).unwrap_or(2)).unwrap_or(""),
            )
        } else {
            (
                rec.get(0).unwrap_or(""),
                rec.get(1).unwrap_or(""),
                rec.get(2).unwrap_or(""),
            )
        };
        // Skip empty rows
        if title.trim().is_empty() && artist.trim().is_empty() {
            continue;
        }
        result.push(TrackInfo {
            title: title.trim().to_string(),
            artist: artist.trim().to_string(),
            album: album.trim().to_string(),
        });
    }
    Ok(result)
}