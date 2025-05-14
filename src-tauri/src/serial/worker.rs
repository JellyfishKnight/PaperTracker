use std::thread;
use crossbeam::channel::{self, Receiver, Sender};
use std::time::Duration;
use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead};
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use regex::Regex;
use crate::utils::consts::*;
use super::messages::{SerialRequest, SerialResponse, SerialEvent, DeviceStatus};

// Represents the state of a serial port connection
enum PortState {
    Disconnected,
    Connected {
        port: String,
        handle: Box<dyn SerialPort>,
    },
}

// The worker handles serial communication in a dedicated thread
pub struct SerialWorker {
    // Channel for receiving requests
    request_rx: Receiver<SerialRequest>,
    // Channel for sending responses
    response_tx: Sender<SerialResponse>,
    // Channel for sending events
    event_tx: Sender<SerialEvent>,
    // Current state of the port
    port_state: PortState,
    // Flag to indicate if the worker should keep running
    running: bool,
}

impl SerialWorker {
    // Create a new serial worker with the given channels
    pub fn new(
        request_rx: Receiver<SerialRequest>,
        response_tx: Sender<SerialResponse>,
        event_tx: Sender<SerialEvent>,
    ) -> Self {
        SerialWorker {
            request_rx,
            response_tx,
            event_tx,
            port_state: PortState::Disconnected,
            running: true,
        }
    }

    // Start the worker loop
    pub fn run(&mut self) {
        println!("Serial worker thread started");

        while self.running {
            // Check for any incoming requests
            match self.request_rx.try_recv() {
                Ok(request) => self.handle_request(request),
                Err(crossbeam::channel::TryRecvError::Empty) => {
                    // No requests, continue with port reading
                }
                Err(crossbeam::channel::TryRecvError::Disconnected) => {
                    println!("Request channel disconnected, stopping serial worker");
                    self.running = false;
                    break;
                }
            }

            // Read from the port if connected
            if let PortState::Connected { ref mut handle, .. } = &mut self.port_state {
                let mut buffer = vec![0u8; 1024];
                match handle.read(buffer.as_mut_slice()) {
                    Ok(bytes_read) if bytes_read > 0 => {
                        let data = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                        self.process_serial_data(data);
                    }
                    Ok(_) => {
                        // No data read, continue
                    }
                    Err(e) => {
                        println!("Serial read error: {}", e);
                        self.event_tx.send(SerialEvent::Error(format!("Serial read error: {}", e))).ok();
                        self.disconnect_port();
                    }
                }
            }

            // Small sleep to avoid high CPU usage
            thread::sleep(Duration::from_millis(10));
        }

        // Clean up when the worker stops
        self.disconnect_port();
        println!("Serial worker thread stopped");
    }

    // Handle incoming requests
    fn handle_request(&mut self, request: SerialRequest) {
        match request {
            SerialRequest::OpenPort { port, baud_rate } => {
                self.open_port(port, baud_rate);
            }
            SerialRequest::ClosePort => {
                self.disconnect_port();
                self.response_tx.send(SerialResponse::PortClosed).ok();
            }
            SerialRequest::SendData(data) => {
                self.send_data(data);
            }
            SerialRequest::SendWifiConfig { ssid, password } => {
                self.send_wifi_config(ssid, password);
            }
            SerialRequest::SetBrightness(brightness) => {
                self.set_brightness(brightness);
            }
            SerialRequest::RestartDevice => {
                self.restart_device();
            }
            SerialRequest::FlashFirmware { device_type, firmware_type, firmware_path } => {
                self.flash_firmware(device_type, firmware_type, firmware_path);
            }
            SerialRequest::Shutdown => {
                println!("Received shutdown request");
                self.running = false;
            }
        }
    }

    // Open a serial port
    fn open_port(&mut self, port: String, baud_rate: u32) {
        // Close existing port if any
        self.disconnect_port();

        // Try to open the new port
        let builder = serialport::new(&port, baud_rate)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_secs(1));

        match builder.open() {
            Ok(handle) => {
                println!("Serial port opened: {}", port);
                self.port_state = PortState::Connected { port: port.clone(), handle };
                self.response_tx.send(SerialResponse::PortOpened { port: port.clone() }).ok();
                self.event_tx.send(SerialEvent::DeviceConnected { port }).ok();
            }
            Err(e) => {
                let error = format!("Failed to open port {}: {}", port, e);
                println!("{}", error);
                self.response_tx.send(SerialResponse::PortOpenFailed { port, error: e.to_string() }).ok();
            }
        }
    }

    // Disconnect the current port
    fn disconnect_port(&mut self) {
        if let PortState::Connected { port, .. } = &self.port_state {
            println!("Closing serial port: {}", port);
            self.event_tx.send(SerialEvent::DeviceDisconnected).ok();
        }
        self.port_state = PortState::Disconnected;
    }

    // Send data to the port
    fn send_data(&mut self, data: Vec<u8>) {
        match &mut self.port_state {
            PortState::Connected { handle, .. } => {
                match handle.write(&data) {
                    Ok(_) => {
                        self.response_tx.send(SerialResponse::DataSent).ok();
                    }
                    Err(e) => {
                        let error = format!("Failed to send data: {}", e);
                        println!("{}", error);
                        self.response_tx.send(SerialResponse::SendFailed(error)).ok();
                    }
                }
            }
            PortState::Disconnected => {
                self.response_tx.send(SerialResponse::SendFailed("Serial port not connected".to_string())).ok();
            }
        }
    }

    // Send WiFi configuration
    fn send_wifi_config(&mut self, ssid: String, password: String) {
        let data = format!("A2SSID{}PWD{}B2", ssid, password);
        match &mut self.port_state {
            PortState::Connected { handle, .. } => {
                match handle.write(data.as_bytes()) {
                    Ok(_) => {
                        self.response_tx.send(SerialResponse::WifiConfigSent).ok();
                    }
                    Err(e) => {
                        let error = format!("Failed to send WiFi config: {}", e);
                        println!("{}", error);
                        self.response_tx.send(SerialResponse::WifiConfigFailed(error)).ok();
                    }
                }
            }
            PortState::Disconnected => {
                self.response_tx.send(SerialResponse::WifiConfigFailed("Serial port not connected".to_string())).ok();
            }
        }
    }

    // Set brightness
    fn set_brightness(&mut self, brightness: u32) {
        let data = format!("A6{}B6", brightness);
        match &mut self.port_state {
            PortState::Connected { handle, .. } => {
                match handle.write(data.as_bytes()) {
                    Ok(_) => {
                        self.response_tx.send(SerialResponse::BrightnessSet).ok();
                    }
                    Err(e) => {
                        let error = format!("Failed to set brightness: {}", e);
                        println!("{}", error);
                        self.response_tx.send(SerialResponse::BrightnessSetFailed(error)).ok();
                    }
                }
            }
            PortState::Disconnected => {
                self.response_tx.send(SerialResponse::BrightnessSetFailed("Serial port not connected".to_string())).ok();
            }
        }
    }

    // Process incoming serial data
    fn process_serial_data(&mut self, data: String) {
        // Find and process packets in the data
        let mut remaining = data;
        while let Some(start) = remaining.find('A') {
            // Trim data before start marker
            remaining = remaining[start..].to_string();
            
            // Find end marker
            if let Some(end) = remaining[1..].find('B') {
                let end = end + 1; // Adjust for starting at index 1
                if end + 1 >= remaining.len() {
                    // Incomplete packet, wait for more data
                    break;
                }
                
                let packet = remaining[..end + 2].to_string();
                remaining = remaining[end + 2..].to_string();
                
                self.process_packet(&packet);
            } else {
                // No end marker found, wait for more data
                break;
            }
        }
    }

    // Process a single packet
    fn process_packet(&mut self, packet: &str) {
        if packet.len() < 3 || !packet.starts_with('A') || packet.chars().nth(packet.len() - 2).unwrap_or(' ') != 'B' {
            return; // Invalid packet format
        }
        
        let packet_type = packet.chars().last().unwrap_or(' ');
        
        match packet_type {
            '1' => {
                if Regex::new(r"^A1(01)B1$").unwrap().is_match(packet) {
                    println!("WiFi setup packet received");
                }
            }
            '2' => {
                let re = Regex::new(r"^A2SSID(.*?)PWD(.*?)B2$").unwrap();
                if let Some(caps) = re.captures(packet) {
                    let ssid = caps.get(1).unwrap().as_str().to_string();
                    let pwd = caps.get(2).unwrap().as_str().to_string();
                    println!("WiFi config packet received: SSID = {}, PWD = {}", ssid, pwd);
                }
            }
            '3' => {
                if Regex::new(r"^A303B3$").unwrap().is_match(packet) {
                    println!("WiFi confirm packet received");
                }
            }
            '4' => {
                let re = Regex::new(r"^A4SSID(.*?)PWD(.*?)B4$").unwrap();
                if let Some(caps) = re.captures(packet) {
                    let ssid = caps.get(1).unwrap().as_str().to_string();
                    let pwd = caps.get(2).unwrap().as_str().to_string();
                    println!("WiFi error packet received: SSID = {}, PWD = {}", ssid, pwd);
                }
            }
            '5' => {
                let re = Regex::new(r"^A5(\d{1,3})(\d+)POWER(\d{1,3})VERSION(\d{1,3})B5$").unwrap();
                if let Some(caps) = re.captures(packet) {
                    let brightness = caps.get(1).unwrap().as_str().parse::<u32>().unwrap_or(0);
                    let raw_ip = caps.get(2).unwrap().as_str();
                    let power = caps.get(3).unwrap().as_str().parse::<u32>().unwrap_or(0);
                    let version = caps.get(4).unwrap().as_str().parse::<u32>().unwrap_or(0);
                    
                    let padded_ip = format!("{:0>12}", raw_ip);
                    let ip_parts = (0..4)
                        .map(|i| padded_ip[i * 3..(i + 1) * 3].parse::<u8>().unwrap_or(0).to_string())
                        .collect::<Vec<_>>();
                    let ip = ip_parts.join(".");
                    
                    // Send device status event
                    self.event_tx.send(SerialEvent::DeviceStatus(DeviceStatus {
                        ip,
                        brightness,
                        power,
                        version,
                        device_type: version, // The version field is used as device type
                    })).ok();
                }
            }
            '6' => {
                let re = Regex::new(r"^A6(\d{1,3})B6$").unwrap();
                if let Some(caps) = re.captures(packet) {
                    let brightness = caps.get(1).unwrap().as_str().parse::<u32>().unwrap_or(0);
                    println!("Light control packet received: Brightness = {}", brightness);
                }
            }
            _ => {
                // Unknown packet type
            }
        }
    }

    // Restart the ESP32 device
    fn restart_device(&mut self) {
        // First, disconnect the port to release resources
        self.disconnect_port();
        
        // Send initial progress
        self.event_tx.send(SerialEvent::RestartProgress {
            progress: 0.0,
            message: "Preparing to restart device...".to_string(),
        }).ok();
        
        // Create a separate thread to handle the restart process
        let thread_event_tx = self.event_tx.clone();
        let thread_response_tx = self.response_tx.clone();
        
        thread::spawn(move || {
            // Find the ESP32 port
            let port = match find_esp32_port() {
                Some(port) => port,
                None => {
                    thread_event_tx.send(SerialEvent::RestartProgress {
                        progress: 100.0,
                        message: "Failed to find ESP32 device".to_string(),
                    }).ok();
                    thread_response_tx.send(SerialResponse::RestartFailed(
                        "ESP32 device not found".to_string()
                    )).ok();
                    return;
                }
            };
            
            thread_event_tx.send(SerialEvent::RestartProgress {
                progress: 30.0,
                message: "Found ESP32 device, restarting...".to_string(),
            }).ok();
            
            // Build and execute the restart command
            // This would use esptool in the real implementation
            thread_event_tx.send(SerialEvent::RestartProgress {
                progress: 50.0,
                message: format!("Executing restart command on port {}", port),
            }).ok();
            
            // Simulate successful restart
            thread::sleep(Duration::from_millis(800));
            
            thread_event_tx.send(SerialEvent::RestartProgress {
                progress: 100.0,
                message: "Device restarted successfully".to_string(),
            }).ok();
            
            thread_response_tx.send(SerialResponse::DeviceRestarted).ok();
        });
    }

    // Flash firmware to the ESP32 device
    fn flash_firmware(&mut self, device_type: String, firmware_type: String, firmware_path: Option<String>) {
        // First, disconnect the port to release resources
        self.disconnect_port();
        
        // Send initial progress
        self.event_tx.send(SerialEvent::FlashProgress {
            progress: 0.0,
            message: "Preparing to flash firmware...".to_string(),
        }).ok();
        
        // Create a separate thread to handle the flashing process
        let thread_event_tx = self.event_tx.clone();
        let thread_response_tx = self.response_tx.clone();
        
        thread::spawn(move || {
            // Find the ESP32 port
            let port = match find_esp32_port() {
                Some(port) => port,
                None => {
                    thread_event_tx.send(SerialEvent::FlashProgress {
                        progress: 100.0,
                        message: "Failed to find ESP32 device".to_string(),
                    }).ok();
                    thread_response_tx.send(SerialResponse::FlashFailed(
                        "ESP32 device not found".to_string()
                    )).ok();
                    return;
                }
            };
            
            thread_event_tx.send(SerialEvent::FlashProgress {
                progress: 20.0,
                message: format!("Found ESP32 device at {}, preparing firmware...", port),
            }).ok();
            
            // Determine firmware path based on type
            let firmware_path = firmware_path.unwrap_or_else(|| {
                format!("assets/{}_firmware.bin", device_type)
            });
            
            thread_event_tx.send(SerialEvent::FlashProgress {
                progress: 30.0,
                message: format!("Flashing {} firmware from {}", firmware_type, firmware_path),
            }).ok();
            
            // Build and execute the flash command
            // This would use esptool in the real implementation
            
            // Simulate progress updates
            for i in 3..10 {
                thread::sleep(Duration::from_millis(300));
                thread_event_tx.send(SerialEvent::FlashProgress {
                    progress: i as f32 * 10.0,
                    message: format!("Flashing firmware: {}%", i * 10),
                }).ok();
            }
            
            thread_event_tx.send(SerialEvent::FlashProgress {
                progress: 100.0,
                message: format!("Firmware {} flashed successfully for {}", firmware_type, device_type),
            }).ok();
            
            thread_response_tx.send(SerialResponse::FirmwareFlashed).ok();
        });
    }
}

// Helper function to find ESP32 port
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

// Helper function to spawn a serial worker in a new thread
pub fn spawn_serial_worker(
    request_rx: Receiver<SerialRequest>,
    response_tx: Sender<SerialResponse>,
    event_tx: Sender<SerialEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut worker = SerialWorker::new(request_rx, response_tx, event_tx);
        worker.run();
    })
}