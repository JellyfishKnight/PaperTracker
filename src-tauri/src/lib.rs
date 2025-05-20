mod updater;
mod paper_tracker_config;
mod utils;
mod serial;
mod websocket;
mod integration;

use std::sync::Mutex;

use opencv::{core::MatTraitConst, highgui};
use paper_tracker_config::config::{init_config, FACE_CONFIG, EYE_CONFIG};
use tauri::Manager;
use updater::version_check::check_for_updates;
use ftlog::*;
use utils::consts::DEVICE_TYPE_FACE;
use integration::interface::{restart_esp32, flash_esp32, write_wifi_info};


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
            let mut serial = serial::esp32_serial::Esp32Serial::new();
            let mut face_image_msg_rx = serial.get_message_rx();
            let mut face_image_stream = websocket::image_stream::ImageStream::new(
                face_image_msg_rx, 
                FACE_CONFIG.functional.wifi_ip.clone(), 
                DEVICE_TYPE_FACE);
            let face_image_stream_request_tx = face_image_stream.get_request_tx();
            let mut face_image_stream_response_rx = face_image_stream.get_response_rx();

            let global_req_tx = serial.get_request_tx();
            let global_write_tx = serial.get_write_tx();
            let mut global_resp_rx = serial.get_response_rx();
            let mut global_msg_rx = serial.get_message_rx();
            std::thread::spawn(move || {
                serial.start();
            });
            std::thread::spawn(move || {
                face_image_stream.start();
            });
            std::thread::spawn(move || {
                loop {
                    face_image_stream_request_tx.send(websocket::image_msg::ImageRequest::GetImageOpenCV);
                    if let Ok(websocket::image_msg::ImageResponse::OpenCVImageData(data)) = face_image_stream_response_rx.try_recv() {
                        info!("Received image data :{} {}", data.cols(), data.rows());
                    }
                }
            });
            app.manage(Mutex::new(global_msg_rx));
            app.manage(global_req_tx);
            app.manage(Mutex::new(global_resp_rx));
            app.manage(global_write_tx);
            
            info!("Application initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_for_updates,
            restart_esp32,
            flash_esp32,
            write_wifi_info
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}