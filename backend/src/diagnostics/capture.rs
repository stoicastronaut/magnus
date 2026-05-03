use std::path::PathBuf;

use serde_json::{Map, Value};

use super::{
    append_crash_event, redact_context, redact_message, DiagnosticContext,
    DiagnosticEvent, DiagnosticKind, DiagnosticLevel, DiagnosticSource,
    Diagnostics,
};

#[derive(serde::Deserialize)]
pub struct ClientEvent {
    pub level: DiagnosticLevel,
    pub kind: DiagnosticKind,
    pub message: String,
    pub context: Value,
}

pub fn command_error_event(
    command_name: &str,
    session_id: &str,
    chat_id: Option<&str>,
    provider_id: Option<&str>,
    model_id: Option<&str>,
    error: &str,
) -> DiagnosticEvent {
    DiagnosticEvent::new(
        DiagnosticLevel::Error,
        DiagnosticSource::Backend,
        DiagnosticKind::CommandFailed,
        &format!("{command_name} failed"),
        DiagnosticContext::command_failed(
            command_name,
            Some(session_id),
            chat_id,
            provider_id,
            model_id,
            &summarize_diagnostic_error(error),
        ),
        chrono::Utc::now(),
    )
}

pub fn log_command_error(
    diagnostics: &Diagnostics,
    command_name: &str,
    chat_id: Option<&str>,
    provider_id: Option<&str>,
    model_id: Option<&str>,
    error: &str,
) {
    diagnostics.log(command_error_event(
        command_name,
        diagnostics.session_id(),
        chat_id,
        provider_id,
        model_id,
        error,
    ));
}

pub fn record_result<T>(
    diagnostics: &Diagnostics,
    command_name: &str,
    chat_id: Option<&str>,
    provider_id: Option<&str>,
    model_id: Option<&str>,
    result: Result<T, String>,
) -> Result<T, String> {
    if let Err(error) = &result {
        log_command_error(
            diagnostics,
            command_name,
            chat_id,
            provider_id,
            model_id,
            error,
        );
    }
    result
}

pub fn install_panic_hook(app_data_dir: PathBuf, session_id: String) {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let payload = panic_info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| {
                panic_info
                    .payload()
                    .downcast_ref::<String>()
                    .map(String::as_str)
            })
            .unwrap_or("panic occurred");
        let mut raw = Map::new();
        raw.insert("session_id".into(), Value::String(session_id.clone()));
        if let Some(location) = panic_info
            .location()
            .map(|location| format!("{}:{}", location.file(), location.line()))
        {
            raw.insert("location".into(), Value::String(location));
        }
        if let Some(thread) = std::thread::current().name() {
            raw.insert("thread".into(), Value::String(thread.to_string()));
        }
        let context = DiagnosticContext(
            redact_context(DiagnosticKind::Panic, Value::Object(raw))
                .as_object()
                .cloned()
                .unwrap_or_default(),
        );
        let event = DiagnosticEvent::new(
            DiagnosticLevel::Fatal,
            DiagnosticSource::Backend,
            DiagnosticKind::Panic,
            payload,
            context,
            chrono::Utc::now(),
        );
        let _ = append_crash_event(&app_data_dir, &event);
        previous_hook(panic_info);
    }));
}

fn summarize_diagnostic_error(error: &str) -> String {
    if let Some(rest) = error.strip_prefix("HTTP ") {
        let status = rest
            .split(|ch: char| ch == ':' || ch.is_whitespace())
            .next()
            .unwrap_or("error");
        return format!("HTTP {status}: provider_api_error");
    }
    redact_message(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_error_event_records_command_without_private_context() {
        let event = command_error_event(
            "stream_message",
            "session-1",
            Some("chat-uuid"),
            Some("provider-uuid"),
            Some("gpt-5-mini"),
            "failed with token sk-abcdefghijklmnopqrstuvwxyz1234567890",
        );
        let value = serde_json::to_value(event).unwrap();

        assert_eq!(value["context"]["command_name"], "stream_message");
        assert_eq!(value["context"]["session_id"], "session-1");
        assert_eq!(value["context"]["chat_id"], "chat-uuid");
        assert_eq!(value["context"]["provider_id"], "provider-uuid");
        assert_eq!(value["context"]["model_id"], "gpt-5-mini");
        assert!(!value["context"]["error"].as_str().unwrap().contains("sk-"));
        assert!(value["context"].get("api_key").is_none());
        assert!(value["context"].get("provider").is_none());
    }

    #[test]
    fn command_error_event_summarizes_provider_response_bodies() {
        let event = command_error_event(
            "stream_message",
            "session-1",
            Some("chat-uuid"),
            Some("provider-uuid"),
            Some("gpt-5-mini"),
            "HTTP 400: echoed private prompt text",
        );
        let value = serde_json::to_value(event).unwrap();

        assert_eq!(value["context"]["error"], "HTTP 400: provider_api_error");
        assert!(!value["context"]["error"]
            .as_str()
            .unwrap()
            .contains("private prompt"));
    }
}
