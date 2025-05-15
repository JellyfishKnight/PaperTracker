// integration/mod.rs

pub mod serial_commands;
pub mod video_commands;
pub mod reconnect_manager;
pub mod serial_messages;
pub mod image_ui_updater;

use std::sync::{Arc, Mutex};
use crate::serial::SerialClient;
use crate::websocket::{manager::VideoStreamManager, DeviceType};
use reconnect_manager::ReconnectManager;

pub struct SerialState {
    pub client: Arc<Mutex<SerialClient>>,
}

pub struct VideoState {
    pub manager: Arc<Mutex<VideoStreamManager>>,
}

// Initialize services and start automatic reconnection without blocking
pub fn init_services() -> (Arc<Mutex<SerialClient>>, Arc<Mutex<VideoStreamManager>>) {
    // Create clients
    let serial_client = Arc::new(Mutex::new(SerialClient::new()));
    let video_manager = Arc::new(Mutex::new(VideoStreamManager::new()));

    let mut reconnect_manager: ReconnectManager;
    // Get video clients for each device type (non-blocking)
    {
        let manager = video_manager.lock().unwrap();
        let left_eye_client = manager.get_client(DeviceType::LeftEye);
        let right_eye_client = manager.get_client(DeviceType::RightEye);
        let face_client = manager.get_client(DeviceType::Face);

        serial_messages::listen_for_serial_events(
            serial_client.lock().unwrap().get_message_receiver(), 
            face_client.get_request_sender(),
            left_eye_client.get_request_sender(),
            right_eye_client.get_request_sender());

        // Create and start reconnect manager (connections happen in background threads)
        reconnect_manager = ReconnectManager::new(
            serial_client.clone(),
            left_eye_client,
            right_eye_client,
            face_client
        );
    }
    
    // Start the reconnection monitoring (non-blocking - will try connections in background)
    reconnect_manager.start();
    
    // Store the reconnect manager in a global to keep it alive
    static RECONNECT_MANAGER: std::sync::OnceLock<std::sync::Mutex<ReconnectManager>> = std::sync::OnceLock::new();
    RECONNECT_MANAGER.get_or_init(|| std::sync::Mutex::new(reconnect_manager));
    
    println!("服务初始化完成，在后台监控连接状态");
    
    // Return the state for use in the application
    (serial_client, video_manager)
}