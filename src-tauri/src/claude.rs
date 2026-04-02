use serde::{Serialize, Deserialize};
use futures_util::StreamExt;
use tauri::Emitter;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String
}

pub async fn stream_message(app: tauri::AppHandle, api_key: &str, base_url: &str, messages: &[Message]) -> Result<(), String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": 1024,
        "stream": true,
        "messages": messages.iter().map(|m| serde_json::json!({
            "role": m.role,
            "content": [{"type": "text", "text": m.content}]
        })).collect::<Vec<_>>()
    });
    let response = client
        .post(format!("{}v1/messages", base_url))
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&chunk).to_string();
        for line in text.lines() {
            if !line.starts_with("data: ") {
                continue;
            }

            let Ok(json) =
                serde_json::from_str::<serde_json::Value>(line.trim_start_matches("data: ")) else {
                    continue;
            };
            if json["type"] != "content_block_delta" {
                continue
            }
            let token = json["delta"]["text"].as_str().unwrap_or("");
            if !token.is_empty() {
                app.emit("stream-token", token).unwrap();
            }
        }
    }
    Ok(())
}
