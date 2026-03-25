// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::Manager;
mod config;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_settings, save_settings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
