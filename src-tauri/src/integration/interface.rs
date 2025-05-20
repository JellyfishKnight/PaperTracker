use tauri::{Manager, Runtime, AppHandle};
use crossbeam::channel::{Sender, Receiver};
use std::sync::Mutex;
use crate::serial::serial_msg::{FlashCommand, SerialRequest, SerialResponse};
use ftlog::*;


#[tauri::command]
pub async fn restart_esp32<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    info!("-------------------------------Rstarting ESP32-------------------------------");
    let request_tx = app.state::<Sender<SerialRequest>>().clone();

    let state = app.state::<Mutex<bus::BusReader<SerialResponse>>>();
    let mut response_rx = state.lock().unwrap();
    info!("-------------------------------Getting Status-------------------------------");
    if let Err(e) = request_tx.send(SerialRequest::GetStatus) {
        return Err(format!("Failed to send get status request to ESP32: {}", e));
    }
    loop {
        match response_rx.recv() {
            Ok(SerialResponse::Status((state, _))) => {
                match state {
                    crate::serial::serial_msg::PortState::Disconnected => {
                        return Err("ESP32设备未连接".to_string());
                    }
                    crate::serial::serial_msg::PortState::Connected => {
                        break;
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                return Err(format!("Failed to receive response from ESP32: {}", e));
            }
        }
    }
    info!("-------------------------------Getting Tools-------------------------------");
    match app.path().resolve("assets/esptool", tauri::path::BaseDirectory::Resource) {
        Ok(tool_path) => {
            let tool_path = tool_path.to_str().unwrap().to_string();
            if let Err(e) = request_tx.send(SerialRequest::Restart(tool_path)) {
                return Err(format!("Failed to send restart request to ESP32: {}", e));
            }
        }
        Err(e) => {
            return Err(format!("Failed to resolve tool path: {}", e));
        }
    }
    info!("-------------------------------Waiting for Results-------------------------------");
    loop {
        match response_rx.recv() {
            Ok(SerialResponse::Restart(status, msg)) => {
                if status {
                    info!("ESP32 restarted successfully");
                } else {
                    return Err(format!("Failed to restart ESP32: {}", msg));
                }
                break;
            }
            Ok(_) => {}
            Err(e) => {
                return Err(format!("Failed to receive response from ESP32: {}", e));
            }
        }
    }
    info!("-------------------------------Finished-------------------------------");
    Ok(())
}

#[tauri::command]
pub async fn flash_esp32<R: Runtime>(app: tauri::AppHandle<R>, device_type: i32) -> Result<(), String> {
    let tool_path = app.path().resolve("assets/esptool", tauri::path::BaseDirectory::Resource);
    if tool_path.is_err() {
        return Err("Failed to resolve tool path".to_string());
    }
    let bootloader_path = app.path().resolve("assets/bootloader.bin", tauri::path::BaseDirectory::Resource);
    if bootloader_path.is_err() {
        return Err("Failed to resolve bootloader path".to_string());
    }
    let partition_path = app.path().resolve("assets/partition-table.bin", tauri::path::BaseDirectory::Resource);
    if partition_path.is_err() {
        return Err("Failed to resolve partition path".to_string());
    }
    let firmware_path = match device_type {
        1 => app.path().resolve("assets/face_tracker.bin", tauri::path::BaseDirectory::Resource),
        2 => app.path().resolve("assets/left_eye.bin", tauri::path::BaseDirectory::Resource),
        3 => app.path().resolve("assets/right_eye.bin", tauri::path::BaseDirectory::Resource),
        _ => return Err("Invalid device type".to_string()),
    };
    if firmware_path.is_err() {
        return Err("Failed to resolve firmware path".to_string());
    }
    let request_tx = app.state::<Sender<SerialRequest>>().clone();
    if let Err(e) = request_tx.send(SerialRequest::Flash(FlashCommand {
        tool_path: tool_path.unwrap().to_str().unwrap().to_string(),
        boot_loader_path: bootloader_path.unwrap().to_str().unwrap().to_string(),
        partition_path: partition_path.unwrap().to_str().unwrap().to_string(),
        firmware_path: firmware_path.unwrap().to_str().unwrap().to_string(),
    })) {
        return Err(format!("Failed to send flash request to ESP32: {}", e));
    }
    Ok(())
}