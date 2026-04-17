use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tokio::sync::Mutex;
mod client;
pub use client::{call_tool, connect, connect_server, list_tools, McpClient};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct McpServer {
    pub name: String,
    pub display_name: String,
    pub command: String,
    pub args: Vec<String>,
    pub token: Option<String>,
    pub env_key: Option<String>,
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
            name: name.to_string(),
            display_name: name.to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "some-server".to_string()],
            token: Some("tok".to_string()),
            env_key: Some("MY_TOKEN".to_string()),
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
        save_servers(dir.path(), &[server.clone()]).unwrap();
        let loaded = load_servers(dir.path()).unwrap();
        assert_eq!(loaded[0].command, "npx");
        assert_eq!(loaded[0].args, vec!["-y", "some-server"]);
        assert_eq!(loaded[0].token.as_deref(), Some("tok"));
        assert_eq!(loaded[0].env_key.as_deref(), Some("MY_TOKEN"));
    }
}
