use crate::config::{BuiltInId, Protocol, ProviderConfig, ProviderType};

#[derive(Clone, Debug, serde::Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
}

pub fn models_for(provider: BuiltInId) -> Vec<ModelInfo> {
    match provider {
        BuiltInId::Anthropic => vec![
            ModelInfo {
                id: "claude-haiku-4-5-20251001".into(),
                display_name: "Haiku 4.5".into(),
            },
            ModelInfo {
                id: "claude-sonnet-4-6".into(),
                display_name: "Sonnet 4.6".into(),
            },
            ModelInfo {
                id: "claude-opus-4-7".into(),
                display_name: "Opus 4.7".into(),
            },
        ],
        BuiltInId::OpenAI => vec![
            ModelInfo {
                id: "gpt-5".into(),
                display_name: "GPT-5".into(),
            },
            ModelInfo {
                id: "gpt-5-mini".into(),
                display_name: "GPT-5 mini".into(),
            },
            ModelInfo {
                id: "gpt-4o".into(),
                display_name: "GPT-4o".into(),
            },
        ],
        BuiltInId::Google => vec![
            ModelInfo {
                id: "gemini-2.5-pro".into(),
                display_name: "Gemini 2.5 Pro".into(),
            },
            ModelInfo {
                id: "gemini-2.5-flash".into(),
                display_name: "Gemini 2.5 Flash".into(),
            },
        ],
    }
}

pub fn models_for_provider(p: &ProviderConfig) -> Vec<ModelInfo> {
    match &p._type {
        ProviderType::BuiltIn { which } => models_for(*which),
        ProviderType::Custom { protocol, .. } => match protocol {
            Protocol::Anthropic => models_for(BuiltInId::Anthropic),
            Protocol::OpenAI => models_for(BuiltInId::OpenAI),
            Protocol::Google => models_for(BuiltInId::Google),
        },
    }
}
