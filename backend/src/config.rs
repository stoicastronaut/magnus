use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub api_key: String,
    pub base_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            api_key: String::new(),
            base_url: "https://api.anthropic.com".to_string(),
        }
    }
}

impl Settings {
    pub fn save(&self, app_data_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(app_data_dir).map_err(|e| e.to_string())?;
        let path = app_data_dir.join("settings.json");
        let str_pretty =
            serde_json::to_string_pretty(&self).map_err(|e| e.to_string())?;
        fs::write(path, str_pretty).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load(app_data_dir: &Path) -> Result<Settings, String> {
        let path = app_data_dir.join("settings.json");
        let json_str = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let json_pretty =
            serde_json::from_str(&json_str).map_err(|e| e.to_string())?;
        Ok(json_pretty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_settings() -> Settings {
        Settings {
            api_key: "sk-test-key".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
        }
    }

    #[test]
    fn test_save_creates_file() {
        let dir = tempdir().unwrap();
        let settings = make_settings();
        settings.save(dir.path()).unwrap();
        assert!(dir.path().join("settings.json").exists());
    }

    #[test]
    fn test_load_returns_correct_settings() {
        let dir = tempdir().unwrap();
        let settings = make_settings();
        settings.save(dir.path()).unwrap();
        let loaded = Settings::load(dir.path()).unwrap();
        assert_eq!(loaded.base_url, settings.base_url);
        assert_eq!(loaded.api_key, settings.api_key);
    }

    #[test]
    fn test_load_fails_on_missing_file() {
        let dir = tempdir().unwrap();
        let result = Settings::load(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.api_key, String::new());
        assert_eq!(settings.base_url, "https://api.anthropic.com");
    }
}
