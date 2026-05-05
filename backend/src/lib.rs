use chats::Chat;
use chats::Message;
use std::fs;
use std::path::PathBuf;
use tauri::{Emitter, Manager};
use tauri_plugin_opener::OpenerExt;
mod chats;
mod config;
mod diagnostics;
mod llm;
mod mcp;
mod models;
mod secrets;

use diagnostics::{
    ClientEvent, DiagnosticContext, DiagnosticEvent, DiagnosticSource,
    Diagnostics, ExportOptions, ExportResult, diagnostics_dir,
    diagnostics_summary, export_diagnostics_bundle, install_panic_hook,
    read_recent_diagnostics, record_result, redact_context, start_diagnostics,
    validate_reveal_path,
};

#[tauri::command]
fn get_settings(
    app: tauri::AppHandle,
    diagnostics: tauri::State<'_, Diagnostics>,
) -> Result<config::Settings, String> {
    let result = (|| {
        let app_data_dir =
            app.path().app_data_dir().map_err(|e| e.to_string())?;
        config::Settings::load(&app_data_dir)
    })();
    record_result(&diagnostics, "get_settings", None, None, None, result)
}

#[tauri::command]
fn upsert_provider(
    app: tauri::AppHandle,
    diagnostics: tauri::State<'_, Diagnostics>,
    provider: config::ProviderConfig,
    api_key: Option<String>,
) -> Result<(), String> {
    let provider_id = provider.id.clone();
    let result = (|| {
        let app_data_dir =
            app.path().app_data_dir().map_err(|e| e.to_string())?;
        let mut settings =
            config::Settings::load(&app_data_dir).unwrap_or(config::Settings {
                default_provider_id: None,
                providers: vec![],
            });
        if let Some(key) = api_key
            && !key.is_empty()
        {
            secrets::set_api_key(&provider.id, &key)?;
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
    })();
    record_result(
        &diagnostics,
        "upsert_provider",
        None,
        Some(&provider_id),
        None,
        result,
    )
}

#[tauri::command]
fn delete_provider(
    app: tauri::AppHandle,
    diagnostics: tauri::State<'_, Diagnostics>,
    provider_id: String,
) -> Result<(), String> {
    let result = (|| {
        let app_data_dir =
            app.path().app_data_dir().map_err(|e| e.to_string())?;
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
    })();
    record_result(
        &diagnostics,
        "delete_provider",
        None,
        Some(&provider_id),
        None,
        result,
    )
}

#[tauri::command]
fn set_default_provider(
    app: tauri::AppHandle,
    diagnostics: tauri::State<'_, Diagnostics>,
    provider_id: String,
) -> Result<(), String> {
    let result = (|| {
        let app_data_dir =
            app.path().app_data_dir().map_err(|e| e.to_string())?;
        let mut settings = config::Settings::load(&app_data_dir)?;
        if !settings.providers.iter().any(|p| p.id == provider_id) {
            return Err(format!("Provider '{}' not found.", provider_id));
        }
        settings.default_provider_id = Some(provider_id.clone());
        settings.save(&app_data_dir)
    })();
    record_result(
        &diagnostics,
        "set_default_provider",
        None,
        Some(&provider_id),
        None,
        result,
    )
}

#[tauri::command]
fn list_models(
    app: tauri::AppHandle,
    diagnostics: tauri::State<'_, Diagnostics>,
    provider_id: String,
) -> Result<Vec<models::ModelInfo>, String> {
    let result = (|| {
        let app_data_dir =
            app.path().app_data_dir().map_err(|e| e.to_string())?;
        let settings = config::Settings::load(&app_data_dir)?;
        let provider = settings
            .providers
            .iter()
            .find(|p| p.id == provider_id)
            .ok_or_else(|| format!("Provider '{}' not found.", provider_id))?;
        Ok(models::models_for_provider(provider))
    })();
    record_result(
        &diagnostics,
        "list_models",
        None,
        Some(&provider_id),
        None,
        result,
    )
}

#[tauri::command]
fn has_api_key(provider_id: String) -> bool {
    secrets::get_api_key(&provider_id).is_ok()
}

#[tauri::command]
fn get_session_id(diagnostics: tauri::State<'_, Diagnostics>) -> String {
    diagnostics.session_id().to_string()
}

#[tauri::command]
fn log_client_event(
    diagnostics: tauri::State<'_, Diagnostics>,
    event: ClientEvent,
) -> Result<(), String> {
    let context = DiagnosticContext(
        redact_context(event.kind, event.context)
            .as_object()
            .cloned()
            .unwrap_or_default(),
    );
    diagnostics.log(DiagnosticEvent::new(
        event.level,
        DiagnosticSource::Frontend,
        event.kind,
        &event.message,
        context,
        chrono::Utc::now(),
    ));
    Ok(())
}

#[tauri::command]
fn get_recent_diagnostics(
    diagnostics: tauri::State<'_, Diagnostics>,
    limit: u32,
) -> Result<Vec<DiagnosticEvent>, String> {
    read_recent_diagnostics(diagnostics.app_data_dir(), limit)
}

#[tauri::command]
fn get_diagnostics_summary(
    diagnostics: tauri::State<'_, Diagnostics>,
    _options: ExportOptions,
) -> Result<String, String> {
    diagnostics_summary(
        diagnostics.app_data_dir(),
        diagnostics.session_id(),
        env!("CARGO_PKG_VERSION"),
    )
}

#[tauri::command]
fn export_diagnostics(
    diagnostics: tauri::State<'_, Diagnostics>,
    options: ExportOptions,
) -> Result<ExportResult, String> {
    export_diagnostics_bundle(
        diagnostics.app_data_dir(),
        diagnostics.session_id(),
        env!("CARGO_PKG_VERSION"),
        option_env!("MAGNUS_BUILD_SHA"),
        options,
    )
}

#[tauri::command]
fn reveal_diagnostics_folder(
    app: tauri::AppHandle,
    diagnostics: tauri::State<'_, Diagnostics>,
) -> Result<(), String> {
    let dir = diagnostics_dir(diagnostics.app_data_dir());
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(dir.to_string_lossy().to_string(), None::<String>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn reveal_path(
    app: tauri::AppHandle,
    diagnostics: tauri::State<'_, Diagnostics>,
    path: String,
) -> Result<(), String> {
    let path = PathBuf::from(path);
    validate_reveal_path(diagnostics.app_data_dir(), &path)?;
    app.opener()
        .reveal_item_in_dir(path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn test_mcp_connection(
    _pool: tauri::State<'_, mcp::McpPool>,
    diagnostics: tauri::State<'_, Diagnostics>,
) -> Result<(), String> {
    let result = async {
        mcp::connect().await.map_err(|e| e.to_string())?;
        Ok(())
    }
    .await;
    record_result(
        &diagnostics,
        "test_mcp_connection",
        None,
        None,
        None,
        result,
    )
}

#[tauri::command]
async fn connect_server(
    pool: tauri::State<'_, mcp::McpPool>,
    diagnostics: tauri::State<'_, Diagnostics>,
    server: mcp::McpServer,
) -> Result<(), String> {
    let result = async {
        let client = mcp::connect_server(&server)
            .await
            .map_err(|e| e.to_string())?;
        pool.connections
            .lock()
            .await
            .insert(server.name.clone(), client);
        Ok(())
    }
    .await;
    record_result(&diagnostics, "connect_server", None, None, None, result)
}

#[tauri::command]
async fn list_tools(
    pool: tauri::State<'_, mcp::McpPool>,
    diagnostics: tauri::State<'_, Diagnostics>,
    server: mcp::McpServer,
) -> Result<Vec<rmcp::model::Tool>, String> {
    let result = async {
        let guard = pool.connections.lock().await;
        let client = guard
            .get(&server.name)
            .ok_or("Server not connected".to_string())?;
        mcp::list_tools(client).await.map_err(|e| e.to_string())
    }
    .await;
    record_result(&diagnostics, "list_tools", None, None, None, result)
}

#[tauri::command]
async fn execute_tool_call(
    pool: tauri::State<'_, mcp::McpPool>,
    diagnostics: tauri::State<'_, Diagnostics>,
    server_name: String,
    tool_name: String,
    arguments: serde_json::Value,
) -> Result<String, String> {
    let result = async {
        let guard = pool.connections.lock().await;
        let client = guard
            .get(&server_name)
            .ok_or("Server not connected".to_string())?;
        mcp::call_tool(client, &tool_name, arguments)
            .await
            .map_err(|e| e.to_string())
    }
    .await;
    record_result(&diagnostics, "execute_tool_call", None, None, None, result)
}

#[tauri::command]
fn save_mcp_servers(
    app: tauri::AppHandle,
    diagnostics: tauri::State<'_, Diagnostics>,
    servers: Vec<mcp::McpServer>,
) -> Result<(), String> {
    let result = (|| {
        let app_data_dir =
            app.path().app_data_dir().map_err(|e| e.to_string())?;
        mcp::save_servers(&app_data_dir, &servers)
    })();
    record_result(&diagnostics, "save_mcp_servers", None, None, None, result)
}

#[tauri::command]
fn load_mcp_servers(
    app: tauri::AppHandle,
    diagnostics: tauri::State<'_, Diagnostics>,
) -> Result<Vec<mcp::McpServer>, String> {
    let result = (|| {
        let app_data_dir =
            app.path().app_data_dir().map_err(|e| e.to_string())?;
        mcp::load_servers(&app_data_dir)
    })();
    record_result(&diagnostics, "load_mcp_servers", None, None, None, result)
}

#[tauri::command]
async fn disconnect_server(
    pool: tauri::State<'_, mcp::McpPool>,
    _diagnostics: tauri::State<'_, Diagnostics>,
    server_name: String,
) -> Result<(), String> {
    pool.connections.lock().await.remove(&server_name);
    Ok(())
}

#[tauri::command]
async fn get_connected_servers(
    pool: tauri::State<'_, mcp::McpPool>,
    _diagnostics: tauri::State<'_, Diagnostics>,
) -> Result<Vec<String>, String> {
    let names = pool.connections.lock().await.keys().cloned().collect();
    Ok(names)
}

#[tauri::command]
async fn stream_message(
    app: tauri::AppHandle,
    pool: tauri::State<'_, mcp::McpPool>,
    diagnostics: tauri::State<'_, Diagnostics>,
    provider_id: String,
    model_id: String,
    messages: Vec<Message>,
) -> Result<(), String> {
    let result = async {
        let app_data_dir =
            app.path().app_data_dir().map_err(|e| e.to_string())?;
        let settings = config::Settings::load(&app_data_dir)?;
        let provider = settings
            .providers
            .iter()
            .find(|p| p.id == provider_id)
            .ok_or_else(|| format!("Provider '{}' not found.", provider_id))?;
        let api_key = secrets::get_api_key(&provider_id)?;
        let http = reqwest::Client::new();
        let client = llm::client_for(provider, api_key, http);

        let tools: Vec<serde_json::Value> = {
            let guard = pool.connections.lock().await;
            let mut all_tools = Vec::new();
            for mcp_client in guard.values() {
                if let Ok(server_tools) = mcp::list_tools(mcp_client).await {
                    for tool in server_tools {
                        all_tools.push(serde_json::json!({
                            "name": tool.name,
                            "description": tool.description,
                            "input_schema": tool.input_schema,
                        }));
                    }
                }
            }
            all_tools
        };

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
            let (assistant_blocks, tool_uses) = client
                .stream_raw(&app, &json_messages, &tools, &model_id)
                .await
                .map_err(|e| e.to_string())?;

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
                app.emit(
                    "tool-call",
                    serde_json::json!({ "name": tool_use.name }),
                )
                .unwrap();
                let mut result_content = "Tool not found".to_string();
                for mcp_client in guard.values() {
                    if let Ok(result) = mcp::call_tool(
                        mcp_client,
                        &tool_use.name,
                        tool_use.input.clone(),
                    )
                    .await
                    {
                        result_content = result;
                        break;
                    }
                }
                tool_results.push(serde_json::json!({
                    "type": "tool_result",
                    "tool_use_id": tool_use.id,
                    "content": result_content,
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
    .await;

    record_result(
        &diagnostics,
        "stream_message",
        None,
        Some(&provider_id),
        Some(&model_id),
        result,
    )
}

#[tauri::command]
async fn save_chat(
    app: tauri::AppHandle,
    diagnostics: tauri::State<'_, Diagnostics>,
    chat: Chat,
) -> Result<(), String> {
    let chat_id = chat.id.clone();
    let provider_id = chat.provider_id.clone();
    let result = async {
        let chats_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| e.to_string())?
            .join("chats");
        chat.save(&chats_dir)
    }
    .await;
    record_result(
        &diagnostics,
        "save_chat",
        Some(&chat_id),
        Some(&provider_id),
        None,
        result,
    )
}

#[tauri::command]
async fn rename_chat(
    app: tauri::AppHandle,
    diagnostics: tauri::State<'_, Diagnostics>,
    provider_id: String,
    model_id: String,
    chat: Chat,
) -> Result<String, String> {
    let chat_id = chat.id.clone();
    let result = async {
        let app_data_dir =
            app.path().app_data_dir().map_err(|e| e.to_string())?;
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
    .await;
    record_result(
        &diagnostics,
        "rename_chat",
        Some(&chat_id),
        Some(&provider_id),
        Some(&model_id),
        result,
    )
}

#[tauri::command]
fn load_chats(
    app: tauri::AppHandle,
    diagnostics: tauri::State<'_, Diagnostics>,
) -> Result<Vec<Chat>, String> {
    let result = (|| {
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
    })();
    record_result(&diagnostics, "load_chats", None, None, None, result)
}

#[tauri::command]
fn delete_chat(
    app: tauri::AppHandle,
    diagnostics: tauri::State<'_, Diagnostics>,
    chat: Chat,
) -> Result<(), String> {
    let chat_id = chat.id.clone();
    let provider_id = chat.provider_id.clone();
    let result = (|| {
        let chats_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| e.to_string())?
            .join("chats");
        chat.delete(&chats_dir)
    })();
    record_result(
        &diagnostics,
        "delete_chat",
        Some(&chat_id),
        Some(&provider_id),
        None,
        result,
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .manage(mcp::McpPool::new())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let diagnostics = start_diagnostics(app_data_dir.clone(), 256);
            install_panic_hook(
                app_data_dir,
                diagnostics.session_id().to_string(),
            );
            app.manage(diagnostics);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            upsert_provider,
            delete_provider,
            set_default_provider,
            list_models,
            has_api_key,
            get_session_id,
            log_client_event,
            get_recent_diagnostics,
            get_diagnostics_summary,
            export_diagnostics,
            reveal_diagnostics_folder,
            reveal_path,
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
            disconnect_server,
            get_connected_servers,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if matches!(event, tauri::RunEvent::Exit)
            && let Some(diagnostics) = app_handle.try_state::<Diagnostics>()
        {
            tauri::async_runtime::block_on(diagnostics.flush());
        }
    });
}
