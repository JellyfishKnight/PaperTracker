// integration/serial_commands.rs

use std::sync::{Arc, Mutex};
use tauri::{command, State};
use crate::serial::SerialClient;

type SerialClientState = Arc<Mutex<SerialClient>>;

#[command]
pub fn write_ssid_and_password(state: State<SerialClientState>, ssid: String, password: String) -> Result<(), String> {
    let client = state.lock().unwrap();
    client.send_wifi_config(ssid, password)
}

#[command]
pub fn write_brightness(state: State<SerialClientState>, brightness: u32) -> Result<(), String> {
    let client = state.lock().unwrap();
    client.set_brightness(brightness)
}

#[command]
pub fn restart_esp32(state: State<SerialClientState>) -> Result<(), String> {
    let client = state.lock().unwrap();
    client.restart_device()
}

#[command]
pub fn flash_esp32(
    state: State<SerialClientState>, 
    device_type: String, 
    firmware_type: String,
    firmware_path: Option<String>
) -> Result<(), String> {
    let client = state.lock().unwrap();
    client.flash_firmware(device_type, firmware_type, firmware_path)
}