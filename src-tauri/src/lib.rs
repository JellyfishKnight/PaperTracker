mod updater;
mod paper_tracker_config;
mod utils;
mod serial;
mod websocket;
mod integration;

use paper_tracker_config::config::init_config;
use updater::version_check::check_for_updates;
use ftlog::*;
use integration::interface::{
    restart_esp32, 
    flash_esp32, 
    write_wifi_info, 
    start_face_image_stream,
    start_left_eye_image_stream,
    start_right_eye_image_stream,
};
use integration::init::init_device;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // build logger
            ftlog::Builder::new()
                .max_log_level(ftlog::LevelFilter::Debug)
                .unbounded()
                .print_omitted_count(true)
                .build()
                .expect("Failed to initialize logger")
                .init()
                .expect("Failed to set logger");
            ftlog::logger().flush();
            // create config 
            init_config(app.handle())?;
            init_device(app.handle());            
            info!("Application initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_for_updates,
            restart_esp32,
            flash_esp32,
            write_wifi_info,
            start_face_image_stream,
            start_left_eye_image_stream,
            start_right_eye_image_stream,
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}