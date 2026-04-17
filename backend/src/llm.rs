use crate::chats::Message;
use futures_util::StreamExt;
use tauri::Emitter;

pub struct ToolUse {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

pub async fn change_chat_name(
    api_key: &str,
    base_url: &str,
    message: &Message,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": 1024,
        "messages": [{
            "role": "user",
            "content": format!(
                "Summarize the following message in 5 to 10 words: {}", message.content
            ),
        }]
    });

    let response = client
        .post(format!("{}v1/messages", base_url))
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value =
        response.json().await.map_err(|e| e.to_string())?;
    let text = json["content"][0]["text"].as_str().unwrap_or("no response");
    Ok(text.trim_matches('"').trim().to_string())
}

/// Streams one turn of the conversation. Returns the assistant content blocks
/// and any tool_use calls Claude wants to make.
pub async fn stream_message(
    app: &tauri::AppHandle,
    api_key: &str,
    base_url: &str,
    messages: &[serde_json::Value],
    tools: &[serde_json::Value],
) -> Result<(Vec<serde_json::Value>, Vec<ToolUse>), String> {
    let client = reqwest::Client::new();

    let mut body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": 4096,
        "stream": true,
        "messages": messages,
    });

    if !tools.is_empty() {
        body["tools"] = serde_json::json!(tools);
    }

    let response = client
        .post(format!("{}v1/messages", base_url))
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let mut stream = response.bytes_stream();

    let mut content_blocks: Vec<serde_json::Value> = Vec::new();
    let mut tool_uses: Vec<ToolUse> = Vec::new();
    let mut current_text = String::new();
    let mut current_tool_input = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&chunk).to_string();

        for line in text.lines() {
            if !line.starts_with("data: ") {
                continue;
            }
            let data = line.trim_start_matches("data: ");
            if data == "[DONE]" {
                continue;
            }
            let Ok(json) = serde_json::from_str::<serde_json::Value>(data) else {
                continue;
            };

            match json["type"].as_str() {
                Some("content_block_start") => {
                    current_text.clear();
                    current_tool_input.clear();
                    let block = &json["content_block"];
                    if block["type"] == "tool_use" {
                        tool_uses.push(ToolUse {
                            id: block["id"].as_str().unwrap_or("").to_string(),
                            name: block["name"].as_str().unwrap_or("").to_string(),
                            input: serde_json::Value::Null,
                        });
                    }
                }
                Some("content_block_delta") => {
                    let delta = &json["delta"];
                    match delta["type"].as_str() {
                        Some("text_delta") => {
                            let token = delta["text"].as_str().unwrap_or("");
                            if !token.is_empty() {
                                current_text.push_str(token);
                                app.emit("stream-token", token).unwrap();
                            }
                        }
                        Some("input_json_delta") => {
                            let partial = delta["partial_json"].as_str().unwrap_or("");
                            current_tool_input.push_str(partial);
                        }
                        _ => {}
                    }
                }
                Some("content_block_stop") => {
                    if !current_text.is_empty() {
                        content_blocks.push(serde_json::json!({
                            "type": "text",
                            "text": current_text.clone()
                        }));
                        current_text.clear();
                    }
                    if !current_tool_input.is_empty() {
                        if let Some(tool) = tool_uses.last_mut() {
                            tool.input = serde_json::from_str(&current_tool_input)
                                .unwrap_or(serde_json::Value::Null);
                            content_blocks.push(serde_json::json!({
                                "type": "tool_use",
                                "id": tool.id,
                                "name": tool.name,
                                "input": tool.input,
                            }));
                        }
                        current_tool_input.clear();
                    }
                }
                _ => {}
            }
        }
    }

    Ok((content_blocks, tool_uses))
}
