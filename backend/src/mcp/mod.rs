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
    pub env_key: Option<String>,
    #[serde(default)]
    pub locally_created: bool,
}

// Legacy struct for migration: includes token field that is no longer part of McpServer
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum LegacyOrNewServer {
    Legacy {
        id: Option<String>,
        name: String,
        display_name: String,
        command: String,
        args: Vec<String>,
        token: Option<String>,
        env_key: Option<String>,
        #[serde(default)]
        locally_created: bool,
    },
    New {
        id: String,
        name: String,
        display_name: String,
        command: String,
        args: Vec<String>,
        env_key: Option<String>,
        #[serde(default)]
        locally_created: bool,
    },
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
    let legacy: Vec<LegacyOrNewServer> =
        serde_json::from_str(&json).map_err(|e| e.to_string())?;

    let mut servers = Vec::new();
    let mut migrated_count = 0;

    for item in legacy {
        match item {
            LegacyOrNewServer::Legacy {
                id,
                name,
                display_name,
                command,
                args,
                token,
                env_key,
                locally_created,
            } => {
                let server_id = id.unwrap_or_else(default_server_id);
                // Migrate token to keychain if present
                if let Some(token_value) = token {
                    let _ =
                        crate::secrets::set_mcp_token(&server_id, &token_value);
                    migrated_count += 1;
                }
                servers.push(McpServer {
                    id: server_id,
                    name,
                    display_name,
                    command,
                    args,
                    env_key,
                    locally_created,
                });
            }
            LegacyOrNewServer::New {
                id,
                name,
                display_name,
                command,
                args,
                env_key,
                locally_created,
            } => {
                servers.push(McpServer {
                    id,
                    name,
                    display_name,
                    command,
                    args,
                    env_key,
                    locally_created,
                });
            }
        }
    }

    // If we migrated any tokens, rewrite the file without them
    if migrated_count > 0 {
        eprintln!("[MCP] Migrated {} MCP tokens to keychain", migrated_count);
        save_servers(app_data_dir, &servers)?;
    }

    Ok(servers)
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
        let original_id = server.id.clone();
        save_servers(dir.path(), std::slice::from_ref(&server)).unwrap();
        let loaded = load_servers(dir.path()).unwrap();
        assert_eq!(loaded[0].command, "npx");
        assert_eq!(loaded[0].args, vec!["-y", "some-server"]);
        assert_eq!(loaded[0].env_key.as_deref(), Some("MY_TOKEN"));
        assert_eq!(loaded[0].id, original_id);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_legacy_migration_moves_token_to_keychain() {
        let dir = tempdir().unwrap();
        let server_id = uuid::Uuid::new_v4().to_string();

        // Create a legacy JSON with token field
        let legacy_json = serde_json::json!([{
            "id": server_id,
            "name": "github",
            "display_name": "GitHub",
            "command": "npx",
            "args": ["-y", "@modelcontextprotocol/server-github"],
            "token": "secret-token-value",
            "env_key": "GITHUB_TOKEN"
        }]);

        let path = dir.path().join("mcp_servers.json");
        std::fs::write(&path, legacy_json.to_string())
            .expect("Failed to write legacy JSON");

        // Load the servers (should trigger migration)
        let loaded = load_servers(dir.path()).expect("Failed to load servers");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "github");
        // Token should not be in the loaded server anymore
        // Verify it was moved to keychain
        let token_in_keychain = crate::secrets::get_mcp_token(&server_id)
            .expect("Failed to get token from keychain");
        assert_eq!(token_in_keychain, Some("secret-token-value".to_string()));

        // Verify the file was rewritten without the token
        let rewritten_json = std::fs::read_to_string(&path)
            .expect("Failed to read rewritten JSON");
        assert!(!rewritten_json.contains("secret-token-value"));

        // Cleanup
        let _ = crate::secrets::delete_mcp_token(&server_id);
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
