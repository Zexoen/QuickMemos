use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_url: String,
    pub access_token: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            access_token: String::new(),
        }
    }
}

impl AppConfig {
    fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("quickmemos"))
    }

    fn config_file() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("config.toml"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_file() else {
            return Self::default();
        };
        let Ok(content) = fs::read_to_string(&path) else {
            return Self::default();
        };
        toml::from_str(&content).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let Some(dir) = Self::config_dir() else {
            return Err("Cannot determine config directory".into());
        };
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join("config.toml");
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }
}
