// integration/video_commands.rs

use std::sync::{Arc, Mutex};
use tauri::{command, State};
use crate::websocket::manager::VideoStreamManager;
use crate::websocket::DeviceType;

type VideoManagerState = Arc<Mutex<VideoStreamManager>>;

#[command]
pub fn update_stream_ip(state: State<VideoManagerState>, device_type_str: String, ip: String) -> Result<(), String> {
    let device_type = match device_type_str.as_str() {
        "face" => DeviceType::Face,
        "left_eye" => DeviceType::LeftEye,
        "right_eye" => DeviceType::RightEye,
        _ => DeviceType::Unknown,
    };
    
    if device_type == DeviceType::Unknown {
        return Err(format!("Invalid device type: {}", device_type_str));
    }
    
    let manager = state.lock().unwrap();
    manager.update_device_ip(device_type, ip)
}

#[command]
pub fn is_device_connected(state: State<VideoManagerState>, device_type_str: String) -> bool {
    let device_type = match device_type_str.as_str() {
        "face" => DeviceType::Face,
        "left_eye" => DeviceType::LeftEye,
        "right_eye" => DeviceType::RightEye,
        _ => DeviceType::Unknown,
    };
    
    if device_type == DeviceType::Unknown {
        return false;
    }
    
    let manager = state.lock().unwrap();
    manager.is_connected(device_type)
}

#[command]
pub fn get_device_status(state: State<VideoManagerState>, device_type_str: String) -> Result<(bool, String, Option<f32>, Option<i32>), String> {
    let device_type = match device_type_str.as_str() {
        "face" => DeviceType::Face,
        "left_eye" => DeviceType::LeftEye,
        "right_eye" => DeviceType::RightEye,
        _ => DeviceType::Unknown,
    };
    
    if device_type == DeviceType::Unknown {
        return Err(format!("Invalid device type: {}", device_type_str));
    }
    
    let manager = state.lock().unwrap();
    let client = manager.get_client(device_type);
    client.check_status()
}