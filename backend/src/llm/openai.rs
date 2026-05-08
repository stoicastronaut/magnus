use async_trait::async_trait;
use futures_util::StreamExt;
use tauri::Emitter;

use super::LlmError;
use crate::chats::Message;

pub struct OpenAIClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
}

impl OpenAIClient {
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

fn messages_to_openai(messages: &[Message]) -> Vec<serde_json::Value> {
    messages
        .iter()
        .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
        .collect()
}

fn stream_body(messages: &[Message], model_id: &str) -> serde_json::Value {
    serde_json::json!({
        "model": model_id,
        "stream": true,
        "messages": messages_to_openai(messages),
    })
}

fn title_body(messages: &[Message], model_id: &str) -> serde_json::Value {
    serde_json::json!({
        "model": model_id,
        "messages": [{
            "role": "user",
            "content": format!(
                "Summarize the following message in 5 to 10 words: {}",
                messages[0].content
            ),
        }]
    })
}

fn raw_messages_to_chat_messages(
    messages: &[serde_json::Value],
) -> Vec<Message> {
    messages
        .iter()
        .filter_map(|m| {
            Some(Message {
                role: m["role"].as_str()?.to_string(),
                content: m["content"][0]["text"]
                    .as_str()
                    .or_else(|| m["content"].as_str())?
                    .to_string(),
                model_id: None,
            })
        })
        .collect()
}

fn token_from_sse_line(line: &str) -> Option<String> {
    if !line.starts_with("data: ") {
        return None;
    }
    let data = line.trim_start_matches("data: ");
    if data == "[DONE]" {
        return None;
    }
    let json = serde_json::from_str::<serde_json::Value>(data).ok()?;
    let token = json["choices"][0]["delta"]["content"].as_str()?;
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn title_from_response(json: &serde_json::Value) -> String {
    json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("New chat")
        .trim_matches('"')
        .trim()
        .to_string()
}

#[async_trait]
impl super::LlmClient for OpenAIClient {
    async fn stream(
        &self,
        app: &tauri::AppHandle,
        messages: &[Message],
        model_id: &str,
    ) -> Result<String, LlmError> {
        let body = stream_body(messages, model_id);

        let response = self
            .http
            .post(format!("{}chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        let mut stream = response.bytes_stream();
        let mut full_text = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                if let Some(token) = token_from_sse_line(line) {
                    full_text.push_str(&token);
                    app.emit("stream-token", token).unwrap();
                }
            }
        }

        Ok(full_text)
    }

    async fn stream_raw(
        &self,
        app: &tauri::AppHandle,
        messages: &[serde_json::Value],
        _tools: &[serde_json::Value],
        model_id: &str,
    ) -> Result<(Vec<serde_json::Value>, Vec<super::ToolUse>), LlmError> {
        let json_messages = raw_messages_to_chat_messages(messages);
        let text = self.stream(app, &json_messages, model_id).await?;
        let block = serde_json::json!({"type": "text", "text": text});
        Ok((vec![block], vec![]))
    }

    async fn generate_title(
        &self,
        messages: &[Message],
        model_id: &str,
    ) -> Result<String, LlmError> {
        let body = title_body(messages, model_id);

        let response = self
            .http
            .post(format!("{}chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        Ok(title_from_response(&json))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn messages_to_openai_preserves_roles_and_text_content() {
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                model_id: None,
            },
            Message {
                role: "assistant".to_string(),
                content: "Hi".to_string(),
                model_id: Some("gpt-5".to_string()),
            },
        ];

        assert_eq!(
            messages_to_openai(&messages),
            vec![
                json!({ "role": "user", "content": "Hello" }),
                json!({ "role": "assistant", "content": "Hi" }),
            ]
        );
    }

    #[test]
    fn stream_body_uses_chat_completion_stream_shape() {
        let messages = vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            model_id: None,
        }];

        assert_eq!(
            stream_body(&messages, "gpt-5"),
            json!({
                "model": "gpt-5",
                "stream": true,
                "messages": [{ "role": "user", "content": "Hello" }],
            })
        );
    }

    #[test]
    fn raw_messages_to_chat_messages_accepts_block_or_string_content() {
        let raw = vec![
            json!({
                "role": "user",
                "content": [{ "type": "text", "text": "Block text" }]
            }),
            json!({
                "role": "assistant",
                "content": "String text"
            }),
            json!({
                "role": "user",
                "content": [{ "type": "image" }]
            }),
        ];

        let messages = raw_messages_to_chat_messages(&raw);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Block text");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[1].content, "String text");
    }

    #[test]
    fn token_from_sse_line_extracts_non_empty_delta_content() {
        assert_eq!(
            token_from_sse_line(
                "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}"
            ),
            Some("Hello".to_string())
        );
        assert_eq!(token_from_sse_line("data: [DONE]"), None);
        assert_eq!(token_from_sse_line("event: ping"), None);
        assert_eq!(token_from_sse_line("data: not-json"), None);
        assert_eq!(
            token_from_sse_line(
                "data: {\"choices\":[{\"delta\":{\"content\":\"\"}}]}"
            ),
            None
        );
    }

    #[test]
    fn title_from_response_trims_quotes_and_falls_back() {
        let messages = vec![Message {
            role: "user".to_string(),
            content: "Explain coverage".to_string(),
            model_id: None,
        }];
        let body = title_body(&messages, "gpt-5");

        assert_eq!(body["model"], "gpt-5");
        assert_eq!(body["messages"][0]["role"], "user");
        assert!(
            body["messages"][0]["content"]
                .as_str()
                .unwrap()
                .contains("Explain coverage")
        );
        assert_eq!(
            title_from_response(&json!({
                "choices": [{ "message": { "content": "\"Short title\"" } }]
            })),
            "Short title"
        );
        assert_eq!(title_from_response(&json!({})), "New chat");
    }
}
