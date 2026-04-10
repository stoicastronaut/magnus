use chats::Chat;
use chats::Message;
use std::fs;
use tauri::Manager;
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
async fn test_mcp_connection() -> Result<(), String> {
    mcp::connect().await;
    Ok(())
}

#[tauri::command]
async fn stream_message(
    app: tauri::AppHandle,
    api_key: String,
    base_url: String,
    messages: Vec<Message>,
) -> Result<(), String> {
    llm::stream_message(app, &api_key, &base_url, &messages).await
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
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            stream_message,
            save_chat,
            rename_chat,
            load_chats,
            delete_chat,
            test_mcp_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
