use tauri::Manager;
use llm::Message;
use chats::Chat;
use std::fs;
mod config;
mod llm;
mod chats;

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<config::Settings, String> {
    let path = app.path().app_data_dir()
        .map_err(|e| e.to_string())?
        .join("settings.json");
    config::Settings::load(&path)
}

#[tauri::command]
fn save_settings(app: tauri::AppHandle, api_key: String, base_url: String) -> Result<(), String> {
    let path = app.path().app_data_dir()
        .map_err(|e| e.to_string())?
        .join("settings.json");
    let settings = config::Settings{
        api_key: api_key,
        base_url: base_url,
    };
    settings.save(&path)?;
    Ok(())
}

#[tauri::command]
async fn stream_message(app: tauri::AppHandle, api_key: String, base_url: String, messages: Vec<Message>) -> Result<(), String> {
    llm::stream_message(app, &api_key, &base_url, &messages).await
}

#[tauri::command]
fn save_chat(app: tauri::AppHandle, chat: Chat) -> Result<(), String> {
    let chats_dir = app.path().app_data_dir()
        .map_err(|e| e.to_string())?
        .join("chats");
    chat.save(&chats_dir)
}

#[tauri::command]
fn load_chats(app: tauri::AppHandle) -> Result<Vec<Chat>, String> {
    let chats_dir = app.path().app_data_dir()
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
    let chats_dir = app.path().app_data_dir()
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
            load_chats,
            delete_chat,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
