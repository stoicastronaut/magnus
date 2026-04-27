use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::fs_perm;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub default_provider_id: Option<String>,
    pub providers: Vec<ProviderConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProviderConfig {
    pub id: String,
    pub display_name: String,
    #[serde(flatten)]
    pub _type: ProviderType,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProviderType {
    BuiltIn {
        which: BuiltInId,
    },
    Custom {
        protocol: Protocol,
        base_url: String,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BuiltInId {
    Anthropic,
    #[serde(rename = "open_ai")]
    OpenAI,
    Google,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Anthropic,
    #[serde(rename = "open_ai")]
    OpenAI,
    Google,
}

impl Settings {
    pub fn save(&self, app_data_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(app_data_dir).map_err(|e| e.to_string())?;
        let path = app_data_dir.join("settings.json");
        let str_pretty =
            serde_json::to_string_pretty(&self).map_err(|e| e.to_string())?;
        fs::write(&path, str_pretty).map_err(|e| e.to_string())?;
        fs_perm::restrict_permissions(&path).map_err(|e| e.to_string())?;
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

    fn built_in_provider() -> ProviderConfig {
        ProviderConfig {
            id: "anthropic".to_string(),
            display_name: "Anthropic".to_string(),
            _type: ProviderType::BuiltIn {
                which: BuiltInId::Anthropic,
            },
        }
    }

    fn custom_provider() -> ProviderConfig {
        ProviderConfig {
            id: "corp-openai".to_string(),
            display_name: "Corp OpenAI".to_string(),
            _type: ProviderType::Custom {
                protocol: Protocol::OpenAI,
                base_url: "https://proxy.example.com/v1/".to_string(),
            },
        }
    }

    fn make_settings() -> Settings {
        Settings {
            default_provider_id: Some("anthropic".to_string()),
            providers: vec![built_in_provider(), custom_provider()],
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
        assert_eq!(loaded.default_provider_id, settings.default_provider_id);
        assert_eq!(loaded.providers.len(), 2);
        assert_eq!(loaded.providers[0].id, "anthropic");
        assert_eq!(loaded.providers[1].display_name, "Corp OpenAI");
        assert_eq!(
            loaded.providers[1]._type,
            ProviderType::Custom {
                protocol: Protocol::OpenAI,
                base_url: "https://proxy.example.com/v1/".to_string(),
            }
        );
    }

    #[test]
    fn test_load_fails_on_missing_file() {
        let dir = tempdir().unwrap();
        let result = Settings::load(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_serialization_uses_kind_tagged_provider_shape() {
        let settings = make_settings();
        let json = serde_json::to_value(&settings).unwrap();

        assert_eq!(json["providers"][0]["kind"], "built_in");
        assert_eq!(json["providers"][0]["which"], "anthropic");
        assert_eq!(json["providers"][1]["kind"], "custom");
        assert_eq!(json["providers"][1]["protocol"], "open_ai");
        assert_eq!(
            json["providers"][1]["base_url"],
            "https://proxy.example.com/v1/"
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_save_creates_file_with_0600_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let settings = make_settings();
        settings.save(dir.path()).unwrap();
        let path = dir.path().join("settings.json");

        let perms = fs::metadata(&path).unwrap().permissions();
        let mode = perms.mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "Settings file should be readable/writable by owner only, got {:#o}",
            mode
        );
    }
}
