use crate::chats::Chat;
use crate::config::{ProviderType, Settings};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::Cursor;
use std::path::Path;

use super::writer::{
    diagnostics_dir, read_all_diagnostics, read_recent_diagnostics,
    MAX_RECENT_DIAGNOSTICS,
};

pub const MAX_EXPORT_UNCOMPRESSED_BYTES: u64 = 10 * 1024 * 1024;
type ExportFile = (String, Vec<u8>);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOptions {
    pub include_full_endpoint_url: bool,
    pub include_active_chat_transcript: bool,
    pub active_chat_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Included {
    pub full_endpoint_url: bool,
    pub active_chat_transcript: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExportResult {
    pub path: String,
    pub summary: String,
    pub included: Included,
}

pub fn export_diagnostics_bundle(
    app_data_dir: &Path,
    session_id: &str,
    app_version: &str,
    build_sha: Option<&str>,
    options: ExportOptions,
) -> Result<ExportResult, String> {
    let summary = diagnostics_summary(app_data_dir, session_id, app_version)?;
    let (mut files, included) = build_export_files(
        app_data_dir,
        session_id,
        app_version,
        build_sha,
        &options,
        &summary,
    )?;
    files.sort_by(|a, b| a.0.cmp(&b.0));

    enforce_export_size_limit(&files)?;
    let path = write_export_archive(app_data_dir, app_version, &files)?;

    Ok(ExportResult {
        path: path.to_string_lossy().to_string(),
        summary,
        included,
    })
}

fn build_export_files(
    app_data_dir: &Path,
    session_id: &str,
    app_version: &str,
    build_sha: Option<&str>,
    options: &ExportOptions,
    summary: &str,
) -> Result<(Vec<ExportFile>, Included), String> {
    let chat_transcript_file =
        optional_chat_transcript_file(app_data_dir, options)?;
    let included = Included {
        full_endpoint_url: options.include_full_endpoint_url,
        active_chat_transcript: chat_transcript_file.is_some(),
    };
    let mut files = base_export_files(
        app_data_dir,
        session_id,
        app_version,
        build_sha,
        options.include_full_endpoint_url,
        &included,
        summary,
    )?;
    if let Some(file) = chat_transcript_file {
        files.push(file);
    }
    Ok((files, included))
}

fn base_export_files(
    app_data_dir: &Path,
    session_id: &str,
    app_version: &str,
    build_sha: Option<&str>,
    include_full_endpoint_url: bool,
    included: &Included,
    summary: &str,
) -> Result<Vec<ExportFile>, String> {
    Ok(vec![
        ("summary.md".to_string(), summary.as_bytes().to_vec()),
        (
            "environment.json".to_string(),
            serde_json::to_vec_pretty(&environment_json(
                app_data_dir,
                session_id,
                app_version,
                build_sha,
                include_full_endpoint_url,
            )?)
            .map_err(|e| e.to_string())?,
        ),
        (
            "events.jsonl".to_string(),
            active_events_jsonl(app_data_dir)?,
        ),
        ("crashes.jsonl".to_string(), crashes_jsonl(app_data_dir)?),
        (
            "recent-errors.json".to_string(),
            serde_json::to_vec_pretty(&read_recent_diagnostics(
                app_data_dir,
                MAX_RECENT_DIAGNOSTICS,
            )?)
            .map_err(|e| e.to_string())?,
        ),
        (
            "included.json".to_string(),
            serde_json::to_vec_pretty(included).map_err(|e| e.to_string())?,
        ),
    ])
}

fn optional_chat_transcript_file(
    app_data_dir: &Path,
    options: &ExportOptions,
) -> Result<Option<ExportFile>, String> {
    if !options.include_active_chat_transcript {
        return Ok(None);
    }
    let Some(chat_id) = options.active_chat_id.as_deref() else {
        return Ok(None);
    };
    let Some(chat) = load_chat_by_id(app_data_dir, chat_id)? else {
        return Ok(None);
    };
    Ok(Some((
        "active-chat.json".to_string(),
        serde_json::to_vec_pretty(&active_chat_json(chat))
            .map_err(|e| e.to_string())?,
    )))
}

fn active_chat_json(chat: Chat) -> serde_json::Value {
    json!({
        "id": chat.id,
        "provider_id": chat.provider_id,
        "created_at": chat.created_at,
        "messages": chat.messages,
    })
}

fn enforce_export_size_limit(files: &[ExportFile]) -> Result<(), String> {
    let uncompressed_size: u64 =
        files.iter().map(|(_, bytes)| bytes.len() as u64).sum();
    if uncompressed_size > MAX_EXPORT_UNCOMPRESSED_BYTES {
        return Err(format!(
            "Diagnostics export is {uncompressed_size} bytes, above the 10 MB limit. Untick optional extras and try again."
        ));
    }
    Ok(())
}

fn write_export_archive(
    app_data_dir: &Path,
    app_version: &str,
    files: &[ExportFile],
) -> Result<std::path::PathBuf, String> {
    let exports_dir = diagnostics_dir(app_data_dir).join("exports");
    fs::create_dir_all(&exports_dir).map_err(|e| e.to_string())?;
    let path = exports_dir.join(export_filename(app_version));
    write_tar_gz(&path, files)?;
    Ok(path)
}

fn export_filename(app_version: &str) -> String {
    format!(
        "magnus-diagnostics-{}-{}.tar.gz",
        app_version,
        chrono::Utc::now().format("%Y%m%dT%H%M%SZ")
    )
}

pub fn validate_reveal_path(
    app_data_dir: &Path,
    requested_path: &Path,
) -> Result<(), String> {
    let diagnostics_dir = diagnostics_dir(app_data_dir);
    fs::create_dir_all(diagnostics_dir.join("exports"))
        .map_err(|e| e.to_string())?;
    let requested = requested_path.canonicalize().map_err(|e| e.to_string())?;
    let diagnostics =
        diagnostics_dir.canonicalize().map_err(|e| e.to_string())?;
    let exports = diagnostics.join("exports");
    if requested.starts_with(&diagnostics) || requested.starts_with(&exports) {
        Ok(())
    } else {
        Err("Refusing to reveal a path outside diagnostics exports.".into())
    }
}

pub fn diagnostics_summary(
    app_data_dir: &Path,
    session_id: &str,
    app_version: &str,
) -> Result<String, String> {
    let recent = read_recent_diagnostics(app_data_dir, 10)?;
    let mut summary = format!(
        "# Magnus Diagnostics\n\n- App version: {app_version}\n- OS: {} {}\n- Arch: {}\n- Session: {session_id}\n\n## Recent Errors\n",
        std::env::consts::OS,
        std::env::consts::FAMILY,
        std::env::consts::ARCH
    );
    if recent.is_empty() {
        summary.push_str("- None recorded\n");
    } else {
        for event in recent {
            summary.push_str(&format!(
                "- {} {:?}: {}\n",
                event.timestamp, event.level, event.message
            ));
        }
    }
    Ok(summary)
}

fn environment_json(
    app_data_dir: &Path,
    session_id: &str,
    app_version: &str,
    build_sha: Option<&str>,
    include_full_endpoint_url: bool,
) -> Result<serde_json::Value, String> {
    let settings = Settings::load(app_data_dir).unwrap_or(Settings {
        default_provider_id: None,
        providers: vec![],
    });
    let providers: Vec<_> = settings
        .providers
        .iter()
        .map(|provider| match &provider._type {
            ProviderType::BuiltIn { which } => json!({
                "id": provider.id,
                "kind": "built_in",
                "which": which,
            }),
            ProviderType::Custom { protocol, base_url } => {
                let endpoint = if include_full_endpoint_url {
                    json!(base_url)
                } else {
                    json!("[redacted-custom-endpoint]")
                };
                json!({
                    "id": provider.id,
                    "kind": "custom",
                    "protocol": protocol,
                    "endpoint": endpoint,
                })
            }
        })
        .collect();

    Ok(json!({
        "app_version": app_version,
        "build_sha": build_sha,
        "os": {
            "name": std::env::consts::OS,
            "family": std::env::consts::FAMILY,
            "arch": std::env::consts::ARCH,
        },
        "session_id": session_id,
        "providers": providers,
    }))
}

fn active_events_jsonl(app_data_dir: &Path) -> Result<Vec<u8>, String> {
    let mut events = read_all_diagnostics(app_data_dir)?;
    events.sort_by_key(|event| event.timestamp);
    let mut out = Vec::new();
    for event in events {
        serde_json::to_writer(&mut out, &event).map_err(|e| e.to_string())?;
        out.push(b'\n');
    }
    Ok(out)
}

fn crashes_jsonl(app_data_dir: &Path) -> Result<Vec<u8>, String> {
    let path = diagnostics_dir(app_data_dir).join("crashes.jsonl");
    if !path.exists() {
        return Ok(Vec::new());
    }
    fs::read(path).map_err(|e| e.to_string())
}

fn load_chat_by_id(
    app_data_dir: &Path,
    chat_id: &str,
) -> Result<Option<Chat>, String> {
    let chats_dir = app_data_dir.join("chats");
    if !chats_dir.exists() {
        return Ok(None);
    }
    for entry in fs::read_dir(chats_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.path().is_file() {
            continue;
        }
        let chat = Chat::load(&entry.path())?;
        if chat.id == chat_id {
            return Ok(Some(chat));
        }
    }
    Ok(None)
}

fn write_tar_gz(
    path: &Path,
    files: &[(String, Vec<u8>)],
) -> Result<(), String> {
    let file = fs::File::create(path).map_err(|e| e.to_string())?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut archive = tar::Builder::new(encoder);
    for (name, bytes) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive
            .append_data(&mut header, name.as_str(), Cursor::new(bytes))
            .map_err(|e| e.to_string())?;
    }
    archive.finish().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chats::Message;
    use crate::config::{Protocol, ProviderConfig};
    use crate::diagnostics::event::{
        DiagnosticContext, DiagnosticEvent, DiagnosticKind, DiagnosticLevel,
        DiagnosticSource,
    };
    use crate::diagnostics::writer::{append_event, diagnostics_dir};
    use flate2::read::GzDecoder;
    use std::io::Read;

    #[test]
    fn reveal_path_rejects_paths_outside_diagnostics_and_exports() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(diagnostics_dir(dir.path()).join("exports"))
            .unwrap();
        let allowed = diagnostics_dir(dir.path())
            .join("exports")
            .join("bundle.tar.gz");
        fs::write(&allowed, "bundle").unwrap();
        let outside = dir.path().join("settings.json");
        fs::write(&outside, "{}").unwrap();

        assert!(validate_reveal_path(dir.path(), &allowed).is_ok());
        assert!(validate_reveal_path(dir.path(), &outside).is_err());
    }

    #[test]
    fn export_bundle_redacts_defaults_and_includes_explicit_extras() {
        let dir = tempfile::tempdir().unwrap();
        let settings = Settings {
            default_provider_id: Some("custom-1".into()),
            providers: vec![ProviderConfig {
                id: "custom-1".into(),
                display_name: "Private Proxy".into(),
                _type: ProviderType::Custom {
                    protocol: Protocol::OpenAI,
                    base_url:
                        "https://proxy.example.com/v1/private-path?token=secret"
                            .into(),
                },
            }],
        };
        settings.save(dir.path()).unwrap();
        let chat = Chat {
            id: "chat-uuid".into(),
            name: "Private Chat Name".into(),
            messages: vec![
                Message {
                    role: "user".into(),
                    content: "secret prompt text".into(),
                    model_id: None,
                },
                Message {
                    role: "assistant".into(),
                    content: "secret answer text".into(),
                    model_id: Some("gpt-5-mini".into()),
                },
            ],
            created_at: "02-05-26".into(),
            provider_id: "custom-1".into(),
        };
        chat.save(&dir.path().join("chats")).unwrap();
        append_event(
            dir.path(),
            &DiagnosticEvent::new(
                DiagnosticLevel::Error,
                DiagnosticSource::Backend,
                DiagnosticKind::CommandFailed,
                "stream failed",
                DiagnosticContext::command_failed(
                    "stream_message",
                    Some("session-1"),
                    Some("chat-uuid"),
                    Some("custom-1"),
                    Some("gpt-5-mini"),
                    "network",
                ),
                chrono::Utc::now(),
            ),
        )
        .unwrap();

        let basic = export_diagnostics_bundle(
            dir.path(),
            "session-1",
            "0.1.0",
            Some("abc123"),
            ExportOptions {
                include_full_endpoint_url: false,
                include_active_chat_transcript: false,
                active_chat_id: Some("chat-uuid".into()),
            },
        )
        .unwrap();
        let basic_entries = tar_gz_entries(Path::new(&basic.path));
        let basic_joined = basic_entries
            .iter()
            .map(|(_, contents)| contents.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(basic_entries.iter().any(|(name, _)| name == "summary.md"));
        assert!(basic_entries
            .iter()
            .any(|(name, _)| name == "environment.json"));
        assert!(basic_entries.iter().any(|(name, _)| name == "events.jsonl"));
        assert!(basic_entries
            .iter()
            .any(|(name, _)| name == "included.json"));
        assert!(basic_joined.contains("gpt-5-mini"));
        assert!(!basic_joined.contains("proxy.example.com"));
        assert!(!basic_joined.contains("Private Chat Name"));
        assert!(!basic_joined.contains("secret prompt text"));
        assert!(!basic_joined.contains("secret answer text"));

        let requested_missing_chat = export_diagnostics_bundle(
            dir.path(),
            "session-1",
            "0.1.0",
            Some("abc123"),
            ExportOptions {
                include_full_endpoint_url: false,
                include_active_chat_transcript: true,
                active_chat_id: None,
            },
        )
        .unwrap();
        let missing_entries =
            tar_gz_entries(Path::new(&requested_missing_chat.path));
        let missing_included: serde_json::Value = serde_json::from_str(
            entry_contents(&missing_entries, "included.json"),
        )
        .unwrap();
        assert!(!requested_missing_chat.included.active_chat_transcript);
        assert_eq!(missing_included["activeChatTranscript"], false);
        assert!(!missing_entries
            .iter()
            .any(|(name, _)| name == "active-chat.json"));

        let expanded = export_diagnostics_bundle(
            dir.path(),
            "session-1",
            "0.1.0",
            Some("abc123"),
            ExportOptions {
                include_full_endpoint_url: true,
                include_active_chat_transcript: true,
                active_chat_id: Some("chat-uuid".into()),
            },
        )
        .unwrap();
        let expanded_joined = tar_gz_entries(Path::new(&expanded.path))
            .iter()
            .map(|(_, contents)| contents.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(expanded.included.active_chat_transcript);
        assert!(expanded_joined.contains(
            "https://proxy.example.com/v1/private-path?token=secret"
        ));
        assert!(expanded_joined.contains("secret prompt text"));
        assert!(expanded_joined.contains("secret answer text"));
    }

    fn tar_gz_entries(path: &Path) -> Vec<(String, String)> {
        let mut gz = GzDecoder::new(fs::File::open(path).unwrap());
        let mut tar = Vec::new();
        gz.read_to_end(&mut tar).unwrap();
        let mut entries = Vec::new();
        let mut offset = 0usize;
        while offset + 512 <= tar.len() {
            let header = &tar[offset..offset + 512];
            if header.iter().all(|byte| *byte == 0) {
                break;
            }
            let name = String::from_utf8_lossy(&header[0..100])
                .trim_matches(char::from(0))
                .to_string();
            let size_text = String::from_utf8_lossy(&header[124..136])
                .trim_matches(char::from(0))
                .trim()
                .to_string();
            let size = usize::from_str_radix(size_text.trim(), 8).unwrap_or(0);
            offset += 512;
            let contents = String::from_utf8_lossy(&tar[offset..offset + size])
                .to_string();
            entries.push((name, contents));
            offset += size.div_ceil(512) * 512;
        }
        entries
    }

    fn entry_contents<'a>(
        entries: &'a [(String, String)],
        name: &str,
    ) -> &'a str {
        entries
            .iter()
            .find(|(entry_name, _)| entry_name == name)
            .map(|(_, contents)| contents.as_str())
            .unwrap()
    }
}
