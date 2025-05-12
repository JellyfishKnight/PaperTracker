use serialport::{self, DataBits, FlowControl, Parity, SerialPort, StopBits};
use tauri::window;
use std::{collections::HashMap, mem::discriminant, ops::BitOrAssign, sync::{Arc, Mutex}, time::Duration};

#[derive(Debug, PartialEq)]
pub enum Esp32Status {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone)]
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

impl PartialEq for PacketType {
    fn eq(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }    
}

impl Eq for PacketType {}

impl std::hash::Hash for PacketType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
    }
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
            .flow_control(flow_control)
            .timeout(std::time::Duration::from_secs(1));
        
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
                            self.process_serial_buffer(data.to_string());
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
                self.status = Esp32Status::Disconnected;
                self.handle = None;
                self.run = false;
                break;
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

    fn process_serial_buffer(&self, mut buffer: String) {
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
                    PacketType::WifiSetup => {
                        println!("匹配到包类型1 (WiFi 配置提示)");
                        self.handle_packet(PacketType::WifiSetup, packet);
                    }
                    PacketType::WifiSsidPwd(ssid, passwd) => {
                        println!("匹配到包类型2 (WiFi 配置数据): SSID = {}, PWD = {}", ssid, passwd);
                        self.handle_packet(PacketType::WifiSsidPwd(ssid, passwd), packet);
                    }
                    PacketType::WifiConfirm => {
                        println!("匹配到包类型3 (WiFi 配置成功确认)");
                        self.handle_packet(PacketType::WifiConfirm, packet);
                    }
                    PacketType::WifiError(ssid, passwd) => {
                        println!("匹配到包类型4 (WiFi 配置错误): 当前WIFI为 {}, 密码为 {}, 请检查是否有误", ssid, passwd);
                        self.handle_packet(PacketType::WifiError(ssid, passwd), packet);
                    }
                    PacketType::DeviceStatus {ip, brightness, power, version } => {
                        // println!("匹配到包类型5 (设备状态): 亮度 = {}, IP = {}, 电量 = {}, 固件版本 = {}", brightness, ip, power, version);
                        self.handle_packet(PacketType::DeviceStatus{ip, brightness, power, version}, packet);
                    }
                    PacketType::LightControl(brightness) => {
                        println!("匹配到包类型6 (补光灯控制): 亮度 = {}", brightness);
                        self.handle_packet(PacketType::LightControl(brightness), packet);
                    }
                    PacketType::Unknown => println!("[未知数据包]"),
                }
            } else {
                break;
            }
        }
    }

    fn handle_packet(&self, packet_type: PacketType, data: String) {
        // Handle the packet type
        if let Some(callback) = self.callbacks.get(&packet_type) {
             callback(packet_type) 
        } else {
            println!("没有找到处理函数: {:?}, callback: {:?}", packet_type, self.callbacks);
        }
    }

    fn register_callback(&mut self, packet_type: PacketType, callback: fn(PacketType)) {
        // Register a callback for the given packet type
        self.callbacks.insert(packet_type, callback);
    }
}

#[cfg(target_os = "macos")]
pub fn find_esp32_port() -> Option<String> {
    // This function should return the port name of the ESP32 device
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

pub fn serial_watchdog(esp32_serial: Arc<Mutex<Option<Esp32Serial>>>) {
    let retry_interval = Duration::from_secs(5);  // 每5秒检查一次
    
    loop {
        // 检查串口状态
        let need_reconnect = {
            let serial_guard = esp32_serial.lock().unwrap();
            match &*serial_guard {
                Some(serial) => serial.status == Esp32Status::Disconnected,
                None => true  // 如果没有实例，也需要重新连接
            }
        };
        
        if need_reconnect {
            println!("守护线程: 检测到串口断开，尝试重新连接...");
            
            // 尝试找到端口
            if let Some(port) = find_esp32_port() {
                println!("守护线程: 找到ESP32端口: {}", port);
                
                // 创建新的串口实例
                let mut new_serial = Esp32Serial::new(
                    port.clone(), 
                    115200, 
                    DataBits::Eight, 
                    StopBits::One, 
                    Parity::None, 
                    FlowControl::None
                );
                
                // 复制回调函数
                {
                    let serial_guard = esp32_serial.lock().unwrap();
                    if let Some(serial) = &*serial_guard {
                        for (packet_type, callback) in &serial.callbacks {
                            new_serial.register_callback((*packet_type).clone(), *callback);
                        }
                    } else {
                        // 如果没有之前的实例，添加默认回调
                        register_default_callbacks(&mut new_serial);
                    }
                }
                
                // 尝试打开端口
                if new_serial.open().is_ok() {
                    println!("守护线程: 成功连接到串口 {}", port);
                    
                    // 替换旧的实例
                    {
                        let mut serial_guard = esp32_serial.lock().unwrap();
                        *serial_guard = Some(new_serial);
                    }
                    
                    // 启动串口线程
                    let esp32_serial_clone = esp32_serial.clone();
                    std::thread::spawn(move || {
                        let mut serial_guard = esp32_serial_clone.lock().unwrap();
                        if let Some(serial) = &mut *serial_guard {
                            serial.start();
                        }
                    });
                } else {
                    println!("守护线程: 连接到串口 {} 失败", port);
                }
            } else {
                println!("守护线程: 未找到ESP32设备，将在{}秒后重试", retry_interval.as_secs());
            }
        }
        
        // 等待一段时间再检查
        std::thread::sleep(retry_interval);
    }
}   

// 添加一个注册默认回调的辅助函数
fn register_default_callbacks(serial: &mut Esp32Serial) {
    serial.register_callback(PacketType::WifiSsidPwd("".to_string(), "".to_string()), |packet| {
        if let PacketType::WifiSsidPwd(ssid, pwd) = packet {
            println!("Handling SSID: {}, PWD: {}", ssid, pwd);
        }
    });
    
    serial.register_callback(PacketType::WifiConfirm, |_| {
        println!("Handling WiFi Confirm");
    });
    
    serial.register_callback(PacketType::WifiSetup, |_| {
        println!("Handling WiFi Setup");
    });
    
    serial.register_callback(PacketType::WifiError("".to_string(), "".to_string()), |packet| {
        if let PacketType::WifiError(ssid, pwd) = packet {
            println!("Handling WiFi Error: SSID: {}, PWD: {}", ssid, pwd);
        }
    });
    
    serial.register_callback(PacketType::DeviceStatus { ip: "".to_string(), brightness: 0, power: 0, version: 0 }, |packet| {
        if let PacketType::DeviceStatus { ip, brightness, power, version } = packet {
            println!("Handling Device Status: IP: {}, Brightness: {}, Power: {}, Version: {}", ip, brightness, power, version);
        }
    });
    
    serial.register_callback(PacketType::LightControl(0), |packet| {
        if let PacketType::LightControl(brightness) = packet {
            println!("Handling Light Control: Brightness: {}", brightness);
        }
    });
    
    serial.register_callback(PacketType::Unknown, |packet| {
        println!("Handling Unknown Packet: {:?}", packet);
    });
}

pub fn start_serial_mod() {
    // 创建可共享的ESP32串口实例
    let esp32_serial = Arc::new(Mutex::new(None::<Esp32Serial>));
        
    // 尝试初始化串口
    if let Some(port) = find_esp32_port() {
        println!("找到ESP32端口: {}", port);
        
        let mut serial = Esp32Serial::new(
            port.clone(), 
            115200, 
            DataBits::Eight, 
            StopBits::One, 
            Parity::None, 
            FlowControl::None
        );
        
        // 注册回调
        register_default_callbacks(&mut serial);
        
        // 尝试打开串口
        if serial.open().is_ok() {
            println!("成功连接到串口 {}", port);
            
            // 保存实例
            {
                let mut serial_guard = esp32_serial.lock().unwrap();
                *serial_guard = Some(serial);
            }
            
            // 启动串口线程
            let esp32_serial_clone = esp32_serial.clone();
            std::thread::spawn(move || {
                let mut serial_guard = esp32_serial_clone.lock().unwrap();
                if let Some(serial) = &mut *serial_guard {
                    serial.start();
                }
            });
        } else {
            println!("连接到串口 {} 失败，守护线程将自动重试", port);
        }
    } else {
        println!("没有找到ESP32设备，守护线程将自动寻找");
    }

    // 启动守护线程
    let esp32_serial_clone = esp32_serial.clone();
    std::thread::spawn(move || {
        serial_watchdog(esp32_serial_clone);
    });
}