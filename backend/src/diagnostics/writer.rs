use chrono::{NaiveDate, Utc};
use std::cmp::Reverse;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use super::event::{
    DiagnosticContext, DiagnosticEvent, DiagnosticKind, DiagnosticLevel,
    DiagnosticSource,
};
use super::retention::{prune_old_diagnostics, RetentionPolicy};

pub const MAX_RECENT_DIAGNOSTICS: u32 = 200;
const RETENTION_SWEEP_INTERVAL: Duration = Duration::from_secs(60 * 60);
const FLUSH_TIMEOUT: Duration = Duration::from_millis(500);

enum DiagnosticMessage {
    Event(DiagnosticEvent),
    Flush(oneshot::Sender<()>),
}

pub fn diagnostics_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("diagnostics")
}

pub fn events_file_for_date(app_data_dir: &Path, date: NaiveDate) -> PathBuf {
    diagnostics_dir(app_data_dir)
        .join(format!("events-{}.jsonl", date.format("%Y-%m-%d")))
}

pub fn append_event(
    app_data_dir: &Path,
    event: &DiagnosticEvent,
) -> Result<(), String> {
    append_jsonl(
        events_file_for_date(app_data_dir, event.timestamp.date_naive()),
        event,
    )
}

pub fn append_crash_event(
    app_data_dir: &Path,
    event: &DiagnosticEvent,
) -> Result<(), String> {
    append_jsonl(diagnostics_dir(app_data_dir).join("crashes.jsonl"), event)
}

pub fn read_recent_diagnostics(
    app_data_dir: &Path,
    limit: u32,
) -> Result<Vec<DiagnosticEvent>, String> {
    let mut events = read_all_diagnostics(app_data_dir)?;
    events.retain(|event| {
        matches!(
            event.level,
            DiagnosticLevel::Warn
                | DiagnosticLevel::Error
                | DiagnosticLevel::Fatal
        )
    });
    events.sort_by_key(|event| Reverse(event.timestamp));
    events.truncate(limit.min(MAX_RECENT_DIAGNOSTICS) as usize);
    Ok(events)
}

fn append_jsonl(path: PathBuf, event: &DiagnosticEvent) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut file, event).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())
}

pub(crate) fn read_all_diagnostics(
    app_data_dir: &Path,
) -> Result<Vec<DiagnosticEvent>, String> {
    let dir = diagnostics_dir(app_data_dir);
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut events = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !(file_name.starts_with("events-") || file_name == "crashes.jsonl") {
            continue;
        }
        let contents =
            fs::read_to_string(entry.path()).map_err(|e| e.to_string())?;
        for line in contents.lines().filter(|line| !line.trim().is_empty()) {
            if let Ok(event) = serde_json::from_str::<DiagnosticEvent>(line) {
                events.push(event);
            }
        }
    }
    Ok(events)
}

#[derive(Clone)]
pub struct Diagnostics {
    app_data_dir: PathBuf,
    session_id: String,
    sender: mpsc::Sender<DiagnosticMessage>,
    dropped: Arc<AtomicU64>,
}

impl Diagnostics {
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn app_data_dir(&self) -> &Path {
        &self.app_data_dir
    }

    pub fn log(&self, event: DiagnosticEvent) {
        if self
            .sender
            .try_send(DiagnosticMessage::Event(event))
            .is_err()
        {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub async fn flush(&self) {
        let (done, done_rx) = oneshot::channel();
        let sent = tokio::time::timeout(
            FLUSH_TIMEOUT,
            self.sender.send(DiagnosticMessage::Flush(done)),
        )
        .await;

        if matches!(sent, Ok(Ok(()))) {
            let _ = tokio::time::timeout(FLUSH_TIMEOUT, done_rx).await;
        } else {
            let _ = append_dropped_event(
                &self.app_data_dir,
                &self.session_id,
                &self.dropped,
            );
        }
    }

    #[cfg(test)]
    pub fn dropped_count(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub fn increment_dropped_for_test(&self, count: u64) {
        self.dropped.fetch_add(count, Ordering::Relaxed);
    }
}

pub fn start_diagnostics(
    app_data_dir: PathBuf,
    capacity: usize,
) -> Diagnostics {
    let (sender, mut receiver) = mpsc::channel::<DiagnosticMessage>(capacity);
    let dropped = Arc::new(AtomicU64::new(0));
    let writer_dir = app_data_dir.clone();
    let writer_session_id = Uuid::new_v4().to_string();
    let diagnostics_session_id = writer_session_id.clone();
    let writer_dropped = Arc::clone(&dropped);
    let retention_dir = app_data_dir.clone();

    tauri::async_runtime::spawn(async move {
        while let Some(message) = receiver.recv().await {
            match message {
                DiagnosticMessage::Event(event) => {
                    if append_event(&writer_dir, &event).is_ok() {
                        let _ = append_dropped_event(
                            &writer_dir,
                            &writer_session_id,
                            &writer_dropped,
                        );
                    }
                }
                DiagnosticMessage::Flush(done) => {
                    let _ = append_dropped_event(
                        &writer_dir,
                        &writer_session_id,
                        &writer_dropped,
                    );
                    let _ = done.send(());
                }
            }
        }
    });

    tauri::async_runtime::spawn(async move {
        let _ = prune_old_diagnostics(
            &retention_dir,
            Utc::now(),
            RetentionPolicy::default(),
        );
        let mut interval = tokio::time::interval(RETENTION_SWEEP_INTERVAL);
        loop {
            interval.tick().await;
            let _ = prune_old_diagnostics(
                &retention_dir,
                Utc::now(),
                RetentionPolicy::default(),
            );
        }
    });

    Diagnostics {
        app_data_dir,
        session_id: diagnostics_session_id,
        sender,
        dropped,
    }
}

fn append_dropped_event(
    app_data_dir: &Path,
    session_id: &str,
    dropped: &AtomicU64,
) -> Result<(), String> {
    let count = dropped.swap(0, Ordering::Relaxed);
    if count == 0 {
        return Ok(());
    }

    append_event(
        app_data_dir,
        &DiagnosticEvent::new(
            DiagnosticLevel::Warn,
            DiagnosticSource::Backend,
            DiagnosticKind::DroppedEvents,
            "Diagnostics events were dropped",
            DiagnosticContext::dropped_events(session_id, count),
            Utc::now(),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::fs;

    fn empty_context() -> DiagnosticContext {
        DiagnosticContext(serde_json::Map::new())
    }

    #[test]
    fn diagnostics_dir_lives_under_app_data_dir() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(diagnostics_dir(dir.path()), dir.path().join("diagnostics"));
    }

    #[test]
    fn diagnostics_start_does_not_require_current_tokio_reactor() {
        let dir = tempfile::tempdir().unwrap();
        let diagnostics = start_diagnostics(dir.path().to_path_buf(), 8);

        assert!(!diagnostics.session_id().is_empty());
    }

    #[test]
    fn writer_appends_events_as_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let event = DiagnosticEvent::new(
            DiagnosticLevel::Warn,
            DiagnosticSource::Backend,
            DiagnosticKind::AppLifecycle,
            "settings load failed",
            empty_context(),
            chrono::Utc.with_ymd_and_hms(2026, 5, 2, 8, 0, 0).unwrap(),
        );

        append_event(dir.path(), &event).unwrap();

        let path = events_file_for_date(
            dir.path(),
            NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
        );
        let contents = fs::read_to_string(path).unwrap();
        let lines: Vec<_> = contents.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(lines[0]).unwrap()
                ["message"],
            "settings load failed"
        );
    }

    #[test]
    fn recent_diagnostics_excludes_info_and_caps_limit() {
        let dir = tempfile::tempdir().unwrap();
        for index in 0..205 {
            let level = if index == 0 {
                DiagnosticLevel::Info
            } else {
                DiagnosticLevel::Error
            };
            let event = DiagnosticEvent::new(
                level,
                DiagnosticSource::Backend,
                DiagnosticKind::CommandFailed,
                &format!("event {index}"),
                empty_context(),
                chrono::Utc.with_ymd_and_hms(2026, 5, 2, 8, 0, 0).unwrap()
                    + chrono::Duration::seconds(index),
            );
            append_event(dir.path(), &event).unwrap();
        }

        let recent = read_recent_diagnostics(dir.path(), 500).unwrap();

        assert_eq!(recent.len(), 200);
        assert_eq!(recent[0].message, "event 204");
        assert!(recent
            .iter()
            .all(|event| event.level != DiagnosticLevel::Info));
    }

    #[test]
    fn crash_events_append_to_crashes_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let event = DiagnosticEvent::new(
            DiagnosticLevel::Fatal,
            DiagnosticSource::Backend,
            DiagnosticKind::Panic,
            "panic captured",
            empty_context(),
            chrono::Utc.with_ymd_and_hms(2026, 5, 2, 8, 0, 0).unwrap(),
        );

        append_crash_event(dir.path(), &event).unwrap();

        let contents = fs::read_to_string(
            diagnostics_dir(dir.path()).join("crashes.jsonl"),
        )
        .unwrap();
        assert!(contents.contains("panic captured"));
    }

    #[tokio::test]
    async fn async_diagnostics_persists_dropped_event_count() {
        let dir = tempfile::tempdir().unwrap();
        let diagnostics = start_diagnostics(dir.path().to_path_buf(), 1);

        for index in 0..50 {
            diagnostics.log(DiagnosticEvent::new(
                DiagnosticLevel::Error,
                DiagnosticSource::Backend,
                DiagnosticKind::CommandFailed,
                &format!("queued {index}"),
                empty_context(),
                chrono::Utc.with_ymd_and_hms(2026, 5, 2, 8, 0, 0).unwrap(),
            ));
        }

        diagnostics.flush().await;
        let recent = read_recent_diagnostics(dir.path(), 200).unwrap();
        let dropped = recent
            .iter()
            .find(|event| event.kind == DiagnosticKind::DroppedEvents)
            .expect("dropped-event diagnostic");

        assert_eq!(dropped.level, DiagnosticLevel::Warn);
        assert!(dropped.context.0["count"].as_u64().unwrap() > 0);
        assert_eq!(diagnostics.dropped_count(), 0);
    }

    #[tokio::test]
    async fn flush_writes_queued_events_before_returning() {
        let dir = tempfile::tempdir().unwrap();
        let diagnostics = start_diagnostics(dir.path().to_path_buf(), 8);

        diagnostics.log(DiagnosticEvent::new(
            DiagnosticLevel::Error,
            DiagnosticSource::Backend,
            DiagnosticKind::CommandFailed,
            "queued before shutdown",
            empty_context(),
            chrono::Utc.with_ymd_and_hms(2026, 5, 2, 8, 0, 0).unwrap(),
        ));

        diagnostics.flush().await;
        let recent = read_recent_diagnostics(dir.path(), 10).unwrap();

        assert!(recent
            .iter()
            .any(|event| event.message == "queued before shutdown"));
    }

    #[tokio::test]
    async fn flush_writes_dropped_event_without_later_user_event() {
        let dir = tempfile::tempdir().unwrap();
        let diagnostics = start_diagnostics(dir.path().to_path_buf(), 8);

        diagnostics.increment_dropped_for_test(3);
        diagnostics.flush().await;
        let recent = read_recent_diagnostics(dir.path(), 10).unwrap();
        let dropped = recent
            .iter()
            .find(|event| event.kind == DiagnosticKind::DroppedEvents)
            .expect("dropped-event diagnostic");

        assert_eq!(dropped.context.0["count"], 3);
    }
}
