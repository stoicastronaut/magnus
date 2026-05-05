use chrono::{DateTime, NaiveDate, Utc};
use std::fs;
use std::path::{Path, PathBuf};

use super::writer::diagnostics_dir;

const BYTES_PER_MIB: u64 = 1024 * 1024;
pub const MAX_RETENTION_DAYS: i64 = 7;
pub const MAX_ACTIVE_LOG_MIB: u64 = 5;
pub const MAX_ACTIVE_LOG_BYTES: u64 = MAX_ACTIVE_LOG_MIB * BYTES_PER_MIB;

#[derive(Clone, Copy, Debug)]
pub struct RetentionPolicy {
    pub max_days: i64,
    pub max_bytes: u64,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_days: MAX_RETENTION_DAYS,
            max_bytes: MAX_ACTIVE_LOG_BYTES,
        }
    }
}

#[derive(Debug)]
struct LogFile {
    path: PathBuf,
    date: Option<NaiveDate>,
    size: u64,
}

pub fn prune_old_diagnostics(
    app_data_dir: &Path,
    now: DateTime<Utc>,
    policy: RetentionPolicy,
) -> Result<(), String> {
    let diagnostics_dir = diagnostics_dir(app_data_dir);
    if !diagnostics_dir.exists() {
        return Ok(());
    }

    let mut files = collect_log_files(&diagnostics_dir)?;
    let min_date = now.date_naive() - chrono::Duration::days(policy.max_days);

    for file in &files {
        if file.date.is_some_and(|date| date < min_date) {
            let _ = fs::remove_file(&file.path);
        }
    }

    files = collect_log_files(&diagnostics_dir)?;
    files.sort_by(|a, b| {
        a.date
            .unwrap_or(NaiveDate::MAX)
            .cmp(&b.date.unwrap_or(NaiveDate::MAX))
            .then_with(|| a.path.cmp(&b.path))
    });

    let mut total: u64 = files.iter().map(|file| file.size).sum();
    for file in files {
        if total <= policy.max_bytes {
            break;
        }
        fs::remove_file(&file.path).map_err(|e| e.to_string())?;
        total = total.saturating_sub(file.size);
    }

    Ok(())
}

fn collect_log_files(diagnostics_dir: &Path) -> Result<Vec<LogFile>, String> {
    let mut files = Vec::new();
    for entry in fs::read_dir(diagnostics_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        let date = event_file_date(&file_name);
        if date.is_none() && file_name != "crashes.jsonl" {
            continue;
        }
        let size = entry.metadata().map_err(|e| e.to_string())?.len();
        files.push(LogFile { path, date, size });
    }
    Ok(files)
}

fn event_file_date(file_name: &str) -> Option<NaiveDate> {
    let date = file_name.strip_prefix("events-")?.strip_suffix(".jsonl")?;
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::writer::{diagnostics_dir, events_file_for_date};
    use chrono::TimeZone;

    #[test]
    fn retention_prunes_oldest_events_by_age_and_size() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(diagnostics_dir(dir.path())).unwrap();
        fs::write(
            events_file_for_date(
                dir.path(),
                NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
            ),
            "old\n",
        )
        .unwrap();
        fs::write(
            events_file_for_date(
                dir.path(),
                NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            ),
            "large-newer\n",
        )
        .unwrap();
        fs::write(
            events_file_for_date(
                dir.path(),
                NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
            ),
            "large-newest\n",
        )
        .unwrap();

        prune_old_diagnostics(
            dir.path(),
            chrono::Utc.with_ymd_and_hms(2026, 5, 2, 12, 0, 0).unwrap(),
            RetentionPolicy {
                max_days: 7,
                max_bytes: 13,
            },
        )
        .unwrap();

        assert!(
            !events_file_for_date(
                dir.path(),
                NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
            )
            .exists()
        );
        assert!(
            !events_file_for_date(
                dir.path(),
                NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            )
            .exists()
        );
        assert!(
            events_file_for_date(
                dir.path(),
                NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
            )
            .exists()
        );
    }
}
