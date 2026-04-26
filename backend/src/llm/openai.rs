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

#[async_trait]
impl super::LlmClient for OpenAIClient {
    async fn stream(
        &self,
        app: &tauri::AppHandle,
        messages: &[Message],
        model_id: &str,
    ) -> Result<String, LlmError> {
        let json_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            .collect();

        let body = serde_json::json!({
            "model": model_id,
            "stream": true,
            "messages": json_messages,
        });

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
                if let Some(token) =
                    json["choices"][0]["delta"]["content"].as_str()
                {
                    if !token.is_empty() {
                        full_text.push_str(token);
                        app.emit("stream-token", token).unwrap();
                    }
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
        let json_messages: Vec<crate::chats::Message> = messages
            .iter()
            .filter_map(|m| {
                Some(crate::chats::Message {
                    role: m["role"].as_str()?.to_string(),
                    content: m["content"][0]["text"]
                        .as_str()
                        .or_else(|| m["content"].as_str())?
                        .to_string(),
                    model_id: None,
                })
            })
            .collect();
        let text = self.stream(app, &json_messages, model_id).await?;
        let block = serde_json::json!({"type": "text", "text": text});
        Ok((vec![block], vec![]))
    }

    async fn generate_title(
        &self,
        messages: &[Message],
        model_id: &str,
    ) -> Result<String, LlmError> {
        let body = serde_json::json!({
            "model": model_id,
            "messages": [{
                "role": "user",
                "content": format!(
                    "Summarize the following message in 5 to 10 words: {}",
                    messages[0].content
                ),
            }]
        });

        let response = self
            .http
            .post(format!("{}chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("New chat");
        Ok(text.trim_matches('"').trim().to_string())
    }
}
