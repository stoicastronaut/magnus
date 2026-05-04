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

#[async_trait]
impl super::LlmClient for GeminiClient {
    async fn stream(
        &self,
        app: &tauri::AppHandle,
        messages: &[Message],
        model_id: &str,
    ) -> Result<String, LlmError> {
        let contents: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": Self::to_gemini_role(&m.role),
                    "parts": [{"text": m.content}]
                })
            })
            .collect();

        let body = serde_json::json!({ "contents": contents });

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
                if !line.starts_with("data: ") {
                    continue;
                }
                let data = line.trim_start_matches("data: ");
                let Ok(json) = serde_json::from_str::<serde_json::Value>(data)
                else {
                    continue;
                };
                if let Some(token) = json["candidates"][0]["content"]["parts"]
                    [0]["text"]
                    .as_str()
                    && !token.is_empty()
                {
                    full_text.push_str(token);
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
        let chat_messages: Vec<Message> = messages
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
            .collect();
        let text = self.stream(app, &chat_messages, model_id).await?;
        let block = serde_json::json!({"type": "text", "text": text});
        Ok((vec![block], vec![]))
    }

    async fn generate_title(
        &self,
        messages: &[Message],
        model_id: &str,
    ) -> Result<String, LlmError> {
        let body = serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{"text": format!(
                    "Summarize the following message in 5 to 10 words: {}",
                    messages[0].content
                )}]
            }]
        });

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
        let text = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("New chat");
        Ok(text.trim_matches('"').trim().to_string())
    }
}
