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
