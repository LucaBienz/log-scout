use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PatternEntry {
    pub name: String,
    pub pattern: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchProfile {
    pub name: String,
    pub file_path: String,
    pub error_patterns: Vec<PatternEntry>,
}

impl WatchProfile {
    pub fn save(&self, filename: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(filename, json)
    }

    pub fn load(filename: &str) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(filename)?;
        let profile = serde_json::from_str(&content)?;
        Ok(profile)
    }
}