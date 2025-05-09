use serialport::{self, DataBits, FlowControl, Parity, SerialPort, StopBits};
use tauri::window;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Esp32Status {
    Connected,
    Disconnected,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum PacketType {
    WifiSetup,
    WifiSsidPwd(String, String),
    WifiConfirm,
    WifiError(String, String),
    DeviceStatus {
        ip: String,
        brightness: u32,
        power: u32,
        version: u32,
    },
    LightControl(u32),
    Unknown,
}


pub struct Esp32Serial {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub flow_control: FlowControl,
    pub builder: serialport::SerialPortBuilder,
    pub callbacks: HashMap<PacketType, fn(PacketType)>,
    pub status: Esp32Status,
    pub handle: Option<Box<dyn SerialPort + 'static>>,
    pub run: bool,
}

impl Esp32Serial {
    pub fn new(port: String, baud_rate: u32, data_bits: DataBits, stop_bits: StopBits, parity: Parity, flow_control: FlowControl) -> Self {
        let builder = serialport::new(port.as_str(), baud_rate)
            .data_bits(data_bits)
            .stop_bits(stop_bits)
            .parity(parity)
            .flow_control(flow_control);
        
        Esp32Serial {
            port: port.clone(),
            baud_rate,
            data_bits,
            stop_bits,
            parity,
            flow_control,
            builder,
            callbacks: HashMap::new(),
            status: Esp32Status::Disconnected,
            handle: None,
            run: false,
        }
    }

    pub fn add_callback(&mut self, callback: fn(PacketType)) {
        // Add a callback to the serial port
        // This is a placeholder for the actual implementation
        self.callbacks.insert(PacketType::Unknown, callback);
    }

    pub fn open(&mut self) -> Result<(), String> {
        if let Ok(port) = self.builder.clone().open() {
            self.handle = Some(port);
            self.status = Esp32Status::Connected;
            println!("Serial port opened: {}", self.port);
        } else {
            return Err(format!("Failed to open port: {}", self.port));
        }

        Ok(())
    }

    pub fn start(&mut self) {
        self.run = true;
        // Start the serial port
        while self.run {
            if let Some(handle) = &mut self.handle {
                let mut buffer: Vec<u8> = vec![0; 1024];
                match handle.read(buffer.as_mut_slice()) {
                    Ok(bytes_read) => {
                        if bytes_read > 0 {
                            let data = String::from_utf8_lossy(&buffer[..bytes_read]);
                            Self::process_serial_buffer(data.to_string());
                        }
                    }
                    Err(e) => {
                        println!("串口读取出错: {}", e);
                        self.status = Esp32Status::Disconnected;
                        self.handle = None;
                    }
                }
            } else {
                println!("串口未连接或连接失败");
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
        

    pub fn close(&mut self) -> Result<(), String> {
        self.run = false;
        Ok(())
    }

    pub fn parse_packet(packet: &str) -> PacketType {
        let packet = packet.trim();

        if packet.len() < 3 || !packet.starts_with('A') || packet.chars().nth(packet.len() - 2).unwrap_or(' ') != 'B' {
            println!("无效包格式");
            return PacketType::Unknown;
        }
    
        let packet_type = packet.chars().last().unwrap_or(' ');
    
        match packet_type {
            '1' => {
                if regex::Regex::new(r"^A1(01)B1$").unwrap().is_match(packet) {
                    println!("匹配到包类型1 (WiFi 配置提示)");
                    PacketType::WifiSetup
                } else {
                    PacketType::Unknown
                }
            }
            '2' => {
                let re = regex::Regex::new(r"^A2SSID(.*?)PWD(.*?)B2$").unwrap();
                if let Some(caps) = re.captures(packet) {
                    let ssid = caps.get(1).unwrap().as_str().to_string();
                    let pwd = caps.get(2).unwrap().as_str().to_string();
                    println!("匹配到包类型2 (WiFi 配置数据): SSID = {}, PWD = {}", ssid, pwd);
                    PacketType::WifiSsidPwd(ssid, pwd)
                } else {
                    PacketType::Unknown
                }
            }
            '3' => {
                if regex::Regex::new(r"^A303B3$").unwrap().is_match(packet) {
                    println!("匹配到包类型3 (WiFi 配置成功确认)");
                    PacketType::WifiConfirm
                } else {
                    PacketType::Unknown
                }
            }
            '4' => {
                let re = regex::Regex::new(r"^A4SSID(.*?)PWD(.*?)B4$").unwrap();
                if let Some(caps) = re.captures(packet) {
                    let ssid = caps.get(1).unwrap().as_str().to_string();
                    let pwd = caps.get(2).unwrap().as_str().to_string();
    
                    if ssid == "paper" {
                        println!("面捕正在开机中，如果长时间开机失败则为未进行WiFi配置，请输入WIFI信息并点击发送。");
                    } else {
                        println!("(WiFi 配置错误): 当前WIFI为 {}, 密码为 {}, 请检查是否有误", ssid, pwd);
                    }
                    PacketType::WifiError(ssid, pwd)
                } else {
                    PacketType::Unknown
                }
            }
            '5' => {
                let re = regex::Regex::new(r"^A5(\d{1,3})(\d+)POWER(\d{1,3})VERSION(\d{1,3})B5$").unwrap();
                if let Some(caps) = re.captures(packet) {
                    let brightness = caps.get(1).unwrap().as_str().parse::<u32>().unwrap_or(0);
                    let raw_ip = caps.get(2).unwrap().as_str();
                    let power = caps.get(3).unwrap().as_str().parse::<u32>().unwrap_or(0);
                    let version = caps.get(4).unwrap().as_str().parse::<u32>().unwrap_or(0);
    
                    let padded_ip = format!("{:0>12}", raw_ip);
                    let ip_parts = (0..4)
                        .map(|i| padded_ip[i * 3..(i + 1) * 3].parse::<u8>().unwrap_or(0).to_string())
                        .collect::<Vec<_>>();
                    let formatted_ip = ip_parts.join(".");
    
                    println!(
                        "匹配到包类型5 (设备状态): 亮度 = {}, IP = {}, 电量 = {}, 固件版本 = {}",
                        brightness, formatted_ip, power, version
                    );
    
                    PacketType::DeviceStatus {
                        ip: formatted_ip,
                        brightness,
                        power,
                        version,
                    }
                } else {
                    PacketType::Unknown
                }
            }
            '6' => {
                let re = regex::Regex::new(r"^A6(\d{1,3})B6$").unwrap();
                if let Some(caps) = re.captures(packet) {
                    let brightness = caps.get(1).unwrap().as_str().parse::<u32>().unwrap_or(0);
                    println!("匹配到包类型6 (补光灯控制): 亮度 = {}", brightness);
                    PacketType::LightControl(brightness)
                } else {
                    PacketType::Unknown
                }
            }
            _ => PacketType::Unknown,
        }    
    }

    fn process_serial_buffer(mut buffer: String) {
        while let Some(start) = buffer.find('A') {
            // 剪掉起始符之前的无效数据
            buffer = buffer[start..].to_string();
    
            // 查找结束符 B
            if let Some(end) = buffer[1..].find('B') {
                let end = end + 1; // 因为我们从 buffer[1..] 开始查找，所以偏移+1
                if end + 1 >= buffer.len() {
                    // 等待更多数据
                    break;
                }
    
                let packet = buffer[..end + 2].to_string();
                buffer = buffer[end + 2..].to_string();
    
                println!("接收到数据包: {}", packet);
    
                match Self::parse_packet(packet.as_str()) {
                    PacketType::WifiSetup => println!("[WiFi 配置提示] 请配置 WiFi"),
                    PacketType::WifiSsidPwd(_, _) => println!("[WiFi 配置] 发送 SSID/PWD..."),
                    PacketType::WifiConfirm => println!("[WiFi 配置成功]"),
                    PacketType::WifiError(_, _) => (), // log::info!() 已在内部处理
                    PacketType::DeviceStatus { .. } => println!("[设备状态] 更新设备信息..."),
                    PacketType::LightControl(_) => println!("[补光灯设置] 调整亮度"),
                    PacketType::Unknown => println!("[未知数据包]"),
                }
            } else {
                break;
            }
        }
    }

}

#[cfg(target_os = "macos")]
pub fn find_esp32_port() -> Option<String> {
    // This function should return the port name of the ESP32 device
    // This is a placeholder for the actual implementation

    use std::os::unix::process;
    let ports = serialport::available_ports().unwrap();
    for port in ports {
        if port.port_type == serialport::SerialPortType::BluetoothPort ||
            port.port_type == serialport::SerialPortType::PciPort || 
            port.port_type == serialport::SerialPortType::Unknown {
            continue;
        }
        if let serialport::SerialPortType::UsbPort(usb_info) = port.port_type {
            // output id with hexadecimal format
            println!("Found port: {}, USB VID: {:04x}, PID: {:04x}", port.port_name, usb_info.vid, usb_info.pid);
            if usb_info.vid == 0x303a && usb_info.pid == 0x1001 {
                return Some(port.port_name);
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
pub fn find_esp32_port() -> Option<String> {
    unimplemented!()
}

pub fn start_serial_mod() {
    let _ = std::thread::spawn(move || {
        let port = find_esp32_port();
        if let Some(port) = port {
            println!("Found ESP32 port: {}", port);
            let mut esp32_serial = Esp32Serial::new(port, 115200, DataBits::Eight, StopBits::One, Parity::None, FlowControl::None);
            // esp32_serial.add_callback();
            esp32_serial.open().unwrap();
        } else {
            println!("ESP32 port not found");
        }
    });
}