use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub variants: Vec<String>,
    pub duration_min: u32,
    pub duration_max: u32,
    pub transcode_mp3: bool,
    pub generate_m3u: bool,
    pub exclude_instrumentals: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            variants: vec![],
            duration_min: 30,
            duration_max: 600,
            transcode_mp3: false,
            generate_m3u: true,
            exclude_instrumentals: false,
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let json_str = serde_json::to_string_pretty(self)?;
        fs::write(path, json_str)?;
        Ok(())
    }
}