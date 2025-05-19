use std::{os::unix::process::CommandExt, process::Command};

pub fn restart_esp32(tool_path: String, port: String) -> std::io::Result<()> {
    let mut binding = std::process::Command::new(tool_path);
    let command = binding
        .args(["--port", port.as_str()])
        .arg("run");

    let output = command.output()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Output: {}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error: {}", stderr);
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
        println!("Output: {}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error: {}", stderr);
    }
    std::io::Result::Ok(())
}

pub fn find_esp32_port() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        let ports = match serialport::available_ports() {
            Ok(ports) => ports,
            Err(_) => return None,
        };
        for port in ports {
            if let serialport::SerialPortType::UsbPort(usb_info) = port.port_type {
                // Filter by the specific VID:PID of the ESP32 device
                if usb_info.vid == 0x303a && usb_info.pid == 0x1001 {
                    return Some(port.port_name);
                }
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        // Windows implementation would go here
        // For now, just return None
    }
    
    None
}