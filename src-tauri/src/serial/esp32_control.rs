#[cfg(target_os = "macos")]
use std::{os::unix::process::CommandExt};
#[cfg(target_os = "windows")]
use std::{os::windows::process::CommandExt, process::Stdio};

use ftlog::*;

pub fn restart_esp32(tool_path: String, port: String) -> std::io::Result<()> {
    let mut binding = std::process::Command::new(tool_path);
    let command = binding
        .arg("--port")
        .arg(port.as_str())
        .arg("run");
    info!("Restarting ESP32 with command: {:?}", command);
    let output = command.output()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        info!("Output: {}", stdout);
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Error: {}", stdout),
        ));
    } 
    std::io::Result::Ok(())
}


pub fn flash_esp32(tool_path: String, boot_loader_path: String, partition_path: String, firmware_path: String, port: String) -> std::io::Result<()> {
    let mut binding = std::process::Command::new(tool_path);
    let command = binding
        .args(["--chip", "ESP32-S3"])
        .args(["--port", port.as_str()])
        .args(["--baud", "921600"])
        .args(["--before", "default_reset"])
        .args(["--after", "hard_reset"])
        .arg(std::format!("write_flash 0x0000 {} 0x8000 {} 0x10000 {}", boot_loader_path, partition_path, firmware_path));

    let output = command.output()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        info!("Output: {}", stdout);
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        error!("Error: {}", stdout);
    }
    std::io::Result::Ok(())
}

pub fn find_esp32_port() -> Option<String> {    
    let ports = match serialport::available_ports() {
        Ok(ports) => ports,
        Err(_) => return None,
    };
    // ESP32-S3的USB VID和PID
    const ESP32_S3_VID: u16 = 0x303A;  // Espressif Systems
    const ESP32_S3_PID: u16 = 0x1001;  // ESP32-S3

    // 查找匹配的设备
    for port in ports {
        if let serialport::SerialPortType::UsbPort(usb_info) = &port.port_type {
            debug!(
                "检查USB设备 {}: VID={:04X}, PID={:04X}",
                port.port_name,
                usb_info.vid,
                usb_info.pid
            );

            // 检查VID和PID是否匹配ESP32-S3
            if usb_info.vid == ESP32_S3_VID && usb_info.pid == ESP32_S3_PID {
                info!("找到ESP32-S3设备的COM端口: {}", port.port_name);
                return Some(port.port_name);
            }
        }
    }
    
    None
}