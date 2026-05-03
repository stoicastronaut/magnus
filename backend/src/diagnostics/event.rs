use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::redaction::{redact_context, redact_message};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLevel {
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSource {
    Backend,
    Frontend,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticKind {
    AppLifecycle,
    CommandFailed,
    ClientError,
    Panic,
    DroppedEvents,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiagnosticContext(pub Map<String, Value>);

impl DiagnosticContext {
    pub fn dropped_events(session_id: &str, count: u64) -> Self {
        let mut raw = Map::new();
        raw.insert("session_id".into(), Value::String(session_id.into()));
        raw.insert("count".into(), Value::Number(count.into()));
        Self(
            redact_context(DiagnosticKind::DroppedEvents, Value::Object(raw))
                .as_object()
                .cloned()
                .unwrap_or_default(),
        )
    }

    pub fn command_failed(
        command_name: &str,
        session_id: Option<&str>,
        chat_id: Option<&str>,
        provider_id: Option<&str>,
        model_id: Option<&str>,
        error: &str,
    ) -> Self {
        let mut raw = Map::new();
        raw.insert("command_name".into(), Value::String(command_name.into()));
        if let Some(value) = session_id {
            raw.insert("session_id".into(), Value::String(value.into()));
        }
        if let Some(value) = chat_id {
            raw.insert("chat_id".into(), Value::String(value.into()));
        }
        if let Some(value) = provider_id {
            raw.insert("provider_id".into(), Value::String(value.into()));
        }
        if let Some(value) = model_id {
            raw.insert("model_id".into(), Value::String(value.into()));
        }
        raw.insert("error".into(), Value::String(error.into()));
        Self(
            redact_context(DiagnosticKind::CommandFailed, Value::Object(raw))
                .as_object()
                .cloned()
                .unwrap_or_default(),
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiagnosticEvent {
    pub timestamp: DateTime<Utc>,
    pub level: DiagnosticLevel,
    pub source: DiagnosticSource,
    pub kind: DiagnosticKind,
    pub message: String,
    pub context: DiagnosticContext,
}

impl DiagnosticEvent {
    pub fn new(
        level: DiagnosticLevel,
        source: DiagnosticSource,
        kind: DiagnosticKind,
        message: &str,
        context: DiagnosticContext,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            timestamp,
            level,
            source,
            kind,
            message: redact_message(message),
            context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn diagnostic_event_serializes_stable_shape() {
        let event = DiagnosticEvent::new(
            DiagnosticLevel::Error,
            DiagnosticSource::Backend,
            DiagnosticKind::CommandFailed,
            "stream_message failed",
            DiagnosticContext::command_failed(
                "stream_message",
                Some("session-1"),
                Some("chat-uuid"),
                Some("provider-uuid"),
                Some("claude-haiku-4-5-20251001"),
                "network_error",
            ),
            chrono::Utc.with_ymd_and_hms(2026, 5, 2, 12, 0, 0).unwrap(),
        );

        let value = serde_json::to_value(event).unwrap();

        assert_eq!(value["level"], "error");
        assert_eq!(value["source"], "backend");
        assert_eq!(value["kind"], "command_failed");
        assert_eq!(value["message"], "stream_message failed");
        assert_eq!(value["context"]["command_name"], "stream_message");
        assert_eq!(value["context"]["chat_id"], "chat-uuid");
        assert_eq!(value["context"]["provider_id"], "provider-uuid");
        assert_eq!(value["context"]["model_id"], "claude-haiku-4-5-20251001");
        assert!(value["context"].get("chat_name").is_none());
    }
}
