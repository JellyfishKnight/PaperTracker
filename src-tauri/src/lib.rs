mod updater;
mod paper_tracker_config;
mod utils;
mod serial;

use paper_tracker_config::config::init_config;
use updater::version_check::check_for_updates;
use serial::esptools::{flash_esp32, restart_esp32};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            init_config(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet, 
            check_for_updates,
            flash_esp32,
            restart_esp32,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}