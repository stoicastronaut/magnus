use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tokio::sync::Mutex;
use uuid::Uuid;
mod client;
mod trust;
mod validate;
pub use client::{call_tool, connect, connect_server, list_tools, McpClient};
pub use trust::{load_trust_store, save_trust_store, ToolTrust};
pub use validate::validate_command;

fn default_server_id() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct McpServer {
    #[serde(default = "default_server_id")]
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub command: String,
    pub args: Vec<String>,
    pub token: Option<String>,
    pub env_key: Option<String>,
    #[serde(default)]
    pub locally_created: bool,
}

pub struct McpPool {
    pub connections: Mutex<HashMap<String, McpClient>>,
}

impl McpPool {
    pub fn new() -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
        }
    }
}

pub fn save_servers(
    app_data_dir: &Path,
    servers: &[McpServer],
) -> Result<(), String> {
    fs::create_dir_all(app_data_dir).map_err(|e| e.to_string())?;
    let path = app_data_dir.join("mcp_servers.json");
    let json =
        serde_json::to_string_pretty(servers).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_servers(app_data_dir: &Path) -> Result<Vec<McpServer>, String> {
    let path = app_data_dir.join("mcp_servers.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_server(name: &str) -> McpServer {
        McpServer {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            display_name: name.to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "some-server".to_string()],
            token: Some("tok".to_string()),
            env_key: Some("MY_TOKEN".to_string()),
            locally_created: true,
        }
    }

    #[test]
    fn test_save_creates_file() {
        let dir = tempdir().unwrap();
        save_servers(dir.path(), &[make_server("github")]).unwrap();
        assert!(dir.path().join("mcp_servers.json").exists());
    }

    #[test]
    fn test_load_returns_correct_servers() {
        let dir = tempdir().unwrap();
        let servers = vec![make_server("github"), make_server("linear")];
        save_servers(dir.path(), &servers).unwrap();
        let loaded = load_servers(dir.path()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "github");
        assert_eq!(loaded[1].name, "linear");
    }

    #[test]
    fn test_load_returns_empty_when_file_missing() {
        let dir = tempdir().unwrap();
        let loaded = load_servers(dir.path()).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_roundtrip_preserves_fields() {
        let dir = tempdir().unwrap();
        let server = make_server("github");
        save_servers(dir.path(), std::slice::from_ref(&server)).unwrap();
        let loaded = load_servers(dir.path()).unwrap();
        assert_eq!(loaded[0].command, "npx");
        assert_eq!(loaded[0].args, vec!["-y", "some-server"]);
        assert_eq!(loaded[0].token.as_deref(), Some("tok"));
        assert_eq!(loaded[0].env_key.as_deref(), Some("MY_TOKEN"));
    }

    #[test]
    fn test_load_missing_id_generates_and_rewrites() {
        let dir = tempdir().unwrap();
        // Write a legacy server JSON without id
        let legacy_json = r#"[
  {
    "name": "github",
    "display_name": "GitHub",
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-github"],
    "token": null,
    "env_key": null
  }
]"#;
        let path = dir.path().join("mcp_servers.json");
        fs::write(&path, legacy_json).unwrap();

        // Load should auto-generate id with default serde
        let loaded = load_servers(dir.path()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(!loaded[0].id.is_empty());
        // id should be a valid UUID
        assert!(uuid::Uuid::parse_str(&loaded[0].id).is_ok());
    }

    #[test]
    fn test_load_missing_locally_created_defaults_to_false() {
        let dir = tempdir().unwrap();
        // Write a server JSON without locally_created
        let legacy_json = r#"[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "github",
    "display_name": "GitHub",
    "command": "npx",
    "args": ["-y"],
    "token": null,
    "env_key": null
  }
]"#;
        let path = dir.path().join("mcp_servers.json");
        fs::write(&path, legacy_json).unwrap();

        let loaded = load_servers(dir.path()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(!loaded[0].locally_created);
    }
}
