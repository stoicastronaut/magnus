use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub api_key: String,
    pub base_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            api_key: String::new(),
            base_url: "https://api.anthropic.com".to_string()
        }
    }
}

impl Settings {
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let str_pretty = serde_json::to_string_pretty(&self).map_err(|e| e.to_string())?;
        fs::write(path, str_pretty).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Settings, String> {
        let json_str = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let json_pretty = serde_json::from_str(&json_str).map_err(|e| e.to_string())?;
        Ok(json_pretty)
    }
}
