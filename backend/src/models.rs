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

#[cfg(test)]
mod tests {
    use super::*;

    fn provider_with_type(_type: ProviderType) -> ProviderConfig {
        ProviderConfig {
            id: "provider".to_string(),
            display_name: "Provider".to_string(),
            _type,
        }
    }

    #[test]
    fn built_in_model_lists_keep_expected_ids_and_names() {
        let anthropic = models_for(BuiltInId::Anthropic);
        assert_eq!(anthropic[0].id, "claude-haiku-4-5-20251001");
        assert_eq!(anthropic[1].display_name, "Sonnet 4.6");

        let open_ai = models_for(BuiltInId::OpenAI);
        assert_eq!(open_ai[0].id, "gpt-5");
        assert_eq!(open_ai[1].display_name, "GPT-5 mini");

        let google = models_for(BuiltInId::Google);
        assert_eq!(google[0].id, "gemini-2.5-pro");
        assert_eq!(google[1].display_name, "Gemini 2.5 Flash");
    }

    #[test]
    fn custom_provider_models_follow_provider_protocol() {
        let anthropic = provider_with_type(ProviderType::Custom {
            protocol: Protocol::Anthropic,
            base_url: "https://proxy.example.com/".to_string(),
        });
        let open_ai = provider_with_type(ProviderType::Custom {
            protocol: Protocol::OpenAI,
            base_url: "https://proxy.example.com/v1/".to_string(),
        });
        let google = provider_with_type(ProviderType::Custom {
            protocol: Protocol::Google,
            base_url: "https://proxy.example.com/".to_string(),
        });

        assert_eq!(
            models_for_provider(&anthropic)[0].id,
            "claude-haiku-4-5-20251001"
        );
        assert_eq!(models_for_provider(&open_ai)[0].id, "gpt-5");
        assert_eq!(models_for_provider(&google)[0].id, "gemini-2.5-pro");
    }

    #[test]
    fn built_in_provider_models_follow_built_in_identity() {
        let provider = provider_with_type(ProviderType::BuiltIn {
            which: BuiltInId::OpenAI,
        });

        assert_eq!(models_for_provider(&provider)[0].display_name, "GPT-5");
    }
}
