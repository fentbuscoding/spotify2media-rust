use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub transcode_mp3: bool,
    pub generate_m3u: bool,
    pub exclude_instrumentals: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            transcode_mp3: true,
            generate_m3u: true,
            exclude_instrumentals: false,
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Self {
        if let Ok(txt) = fs::read_to_string(path) {
            if let Ok(cfg) = serde_json::from_str(&txt) {
                return cfg;
            }
        }
        Self::default()
    }

    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let txt = serde_json::to_string_pretty(self).unwrap();
        fs::write(path, txt)
    }
}