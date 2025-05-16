// integration/serial_commands.rs

use std::sync::{Arc, Mutex};
use tauri::{command, AppHandle, Manager, State};
use crate::serial::SerialClient;
use super::SerialState;

#[command]
pub fn write_ssid_and_password(state: State<SerialState>, ssid: String, password: String) -> Result<(), String> {
    let client = state.client.lock().unwrap();
    client.send_wifi_config(ssid, password)
}

#[command]
pub fn write_brightness(state: State<SerialState>, brightness: u32) -> Result<(), String> {
    let client = state.client.lock().unwrap();
    client.set_brightness(brightness)
}

#[command]
pub fn restart_esp32(app: AppHandle) -> Result<(), String> {
    let binding = app.state::<SerialState>();
    let client = binding.client.lock().unwrap();
    if let Ok(tool_path) = app.path().resolve("assets/esptool", tauri::path::BaseDirectory::Resource) {
        client.restart_device(tool_path.to_string_lossy().to_string())
    } else {
        Err("Failed to resolve esp tool path".to_string())
    }
}

#[command]
pub fn flash_esp32(
    app: AppHandle, 
    device_type: String, 
    firmware_type: String
) -> Result<(), String> {
    let binding = app.state::<SerialState>();
    let client = binding.client.lock().unwrap();
    let tool_path = app.path().resolve("assets/esptool", tauri::path::BaseDirectory::Resource);
    if let Ok(tool_path) = tool_path {
        let tool_path_str = tool_path.to_string_lossy().to_string();
        if let Ok(firmware_path) = app.path().resolve("assets/firmware", tauri::path::BaseDirectory::Resource) {
            let firmware_path_str = firmware_path.to_string_lossy().to_string();
            client.flash_firmware(tool_path_str, device_type, firmware_type, firmware_path_str)
        } else {
            Err("Failed to resolve firmware path".to_string())
        }
    } else {
        Err("Failed to resolve esp tool path".to_string())
    }
}