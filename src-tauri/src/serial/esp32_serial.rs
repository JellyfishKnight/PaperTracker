use std::{collections::HashMap, vec};
use crossbeam::channel::{Sender, Receiver};
use ftlog::*;
use regex::Regex;
use serialport::SerialPort;
use crate::{integration::interface::StreamEvent, utils::consts::{DEVICE_TYPE_FACE, DEVICE_TYPE_LEFT_EYE, DEVICE_TYPE_RIGHT_EYE}};

use super::{esp32_control::{find_esp32_port, flash_esp32, restart_esp32}, serial_msg::{DeviceStatus, PortState, SerialMessage, SerialRequest, SerialResponse, SerialSendPacket, WifiError}};
use tauri::{AppHandle, Emitter, EventTarget, Runtime};


pub struct Esp32Serial<R: Runtime> {
    // Channel for receiving requests
    request_rx: Receiver<SerialRequest>,
    // Channel for sending requests
    request_tx: Sender<SerialRequest>,
    // Channel for sending responses
    response_tx: bus::Bus<SerialResponse>,
    // 
    message_tx: bus::Bus<SerialMessage>,
    // 
    write_rx: Receiver<SerialSendPacket>,
    //
    write_tx: Sender<SerialSendPacket>,
    // Current state of the port
    port_state: PortState,
    // 
    last_message: HashMap<i32, SerialMessage>,
    // 
    serial_info: (String, i32),
    // 
    run: bool,
    // 
    app_handle: AppHandle<R>,
}

impl<R: Runtime> Esp32Serial<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        let (request_tx, request_rx) = crossbeam::channel::unbounded();
        let response_tx = bus::Bus::new(1);
        let message_tx = bus::Bus::new(1);
        let (write_tx, write_rx) = crossbeam::channel::unbounded();
        Esp32Serial {
            request_rx,
            request_tx,
            response_tx,
            message_tx,
            write_rx,
            write_tx,
            port_state: PortState::Disconnected,
            last_message: HashMap::new(),
            serial_info: ("".to_string(), 0),
            run: false,
            app_handle: app,
        }
    }
    
    pub fn get_message_rx(&mut self) -> bus::BusReader<SerialMessage> {
        self.message_tx.add_rx()
    }

    pub fn get_response_rx(&mut self) -> bus::BusReader<SerialResponse> {
        self.response_tx.add_rx()
    }

    pub fn get_request_tx(&mut self) -> Sender<SerialRequest> {
        self.request_tx.clone()
    }

    pub fn get_write_tx(&mut self) -> Sender<SerialSendPacket> {
        self.write_tx.clone()
    }

    pub fn start(&mut self) {
        let mut port : Option<Box<dyn SerialPort + 'static>> = None;
        self.run = true;
        loop {
            // Check if the port state has changed
            if let PortState::Disconnected = self.port_state {
                if let Err(e) = self.app_handle.emit("face_serial_status", "面捕设备未连接") {
                    error!("Failed to emit serial status event: {}", e);
                }
                if let Err(e) = self.app_handle.emit("left_eye_serial_status", "左眼设备未连接") {
                    error!("Failed to emit serial status event: {}", e);
                }
                if let Err(e) = self.app_handle.emit("right_eye_serial_status", "右眼设备未连接") {
                    error!("Failed to emit serial status event: {}", e);
                }    
            } 
            if let PortState::Connected = self.port_state {
                if let Some(SerialMessage::DeviceStatus(message)) = self.last_message.get(&5) {
                    match message.device_type {
                        DEVICE_TYPE_FACE => {
                            let _ = self.app_handle.emit("face_serial_status", "面捕设备已连接");
                        }
                        DEVICE_TYPE_LEFT_EYE => {
                            let _ = self.app_handle.emit("left_eye_serial_status", "左眼设备已连接");
                        }
                        DEVICE_TYPE_RIGHT_EYE => {
                            let _ = self.app_handle.emit("right_eye_serial_status", "右眼设备已连接");
                        }
                        _ => ()
                    }
                }
            }
            // Check for incoming requests
            match self.request_rx.try_recv() {
                Ok(request) => {
                    info!("Received request: {:?}", request);
                    self.handle_request(request, &mut port);
                }
                Err(crossbeam::channel::TryRecvError::Disconnected) => {
                    // Handle error in receiving requests
                    debug!("disconnected request in serial");
                }
                _ => ()
            }
            // Check if the port is disconnected
            if let PortState::Disconnected = self.port_state {
                if self.run {
                    if self.connect(&mut port) {
                        info!("Connected to ESP32 device");
                        self.port_state = PortState::Connected;
                    } else {
                        error!("Failed to connect to ESP32 device");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;        
                    }
                } else {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    continue;
                }
            }
            if !self.run {
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
            // check for write requests
            match self.write_rx.try_recv() {
                Ok(message) => {
                    // Handle write request
                    self.handle_write_message(message, &mut port);
                }
                Err(crossbeam::channel::TryRecvError::Disconnected) => {
                    // Handle error in receiving requests
                    debug!("Disconnected write in serial");
                }
                _ => ()
            }
            // Read data from the serial port
            let mut buffer = vec![0u8; 1024];
            if let Some(ref mut port) = port {
                match port.read(&mut buffer) {
                    Ok(bytes_read) => {
                        if bytes_read > 0 {
                            // Process the received data
                            let data = &buffer[..bytes_read];
                            // Deserialize the data into a SerialMessage
                            self.process_serial_data(String::from_utf8_lossy(data).to_string());
                        }
                    }
                    Err(e) => {
                        self.port_state = PortState::Disconnected;
                        error!("Error reading from serial port: {}", e);
                    }
                }
            }
        }
    }

    fn connect(&mut self, port: &mut Option<Box<dyn SerialPort + 'static>>) -> bool {
        if let Some(port_name) = find_esp32_port() {
            self.serial_info.0 = port_name;
            info!("ESP32 device found at port: {}", self.serial_info.0);
        } else {
            info!("No ESP32 device found");
            return false;
        }
        // Open the serial port
        *port = match serialport::new(&self.serial_info.0, 115200)
            .timeout(std::time::Duration::from_secs(1))
            .open() {
                Ok(port) => {
                    info!("Serial port opened successfully");
                    Some(port)
                }
                Err(e) => {
                    error!("Failed to open serial port: {}", e);
                    None
                }
            };
        port.is_some()
    }

    fn handle_write_message(&mut self, message: SerialSendPacket, port: &mut Option<Box<dyn SerialPort + 'static>>) {
        match message {
            SerialSendPacket::Brightness(brightness) => {
                info!("Setting brightness to: {}", brightness);
                let packet = format!("A6{}B6", brightness);
                if let Some(ref mut port) = port {
                    if let Err(e) = port.write(packet.as_bytes()) {
                        self.port_state = PortState::Disconnected;
                        error!("Error writing to serial port: {}", e);
                    } else {
                        info!("Sent brightness packet: {}", packet);
                    }
                } else {
                    self.port_state = PortState::Disconnected;
                    error!("Port is not available for writing");
                }
            }
            SerialSendPacket::WifiConfig(config) => {
                let packet = format!("A2SSID{}PWD{}B2", config.ssid, config.password);
                if let Some(ref mut port) = port {
                    if let Err(e) = port.write(packet.as_bytes()) {
                        self.port_state = PortState::Disconnected;
                        error!("Error writing to serial port: {}", e);
                    } else {
                        info!("Sent WiFi config packet: {}", packet);       
                    }
                } else {
                    self.port_state = PortState::Disconnected;
                    error!("Port is not available for writing");
                }
            }
        }
    }

    fn handle_request(&mut self, req: SerialRequest, port: &mut Option<Box<dyn SerialPort + 'static>>) {
        match req {
            SerialRequest::Restart(path) => {
                // Handle restart request
                self.port_state = PortState::Disconnected;
                *port = None;
                let result = restart_esp32(path, self.serial_info.0.clone());
                match result {
                    Ok(_) => {
                        self.response_tx.broadcast(SerialResponse::Restart(true, "Restarted successfully".to_string()));
                    }
                    Err(e) => {
                        error!("Failed to restart ESP32: {}", e);
                        self.response_tx.broadcast(SerialResponse::Restart(false, e.to_string()));
                    }
                }
            }
            SerialRequest::Flash(command) => {
                self.port_state = PortState::Disconnected;
                // Handle flash request
                let result = flash_esp32(
                    command.tool_path, 
                    command.boot_loader_path, 
                    command.partition_path,
                    command.firmware_path,
                    self.serial_info.0.clone()
                );
                match result {
                    Ok(_) => {
                        self.response_tx.broadcast(SerialResponse::Flash(("Flashed successfully".to_string(), 100)));
                    }
                    Err(e) => {
                        error!("Failed to flash ESP32: {}", e);
                        self.response_tx.broadcast(SerialResponse::Flash(("Failed to flash".to_string(), 0)));
                    }
                }
            }
            SerialRequest::GetStatus => {
                self.response_tx.broadcast(SerialResponse::Status((self.port_state.clone(), self.serial_info.1)));
            }
            SerialRequest::Stop => {
                self.run = false;
            }
            SerialRequest::Start => {
                self.run = true;
            }
        }
    }

    fn process_serial_data(&mut self, data: String) {
        // Find and process packets in the data
        let mut remaining = data;
        while let Some(start) = remaining.find('A') {
            // if the char after 'A' is not a digit, skip this packet
            if remaining[start + 1..].chars().next().unwrap_or(' ') < '0' || remaining[start + 1..].chars().next().unwrap_or(' ') > '9' {
                info!("{}", remaining);
                return ;
            }
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

    fn process_packet(&mut self, packet: &str) {
        if packet.len() < 3 || !packet.starts_with('A') || packet.chars().nth(packet.len() - 2).unwrap_or(' ') != 'B' {
            return; // Invalid packet format
        }
        
        let packet_type = packet.chars().last().unwrap_or(' ');
        
        match packet_type {
            '1' => {
                if Regex::new(r"^A1(01)B1$").unwrap().is_match(packet) {
                    let _ = self.message_tx.try_broadcast(
                        SerialMessage::GeneralMessage(
                            "Wifi Setup packet received".to_string(),
                        )
                    );
                    *self.last_message.entry(1).or_insert(SerialMessage::GeneralMessage("Wifi Setup packet received".to_string())) = 
                        SerialMessage::GeneralMessage(
                            "Wifi Setup packet received".to_string(),
                        );
                }
            }
            '2' => {
                let re = Regex::new(r"^A2SSID(.*?)PWD(.*?)B2$").unwrap();
                if let Some(caps) = re.captures(packet) {
                    let ssid = caps.get(1).unwrap().as_str().to_string();
                    let pwd = caps.get(2).unwrap().as_str().to_string();
                    let _ = self.message_tx.try_broadcast(
                        SerialMessage::GeneralMessage(format!("WiFi config packet received: SSID = {}, PWD = {}", ssid, pwd))
                    );
                    *self.last_message.entry(2).or_insert(SerialMessage::GeneralMessage(format!("WiFi config packet received: SSID = {}, PWD = {}", ssid, pwd))) = 
                        SerialMessage::GeneralMessage(format!("WiFi config packet received: SSID = {}, PWD = {}", ssid, pwd));
                }
            }
            '3' => {
                if Regex::new(r"^A303B3$").unwrap().is_match(packet) {
                    let _ = self.message_tx.try_broadcast(
                        SerialMessage::GeneralMessage(
                            "WiFi confirm packet received".to_string(),
                        )
                    );
                    *self.last_message.entry(3).or_insert(SerialMessage::GeneralMessage("WiFi confirm packet received".to_string())) = 
                        SerialMessage::GeneralMessage(
                            "WiFi confirm packet received".to_string(),
                        );
                }
            }
            '4' => {
                let re = Regex::new(r"^A4SSID(.*?)PWD(.*?)B4$").unwrap();
                if let Some(caps) = re.captures(packet) {
                    let ssid = caps.get(1).unwrap().as_str().to_string();
                    let pwd = caps.get(2).unwrap().as_str().to_string();
                    // Send WiFi error event
                    let _ = self.message_tx.try_broadcast(
                        SerialMessage::WifiError(WifiError {
                            ssid: ssid.clone(),
                            password: pwd.clone(),
                        })
                    );
                    *self.last_message.entry(4).or_insert(SerialMessage::WifiError(WifiError {
                        ssid: ssid.clone(),
                        password: pwd.clone(),
                    })) = 
                        SerialMessage::WifiError(WifiError {
                            ssid: ssid.clone(),
                            password: pwd.clone(),
                        });
                }
            }
            '5' => {
                let re = Regex::new(r"^A5(\d{1,3})(\d+)POWER(\d{1,3})VERSION(\d{1,3})B5$").unwrap();
                if let Some(caps) = re.captures(packet) {
                    let brightness = caps.get(1).unwrap().as_str().parse::<i32>().unwrap_or(0);
                    let raw_ip = caps.get(2).unwrap().as_str();
                    let power = caps.get(3).unwrap().as_str().parse::<f32>().unwrap_or(0.0);
                    let version = caps.get(4).unwrap().as_str().parse::<i32>().unwrap_or(0);
                    let padded_ip = format!("{:0>12}", raw_ip);
                    let ip_parts = (0..4)
                        .map(|i| padded_ip[i * 3..(i + 1) * 3].parse::<u8>().unwrap_or(0).to_string())
                        .collect::<Vec<_>>();
                    let ip = ip_parts.join(".");
                    self.serial_info.1 = version;
                    let _ = self.message_tx.try_broadcast(
                        SerialMessage::DeviceStatus(
                            DeviceStatus {
                                ip: ip.clone(),
                                brightness,
                                power,
                                device_type: version,
                            }
                        )
                    );
                    *self.last_message.entry(5).or_insert(SerialMessage::DeviceStatus(
                        DeviceStatus {
                            ip: ip.clone(),
                            brightness,
                            power,
                            device_type: version,
                        }
                    )) = 
                        SerialMessage::DeviceStatus(
                            DeviceStatus {
                                ip: ip.clone(),
                                brightness,
                                power,
                                device_type: version,
                            }
                        );
                }
            }
            '6' => {
                let re = Regex::new(r"^A6(\d{1,3})B6$").unwrap();
                if let Some(caps) = re.captures(packet) {
                    let brightness = caps.get(1).unwrap().as_str().parse::<u32>().unwrap_or(0);
                    let _ = self.message_tx.try_broadcast(
                        SerialMessage::GeneralMessage(format!("Brightness set to: {}", brightness))
                    );
                    *self.last_message.entry(6).or_insert(SerialMessage::GeneralMessage(format!("Brightness set to: {}", brightness))) = 
                        SerialMessage::GeneralMessage(format!("Brightness set to: {}", brightness));
                }
            }
            _ => {
                // Unknown packet type
                info!("Unknown packet type: {}", packet_type);
            }
        }
    }
}

