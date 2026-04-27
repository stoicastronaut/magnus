use chats::Chat;
use chats::Message;
use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{Emitter, Manager};
use tokio::sync::oneshot;

mod chats;
mod config;
mod fs_perm;
mod llm;
mod mcp;
mod models;
mod redact;
mod secrets;

// Approval decision type
#[derive(Clone, Debug, PartialEq)]
enum ToolApprovalDecision {
    AllowOnce,
    AlwaysAllow,
    Deny,
}

// Audit log entry
#[derive(serde::Serialize, Debug)]
struct AuditLogEntry {
    ts: String,
    server: String,
    tool: String,
    decision: String,
    duration_ms: u64,
    output_size: usize,
}

// Tool approval state (using Arc wrapper to allow extraction)
type ApprovalPendingMap = std::sync::Mutex<
    HashMap<String, Option<oneshot::Sender<ToolApprovalDecision>>>,
>;

// Counter for approval request IDs
static APPROVAL_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<config::Settings, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    config::Settings::load(&app_data_dir)
}

#[tauri::command]
fn upsert_provider(
    app: tauri::AppHandle,
    mut provider: config::ProviderConfig,
    api_key: Option<String>,
) -> Result<(), String> {
    // Validate custom provider URLs before saving
    if let config::ProviderType::Custom { base_url, .. } = &mut provider._type {
        let normalized_url = llm::validate_base_url(base_url)?;
        *base_url = normalized_url;
    }

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let mut settings =
        config::Settings::load(&app_data_dir).unwrap_or(config::Settings {
            default_provider_id: None,
            providers: vec![],
        });
    if let Some(key) = api_key {
        if !key.is_empty() {
            secrets::set_api_key(&provider.id, &key)?;
        }
    }
    if let Some(pos) =
        settings.providers.iter().position(|p| p.id == provider.id)
    {
        settings.providers[pos] = provider;
    } else {
        if settings.default_provider_id.is_none() {
            settings.default_provider_id = Some(provider.id.clone());
        }
        settings.providers.push(provider);
    }
    settings.save(&app_data_dir)
}

#[tauri::command]
fn delete_provider(
    app: tauri::AppHandle,
    provider_id: String,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let mut settings = config::Settings::load(&app_data_dir)?;
    if settings.default_provider_id.as_deref() == Some(&provider_id)
        && settings.providers.len() > 1
    {
        return Err(
            "Set a new default provider before deleting this one.".into()
        );
    }
    settings.providers.retain(|p| p.id != provider_id);
    if settings.default_provider_id.as_deref() == Some(&provider_id) {
        settings.default_provider_id =
            settings.providers.first().map(|p| p.id.clone());
    }
    let _ = secrets::delete_api_key(&provider_id);
    settings.save(&app_data_dir)
}

#[tauri::command]
fn set_default_provider(
    app: tauri::AppHandle,
    provider_id: String,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let mut settings = config::Settings::load(&app_data_dir)?;
    if !settings.providers.iter().any(|p| p.id == provider_id) {
        return Err(format!("Provider '{}' not found.", provider_id));
    }
    settings.default_provider_id = Some(provider_id);
    settings.save(&app_data_dir)
}

#[tauri::command]
fn list_models(
    app: tauri::AppHandle,
    provider_id: String,
) -> Result<Vec<models::ModelInfo>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let settings = config::Settings::load(&app_data_dir)?;
    let provider = settings
        .providers
        .iter()
        .find(|p| p.id == provider_id)
        .ok_or_else(|| format!("Provider '{}' not found.", provider_id))?;
    Ok(models::models_for_provider(provider))
}

#[tauri::command]
fn has_api_key(provider_id: String) -> bool {
    secrets::get_api_key(&provider_id).is_ok()
}

#[tauri::command]
async fn test_mcp_connection(
    _pool: tauri::State<'_, mcp::McpPool>,
) -> Result<(), String> {
    let _ = mcp::connect().await;
    Ok(())
}

#[tauri::command]
async fn connect_server(
    pool: tauri::State<'_, mcp::McpPool>,
    server: mcp::McpServer,
) -> Result<(), String> {
    let client = mcp::connect_server(&server)
        .await
        .map_err(|e| e.to_string())?;
    pool.connections
        .lock()
        .await
        .insert(server.name.clone(), client);
    Ok(())
}

#[tauri::command]
async fn list_tools(
    pool: tauri::State<'_, mcp::McpPool>,
    server: mcp::McpServer,
) -> Result<Vec<rmcp::model::Tool>, String> {
    let guard = pool.connections.lock().await;
    let client = guard
        .get(&server.name)
        .ok_or("Server not connected".to_string())?;
    mcp::list_tools(client).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn execute_tool_call(
    pool: tauri::State<'_, mcp::McpPool>,
    server_name: String,
    tool_name: String,
    arguments: serde_json::Value,
) -> Result<String, String> {
    let guard = pool.connections.lock().await;
    let client = guard
        .get(&server_name)
        .ok_or("Server not connected".to_string())?;
    mcp::call_tool(client, &tool_name, arguments)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_mcp_servers(
    app: tauri::AppHandle,
    mut servers: Vec<mcp::McpServer>,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    // Validate and resolve all server commands
    for server in &mut servers {
        let resolved_command =
            mcp::validate_command(&server.command, &server.args)?;
        server.command = resolved_command;

        // Ensure each server has an id (should be auto-generated by default_server_id)
        if server.id.is_empty() {
            server.id = uuid::Uuid::new_v4().to_string();
        }
    }

    mcp::save_servers(&app_data_dir, &servers)
}

#[tauri::command]
fn load_mcp_servers(
    app: tauri::AppHandle,
) -> Result<Vec<mcp::McpServer>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    mcp::load_servers(&app_data_dir)
}

#[tauri::command]
fn set_mcp_token(server_id: String, token: String) -> Result<(), String> {
    secrets::set_mcp_token(&server_id, &token)
}

#[tauri::command]
fn delete_mcp_token(server_id: String) -> Result<(), String> {
    secrets::delete_mcp_token(&server_id)
}

#[tauri::command]
async fn disconnect_server(
    pool: tauri::State<'_, mcp::McpPool>,
    server_name: String,
) -> Result<(), String> {
    pool.connections.lock().await.remove(&server_name);
    Ok(())
}

#[tauri::command]
async fn get_connected_servers(
    pool: tauri::State<'_, mcp::McpPool>,
) -> Result<Vec<String>, String> {
    let names = pool.connections.lock().await.keys().cloned().collect();
    Ok(names)
}

#[tauri::command]
async fn stream_message(
    app: tauri::AppHandle,
    pool: tauri::State<'_, mcp::McpPool>,
    provider_id: String,
    model_id: String,
    messages: Vec<Message>,
) -> Result<(), String> {
    tracing::debug!(
        provider_id,
        model_id,
        message_count = messages.len(),
        "stream_message called"
    );
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let settings = config::Settings::load(&app_data_dir)?;
    let provider = settings
        .providers
        .iter()
        .find(|p| p.id == provider_id)
        .ok_or_else(|| format!("Provider '{}' not found.", provider_id))?;
    let api_key = secrets::get_api_key(&provider_id)?;
    tracing::debug!("retrieved api key");
    let http = reqwest::Client::new();
    let client = llm::client_for(provider, api_key, http);

    // Load trust store
    let trust_store = mcp::load_trust_store(&app_data_dir)?;

    // Load servers and build a map of server_name -> (server_id, locally_created)
    let servers = mcp::load_servers(&app_data_dir)?;
    let server_info: HashMap<String, (String, bool)> = servers
        .iter()
        .map(|s| (s.name.clone(), (s.id.clone(), s.locally_created)))
        .collect();

    // Build tools_with_owner: Vec<(server_name, Tool)>
    let tools_with_owner: Vec<(String, rmcp::model::Tool)> = {
        let guard = pool.connections.lock().await;
        let mut out = Vec::new();
        for (server_name, c) in guard.iter() {
            if let Ok(ts) = mcp::list_tools(c).await {
                for t in ts {
                    out.push((server_name.clone(), t));
                }
            }
        }
        out
    };

    // Filter and build tools_for_model, excluding disabled tools
    let tools_for_model: Vec<serde_json::Value> = tools_with_owner
        .iter()
        .filter(|(s, t)| {
            if let Some((server_id, _)) = server_info.get(s) {
                !trust_store.is_tool_disabled(server_id, &t.name)
            } else {
                false
            }
        })
        .map(|(_, t)| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "input_schema": t.input_schema,
            })
        })
        .collect();

    let mut json_messages: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": [{"type": "text", "text": m.content}]
            })
        })
        .collect();

    loop {
        tracing::debug!("calling client.stream_raw");
        let (assistant_blocks, tool_uses) = client
            .stream_raw(&app, &json_messages, &tools_for_model, &model_id)
            .await
            .map_err(|e| {
                tracing::warn!("stream_raw error: {}", e);
                e.to_string()
            })?;
        tracing::debug!(
            blocks = assistant_blocks.len(),
            tool_uses = tool_uses.len(),
            "stream_raw returned"
        );

        json_messages.push(serde_json::json!({
            "role": "assistant",
            "content": assistant_blocks,
        }));

        if tool_uses.is_empty() {
            break;
        }

        let guard = pool.connections.lock().await;
        let mut tool_results = Vec::new();

        for tool_use in &tool_uses {
            // Find which server owns this tool
            let (owning_server, _tool_obj) = tools_with_owner
                .iter()
                .find(|(_, t)| t.name == tool_use.name)
                .ok_or_else(|| format!("Tool '{}' not found", tool_use.name))?;

            // Get server_id and locally_created flag
            let (server_id, locally_created) =
                server_info.get(owning_server).cloned().ok_or_else(|| {
                    format!("Server '{}' not found in config", owning_server)
                })?;

            let start_time = std::time::Instant::now();

            // Check trust level: if server is not locally_created, force AskEveryTime
            let can_execute = if !locally_created {
                false // Imported servers always ask
            } else {
                trust_store
                    .can_execute_without_prompt(&server_id, &tool_use.name)
            };

            let decision = if can_execute {
                ToolApprovalDecision::AllowOnce
            } else {
                // Request approval from UI
                let approval_id = format!(
                    "{}",
                    APPROVAL_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
                );
                let (tx, rx) = oneshot::channel();

                // Store pending approval
                (|| -> Result<(), String> {
                    let state = app.state::<ApprovalPendingMap>();
                    let mut pending =
                        state.lock().map_err(|e| e.to_string())?;
                    pending.insert(approval_id.clone(), Some(tx));
                    Ok(())
                })()?;

                // Emit approval request event with pretty-printed input
                let input_preview =
                    serde_json::to_string_pretty(&tool_use.input)
                        .unwrap_or_else(|_| format!("{:?}", tool_use.input));

                let _ = app.emit(
                    "tool-approval-request",
                    serde_json::json!({
                        "id": approval_id,
                        "server": owning_server,
                        "tool": tool_use.name,
                        "input_preview": input_preview,
                    }),
                );

                // Wait for response with 5-minute timeout
                match tokio::time::timeout(
                    std::time::Duration::from_secs(300),
                    rx,
                )
                .await
                {
                    Ok(Ok(dec)) => dec,
                    Ok(Err(_)) => ToolApprovalDecision::Deny,
                    Err(_) => {
                        tracing::warn!(
                            "tool approval timeout: {}",
                            approval_id
                        );
                        ToolApprovalDecision::Deny
                    }
                }
            };

            let duration_ms = start_time.elapsed().as_millis() as u64;

            // If AlwaysAllow, persist the decision
            if decision == ToolApprovalDecision::AlwaysAllow {
                let mut updated_store = trust_store.clone();
                updated_store.set_tool_trust(
                    &server_id,
                    &tool_use.name,
                    mcp::ToolTrust::AlwaysAllow,
                );
                let _ = mcp::save_trust_store(&app_data_dir, &updated_store);
            }

            // Execute or deny
            let result_content = match decision {
                ToolApprovalDecision::Deny => {
                    let _ = write_audit_log(
                        &app_data_dir,
                        &server_id,
                        &tool_use.name,
                        "deny",
                        duration_ms,
                        0,
                    );
                    "User denied tool execution".to_string()
                }
                _ => {
                    // Execute the tool
                    let client = guard.get(owning_server).ok_or_else(|| {
                        format!("Server '{}' not connected", owning_server)
                    })?;

                    let call_start = std::time::Instant::now();

                    let exec_result = match tokio::time::timeout(
                        std::time::Duration::from_secs(30),
                        mcp::call_tool(
                            client,
                            &tool_use.name,
                            tool_use.input.clone(),
                        ),
                    )
                    .await
                    {
                        Ok(Ok(result)) => result,
                        Ok(Err(e)) => {
                            let _ = write_audit_log(
                                &app_data_dir,
                                &server_id,
                                &tool_use.name,
                                "error",
                                call_start.elapsed().as_millis() as u64,
                                0,
                            );
                            format!("Tool execution error: {}", e)
                        }
                        Err(_) => {
                            let _ = write_audit_log(
                                &app_data_dir,
                                &server_id,
                                &tool_use.name,
                                "timeout",
                                30000,
                                0,
                            );
                            "Tool execution timeout (30s)".to_string()
                        }
                    };

                    let output_size = exec_result.len();
                    let decision_str = match decision {
                        ToolApprovalDecision::AllowOnce => "allow_once",
                        ToolApprovalDecision::AlwaysAllow => "always_allow",
                        ToolApprovalDecision::Deny => "deny",
                    };
                    let _ = write_audit_log(
                        &app_data_dir,
                        &server_id,
                        &tool_use.name,
                        decision_str,
                        call_start.elapsed().as_millis() as u64,
                        output_size,
                    );

                    // Truncate to 64 KB
                    if exec_result.len() > 65536 {
                        let truncated = exec_result[..65536].to_string();
                        format!(
                            "{}\n[truncated, original {} bytes]",
                            truncated,
                            exec_result.len()
                        )
                    } else {
                        exec_result
                    }
                }
            };

            tool_results.push(serde_json::json!({
                "type": "tool_result",
                "tool_use_id": tool_use.id,
                "content": result_content,
                "is_error": decision == ToolApprovalDecision::Deny,
            }));
        }
        drop(guard);

        json_messages.push(serde_json::json!({
            "role": "user",
            "content": tool_results,
        }));
    }

    Ok(())
}

#[tauri::command]
async fn save_chat(app: tauri::AppHandle, chat: Chat) -> Result<(), String> {
    let chats_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("chats");
    chat.save(&chats_dir)
}

#[tauri::command]
async fn rename_chat(
    app: tauri::AppHandle,
    provider_id: String,
    model_id: String,
    chat: Chat,
) -> Result<String, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let chats_dir = app_data_dir.join("chats");
    let settings = config::Settings::load(&app_data_dir)?;
    let provider = settings
        .providers
        .iter()
        .find(|p| p.id == provider_id)
        .ok_or_else(|| format!("Provider '{}' not found.", provider_id))?;
    let api_key = secrets::get_api_key(&provider_id)?;
    let http = reqwest::Client::new();
    let client = llm::client_for(provider, api_key, http);
    let chat_name = client
        .generate_title(&chat.messages, &model_id)
        .await
        .map_err(|e| e.to_string())?;
    let updated_chat = Chat {
        name: chat_name.clone(),
        ..chat
    };
    updated_chat.save(&chats_dir)?;
    Ok(chat_name)
}

#[tauri::command]
fn load_chats(app: tauri::AppHandle) -> Result<Vec<Chat>, String> {
    let chats_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("chats");

    if !chats_dir.exists() {
        return Ok(vec![]);
    }

    let mut chats = vec![];
    for entry in fs::read_dir(&chats_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let chat = Chat::load(&entry.path())?;
        chats.push(chat);
    }
    Ok(chats)
}

#[tauri::command]
fn delete_chat(app: tauri::AppHandle, chat: Chat) -> Result<(), String> {
    let chats_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("chats");
    chat.delete(&chats_dir)
}

#[tauri::command]
fn set_tool_trust(
    app: tauri::AppHandle,
    server_name: String,
    tool_name: String,
    trust: String,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let mut store = mcp::load_trust_store(&app_data_dir)?;

    let tool_trust = match trust.as_str() {
        "disabled" => mcp::ToolTrust::Disabled,
        "ask_every_time" => mcp::ToolTrust::AskEveryTime,
        "always_allow" => mcp::ToolTrust::AlwaysAllow,
        _ => return Err("Invalid trust level".into()),
    };

    store.set_tool_trust(&server_name, &tool_name, tool_trust);
    mcp::save_trust_store(&app_data_dir, &store)
}

#[tauri::command]
fn respond_tool_approval(
    app: tauri::AppHandle,
    approval_id: String,
    allow_once: bool,
    always_allow: bool,
    deny: bool,
) -> Result<(), String> {
    let decision = if deny {
        ToolApprovalDecision::Deny
    } else if always_allow {
        ToolApprovalDecision::AlwaysAllow
    } else if allow_once {
        ToolApprovalDecision::AllowOnce
    } else {
        return Err(
            "Must choose one decision: allow_once, always_allow, or deny"
                .into(),
        );
    };

    (|| -> Result<(), String> {
        let state = app.state::<ApprovalPendingMap>();
        let mut pending_map = state.lock().map_err(|e| e.to_string())?;
        if let Some(Some(sender)) = pending_map.remove(&approval_id) {
            let _ = sender.send(decision);
        }
        Ok(())
    })()
}

fn write_audit_log(
    app_data_dir: &std::path::Path,
    server: &str,
    tool: &str,
    decision: &str,
    duration_ms: u64,
    output_size: usize,
) -> Result<(), String> {
    use chrono::Utc;
    use std::io::Write;

    std::fs::create_dir_all(app_data_dir).map_err(|e| e.to_string())?;
    let log_path = app_data_dir.join("mcp_audit.log");

    let ts = Utc::now().to_rfc3339();
    let entry = AuditLogEntry {
        ts,
        server: server.to_string(),
        tool: tool.to_string(),
        decision: decision.to_string(),
        duration_ms,
        output_size,
    };

    let json = serde_json::to_string(&entry).map_err(|e| e.to_string())?;

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| e.to_string())?;

    writeln!(file, "{}", json).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "magnus=info".into()),
        )
        .init();

    tauri::Builder::default()
        .manage(mcp::McpPool::new())
        .manage(ApprovalPendingMap::new(std::collections::HashMap::new()))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            upsert_provider,
            delete_provider,
            set_default_provider,
            list_models,
            has_api_key,
            connect_server,
            stream_message,
            save_chat,
            rename_chat,
            load_chats,
            delete_chat,
            test_mcp_connection,
            list_tools,
            execute_tool_call,
            save_mcp_servers,
            load_mcp_servers,
            set_mcp_token,
            delete_mcp_token,
            disconnect_server,
            get_connected_servers,
            set_tool_trust,
            respond_tool_approval,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
