mod anthropic;
mod gemini;
mod openai;

pub use anthropic::AnthropicClient;
pub use gemini::GeminiClient;
pub use openai::OpenAIClient;

use crate::chats::Message;
use crate::config::{BuiltInId, Protocol, ProviderConfig, ProviderType};

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("HTTP error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("API error: {0}")]
    Api(String),
}

pub struct ToolUse {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn stream(
        &self,
        app: &tauri::AppHandle,
        messages: &[Message],
        model_id: &str,
    ) -> Result<String, LlmError>;

    async fn generate_title(
        &self,
        messages: &[Message],
        model_id: &str,
    ) -> Result<String, LlmError>;

    /// Streams a response and returns content blocks + any tool calls.
    /// Non-Anthropic providers return an empty tool_uses vec.
    async fn stream_raw(
        &self,
        app: &tauri::AppHandle,
        messages: &[serde_json::Value],
        tools: &[serde_json::Value],
        model_id: &str,
    ) -> Result<(Vec<serde_json::Value>, Vec<ToolUse>), LlmError>;
}

pub fn client_for(provider: &ProviderConfig, api_key: String, http: reqwest::Client) -> Box<dyn LlmClient> {
    match &provider._type {
        ProviderType::BuiltIn { which: BuiltInId::Anthropic } => Box::new(
            AnthropicClient::new("https://api.anthropic.com/".into(), api_key, http),
        ),
        ProviderType::BuiltIn { which: BuiltInId::OpenAI } => Box::new(
            OpenAIClient::new("https://api.openai.com/v1/".into(), api_key, http),
        ),
        ProviderType::BuiltIn { which: BuiltInId::Google } => Box::new(
            GeminiClient::new(
                "https://generativelanguage.googleapis.com/v1beta/".into(),
                api_key,
                http,
            ),
        ),
        ProviderType::Custom { protocol: Protocol::Anthropic, base_url } => {
            Box::new(AnthropicClient::new(base_url.clone(), api_key, http))
        }
        ProviderType::Custom { protocol: Protocol::OpenAI, base_url } => {
            Box::new(OpenAIClient::new(base_url.clone(), api_key, http))
        }
        ProviderType::Custom { protocol: Protocol::Google, base_url } => {
            Box::new(GeminiClient::new(base_url.clone(), api_key, http))
        }
    }
}
