use tauri::{Window, State, Emitter};
use std::sync::{Arc, Mutex};
use crate::websocket::manager::VideoStreamManager;
use crate::websocket::DeviceType;

// Shared state for video streams
pub struct VideoState {
    manager: Arc<Mutex<VideoStreamManager>>,
}

impl VideoState {
    pub fn new() -> Self {
        VideoState {
            manager: Arc::new(Mutex::new(VideoStreamManager::new())),
        }
    }
}

// Convert string to device type
fn string_to_device_type(device_type_str: &str) -> DeviceType {
    match device_type_str.to_lowercase().as_str() {
        "face" => DeviceType::Face,
        "lefteye" => DeviceType::LeftEye,
        "left_eye" => DeviceType::LeftEye,
        "righteye" => DeviceType::RightEye,
        "right_eye" => DeviceType::RightEye,
        _ => DeviceType::Unknown,
    }
}

// Tauri command to update device IP
#[tauri::command]
pub fn update_stream_ip(
    device_type_str: String,
    ip: String,
    state: State<'_, VideoState>,
) -> Result<(), String> {
    let device_type = string_to_device_type(&device_type_str);
    if device_type == DeviceType::Unknown {
        return Err(format!("Unknown device type: {}", device_type_str));
    }
    
    let manager = state.manager.lock().unwrap();
    manager.update_device_ip(device_type, ip)
}

// Tauri command to check if a device is connected
#[tauri::command]
pub fn is_device_connected(
    device_type_str: String,
    state: State<'_, VideoState>,
) -> Result<bool, String> {
    let device_type = string_to_device_type(&device_type_str);
    if device_type == DeviceType::Unknown {
        return Err(format!("Unknown device type: {}", device_type_str));
    }
    
    let manager = state.manager.lock().unwrap();
    Ok(manager.is_connected(device_type))
}

// Tauri command to get device status
#[tauri::command]
pub fn get_device_status(
    device_type_str: String,
    state: State<'_, VideoState>,
) -> Result<serde_json::Value, String> {
    let device_type = string_to_device_type(&device_type_str);
    if device_type == DeviceType::Unknown {
        return Err(format!("Unknown device type: {}", device_type_str));
    }
    
    let manager = state.manager.lock().unwrap();
    let client = manager.get_client(device_type);
    
    match client.check_status() {
        Ok((connected, url, battery, brightness)) => {
            Ok(serde_json::json!({
                "connected": connected,
                "url": url,
                "battery": battery,
                "brightness": brightness,
            }))
        }
        Err(e) => Err(e),
    }
}