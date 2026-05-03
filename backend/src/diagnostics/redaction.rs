use regex::Regex;
use serde_json::{Map, Value};
use std::sync::OnceLock;

use super::event::DiagnosticKind;

fn secret_regex() -> &'static Regex {
    static SECRET_RE: OnceLock<Regex> = OnceLock::new();
    SECRET_RE.get_or_init(|| {
        Regex::new(
            r#"(?ix)
            \bsk-[A-Za-z0-9_\-]{20,}\b
            |
            \b(?:key|token|auth|secret)[=: ]+['\"]?[A-Za-z0-9+/=_\-]{24,}['\"]?
            "#,
        )
        .expect("valid secret regex")
    })
}

fn url_regex() -> &'static Regex {
    static URL_RE: OnceLock<Regex> = OnceLock::new();
    URL_RE.get_or_init(|| {
        Regex::new(r#"https?://[^\s)>\]"']+"#).expect("valid url regex")
    })
}

fn unix_path_regex() -> &'static Regex {
    static PATH_RE: OnceLock<Regex> = OnceLock::new();
    PATH_RE.get_or_init(|| {
        Regex::new(
            r#"(?i)(?:/Users|/home|/private/var|/var/folders)/[^\s)>\]"']+"#,
        )
        .expect("valid path regex")
    })
}

fn windows_path_regex() -> &'static Regex {
    static PATH_RE: OnceLock<Regex> = OnceLock::new();
    PATH_RE.get_or_init(|| {
        Regex::new(r#"(?i)[A-Z]:\\Users\\[^\s)>\]"']+"#)
            .expect("valid windows path regex")
    })
}

pub fn redact_message(message: &str) -> String {
    let value = secret_regex().replace_all(message, "[redacted-secret]");
    let value = url_regex().replace_all(&value, "[redacted-url]");
    let value = unix_path_regex().replace_all(&value, "[redacted-path]");
    let value = windows_path_regex().replace_all(&value, "[redacted-path]");
    value.into_owned()
}

pub fn redact_context(kind: DiagnosticKind, raw: Value) -> Value {
    let Some(raw) = raw.as_object() else {
        return Value::Object(Map::new());
    };

    let allowed = match kind {
        DiagnosticKind::CommandFailed => &[
            "command_name",
            "session_id",
            "chat_id",
            "provider_id",
            "model_id",
            "error",
            "error_kind",
            "status",
        ][..],
        DiagnosticKind::ClientError => &[
            "session_id",
            "chat_id",
            "provider_id",
            "model_id",
            "error",
            "error_kind",
        ][..],
        DiagnosticKind::AppLifecycle => &["session_id", "phase"][..],
        DiagnosticKind::Panic => &["session_id", "thread", "location"][..],
        DiagnosticKind::DroppedEvents => &["session_id", "count"][..],
    };

    let mut out = Map::new();
    for key in allowed {
        if let Some(value) = raw.get(*key) {
            out.insert((*key).to_string(), redact_value(value));
        }
    }
    Value::Object(out)
}

fn redact_value(value: &Value) -> Value {
    match value {
        Value::String(value) => Value::String(redact_message(value)),
        Value::Array(values) => {
            Value::Array(values.iter().map(redact_value).collect())
        }
        Value::Object(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), redact_value(value)))
                .collect(),
        ),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn redaction_drops_unknown_context_and_keeps_allowed_ids() {
        let context = redact_context(
            DiagnosticKind::CommandFailed,
            json!({
                "command_name": "stream_message",
                "session_id": "session-1",
                "chat_id": "chat-uuid",
                "provider_id": "provider-uuid",
                "model_id": "gpt-5-mini",
                "error": "request failed",
                "chat_name": "Personal budget",
                "prompt": "my private prompt",
                "tool_arguments": {"query": "secret"}
            }),
        );

        assert_eq!(context["command_name"], "stream_message");
        assert_eq!(context["session_id"], "session-1");
        assert_eq!(context["chat_id"], "chat-uuid");
        assert_eq!(context["provider_id"], "provider-uuid");
        assert_eq!(context["model_id"], "gpt-5-mini");
        assert!(context.get("chat_name").is_none());
        assert!(context.get("prompt").is_none());
        assert!(context.get("tool_arguments").is_none());
    }

    #[test]
    fn redaction_scrubs_secrets_urls_and_paths() {
        let scrubbed = redact_message(
            "token sk-abcdefghijklmnopqrstuvwxyz1234567890 at https://proxy.example.com/v1/messages and /Users/alice/secrets.txt",
        );

        assert!(!scrubbed.contains("sk-abcdefghijklmnopqrstuvwxyz1234567890"));
        assert!(!scrubbed.contains("https://proxy.example.com/v1/messages"));
        assert!(!scrubbed.contains("/Users/alice"));
        assert!(scrubbed.contains("[redacted-secret]"));
        assert!(scrubbed.contains("[redacted-url]"));
        assert!(scrubbed.contains("[redacted-path]"));
    }
}
