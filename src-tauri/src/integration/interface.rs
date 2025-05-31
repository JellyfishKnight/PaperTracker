use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, Emitter, Manager, Runtime};
use crossbeam::channel::{Receiver, Sender};
use std::sync::{mpsc::TryRecvError, Mutex};
use crate::{serial::serial_msg::{self, FlashCommand, SerialRequest, SerialResponse, SerialSendPacket, WifiConfig}, websocket::image_msg::{ImageRequest, ImageResponse, StreamSettingRequest, StreamSettingResponse}};
use ftlog::*;

use super::init::{ImageStreamState, SerialState};



#[tauri::command]
pub async fn restart_esp32<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    let state = app.state::<SerialState>();
    let request_tx = state.global_req_tx.clone();
    let mut response_rx = state.global_resp_rx.lock().unwrap();
    if let Err(e) = request_tx.send(SerialRequest::GetStatus) {
        return Err(format!("Failed to send get status request to ESP32: {}", e));
    }
    let start_restart_time = std::time::Instant::now();
    loop {
        match response_rx.try_recv() {
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
            Err(TryRecvError::Disconnected) => {
                return Err("软件内部错误，请重启应用".to_string());
            }
            _ => ()
        }
        // 检查是否超过5秒
        if start_restart_time.elapsed().as_secs() > 5 {
            return Err("ESP32设备未响应，请检查连接".to_string());
        }
    }
    #[cfg(target_os = "macos")]
    {
        match app.path().resolve("assets/esptool", tauri::path::BaseDirectory::Resource) {
            Ok(tool_path) => {
                let tool_path = tool_path.to_str().unwrap().to_string();
                if let Err(e) = request_tx.send(SerialRequest::Restart(tool_path)) {
                    return Err(format!("Failed to send restart request to ESP32: {}", e));
                }
            }
            Err(_) => {
                return Err("软件不完整，请重新安装".to_string());
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        match app.path().resolve("assets/esptool.exe", tauri::path::BaseDirectory::Resource) {
            Ok(tool_path) => {
                let tool_path = crate::utils::platform::normalize_windows_path(tool_path.to_str().unwrap()).to_string();
                if let Err(e) = request_tx.send(SerialRequest::Restart(tool_path)) {
                    return Err(format!("Failed to send restart request to ESP32: {}", e));
                }
            }
            Err(_) => {
                return Err("软件不完整，请重新安装".to_string());
            }
        }
    }

    let start_restart_time = std::time::Instant::now();
    loop {
        match response_rx.try_recv() {
            Ok(SerialResponse::Restart(status, msg)) => {
                if status {
                    info!("ESP32 restarted successfully");
                } else {
                    return Err(format!("重启ESP32失败: {}", msg));
                }
                break;
            }
            Err(TryRecvError::Disconnected) => {
                return Err("Failed to receive response from disconnected channel".to_string());
            }
            _ => ()
        }
        // 检查是否超过5秒
        if start_restart_time.elapsed().as_secs() > 10 {
            return Err("ESP32设备未响应，请检查连接".to_string());
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn flash_esp32<R: Runtime>(app: tauri::AppHandle<R>, device_type: i32) -> Result<(), String> {
    let state = app.state::<SerialState>();
    let request_tx = state.global_req_tx.clone();
    let mut response_rx = state.global_resp_rx.lock().unwrap();
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
    #[cfg(target_os = "macos")]
    let tool_path = app.path().resolve("assets/esptool", tauri::path::BaseDirectory::Resource);
    #[cfg(target_os = "windows")]
    let tool_path = app.path().resolve("assets/esptool.exe", tauri::path::BaseDirectory::Resource);
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
    if let Err(e) = request_tx.send(SerialRequest::Flash(FlashCommand {
        #[cfg(target_os = "macos")]
        tool_path: tool_path.unwrap().to_str().unwrap().to_string(),
        #[cfg(target_os = "windows")]
        tool_path: crate::utils::platform::normalize_windows_path(tool_path.unwrap().as_path().to_str().unwrap()).to_string(),
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
    let state = app.state::<SerialState>().clone();
    let write_tx = state.global_write_tx.clone();
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
        serial: bool,
        ip: String,
        battery: f32,
        brightness: i32,
        device_type: i32,
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
pub fn start_left_eye_image_stream<R: Runtime>(
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
pub fn start_right_eye_image_stream<R: Runtime>(
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
pub fn set_brightness(
    app: tauri::AppHandle<impl Runtime>, 
    brightness: u8
) -> Result<(), String> {
    let state = app.state::<SerialState>();
    let write_tx = state.global_write_tx.clone();
    if let Err(e) = write_tx.send(SerialSendPacket::Brightness(brightness as i32)) {
        return Err(format!("Failed to send brightness request to ESP32: {}", e));
    }
    info!("Brightness set to {}", brightness);
    Ok(())
}


#[tauri::command]
pub fn set_rotation(
    app: tauri::AppHandle<impl Runtime>, 
    rotation: f64,
    device_type: i32
) -> Result<(), String> {
    let state = app.state::<ImageStreamState>();
    let send_tx = match device_type {
        1 => state.face_setting_req.clone(),
        2 => state.left_eye_setting_req.clone(),
        3 => state.right_eye_setting_req.clone(),
        _ => return Err("Invalid device type".to_string()),
    };
    if let Err(e) = send_tx.send(StreamSettingRequest::SetRotateAngle(rotation)) {
        return Err(format!("Failed to send rotation request: {}", e));
    }
    Ok(())
}