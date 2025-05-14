use tauri::{Window, State, Emitter};
use std::sync::{Arc, Mutex};
use crate::serial::{SerialClient, SerialEvent};

// Shared state for serial communication
pub struct SerialState {
    pub client: Arc<Mutex<SerialClient>>,
}

impl SerialState {
    pub fn new() -> Self {
        SerialState {
            client: Arc::new(Mutex::new(SerialClient::new())),
        }
    }
}

// Tauri command to write WiFi SSID and password
#[tauri::command]
pub fn write_ssid_and_password(
    ssid: String,
    password: String,
    state: State<'_, SerialState>,
) -> Result<(), String> {
    let client = state.client.lock().unwrap();
    client.send_wifi_config(ssid, password)
}

// Tauri command to set brightness
#[tauri::command]
pub fn write_brightness(
    brightness: u32,
    state: State<'_, SerialState>,
) -> Result<(), String> {
    let client = state.client.lock().unwrap();
    client.set_brightness(brightness)
}

// Tauri command to restart ESP32
#[tauri::command]
pub fn restart_esp32(
    window: Window,
    state: State<'_, SerialState>,
) -> Result<(), String> {
    // Create a clone of the window to use in the event handler
    let window_clone = window.clone();
    
    // Set up event forwarding
    let client = state.client.lock().unwrap();
    let event_rx = client.get_event_receiver();
    
    std::thread::spawn(move || {
        while let Ok(event) = event_rx.recv() {
            match event {
                SerialEvent::RestartProgress { progress, message } => {
                    let _ = window_clone.emit("esp32_operation", serde_json::json!({
                        "progress": progress,
                        "message": message,
                        "status": if progress >= 100.0 { "success" } else { "running" },
                    }));
                }
                SerialEvent::Error(error) => {
                    let _ = window_clone.emit("esp32_operation", serde_json::json!({
                        "progress": 100.0,
                        "message": error,
                        "status": "error",
                    }));
                }
                _ => {}
            }
        }
    });
    
    // Send restart command
    client.restart_device()
}

// Tauri command to flash ESP32 firmware
#[tauri::command]
pub fn flash_esp32(
    window: Window,
    device_type: String,
    firmware_type: String,
    firmware_path: Option<String>,
    state: State<'_, SerialState>,
) -> Result<(), String> {
    // Create a clone of the window to use in the event handler
    let window_clone = window.clone();
    
    // Set up event forwarding
    let client = state.client.lock().unwrap();
    let event_rx = client.get_event_receiver();
    
    std::thread::spawn(move || {
        while let Ok(event) = event_rx.recv() {
            match event {
                SerialEvent::FlashProgress { progress, message } => {
                    let _ = window_clone.emit("esp32_operation", serde_json::json!({
                        "progress": progress,
                        "message": message,
                        "status": if progress >= 100.0 { "success" } else { "running" },
                    }));
                }
                SerialEvent::Error(error) => {
                    let _ = window_clone.emit("esp32_operation", serde_json::json!({
                        "progress": 100.0,
                        "message": error,
                        "status": "error",
                    }));
                }
                _ => {}
            }
        }
    });
    
    // Send flash command
    client.flash_firmware(device_type, firmware_type, firmware_path)
}