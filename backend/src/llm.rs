use crate::chats::Message;
use futures_util::StreamExt;
use tauri::Emitter;

pub struct ToolUse {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

// ── SSE parsing ──────────────────────────────────────────────────────────────

pub struct SseState {
    pub current_text: String,
    pub current_tool_input: String,
    pub content_blocks: Vec<serde_json::Value>,
    pub tool_uses: Vec<ToolUse>,
}

impl SseState {
    pub fn new() -> Self {
        Self {
            current_text: String::new(),
            current_tool_input: String::new(),
            content_blocks: Vec::new(),
            tool_uses: Vec::new(),
        }
    }

    /// Process one parsed SSE event. Returns a text token to emit if any.
    pub fn process_event(
        &mut self,
        json: &serde_json::Value,
    ) -> Option<String> {
        match json["type"].as_str() {
            Some("content_block_start") => {
                self.current_text.clear();
                self.current_tool_input.clear();
                let block = &json["content_block"];
                if block["type"] == "tool_use" {
                    self.tool_uses.push(ToolUse {
                        id: block["id"].as_str().unwrap_or("").to_string(),
                        name: block["name"].as_str().unwrap_or("").to_string(),
                        input: serde_json::Value::Null,
                    });
                }
                None
            }
            Some("content_block_delta") => {
                let delta = &json["delta"];
                match delta["type"].as_str() {
                    Some("text_delta") => {
                        let token = delta["text"].as_str().unwrap_or("");
                        if !token.is_empty() {
                            self.current_text.push_str(token);
                            return Some(token.to_string());
                        }
                        None
                    }
                    Some("input_json_delta") => {
                        let partial =
                            delta["partial_json"].as_str().unwrap_or("");
                        self.current_tool_input.push_str(partial);
                        None
                    }
                    _ => None,
                }
            }
            Some("content_block_stop") => {
                if !self.current_text.is_empty() {
                    self.content_blocks.push(serde_json::json!({
                        "type": "text",
                        "text": self.current_text.clone()
                    }));
                    self.current_text.clear();
                }
                if !self.current_tool_input.is_empty() {
                    if let Some(tool) = self.tool_uses.last_mut() {
                        tool.input =
                            serde_json::from_str(&self.current_tool_input)
                                .unwrap_or(serde_json::Value::Null);
                        self.content_blocks.push(serde_json::json!({
                            "type": "tool_use",
                            "id": tool.id,
                            "name": tool.name,
                            "input": tool.input,
                        }));
                    }
                    self.current_tool_input.clear();
                }
                None
            }
            _ => None,
        }
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

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
    let mut state = SseState::new();

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
            let Ok(json) = serde_json::from_str::<serde_json::Value>(data)
            else {
                continue;
            };
            if let Some(token) = state.process_event(&json) {
                app.emit("stream-token", token).unwrap();
            }
        }
    }

    Ok((state.content_blocks, state.tool_uses))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn text_start() -> serde_json::Value {
        json!({"type": "content_block_start", "content_block": {"type": "text"}})
    }

    fn text_delta(text: &str) -> serde_json::Value {
        json!({"type": "content_block_delta", "delta": {"type": "text_delta", "text": text}})
    }

    fn input_delta(partial: &str) -> serde_json::Value {
        json!({"type": "content_block_delta", "delta": {"type": "input_json_delta", "partial_json": partial}})
    }

    fn block_stop() -> serde_json::Value {
        json!({"type": "content_block_stop"})
    }

    fn tool_start(id: &str, name: &str) -> serde_json::Value {
        json!({"type": "content_block_start", "content_block": {"type": "tool_use", "id": id, "name": name}})
    }

    #[test]
    fn test_text_block_accumulates_tokens() {
        let mut state = SseState::new();
        state.process_event(&text_start());
        let t1 = state.process_event(&text_delta("Hello"));
        let t2 = state.process_event(&text_delta(", world"));
        state.process_event(&block_stop());

        assert_eq!(t1, Some("Hello".to_string()));
        assert_eq!(t2, Some(", world".to_string()));
        assert_eq!(state.content_blocks.len(), 1);
        assert_eq!(state.content_blocks[0]["text"], "Hello, world");
    }

    #[test]
    fn test_empty_text_delta_emits_nothing() {
        let mut state = SseState::new();
        state.process_event(&text_start());
        let token = state.process_event(&text_delta(""));
        assert_eq!(token, None);
    }

    #[test]
    fn test_tool_use_block_parsed() {
        let mut state = SseState::new();
        state.process_event(&tool_start("tu_1", "search_repos"));
        state.process_event(&input_delta(r#"{"query":"#));
        state.process_event(&input_delta(r#""rust"}"#));
        state.process_event(&block_stop());

        assert_eq!(state.tool_uses.len(), 1);
        assert_eq!(state.tool_uses[0].id, "tu_1");
        assert_eq!(state.tool_uses[0].name, "search_repos");
        assert_eq!(state.tool_uses[0].input["query"], "rust");

        assert_eq!(state.content_blocks[0]["type"], "tool_use");
        assert_eq!(state.content_blocks[0]["name"], "search_repos");
    }

    #[test]
    fn test_multiple_blocks_in_sequence() {
        let mut state = SseState::new();

        state.process_event(&text_start());
        state.process_event(&text_delta("Before tool."));
        state.process_event(&block_stop());

        state.process_event(&tool_start("tu_2", "list_issues"));
        state.process_event(&input_delta(r#"{"repo":"magnus"}"#));
        state.process_event(&block_stop());

        assert_eq!(state.content_blocks.len(), 2);
        assert_eq!(state.content_blocks[0]["type"], "text");
        assert_eq!(state.content_blocks[1]["type"], "tool_use");
        assert_eq!(state.tool_uses.len(), 1);
    }

    #[test]
    fn test_invalid_tool_input_json_defaults_to_null() {
        let mut state = SseState::new();
        state.process_event(&tool_start("tu_3", "bad_tool"));
        state.process_event(&input_delta("not valid json{{"));
        state.process_event(&block_stop());

        assert!(state.tool_uses[0].input.is_null());
    }

    #[test]
    fn test_unknown_event_type_ignored() {
        let mut state = SseState::new();
        let result = state
            .process_event(&json!({"type": "message_start", "message": {}}));
        assert_eq!(result, None);
        assert!(state.content_blocks.is_empty());
    }
}
