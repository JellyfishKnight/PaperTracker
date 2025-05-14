use tauri::{Manager, AppHandle, Runtime, WebviewWindow, command, Window, Emitter};
use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use regex::Regex;
use super::global::ESP32_SERIAL;
use super::esp32::{Esp32Status, find_esp32_port};

// 进度更新结构体
#[derive(Clone, serde::Serialize)]
struct ProgressUpdate {
    progress: f64,
    message: String,
    status: String,
}

#[tauri::command]
pub fn restart_esp32<R: Runtime>(window: Window<R>) -> Result<(), String> {
    // 检查ESP32设备是否连接
    let port = match find_esp32_port() {
        Some(port) => port,
        None => {
            let _ = window.emit("esp32_operation", ProgressUpdate {
                progress: 0.0,
                message: "ESP32设备未连接".to_string(),
                status: "error".to_string(),
            });
            return Err("ESP32设备未连接".to_string());
        }
    };

    // 获取esptool路径
    let app_handle = window.app_handle();
    let esp_tool_path = match app_handle.path().resolve("assets/esptool", tauri::path::BaseDirectory::Resource) {
        Ok(path) => path,
        Err(_) => return Err("无法解析ESP TOOL资源路径".to_string()),
    };
    
    // 创建进度更新闭包
    let progress_sender = move |progress: f64, message: &str, status: &str| {
        let _ = window.emit("esp32_operation", ProgressUpdate {
            progress,
            message: message.to_string(),
            status: status.to_string(),
        });
    };

    // 发送初始进度
    progress_sender(5.0, "正在准备重启设备...", "running");

    // 在新线程中执行重启操作
    thread::spawn(move || {
        progress_sender(10.0, "正在释放串口资源...", "running");

        // 先关闭现有串口连接
        let mut esp32_serial_lock = ESP32_SERIAL.lock().unwrap();
        if let Some(serial) = esp32_serial_lock.as_mut() {
            let _ = serial.close();
        }
        *esp32_serial_lock = None;
        drop(esp32_serial_lock);

        progress_sender(30.0, "正在执行重启命令...", "running");

        // 构建重启命令
        let mut command = if cfg!(target_os = "windows") {
            let mut cmd = Command::new(&esp_tool_path);
            cmd.args(["--chip", "ESP32-S3", "--port", &port, "--baud", "921600", "run"]);
            cmd
        } else {
            let mut cmd = Command::new(&esp_tool_path);
            cmd.args(["--chip", "ESP32-S3", "--port", &port, "--baud", "921600", "run"]);
            cmd
        };

        // 设置标准输出和错误输出
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // 执行命令
        match command.spawn() {
            Ok(mut child) => {
                // 读取输出
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines().map_while(Result::ok) {
                        progress_sender(50.0, &format!("设备输出: {}", line), "running");
                    }
                }

                // 等待命令完成
                match child.wait() {
                    Ok(status) => {
                        if status.success() {
                            progress_sender(100.0, "ESP32设备重启成功!", "success");
                        } else {
                            progress_sender(100.0, &format!("ESP32设备重启失败，退出码: {:?}", status.code()), "error");
                        }
                    },
                    Err(e) => {
                        progress_sender(100.0, &format!("等待命令完成时出错: {}", e), "error");
                    }
                }
            },
            Err(e) => {
                progress_sender(100.0, &format!("启动命令失败: {}", e), "error");
            }
        }

        // 重新初始化串口
        // super::esp32::start_serial_mod();
    });

    Ok(())
}

#[tauri::command]
pub fn flash_esp32<R: Runtime>(
    window: Window<R>, 
    device_type: String, 
    firmware_type: String,
    firmware_path: Option<String>
) -> Result<(), String> {
    // 检查ESP32设备是否连接
    let port = match find_esp32_port() {
        Some(port) => port,
        None => {
            let _ = window.emit("esp32_operation", ProgressUpdate {
                progress: 0.0,
                message: "ESP32设备未连接".to_string(),
                status: "error".to_string(),
            });
            return Err("ESP32设备未连接".to_string());
        }
    };

    // 获取资源路径
    let esp_tool_path = match window.app_handle().path().resolve("assets/esptool", tauri::path::BaseDirectory::Resource) {
        Ok(path) => path,
        Err(_) => return Err("无法解析ESP TOOL资源路径".to_string()),
    };

    let bootloader_path = match window.app_handle().path().resolve("assets/bootloader.bin", tauri::path::BaseDirectory::Resource) {
        Ok(path) => path,
        Err(_) => return Err("无法解析bootloader资源路径".to_string()),
    };

    let partition_path = match window.app_handle().path().resolve("assets/partition-table.bin", tauri::path::BaseDirectory::Resource) {
        Ok(path) => path,
        Err(_) => return Err("无法解析partition-table资源路径".to_string()),
    };
    // 确定固件路径
    let firmware_path = if let Some(custom_path) = firmware_path {
        // 使用用户提供的自定义固件路径
        custom_path
    } else {
        // 根据设备类型选择默认固件
        let firmware_filename = match device_type.as_str() {
            "face" => "face_tracker.bin",
            "left_eye" => "left_eye.bin",
            "right_eye" => "right_eye.bin",
            _ => "face_tracker.bin", // 默认为面捕固件
        };

        // 如果是测试版固件，添加beta前缀
        let firmware_filename = if firmware_type == "beta" {
            format!("beta_{}", firmware_filename)
        } else {
            firmware_filename.to_string()
        };

        // 解析固件路径
        match window.app_handle().path().resolve(format!("assets/{}", firmware_filename), tauri::path::BaseDirectory::Resource) {
            Ok(path) => path.to_string_lossy().to_string(),
            Err(_) => return Err(format!("无法解析固件资源路径: {}", firmware_filename)),
        }
    };

    // 创建进度更新闭包
    let progress_sender = move |progress: f64, message: &str, status: &str| {
        let _ = window.emit("esp32_operation", ProgressUpdate {
            progress,
            message: message.to_string(),
            status: status.to_string(),
        });
    };

    // 发送初始进度
    progress_sender(5.0, "正在准备刷写固件...", "running");

    // 在新线程中执行刷写操作
    thread::spawn(move || {
        progress_sender(10.0, "正在释放串口资源...", "running");

        // 先关闭现有串口连接
        let mut esp32_serial_lock = ESP32_SERIAL.lock().unwrap();
        if let Some(serial) = esp32_serial_lock.as_mut() {
            let _ = serial.close();
        }
        *esp32_serial_lock = None;
        drop(esp32_serial_lock);

        progress_sender(20.0, "正在执行固件刷写命令...", "running");

        // 构建刷写命令
        let mut command = if cfg!(target_os = "windows") {
            let mut cmd = Command::new(&esp_tool_path);
            cmd.args([
                "--chip", "ESP32-S3",
                "--port", &port,
                "--baud", "921600",
                "--before", "default_reset",
                "--after", "hard_reset",
                "write_flash",
                "0x0000", &bootloader_path.to_string_lossy(),
                "0x8000", &partition_path.to_string_lossy(),
                "0x10000", &firmware_path
            ]);
            cmd
        } else {
            let mut cmd = Command::new(&esp_tool_path);
            cmd.args([
                "--chip", "ESP32-S3",
                "--port", &port,
                "--baud", "921600",
                "--before", "default_reset",
                "--after", "hard_reset",
                "write_flash",
                "0x0000", &bootloader_path.to_string_lossy(),
                "0x8000", &partition_path.to_string_lossy(),
                "0x10000", &firmware_path
            ]);
            cmd
        };

        // 设置标准输出和错误输出
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // 执行命令
        match command.spawn() {
            Ok(mut child) => {
                // 获取输出流
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    
                    // 创建一个正则表达式来匹配进度
                    let re = Regex::new(r"(\d+)%").unwrap();
                    
                    for line in reader.lines().map_while(Result::ok) {
                        // 检查是否包含进度信息
                        if let Some(cap) = re.captures(&line) {
                            if let Some(percent_match) = cap.get(1) {
                                if let Ok(percent) = percent_match.as_str().parse::<u32>() {
                                    // 计算总进度：20% 基础进度 + 刷写进度的75%
                                    let total_progress = 20.0 + (percent as f64 * 0.75);
                                    progress_sender(total_progress, &format!("刷写进度: {}%", percent), "running");
                                }
                            }
                        } else {
                            progress_sender(30.0, &format!("设备输出: {}", line), "running");
                        }
                    }
                }

                // 等待命令完成
                match child.wait() {
                    Ok(status) => {
                        if status.success() {
                            progress_sender(100.0, "ESP32固件刷写成功!", "success");
                        } else {
                            progress_sender(100.0, &format!("ESP32固件刷写失败，退出码: {:?}", status.code()), "error");
                        }
                    },
                    Err(e) => {
                        progress_sender(100.0, &format!("等待命令完成时出错: {}", e), "error");
                    }
                }
            },
            Err(e) => {
                progress_sender(100.0, &format!("启动命令失败: {}", e), "error");
            }
        }

        // 重新初始化串口
        super::esp32::start_serial_mod();
    });

    Ok(())
}