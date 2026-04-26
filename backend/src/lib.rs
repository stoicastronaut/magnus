use chats::Chat;
use chats::Message;
use std::fs;
use tauri::{Emitter, Manager};
mod chats;
mod config;
mod llm;
mod mcp;
mod models;
mod secrets;

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<config::Settings, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    config::Settings::load(&app_data_dir)
}

#[tauri::command]
fn upsert_provider(
    app: tauri::AppHandle,
    provider: config::ProviderConfig,
    api_key: Option<String>,
) -> Result<(), String> {
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
    servers: Vec<mcp::McpServer>,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
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
    eprintln!(
        "[stream_message] called provider_id={} model_id={} message_count={}",
        provider_id,
        model_id,
        messages.len()
    );
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let settings = config::Settings::load(&app_data_dir)?;
    let provider = settings
        .providers
        .iter()
        .find(|p| p.id == provider_id)
        .ok_or_else(|| format!("Provider '{}' not found.", provider_id))?;
    eprintln!("[stream_message] found provider: {:?}", provider);
    let api_key = secrets::get_api_key(&provider_id)?;
    eprintln!("[stream_message] got api key (len={})", api_key.len());
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
        eprintln!("[stream_message] calling client.stream_raw");
        let (assistant_blocks, tool_uses) = client
            .stream_raw(&app, &json_messages, &tools, &model_id)
            .await
            .map_err(|e| {
                eprintln!("[stream_message] stream_raw error: {}", e);
                e.to_string()
            })?;
        eprintln!(
            "[stream_message] stream_raw returned {} blocks, {} tool_uses",
            assistant_blocks.len(),
            tool_uses.len()
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
            app.emit("tool-call", serde_json::json!({ "name": tool_use.name }))
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(mcp::McpPool::new())
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
            disconnect_server,
            get_connected_servers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
