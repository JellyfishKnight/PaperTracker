use std::{
    collections::VecDeque,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use thiserror::Error;
use url::Url;
use tungstenite::{connect, Message, WebSocket};
use opencv::{core::Mat, imgcodecs, prelude::*};
use serde::Deserialize;
use mdns_sd::{ServiceDaemon, ServiceEvent};

/// 设备类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Unknown,
    Face,
    LeftEye,
    RightEye,
}

/// 与 ESP32 WebSocket 通信的错误类型
#[derive(Error, Debug)]
pub enum ESP32Error {
    #[error("无效的 URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    
    #[error("WebSocket 连接错误: {0}")]
    WebSocketError(#[from] tungstenite::Error),
    
    #[error("未初始化的视频流")]
    NotInitialized,
    
    #[error("图像处理错误: {0}")]
    ImageError(String),
    
    #[error("mDNS 错误: {0}")]
    MdnsError(String),
    
    #[error("内部错误: {0}")]
    InternalError(String),
}

/// 表示流的状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamStatus {
    Disconnected,
    Connecting,
    Connected,
}

/// ESP32 设备状态信息
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceStatus {
    pub battery: Option<f32>,
    pub brightness: Option<i32>,
}

/// 包含图像帧和相关元数据的结构
#[derive(Debug, Clone)]
pub struct Frame {
    pub image: Mat,
    pub timestamp: Instant,
}

/// ESP32 视频流的配置选项
#[derive(Debug, Clone)]
pub struct ESP32Config {
    /// 连接超时时间（毫秒）
    pub connection_timeout_ms: u64,
    
    /// 心跳超时时间（毫秒）
    pub heartbeat_timeout_ms: u64,
    
    /// 最大重连尝试次数
    pub max_reconnect_attempts: usize,
    
    /// 是否启用 mDNS 查找
    pub enable_mdns: bool,
}

impl Default for ESP32Config {
    fn default() -> Self {
        Self {
            connection_timeout_ms: 3000,
            heartbeat_timeout_ms: 2500,
            max_reconnect_attempts: 5,
            enable_mdns: true,
        }
    }
}

/// ESP32 视频流处理器的内部状态
pub struct ESP32State {
    pub status: StreamStatus,
    pub device_status: DeviceStatus,
    pub current_url: String,
    pub connection_urls: Vec<String>,
    pub image_buffer: VecDeque<Frame>,
    pub last_frame_time: Instant,
    pub missed_heartbeats: usize,
}

/// ESP32 视频流控制器 - 公共接口
pub struct ESP32StreamController {
    pub state: Arc<Mutex<ESP32State>>,
    pub config: ESP32Config,
    
    // 通信通道
    pub shutdown_tx: Option<mpsc::Sender<()>>,
    pub status_rx: Arc<Mutex<mpsc::Receiver<DeviceStatus>>>,
    
    // 工作线程句柄
    pub worker_thread: Option<thread::JoinHandle<()>>,
}

/// 表示一个 ESP32 视频流会话的句柄
/// 这是一个轻量级的克隆类型，用于从主控制器访问流
#[derive(Clone)]
pub struct ESP32Stream {
    pub state: Arc<Mutex<ESP32State>>,
    pub status_rx: Arc<Mutex<mpsc::Receiver<DeviceStatus>>>,
}

impl ESP32StreamController {
    /// 创建一个新的 ESP32 视频流控制器
    pub fn new(config: ESP32Config) -> Self {
        let (status_tx, status_rx) = mpsc::channel();
        let status_rx = Arc::new(Mutex::new(status_rx));
        
        // 创建初始状态
        let state = Arc::new(Mutex::new(ESP32State {
            status: StreamStatus::Disconnected,
            device_status: DeviceStatus {
                battery: None,
                brightness: None,
            },
            current_url: String::new(),
            connection_urls: Vec::new(),
            image_buffer: VecDeque::with_capacity(1),
            last_frame_time: Instant::now(),
            missed_heartbeats: 0,
        }));
        
        Self {
            state,
            config,
            shutdown_tx: None,
            status_rx,
            worker_thread: None,
        }
    }
    
    /// 使用默认配置创建一个新的控制器
    pub fn default() -> Self {
        Self::new(ESP32Config::default())
    }
    
    /// 初始化与 ESP32 的连接
    /// 
    /// `url` - WebSocket URL 或 HTTP URL（将自动转换）
    /// `device_type` - 设备类型，用于自动添加默认 mDNS 地址
    /// 
    /// 返回 `Result<(), ESP32Error>` 表示初始化是否成功
    pub fn init(&mut self, url: &str, device_type: DeviceType) -> Result<(), ESP32Error> {
        let mut state = self.state.lock().unwrap();
        
        // 如果 URL 为空，检查是否有现有 URL
        if url.is_empty() {
            if state.current_url.is_empty() {
                return Err(ESP32Error::InvalidUrl(url::ParseError::EmptyHost));
            }
            return Ok(()); // 保持现有 URL
        }
        
        // 清除现有连接 URL 列表
        state.connection_urls.clear();
        
        // 保存原始 URL
        state.current_url = url.to_string();
        
        // 处理 URL 并添加到连接列表
        self.add_connection_url(&mut state, url)?;
        
        // 添加基于设备类型的默认 mDNS 地址
        match device_type {
            DeviceType::Face => {
                self.add_unique_url(&mut state, "ws://paper1.local:80/ws");
            },
            DeviceType::LeftEye => {
                self.add_unique_url(&mut state, "ws://paper2.local:80/ws");
            },
            DeviceType::RightEye => {
                self.add_unique_url(&mut state, "ws://paper3.local:80/ws");
            },
            DeviceType::Unknown => {
                // 基于 URL 关键字推断设备类型
                if url.contains("face") || url.contains("paper1") {
                    self.add_unique_url(&mut state, "ws://paper1.local:80/ws");
                } else if url.contains("left") || url.contains("paper2") {
                    self.add_unique_url(&mut state, "ws://paper2.local:80/ws");
                } else if url.contains("right") || url.contains("paper3") {
                    self.add_unique_url(&mut state, "ws://paper3.local:80/ws");
                }
            }
        }
        
        // 如果是 mDNS 地址且启用了 mDNS，启动解析
        if self.config.enable_mdns && url.contains(".local") {
            let hostname = extract_hostname(url);
            if !hostname.is_empty() {
                println!("发现 mDNS 地址: {}", hostname);
                self.start_mdns_lookup(hostname);
            }
        }
        
        println!("初始化连接 URL 列表: {:?}", state.connection_urls);
        Ok(())
    }
    
    /// 启动视频流
    pub fn start(&mut self) -> Result<(), ESP32Error> {
        let mut state = self.state.lock().unwrap();
        
        // 检查是否已初始化
        if state.connection_urls.is_empty() && state.current_url.is_empty() {
            return Err(ESP32Error::NotInitialized);
        }
        
        // 检查是否已经在运行
        if state.status != StreamStatus::Disconnected {
            println!("流已经在运行中");
            return Ok(());
        }
        
        // 设置状态为连接中
        state.status = StreamStatus::Connecting;
        
        // 创建一个通道用于发送关闭信号
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        self.shutdown_tx = Some(shutdown_tx);
        
        // 克隆需要的状态和配置
        let state_arc = self.state.clone();
        let config = self.config.clone();
        let (status_tx, _) = mpsc::channel(); // 创建了新的发送端，接收端已经存在
        
        // 启动工作线程
        let handle = thread::spawn(move || {
            run_esp32_stream(state_arc, config, shutdown_rx, status_tx);
        });
        
        self.worker_thread = Some(handle);
        
        Ok(())
    }
    
    /// 停止视频流
    pub fn stop(&mut self) {
        // 发送关闭信号
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        
        // 等待工作线程结束
        if let Some(handle) = self.worker_thread.take() {
            let _ = handle.join();
        }
        
        // 更新状态
        let mut state = self.state.lock().unwrap();
        state.status = StreamStatus::Disconnected;
    }
    
    /// 获取 ESP32 视频流的句柄，可用于从其他线程访问流
    pub fn stream_handle(&self) -> ESP32Stream {
        ESP32Stream {
            state: Arc::clone(&self.state),
            status_rx: Arc::clone(&self.status_rx),
        }
    }
    
    /// 检查流是否已连接并正在运行
    pub fn is_connected(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.status == StreamStatus::Connected
    }
    
    // 私有辅助方法
    fn add_connection_url(&self, state: &mut ESP32State, url: &str) -> Result<(), ESP32Error> {
        // 转换 HTTP URL 为 WebSocket URL
        let ws_url = if !url.starts_with("ws://") && !url.starts_with("wss://") {
            if url.starts_with("http://") {
                format!("ws://{}/ws", url.strip_prefix("http://").unwrap_or(url))
            } else if url.starts_with("https://") {
                format!("wss://{}/ws", url.strip_prefix("https://").unwrap_or(url))
            } else {
                // 不是标准 URL，假设是主机地址
                let mut ws_url = format!("ws://{}", url);
                
                // 确保 URL 有正确的端口和路径
                if !ws_url.contains(':') {
                    ws_url.push_str(":80");
                }
                if !ws_url.contains("/ws") {
                    ws_url.push_str("/ws");
                }
                
                ws_url
            }
        } else {
            url.to_string()
        };
        
        // 验证 URL 是否有效
        let _ = Url::parse(&ws_url)?;
        
        // 添加到连接列表
        self.add_unique_url(state, &ws_url);
        
        Ok(())
    }
    
    fn add_unique_url(&self, state: &mut ESP32State, url: &str) {
        if !state.connection_urls.contains(&url.to_string()) {
            state.connection_urls.push(url.to_string());
        }
    }
    
    fn start_mdns_lookup(&self, hostname: String) {
        // 克隆需要的状态
        let state_arc = self.state.clone();
        
        thread::spawn(move || {
            match ServiceDaemon::new() {
                Ok(daemon) => {
                    println!("启动 mDNS 查找: {}", hostname);
                    
                    // 创建服务浏览器
                    if let Ok(browser) = daemon.browse("_http._tcp.local.") {
                        let receiver = browser.recv();
                        let timeout = Duration::from_secs(5);
                        
                        // 处理服务事件
                        while let Ok(event) = &receiver {
                            if let ServiceEvent::ServiceResolved(info) = event {
                                // 检查是否是我们要查找的主机名
                                if info.get_hostname().contains(&hostname) {
                                    for address in info.get_addresses() {
                                        let ip_address = address.to_string();
                                        println!("mDNS 解析成功: {} -> {}", hostname, ip_address);
                                        
                                        // 构建 WebSocket URL
                                        let ws_url = format!("ws://{}:{}/ws", ip_address, info.get_port());
                                        
                                        // 添加到连接列表
                                        let mut state = state_arc.lock().unwrap();
                                        if !state.connection_urls.contains(&ws_url) {
                                            // 将 mDNS 解析的地址放在列表前面，优先尝试
                                            state.connection_urls.insert(0, ws_url);
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                Err(e) => println!("mDNS 服务创建失败: {}", e),
            }
        });
    }
}

impl Drop for ESP32StreamController {
    fn drop(&mut self) {
        self.stop();
    }
}

impl ESP32Stream {
    /// 获取最新的视频帧
    pub fn get_latest_frame(&self) -> Option<Mat> {
        let state = self.state.lock().unwrap();
        
        state.image_buffer.front().map(|frame| frame.image.clone())
    }
    
    /// 获取流的当前状态
    pub fn status(&self) -> StreamStatus {
        let state = self.state.lock().unwrap();
        state.status
    }
    
    /// 获取设备状态
    pub fn device_status(&self) -> DeviceStatus {
        match self.status_rx.lock().unwrap().try_recv() {
            Ok(status) => status,
            Err(_) => {
                // 使用现有状态
                let state = self.state.lock().unwrap();
                state.device_status.clone()
            }
        }
    }
    
    /// 获取电池百分比
    pub fn battery_percentage(&self) -> Option<f32> {
        self.device_status().battery
    }
    
    /// 获取亮度值
    pub fn brightness_value(&self) -> Option<i32> {
        self.device_status().brightness
    }
}

// 工作线程主函数
pub fn run_esp32_stream(
    state_arc: Arc<Mutex<ESP32State>>,
    config: ESP32Config,
    shutdown_rx: mpsc::Receiver<()>,
    status_tx: mpsc::Sender<DeviceStatus>,
) {
    let mut attempt_index = 0;
    let mut current_socket: Option<WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>> = None;
    let mut reconnect_attempt = 0;
    
    // 连接到下一个可用地址
    fn try_connect(
        state_arc: &Arc<Mutex<ESP32State>>,
        attempt_index: &mut usize,
        config: &ESP32Config,
    ) -> Option<WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>> {
        let mut state = state_arc.lock().unwrap();
        
        // 检查是否还有可用的 URL
        if *attempt_index >= state.connection_urls.len() {
            println!("所有连接尝试都失败");
            std::thread::sleep(Duration::from_secs(5));
            state.status = StreamStatus::Disconnected;
            return None;
        }
        
        let url_to_try = state.connection_urls[*attempt_index].clone();
        *attempt_index += 1;
        
        println!("尝试连接到 URL ({}/{}): {}", 
             *attempt_index, 
             state.connection_urls.len(), 
             url_to_try);
        
        // 释放锁以避免连接期间的阻塞
        drop(state);
        
        // 尝试解析 URL 并连接
        match Url::parse(&url_to_try) {
            Ok(parsed_url) => {
                match connect(parsed_url) {
                    Ok((socket, response)) => {
                        println!("成功连接到 WebSocket: {} (HTTP 状态: {})", 
                             url_to_try, 
                             response.status());
                        
                        // 更新状态
                        let mut state = state_arc.lock().unwrap();
                        state.current_url = url_to_try;
                        state.status = StreamStatus::Connected;
                        state.missed_heartbeats = 0;
                        state.last_frame_time = Instant::now();
                        
                        Some(socket)
                    },
                    Err(e) => {
                        println!("连接到 {} 失败: {}", url_to_try, e);
                        None
                    }
                }
            },
            Err(e) => {
                println!("URL 解析失败: {} - {}", url_to_try, e);
                None
            }
        }
    }
    
    // 主循环
    'main_loop: loop {
        // 检查是否有关闭信号
        if shutdown_rx.try_recv().is_ok() {
            println!("收到关闭信号，正在终止工作线程");
            if let Some(ref mut socket) = current_socket {
                let _ = socket.close(None);
            }
            break;
        }
        
        // 如果没有活动连接，尝试连接
        if current_socket.is_none() {            
            match try_connect(&state_arc, &mut attempt_index, &config) {
                Some(socket) => {
                    current_socket = Some(socket);
                },
                None => {
                    // 没有成功连接，等待一段时间后重试
                    thread::sleep(Duration::from_millis(500));
                    reconnect_attempt += 1;
                    continue;
                }
            }
        }
        
        // 检查心跳
        {
            let mut state = state_arc.lock().unwrap();
            let elapsed = Instant::now().duration_since(state.last_frame_time);
            
            if elapsed.as_millis() as u64 > config.heartbeat_timeout_ms {
                state.missed_heartbeats += 1;
                
                if state.missed_heartbeats > 3 {
                    println!("心跳检测失败，尝试重新连接");
                    state.status = StreamStatus::Connecting;
                    state.missed_heartbeats = 0;
                    
                    // 关闭当前连接
                    if let Some(ref mut socket) = current_socket {
                        let _ = socket.close(None);
                    }
                    current_socket = None;
                    
                    // 重置连接尝试索引以便从头开始
                    attempt_index = 0;
                    continue 'main_loop;
                }
            }
        }
        
        // 处理 WebSocket 消息
        if let Some(ref mut socket) = current_socket {
            match socket.read() {
                Ok(message) => {
                    // 更新最后接收时间
                    let mut state = state_arc.lock().unwrap();
                    state.last_frame_time = Instant::now();
                    state.missed_heartbeats = 0;
                    
                    match message {
                        Message::Binary(data) => {
                            // 处理二进制消息（图像数据）
                            if data.len() < 10 {
                                println!("接收到的数据太短，不可能是有效图像");
                                continue;
                            }
                            
                            // 使用 OpenCV 解码图像
                            match imgcodecs::imdecode(&Mat::from_slice(&data).unwrap(), imgcodecs::IMREAD_COLOR) {
                                Ok(frame) if !frame.empty() => {
                                    // 将图像放入缓冲区
                                    let new_frame = Frame {
                                        image: frame,
                                        timestamp: Instant::now(),
                                    };
                                    
                                    // 保留单帧以节省内存
                                    state.image_buffer.clear();
                                    state.image_buffer.push_back(new_frame);
                                },
                                Ok(_) => println!("解码图像为空"),
                                Err(e) => println!("图像解码失败: {}", e),
                            }
                        },
                        Message::Text(text) => {
                            // 处理文本消息（状态信息）
                            match serde_json::from_str::<DeviceStatus>(&text) {
                                Ok(status) => {
                                    // 更新设备状态
                                    state.device_status = status.clone();
                                    
                                    // 通过通道发送状态更新
                                    let _ = status_tx.send(status);
                                    
                                    if let Some(battery) = state.device_status.battery {
                                        println!("收到电池电量: {}%", battery);
                                    }
                                    
                                    if let Some(brightness) = state.device_status.brightness {
                                        println!("收到亮度值: {}", brightness);
                                    }
                                },
                                Err(e) => println!("解析状态消息失败: {}, 消息: {}", e, text),
                            }
                        },
                        Message::Close(_) => {
                            println!("WebSocket 连接关闭");
                            current_socket = None;
                            state.status = StreamStatus::Connecting;
                        },
                        _ => {} // 忽略其他类型的消息
                    }
                },
                Err(e) => {
                    println!("读取 WebSocket 消息失败: {}", e);
                    
                    let mut state = state_arc.lock().unwrap();
                    state.status = StreamStatus::Connecting;
                    
                    // 关闭当前连接并尝试下一个
                    current_socket = None;
                }
            }
        }
        
        // 短暂休眠以避免 CPU 占用过高
        thread::sleep(Duration::from_millis(5));
    }
    
    println!("工作线程已终止");
}

// 辅助函数：从 URL 中提取主机名
fn extract_hostname(url: &str) -> String {
    // 尝试解析为标准 URL
    if let Ok(parsed_url) = Url::parse(url) {
        if let Some(host) = parsed_url.host_str() {
            return host.to_string();
        }
    }
    
    // 尝试从字符串中提取主机名
    let parts: Vec<&str> = url.split(&['/', ':', '?'][..]).collect();
    if parts.len() > 1 {
        return parts[1].to_string();
    }
    
    url.to_string()
}