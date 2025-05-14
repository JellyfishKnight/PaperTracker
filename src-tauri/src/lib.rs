mod updater;
mod paper_tracker_config;
mod utils;
mod serial;
mod websocket;
mod integration;

use paper_tracker_config::config::init_config;
use tauri::Manager;
use updater::version_check::check_for_updates;
use integration::serial_commands::{write_ssid_and_password, write_brightness, restart_esp32, flash_esp32};
use integration::video_commands::{update_stream_ip, is_device_connected, get_device_status};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            init_config(app.handle())?;
            
            // Initialize services
            let (serial_state, video_state) = integration::init_services();
            
            // Register states
            app.manage(serial_state);
            app.manage(video_state);
            
            println!("Application initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_for_updates,
            write_ssid_and_password,
            write_brightness,
            restart_esp32,
            flash_esp32,
            update_stream_ip,
            is_device_connected,
            get_device_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}