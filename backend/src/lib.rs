use chats::Chat;
use chats::Message;
use std::fs;
use tauri::{Emitter, Manager};
mod chats;
mod config;
mod llm;
mod mcp;

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<config::Settings, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    config::Settings::load(&app_data_dir)
}

#[tauri::command]
fn save_settings(
    app: tauri::AppHandle,
    api_key: String,
    base_url: String,
) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let settings = config::Settings { api_key, base_url };
    settings.save(&app_data_dir)
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
    api_key: String,
    base_url: String,
    messages: Vec<Message>,
) -> Result<(), String> {
    let mut json_messages: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": [{"type": "text", "text": m.content}]
            })
        })
        .collect();

    let tools: Vec<serde_json::Value> = {
        let guard = pool.connections.lock().await;
        let mut all_tools = Vec::new();
        for client in guard.values() {
            if let Ok(server_tools) = mcp::list_tools(client).await {
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

    loop {
        let (assistant_blocks, tool_uses) = llm::stream_message(
            &app,
            &api_key,
            &base_url,
            &json_messages,
            &tools,
        )
        .await?;

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
            for client in guard.values() {
                if let Ok(result) = mcp::call_tool(
                    client,
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
    api_key: String,
    base_url: String,
    chat: Chat,
) -> Result<String, String> {
    let chats_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("chats");
    let message = &chat.messages[0];
    let chat_name = llm::change_chat_name(&api_key, &base_url, message).await?;
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
            connect_server,
            save_settings,
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
