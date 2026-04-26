use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ToolTrust {
    Disabled,
    AskEveryTime,
    AlwaysAllow,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct TrustStore {
    /// key = "{server_id}::{tool_name}"
    pub tools: HashMap<String, ToolTrust>,
    pub destructive_patterns: Vec<String>,
}

impl TrustStore {
    /// Create a new TrustStore with default destructive patterns
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            destructive_patterns: vec![
                "*delete*".to_string(),
                "*remove*".to_string(),
                "*write*".to_string(),
                "*exec*".to_string(),
                "*run*".to_string(),
                "*kill*".to_string(),
            ],
        }
    }

    /// Check if a tool name matches any destructive pattern
    pub fn is_destructive(&self, tool_name: &str) -> bool {
        self.destructive_patterns
            .iter()
            .any(|pattern| glob_match(tool_name, pattern))
    }

    /// Get the trust level for a tool
    /// Defaults to AskEveryTime if not found
    pub fn get_tool_trust(
        &self,
        server_id: &str,
        tool_name: &str,
    ) -> ToolTrust {
        let key = format!("{}::{}", server_id, tool_name);
        self.tools
            .get(&key)
            .cloned()
            .unwrap_or(ToolTrust::AskEveryTime)
    }

    /// Set the trust level for a tool
    pub fn set_tool_trust(
        &mut self,
        server_id: &str,
        tool_name: &str,
        trust: ToolTrust,
    ) {
        let key = format!("{}::{}", server_id, tool_name);
        self.tools.insert(key, trust);
    }

    /// Check if a tool should be executed without prompting
    /// Returns true if AlwaysAllow AND not destructive
    /// Always returns false for Disabled or destructive patterns
    pub fn can_execute_without_prompt(
        &self,
        server_id: &str,
        tool_name: &str,
    ) -> bool {
        if self.is_destructive(tool_name) {
            return false;
        }
        self.get_tool_trust(server_id, tool_name) == ToolTrust::AlwaysAllow
    }

    /// Check if a tool is disabled
    pub fn is_tool_disabled(&self, server_id: &str, tool_name: &str) -> bool {
        self.get_tool_trust(server_id, tool_name) == ToolTrust::Disabled
    }
}

/// Simple glob pattern matching for wildcard patterns like "*delete*"
fn glob_match(text: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    // Handle patterns like "*delete*", "delete*", "*delete"
    if pattern.starts_with('*') && pattern.ends_with('*') {
        let middle = &pattern[1..pattern.len() - 1];
        text.contains(middle)
    } else if let Some(end) = pattern.strip_prefix('*') {
        text.ends_with(end)
    } else if let Some(start) = pattern.strip_suffix('*') {
        text.starts_with(start)
    } else {
        text == pattern
    }
}

pub fn save_trust_store(
    app_data_dir: &Path,
    store: &TrustStore,
) -> Result<(), String> {
    fs::create_dir_all(app_data_dir).map_err(|e| e.to_string())?;
    let path = app_data_dir.join("mcp_trust.json");
    let json =
        serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_trust_store(app_data_dir: &Path) -> Result<TrustStore, String> {
    let path = app_data_dir.join("mcp_trust.json");
    if !path.exists() {
        return Ok(TrustStore::new());
    }
    let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut store: TrustStore =
        serde_json::from_str(&json).map_err(|e| e.to_string())?;

    // Ensure destructive_patterns are initialized if missing
    if store.destructive_patterns.is_empty() {
        store.destructive_patterns = vec![
            "*delete*".to_string(),
            "*remove*".to_string(),
            "*write*".to_string(),
            "*exec*".to_string(),
            "*run*".to_string(),
            "*kill*".to_string(),
        ];
    }

    Ok(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_glob_match_middle() {
        assert!(glob_match("filesystem.delete_file", "*delete*"));
        assert!(glob_match("tool_delete_db", "*delete*"));
        assert!(!glob_match("filesystem.read_file", "*delete*"));
    }

    #[test]
    fn test_glob_match_start() {
        assert!(glob_match("delete_file", "delete*"));
        assert!(!glob_match("file_delete", "delete*"));
    }

    #[test]
    fn test_glob_match_end() {
        assert!(glob_match("file_delete", "*delete"));
        assert!(!glob_match("delete_file", "*delete"));
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("delete", "delete"));
        assert!(!glob_match("delete_file", "delete"));
    }

    #[test]
    fn test_is_destructive() {
        let store = TrustStore::new();
        assert!(store.is_destructive("filesystem.delete_file"));
        assert!(store.is_destructive("filesystem.remove_dir"));
        assert!(store.is_destructive("filesystem.write_file"));
        assert!(store.is_destructive("shell.exec"));
        assert!(!store.is_destructive("filesystem.read_file"));
    }

    #[test]
    fn test_default_trust_is_ask_every_time() {
        let store = TrustStore::new();
        assert_eq!(
            store.get_tool_trust("my_server_id", "my_tool"),
            ToolTrust::AskEveryTime
        );
    }

    #[test]
    fn test_set_and_get_tool_trust() {
        let mut store = TrustStore::new();
        store.set_tool_trust("server-id-1", "tool1", ToolTrust::AlwaysAllow);
        assert_eq!(
            store.get_tool_trust("server-id-1", "tool1"),
            ToolTrust::AlwaysAllow
        );
    }

    #[test]
    fn test_always_allow_overridden_by_destructive() {
        let mut store = TrustStore::new();
        store.set_tool_trust(
            "server-id-1",
            "filesystem.delete",
            ToolTrust::AlwaysAllow,
        );
        assert!(!store
            .can_execute_without_prompt("server-id-1", "filesystem.delete"));
    }

    #[test]
    fn test_disabled_tool() {
        let mut store = TrustStore::new();
        store.set_tool_trust("server-id-1", "tool1", ToolTrust::Disabled);
        assert!(store.is_tool_disabled("server-id-1", "tool1"));
    }

    #[test]
    fn test_roundtrip_serialization() {
        let dir = tempdir().unwrap();
        let mut store = TrustStore::new();
        store.set_tool_trust("server-id-1", "tool1", ToolTrust::AlwaysAllow);
        store.set_tool_trust("server-id-2", "tool2", ToolTrust::Disabled);

        save_trust_store(dir.path(), &store).unwrap();
        let loaded = load_trust_store(dir.path()).unwrap();

        assert_eq!(
            loaded.get_tool_trust("server-id-1", "tool1"),
            ToolTrust::AlwaysAllow
        );
        assert_eq!(
            loaded.get_tool_trust("server-id-2", "tool2"),
            ToolTrust::Disabled
        );
        assert_eq!(loaded.destructive_patterns.len(), 6);
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let dir = tempdir().unwrap();
        let store = load_trust_store(dir.path()).unwrap();
        assert_eq!(store.tools.len(), 0);
        assert!(!store.destructive_patterns.is_empty());
    }
}
