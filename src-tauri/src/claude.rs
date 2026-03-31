pub async fn test_connect(api_key: &str, base_url: &str) -> Result<String, String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": 100,
        "messages": [{"role": "user", "content": "Hello Claude"}]
    });
    let response = client
        .post(format!("{}v1/messages", base_url))
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let text = json["content"][0]["text"].as_str().unwrap_or("no response");
    Ok(text.to_string())
}
