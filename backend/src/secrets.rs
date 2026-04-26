const SERVICE: &str = "com.stoicastronaut.magnus";

pub fn set_api_key(provider_id: &str, key: &str) -> Result<(), String> {
    keyring::Entry::new(SERVICE, provider_id)
        .and_then(|e| e.set_password(key))
        .map_err(|e| e.to_string())
}

pub fn get_api_key(provider_id: &str) -> Result<String, String> {
    keyring::Entry::new(SERVICE, provider_id)
        .and_then(|e| e.get_password())
        .map_err(|e| e.to_string())
}

pub fn delete_api_key(provider_id: &str) -> Result<(), String> {
    keyring::Entry::new(SERVICE, provider_id)
        .and_then(|e| e.delete_credential())
        .map_err(|e| e.to_string())
}

// MCP token helpers - store in keychain with mcp:: prefix
pub fn set_mcp_token(server_id: &str, token: &str) -> Result<(), String> {
    keyring::Entry::new(SERVICE, &format!("mcp::{}", server_id))
        .and_then(|e| e.set_password(token))
        .map_err(|e| e.to_string())
}

pub fn get_mcp_token(server_id: &str) -> Result<Option<String>, String> {
    match keyring::Entry::new(SERVICE, &format!("mcp::{}", server_id))
        .and_then(|e| e.get_password())
    {
        Ok(token) => Ok(Some(token)),
        Err(keyring::error::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn delete_mcp_token(server_id: &str) -> Result<(), String> {
    keyring::Entry::new(SERVICE, &format!("mcp::{}", server_id))
        .and_then(|e| e.delete_credential())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn test_mcp_token_roundtrip() {
        let server_id = format!("test-{}", uuid::Uuid::new_v4());
        let token = "test-token-secret";

        // Set token
        set_mcp_token(&server_id, token).expect("Failed to set token");

        // Get token
        let retrieved = get_mcp_token(&server_id)
            .expect("Failed to get token")
            .expect("Token should exist");
        assert_eq!(retrieved, token);

        // Delete token
        delete_mcp_token(&server_id).expect("Failed to delete token");

        // Verify deletion
        let after_delete =
            get_mcp_token(&server_id).expect("Failed to get after delete");
        assert_eq!(after_delete, None);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_get_nonexistent_mcp_token_returns_none() {
        let server_id = format!("nonexistent-{}", uuid::Uuid::new_v4());
        let result =
            get_mcp_token(&server_id).expect("Failed to query keychain");
        assert_eq!(result, None);
    }
}
