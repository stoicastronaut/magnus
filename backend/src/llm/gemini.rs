use async_trait::async_trait;
use futures_util::StreamExt;
use tauri::Emitter;

use super::LlmError;
use crate::chats::Message;

pub struct GeminiClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
}

impl GeminiClient {
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

    fn to_gemini_role(role: &str) -> &str {
        if role == "assistant" { "model" } else { role }
    }
}

fn messages_to_gemini_contents(messages: &[Message]) -> Vec<serde_json::Value> {
    messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": GeminiClient::to_gemini_role(&m.role),
                "parts": [{"text": m.content}]
            })
        })
        .collect()
}

fn stream_body(messages: &[Message]) -> serde_json::Value {
    serde_json::json!({ "contents": messages_to_gemini_contents(messages) })
}

fn title_body(messages: &[Message]) -> serde_json::Value {
    serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": [{"text": format!(
                "Summarize the following message in 5 to 10 words: {}",
                messages[0].content
            )}]
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
    let json = serde_json::from_str::<serde_json::Value>(data).ok()?;
    let token =
        json["candidates"][0]["content"]["parts"][0]["text"].as_str()?;
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn title_from_response(json: &serde_json::Value) -> String {
    json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("New chat")
        .trim_matches('"')
        .trim()
        .to_string()
}

#[async_trait]
impl super::LlmClient for GeminiClient {
    async fn stream(
        &self,
        app: &tauri::AppHandle,
        messages: &[Message],
        model_id: &str,
    ) -> Result<String, LlmError> {
        let body = stream_body(messages);

        let response = self
            .http
            .post(format!(
                "{}models/{}:streamGenerateContent?alt=sse&key={}",
                self.base_url, model_id, self.api_key
            ))
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
        let chat_messages = raw_messages_to_chat_messages(messages);
        let text = self.stream(app, &chat_messages, model_id).await?;
        let block = serde_json::json!({"type": "text", "text": text});
        Ok((vec![block], vec![]))
    }

    async fn generate_title(
        &self,
        messages: &[Message],
        model_id: &str,
    ) -> Result<String, LlmError> {
        let body = title_body(messages);

        let response = self
            .http
            .post(format!(
                "{}models/{}:generateContent?key={}",
                self.base_url, model_id, self.api_key
            ))
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

    #[test]
    fn to_gemini_role_maps_assistant_to_model() {
        assert_eq!(GeminiClient::to_gemini_role("assistant"), "model");
        assert_eq!(GeminiClient::to_gemini_role("user"), "user");
    }

    #[test]
    fn messages_to_gemini_contents_maps_roles_and_text_parts() {
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                model_id: None,
            },
            Message {
                role: "assistant".to_string(),
                content: "Hi".to_string(),
                model_id: Some("gemini-2.5-pro".to_string()),
            },
        ];

        assert_eq!(
            messages_to_gemini_contents(&messages),
            vec![
                serde_json::json!({ "role": "user", "parts": [{ "text": "Hello" }] }),
                serde_json::json!({ "role": "model", "parts": [{ "text": "Hi" }] }),
            ]
        );
    }

    #[test]
    fn stream_body_uses_generate_content_shape() {
        let messages = vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            model_id: None,
        }];

        assert_eq!(
            stream_body(&messages),
            serde_json::json!({
                "contents": [{
                    "role": "user",
                    "parts": [{ "text": "Hello" }]
                }]
            })
        );
    }

    #[test]
    fn raw_messages_to_chat_messages_accepts_block_or_string_content() {
        let raw = vec![
            serde_json::json!({
                "role": "user",
                "content": [{ "type": "text", "text": "Block text" }]
            }),
            serde_json::json!({
                "role": "assistant",
                "content": "String text"
            }),
            serde_json::json!({
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
    fn token_from_sse_line_extracts_candidate_part_text() {
        assert_eq!(
            token_from_sse_line(
                "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Hello\"}]}}]}"
            ),
            Some("Hello".to_string())
        );
        assert_eq!(token_from_sse_line("event: ping"), None);
        assert_eq!(token_from_sse_line("data: not-json"), None);
        assert_eq!(
            token_from_sse_line(
                "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"\"}]}}]}"
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
        let body = title_body(&messages);

        assert_eq!(body["contents"][0]["role"], "user");
        assert!(
            body["contents"][0]["parts"][0]["text"]
                .as_str()
                .unwrap()
                .contains("Explain coverage")
        );
        assert_eq!(
            title_from_response(&serde_json::json!({
                "candidates": [{
                    "content": { "parts": [{ "text": "\"Gemini title\"" }] }
                }]
            })),
            "Gemini title"
        );
        assert_eq!(title_from_response(&serde_json::json!({})), "New chat");
    }
}
