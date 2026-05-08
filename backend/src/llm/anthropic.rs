use async_trait::async_trait;
use futures_util::StreamExt;
use tauri::Emitter;

use super::{LlmError, ToolUse};
use crate::chats::Message;

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
                        self.current_tool_input.push_str(
                            delta["partial_json"].as_str().unwrap_or(""),
                        );
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

pub struct AnthropicClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
}

impl AnthropicClient {
    pub fn new(
        base_url: String,
        api_key: String,
        http: reqwest::Client,
    ) -> Self {
        Self {
            base_url,
            api_key,
            http,
        }
    }
}

fn messages_to_anthropic(messages: &[Message]) -> Vec<serde_json::Value> {
    messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": [{ "type": "text", "text": m.content }]
            })
        })
        .collect()
}

fn stream_body(
    messages: &[serde_json::Value],
    tools: &[serde_json::Value],
    model_id: &str,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "model": model_id,
        "max_tokens": 4096,
        "stream": true,
        "messages": messages,
    });
    if !tools.is_empty() {
        body["tools"] = serde_json::json!(tools);
    }
    body
}

fn title_body(messages: &[Message], model_id: &str) -> serde_json::Value {
    serde_json::json!({
        "model": model_id,
        "max_tokens": 64,
        "messages": [{
            "role": "user",
            "content": format!(
                "Summarize the following message in 5 to 10 words: {}",
                messages[0].content
            ),
        }]
    })
}

fn title_from_response(json: &serde_json::Value) -> String {
    json["content"][0]["text"]
        .as_str()
        .unwrap_or("New chat")
        .trim_matches('"')
        .trim()
        .to_string()
}

#[async_trait]
impl super::LlmClient for AnthropicClient {
    async fn stream(
        &self,
        app: &tauri::AppHandle,
        messages: &[Message],
        model_id: &str,
    ) -> Result<String, LlmError> {
        let json_messages = messages_to_anthropic(messages);
        let (blocks, _) =
            self.stream_raw(app, &json_messages, &[], model_id).await?;
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
        let body = title_body(messages, model_id);

        let response = self
            .http
            .post(format!("{}v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        Ok(title_from_response(&json))
    }

    async fn stream_raw(
        &self,
        app: &tauri::AppHandle,
        messages: &[serde_json::Value],
        tools: &[serde_json::Value],
        model_id: &str,
    ) -> Result<(Vec<serde_json::Value>, Vec<ToolUse>), LlmError> {
        let body = stream_body(messages, tools, model_id);

        let url = format!("{}v1/messages", self.base_url);
        let response = self
            .http
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(super::LlmError::Api(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_state_accumulates_text_blocks_and_returns_tokens() {
        let mut state = SseState::new();

        assert_eq!(
            state.process_event(&serde_json::json!({
                "type": "content_block_start",
                "content_block": { "type": "text" }
            })),
            None
        );
        assert_eq!(
            state.process_event(&serde_json::json!({
                "type": "content_block_delta",
                "delta": { "type": "text_delta", "text": "Hello" }
            })),
            Some("Hello".to_string())
        );
        assert_eq!(
            state.process_event(&serde_json::json!({
                "type": "content_block_delta",
                "delta": { "type": "text_delta", "text": " there" }
            })),
            Some(" there".to_string())
        );
        assert_eq!(
            state.process_event(
                &serde_json::json!({ "type": "content_block_stop" })
            ),
            None
        );

        assert_eq!(
            state.content_blocks,
            vec![serde_json::json!({ "type": "text", "text": "Hello there" })]
        );
        assert!(state.tool_uses.is_empty());
    }

    #[test]
    fn sse_state_accumulates_tool_use_blocks() {
        let mut state = SseState::new();

        state.process_event(&serde_json::json!({
            "type": "content_block_start",
            "content_block": {
                "type": "tool_use",
                "id": "toolu_1",
                "name": "search"
            }
        }));
        state.process_event(&serde_json::json!({
            "type": "content_block_delta",
            "delta": {
                "type": "input_json_delta",
                "partial_json": "{\"query\":"
            }
        }));
        state.process_event(&serde_json::json!({
            "type": "content_block_delta",
            "delta": {
                "type": "input_json_delta",
                "partial_json": "\"magnus\"}"
            }
        }));
        state.process_event(
            &serde_json::json!({ "type": "content_block_stop" }),
        );

        assert_eq!(state.tool_uses.len(), 1);
        assert_eq!(state.tool_uses[0].id, "toolu_1");
        assert_eq!(state.tool_uses[0].name, "search");
        assert_eq!(
            state.tool_uses[0].input,
            serde_json::json!({ "query": "magnus" })
        );
        assert_eq!(
            state.content_blocks,
            vec![serde_json::json!({
                "type": "tool_use",
                "id": "toolu_1",
                "name": "search",
                "input": { "query": "magnus" },
            })]
        );
    }

    #[test]
    fn sse_state_ignores_unknown_or_empty_delta_events() {
        let mut state = SseState::new();

        assert_eq!(
            state.process_event(&serde_json::json!({
                "type": "content_block_delta",
                "delta": { "type": "text_delta", "text": "" }
            })),
            None
        );
        assert_eq!(
            state.process_event(&serde_json::json!({
                "type": "content_block_delta",
                "delta": { "type": "other_delta" }
            })),
            None
        );
        assert_eq!(
            state.process_event(&serde_json::json!({ "type": "message_stop" })),
            None
        );
        assert!(state.content_blocks.is_empty());
        assert!(state.tool_uses.is_empty());
    }

    #[test]
    fn messages_to_anthropic_wraps_text_content_blocks() {
        let messages = vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            model_id: None,
        }];

        assert_eq!(
            messages_to_anthropic(&messages),
            vec![serde_json::json!({
                "role": "user",
                "content": [{ "type": "text", "text": "Hello" }]
            })]
        );
    }

    #[test]
    fn stream_body_includes_tools_only_when_present() {
        let messages = vec![serde_json::json!({
            "role": "user",
            "content": [{ "type": "text", "text": "Hello" }]
        })];
        let without_tools = stream_body(&messages, &[], "claude-sonnet-4-6");

        assert_eq!(without_tools["model"], "claude-sonnet-4-6");
        assert_eq!(without_tools["max_tokens"], 4096);
        assert_eq!(without_tools["stream"], true);
        assert!(without_tools.get("tools").is_none());

        let tools = vec![serde_json::json!({ "name": "search" })];
        let with_tools = stream_body(&messages, &tools, "claude-sonnet-4-6");
        assert_eq!(
            with_tools["tools"],
            serde_json::json!([{ "name": "search" }])
        );
    }

    #[test]
    fn title_body_and_title_response_use_expected_anthropic_shape() {
        let messages = vec![Message {
            role: "user".to_string(),
            content: "Explain coverage".to_string(),
            model_id: None,
        }];
        let body = title_body(&messages, "claude-sonnet-4-6");

        assert_eq!(body["model"], "claude-sonnet-4-6");
        assert_eq!(body["max_tokens"], 64);
        assert_eq!(body["messages"][0]["role"], "user");
        assert!(
            body["messages"][0]["content"]
                .as_str()
                .unwrap()
                .contains("Explain coverage")
        );
        assert_eq!(
            title_from_response(
                &serde_json::json!({ "content": [{ "text": "\"Coverage plan\"" }] })
            ),
            "Coverage plan"
        );
        assert_eq!(title_from_response(&serde_json::json!({})), "New chat");
    }
}
