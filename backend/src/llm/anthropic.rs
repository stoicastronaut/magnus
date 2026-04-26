use async_trait::async_trait;
use futures_util::StreamExt;
use tauri::Emitter;

use crate::chats::Message;
use super::{LlmError, ToolUse};

struct SseState {
    current_text: String,
    current_tool_input: String,
    content_blocks: Vec<serde_json::Value>,
    tool_uses: Vec<ToolUse>,
}

impl SseState {
    fn new() -> Self {
        Self {
            current_text: String::new(),
            current_tool_input: String::new(),
            content_blocks: Vec::new(),
            tool_uses: Vec::new(),
        }
    }

    fn process_event(&mut self, json: &serde_json::Value) -> Option<String> {
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
                        self.current_tool_input
                            .push_str(delta["partial_json"].as_str().unwrap_or(""));
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
                        tool.input = serde_json::from_str(&self.current_tool_input)
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

pub struct AnthropicClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
}

impl AnthropicClient {
    pub fn new(base_url: String, api_key: String, http: reqwest::Client) -> Self {
        Self { base_url, api_key, http }
    }
}

#[async_trait]
impl super::LlmClient for AnthropicClient {
    async fn stream(
        &self,
        app: &tauri::AppHandle,
        messages: &[Message],
        model_id: &str,
    ) -> Result<String, LlmError> {
        let json_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| serde_json::json!({"role": m.role, "content": [{"type": "text", "text": m.content}]}))
            .collect();
        let (blocks, _) = self.stream_raw(app, &json_messages, &[], model_id).await?;
        let text = blocks
            .iter()
            .filter(|b| b["type"] == "text")
            .map(|b| b["text"].as_str().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("");
        Ok(text)
    }

    async fn generate_title(
        &self,
        messages: &[Message],
        model_id: &str,
    ) -> Result<String, LlmError> {
        let body = serde_json::json!({
            "model": model_id,
            "max_tokens": 64,
            "messages": [{
                "role": "user",
                "content": format!(
                    "Summarize the following message in 5 to 10 words: {}",
                    messages[0].content
                ),
            }]
        });

        let response = self.http
            .post(format!("{}v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let text = json["content"][0]["text"].as_str().unwrap_or("New chat");
        Ok(text.trim_matches('"').trim().to_string())
    }

    async fn stream_raw(
        &self,
        app: &tauri::AppHandle,
        messages: &[serde_json::Value],
        tools: &[serde_json::Value],
        model_id: &str,
    ) -> Result<(Vec<serde_json::Value>, Vec<ToolUse>), LlmError> {
        let mut body = serde_json::json!({
            "model": model_id,
            "max_tokens": 4096,
            "stream": true,
            "messages": messages,
        });
        if !tools.is_empty() {
            body["tools"] = serde_json::json!(tools);
        }

        let response = self.http
            .post(format!("{}v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        let mut stream = response.bytes_stream();
        let mut state = SseState::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);
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
                if let Some(token) = state.process_event(&json) {
                    app.emit("stream-token", token).unwrap();
                }
            }
        }

        Ok((state.content_blocks, state.tool_uses))
    }
}
