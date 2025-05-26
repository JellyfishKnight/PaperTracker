use serde::Serialize;
use tauri::{Manager, Runtime, AppHandle, ipc::Channel};
use crossbeam::channel::{Receiver, Sender};
use std::sync::{mpsc::TryRecvError, Mutex};
use crate::{serial::serial_msg::{FlashCommand, SerialRequest, SerialResponse, SerialSendPacket, WifiConfig}, websocket::image_msg::{ImageRequest, ImageResponse}};
use ftlog::*;

use super::init::ImageStreamState;


#[tauri::command]
pub async fn restart_esp32<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    let request_tx = app.state::<Sender<SerialRequest>>().clone();

    let state = app.state::<Mutex<bus::BusReader<SerialResponse>>>();
    let mut response_rx = state.lock().unwrap();
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

#[tauri::command]
pub async fn write_wifi_info<R: Runtime>(app: tauri::AppHandle<R>, ssid: String, password: String) -> Result<(), String> {
    let write_tx = app.state::<Sender<SerialSendPacket>>().clone();
    if let Err(e) = write_tx.send(SerialSendPacket::WifiConfig(WifiConfig {
        ssid,
        password,
    })) {
        return Err(format!("Failed to send wifi config request to ESP32: {}", e));
    }
    Ok(())
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum StreamEvent {
    Image {
        base64: String,
        device: String,
    },
    Status {
        wifi: String,
        serial: String,
        ip: String,
        battery: u8,
        brightness: u8,
    },
    Log {
        message: String,
    },
}

#[tauri::command]
pub fn start_face_image_stream<R: Runtime>(
    app: tauri::AppHandle<R>, 
    on_event: Channel<StreamEvent>
) {
    info!("Starting Face Image Stream");
    
    // 发送初始连接消息
    on_event.send(StreamEvent::Log {
        message: "Face image stream connected".to_string()
    }).unwrap();
    
    let state = app.state::<ImageStreamState>();
    let face_stream_req = state.face_stream_req.clone();
    let face_stream_resp = state.face_stream_resp.clone();
    
    std::thread::spawn(move || {
        let mut face_stream_resp = face_stream_resp.lock().unwrap();
        
        loop {
            if let Err(e) = face_stream_req.send(ImageRequest::GetImageBase64) {
                error!("Failed to send face image request: {}", e);
                on_event.send(StreamEvent::Log {
                    message: format!("Failed to send request: {}", e)
                }).ok();
                continue;
            }
            
            match face_stream_resp.try_recv() {
                Ok(ImageResponse::Base64ImageData(data)) => {
                    // 将 Vec<u8> 转换回 base64 字符串
                    let base64_string = String::from_utf8(data).unwrap_or_else(|_| {
                        error!("Invalid UTF-8 in base64 data");
                        String::new()
                    });
                    
                    // 发送图像事件
                    on_event.send(StreamEvent::Image {
                        base64: base64_string,
                        device: "face".to_string(),
                    }).unwrap();
                }
                Err(TryRecvError::Disconnected) => {
                    error!("Failed to receive data from disconnected channel");
                    break;
                }
                _ => ()
            }
            
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });
}


#[tauri::command]
pub async fn start_left_eye_image_stream<R: Runtime>(
    app: tauri::AppHandle<R>, 
    on_event: Channel<StreamEvent>
) {
    info!("Starting Left Eye Image Stream");
    
    // 发送初始连接消息
    on_event.send(StreamEvent::Log {
        message: "Left Eye image stream connected".to_string()
    }).unwrap();

    let state = app.state::<ImageStreamState>();
    let left_eye_stream_req = state.left_eye_stream_req.clone();
    let left_eye_stream_resp = state.left_eye_stream_resp.clone();
    std::thread::spawn(move || {
        let mut left_eye_stream_resp = left_eye_stream_resp.lock().unwrap();
        loop {
            if let Err(e) = left_eye_stream_req.send(ImageRequest::GetImageBase64) {
                error!("Failed to send left eye image request: {}", e);
                on_event.send(StreamEvent::Log {
                    message: format!("Failed to send request: {}", e)
                }).ok();
                continue;
            }
            match left_eye_stream_resp.try_recv() {
                Ok(ImageResponse::Base64ImageData(data)) => {
                    // 将 Vec<u8> 转换回 base64 字符串
                    let base64_string = String::from_utf8(data).unwrap_or_else(|_| {
                        error!("Invalid UTF-8 in base64 data");
                        String::new()
                    });
                    
                    // 发送图像事件
                    on_event.send(StreamEvent::Image {
                        base64: base64_string,
                        device: "left_eye".to_string(),
                    }).unwrap();
                }
                Err(TryRecvError::Disconnected) => {
                    error!("Failed to receive data from disconnected channel");
                    break;
                }
                _ => ()
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });
}

#[tauri::command]
pub async fn start_right_eye_image_stream<R: Runtime>(
    app: tauri::AppHandle<R>, 
    on_event: Channel<StreamEvent>
) {
    let state = app.state::<ImageStreamState>();
    let right_eye_stream_req = state.right_eye_stream_req.clone();
    let right_eye_stream_resp = state.right_eye_stream_resp.clone();
    std::thread::spawn(move || {
        let mut right_eye_stream_resp = right_eye_stream_resp.lock().unwrap();
        loop {
            if let Err(e) = right_eye_stream_req.send(ImageRequest::GetImageBase64) {
                error!("Failed to send right eye image request: {}", e);
                on_event.send(StreamEvent::Log {
                    message: format!("Failed to send request: {}", e)
                }).ok();
                continue;
            }
            match right_eye_stream_resp.try_recv() {
                Ok(ImageResponse::Base64ImageData(data)) => {
                    // 将 Vec<u8> 转换回 base64 字符串
                    let base64_string = String::from_utf8(data).unwrap_or_else(|_| {
                        error!("Invalid UTF-8 in base64 data");
                        String::new()
                    });
                    
                    // 发送图像事件
                    on_event.send(StreamEvent::Image {
                        base64: base64_string,
                        device: "right_eye".to_string(),
                    }).unwrap();
                }
                Err(TryRecvError::Disconnected) => {
                    error!("Failed to receive data from disconnected channel");
                    break;
                }
                _ => ()
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });
}

#[tauri::command]
pub async fn set_brightness(
    app: tauri::AppHandle<impl Runtime>, 
    brightness: u8
) -> Result<(), String> {
    let write_tx = app.state::<Sender<SerialSendPacket>>().clone();
    if let Err(e) = write_tx.send(SerialSendPacket::Brightness(brightness as i32)) {
        return Err(format!("Failed to send brightness request to ESP32: {}", e));
    }
    Ok(())
}
