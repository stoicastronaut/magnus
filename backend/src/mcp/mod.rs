use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tokio::sync::Mutex;
mod client;
pub use client::{connect, connect_server, call_tool, list_tools, McpClient};

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

pub fn save_servers(app_data_dir: &Path, servers: &[McpServer]) -> Result<(), String> {
    fs::create_dir_all(app_data_dir).map_err(|e| e.to_string())?;
    let path = app_data_dir.join("mcp_servers.json");
    let json = serde_json::to_string_pretty(servers).map_err(|e| e.to_string())?;
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
