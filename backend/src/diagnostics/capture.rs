use std::{any::Any, path::PathBuf};

use serde_json::{Map, Value};

use super::{
    DiagnosticContext, DiagnosticEvent, DiagnosticKind, DiagnosticLevel,
    DiagnosticSource, Diagnostics, append_crash_event, redact_context,
    redact_message,
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
        let payload = panic_payload_message(panic_info.payload());
        let location = panic_info
            .location()
            .map(|location| format!("{}:{}", location.file(), location.line()));
        let thread = std::thread::current().name().map(str::to_string);
        let event = panic_event(&session_id, payload, location, thread);
        let _ = append_crash_event(&app_data_dir, &event);
        previous_hook(panic_info);
    }));
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("panic occurred")
}

fn panic_event(
    session_id: &str,
    payload: &str,
    location: Option<String>,
    thread: Option<String>,
) -> DiagnosticEvent {
    let mut raw = Map::new();
    raw.insert("session_id".into(), Value::String(session_id.to_string()));
    if let Some(location) = location {
        raw.insert("location".into(), Value::String(location));
    }
    if let Some(thread) = thread {
        raw.insert("thread".into(), Value::String(thread));
    }
    let context = DiagnosticContext(
        redact_context(DiagnosticKind::Panic, Value::Object(raw))
            .as_object()
            .cloned()
            .unwrap_or_default(),
    );
    DiagnosticEvent::new(
        DiagnosticLevel::Fatal,
        DiagnosticSource::Backend,
        DiagnosticKind::Panic,
        payload,
        context,
        chrono::Utc::now(),
    )
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
    use crate::diagnostics::read_recent_diagnostics;

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
        assert!(
            !value["context"]["error"]
                .as_str()
                .unwrap()
                .contains("private prompt")
        );
    }

    #[tokio::test]
    async fn record_result_returns_ok_without_logging() {
        let dir = tempfile::tempdir().unwrap();
        let diagnostics =
            crate::diagnostics::start_diagnostics(dir.path().to_path_buf(), 8);

        let result = record_result(
            &diagnostics,
            "load_settings",
            None,
            None,
            None,
            Ok::<_, String>("settings"),
        );

        diagnostics.flush().await;
        assert_eq!(result.unwrap(), "settings");
        assert!(read_recent_diagnostics(dir.path(), 10).unwrap().is_empty());
    }

    #[tokio::test]
    async fn record_result_logs_error_and_returns_it() {
        let dir = tempfile::tempdir().unwrap();
        let diagnostics =
            crate::diagnostics::start_diagnostics(dir.path().to_path_buf(), 8);

        let result = record_result::<()>(
            &diagnostics,
            "stream_message",
            Some("chat-1"),
            Some("provider-1"),
            Some("model-1"),
            Err("HTTP 503 upstream unavailable with private payload"
                .to_string()),
        );

        diagnostics.flush().await;
        let events = read_recent_diagnostics(dir.path(), 10).unwrap();
        let event = events
            .iter()
            .find(|event| event.kind == DiagnosticKind::CommandFailed)
            .expect("command failed event");

        assert_eq!(
            result.unwrap_err(),
            "HTTP 503 upstream unavailable with private payload"
        );
        assert_eq!(event.message, "stream_message failed");
        assert_eq!(event.context.0["command_name"], "stream_message");
        assert_eq!(event.context.0["chat_id"], "chat-1");
        assert_eq!(event.context.0["provider_id"], "provider-1");
        assert_eq!(event.context.0["model_id"], "model-1");
        assert_eq!(event.context.0["error"], "HTTP 503: provider_api_error");
    }

    #[test]
    fn panic_payload_message_supports_common_payload_shapes() {
        let string_payload = String::from("owned panic");
        let unknown_payload = 42_u8;

        assert_eq!(panic_payload_message(&"borrowed panic"), "borrowed panic");
        assert_eq!(panic_payload_message(&string_payload), "owned panic");
        assert_eq!(panic_payload_message(&unknown_payload), "panic occurred");
    }

    #[test]
    fn panic_event_records_redacted_backend_panic_context() {
        let event = panic_event(
            "session-1",
            "panic with sk-abcdefghijklmnopqrstuvwxyz1234567890",
            Some("/Users/person/project/src/main.rs:12".to_string()),
            Some("main".to_string()),
        );
        let value = serde_json::to_value(event).unwrap();

        assert_eq!(value["level"], "fatal");
        assert_eq!(value["source"], "backend");
        assert_eq!(value["kind"], "panic");
        assert!(!value["message"].as_str().unwrap().contains("sk-"));
        assert_eq!(value["context"]["session_id"], "session-1");
        assert_eq!(value["context"]["thread"], "main");
        assert!(
            !value["context"]["location"]
                .as_str()
                .unwrap()
                .contains("/Users/person")
        );
    }
}
