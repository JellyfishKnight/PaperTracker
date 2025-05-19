mod updater;
mod paper_tracker_config;
mod utils;
mod serial;
mod websocket;
mod integration;

use paper_tracker_config::config::init_config;
use tauri::Manager;
use updater::version_check::check_for_updates;
use ftlog::*;
use serial::esp32_serial;


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
            let mut msg_rx = serial.get_message_rx();
            let req_tx = serial.get_request_tx();
            let mut resp_rx = serial.get_response_rx();
            std::thread::spawn(move || {
                serial.start();
            });
        
            std::thread::spawn(move || {
                loop {
                    println!("Waiting for messages...");
                    if let Ok(msg) = msg_rx.recv() {
                        println!("Received message: {:?}", msg);
                    }
                    if let Ok(req) = req_tx.send(serial::serial_msg::SerialRequest::GetStatus) {
                        // println!("Sendf request: {:?}", req);
                    }
                    if let Ok(resp) = resp_rx.recv() {
                        println!("Received response: {:?}", resp);
                    }
                }
            });
        
            info!("Application initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_for_updates,
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}