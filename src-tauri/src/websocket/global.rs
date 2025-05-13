// File: websocket/global.rs

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use std::time::Duration;
use crate::serial::global::{LAST_FACE_IP, LAST_LEFT_EYE_IP, LAST_RIGHT_EYE_IP};
use crate::websocket::esp32_video_stream::{ESP32StreamController, ESP32Stream, ESP32Config, DeviceType, StreamStatus};
use crate::paper_tracker_config::config::{FACE_CONIG, EYE_CONFIG};

// 定义全局静态变量
pub static FACE_STREAM: Lazy<Arc<Mutex<Option<ESP32StreamController>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub static LEFT_EYE_STREAM: Lazy<Arc<Mutex<Option<ESP32StreamController>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub static RIGHT_EYE_STREAM: Lazy<Arc<Mutex<Option<ESP32StreamController>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

// 初始化所有视频流
pub fn init_global_video_streams() {
    println!("正在初始化全局视频流...");
    
    // 初始化上次IP值
    {
        let face_ip = FACE_CONIG.functional.wifi_ip.clone();
        if !face_ip.is_empty() {
            *LAST_FACE_IP.lock().unwrap() = face_ip.clone();
        }
        
        let left_eye_ip = EYE_CONFIG.functional.left_ip.clone();
        if !left_eye_ip.is_empty() {
            *LAST_LEFT_EYE_IP.lock().unwrap() = left_eye_ip.clone();
        }
        
        let right_eye_ip = EYE_CONFIG.functional.right_ip.clone();
        if !right_eye_ip.is_empty() {
            *LAST_RIGHT_EYE_IP.lock().unwrap() = right_eye_ip.clone();
        }
    }
    // 创建默认配置，设置"无限"重连
    let config = ESP32Config {
        connection_timeout_ms: 5000,
        heartbeat_timeout_ms: 2000,
        max_reconnect_attempts: usize::MAX, // 实际上是"无限"重连
        enable_mdns: true,
    };
    
    // 初始化面部流
    let face_ip = FACE_CONIG.functional.wifi_ip.clone();
    if !face_ip.is_empty() {
        println!("初始化面部流，IP: {}", face_ip);
        let _ = init_stream(&FACE_STREAM, DeviceType::Face, &face_ip, config.clone());
    }
    
    // 初始化左眼流
    let left_eye_ip = EYE_CONFIG.functional.left_ip.clone();
    if !left_eye_ip.is_empty() {
        println!("初始化左眼流，IP: {}", left_eye_ip);
        let _ = init_stream(&LEFT_EYE_STREAM, DeviceType::LeftEye, &left_eye_ip, config.clone());
    }
    
    // 初始化右眼流
    let right_eye_ip = EYE_CONFIG.functional.right_ip.clone();
    if !right_eye_ip.is_empty() {
        println!("初始化右眼流，IP: {}", right_eye_ip);
        let _ = init_stream(&RIGHT_EYE_STREAM, DeviceType::RightEye, &right_eye_ip, config.clone());
    }
    
    // 启动守护线程，只用于监控配置变更
    start_config_monitor();
    
    println!("全局视频流初始化完成");
}

// 初始化单个流
fn init_stream(
    stream_arc: &Arc<Mutex<Option<ESP32StreamController>>>,
    device_type: DeviceType,
    ip: &str,
    config: ESP32Config,
) -> Result<(), String> {
    let mut stream_guard = stream_arc.lock().unwrap();
    // 检查IP是否有效
    if ip.is_empty() || ip == "0.0.0.0" {
        return Err(format!("无效的IP地址: '{}'", ip));
    }
    
    // 创建控制器
    let mut controller = ESP32StreamController::new(config);
    
    // 初始化和启动
    if let Err(e) = controller.init(ip, device_type) {
        return Err(format!("初始化视频流失败: {}", e));
    }
    
    if let Err(e) = controller.start() {
        return Err(format!("启动视频流失败: {}", e));
    }
    
    // 保存控制器
    *stream_guard = Some(controller);
    
    Ok(())
}

// 更新流的IP - 修改后的版本，避免死锁
pub fn update_stream_ip(device_type: DeviceType, ip: String) -> Result<(), String> {
    // 检查IP是否有效
    if ip.is_empty() || ip == "0.0.0.0" {
        return Err(format!("无效的IP地址: '{}'", ip));
    }
    
    // 默认配置
    let config = ESP32Config {
        connection_timeout_ms: 5000,
        heartbeat_timeout_ms: 2000,
        max_reconnect_attempts: usize::MAX,
        enable_mdns: true,
    };
    
    // 获取对应的流引用
    let stream_arc = match device_type {
        DeviceType::Face => &FACE_STREAM,
        DeviceType::LeftEye => &LEFT_EYE_STREAM,
        DeviceType::RightEye => &RIGHT_EYE_STREAM,
        _ => return Err("未知设备类型".to_string()),
    };
    
    // 使用一个作用域来确保锁被及时释放
    {
        // 尝试锁定，设置超时避免永久阻塞
        let mut stream_option = match stream_arc.try_lock() {
            Ok(guard) => guard,
            Err(_) => {
                println!("警告: 无法获取流锁，设备可能正忙，稍后将重试");
                return Err("无法获取流锁，设备正忙".to_string());
            }
        };
        
        // 检查流是否存在
        if stream_option.is_none() {
            // 创建新的控制器
            let mut controller = ESP32StreamController::new(config);
            
            // 初始化和启动
            if let Err(e) = controller.init(&ip, device_type) {
                return Err(format!("初始化视频流失败: {}", e));
            }
            
            if let Err(e) = controller.start() {
                return Err(format!("启动视频流失败: {}", e));
            }
            
            // 储存控制器
            *stream_option = Some(controller);
            println!("成功创建 {:?} 设备的视频流，IP: {}", device_type, ip);
        } else if let Some(controller) = stream_option.as_mut() {
            // 停止现有流
            controller.stop();
            
            // 使用新IP更新
            if let Err(e) = controller.init(&ip, device_type) {
                return Err(format!("初始化视频流失败: {}", e));
            }
            
            // 重启流
            if let Err(e) = controller.start() {
                return Err(format!("启动视频流失败: {}", e));
            }
            
            println!("成功更新 {:?} 设备的IP为 {}", device_type, ip);
        }
    } // 锁在这里被释放
    
    Ok(())
}
// 更新单个流
fn update_stream(
    stream_arc: &Arc<Mutex<Option<ESP32StreamController>>>,
    device_type: DeviceType,
    ip: &str,
    config: ESP32Config,
) -> Result<(), String> {
    let mut stream_guard = stream_arc.lock().unwrap();

    match &mut *stream_guard {
        Some(controller) => {
            if controller.is_connected() {
                println!("{:?} 当前流已连接，忽略更新", device_type);
                return Ok(());
            }
            // 停止现有流
            controller.stop();
            
            // 使用新IP更新
            if let Err(e) = controller.init(ip, device_type) {
                return Err(format!("初始化视频流失败: {}", e));
            }
            
            // 重启流
            if let Err(e) = controller.start() {
                return Err(format!("启动视频流失败: {}", e));
            }
            
            println!("已成功更新 {:?} 设备的IP为 {}", device_type, ip);
            Ok(())
        },
        None => {
            // 如果流不存在，创建新的
            println!("流不存在，为 {:?} 创建新流，IP: {}", device_type, ip);
            init_stream(stream_arc, device_type, ip, config)
        }
    }
}

// 启动配置监控线程
fn start_config_monitor() {
    // 监控配置变化的线程
    std::thread::spawn(|| {
        let check_interval = Duration::from_secs(30); // 每30秒检查一次配置
        let mut last_face_ip = FACE_CONIG.functional.wifi_ip.clone();
        let mut last_left_eye_ip = EYE_CONFIG.functional.left_ip.clone();
        let mut last_right_eye_ip = EYE_CONFIG.functional.right_ip.clone();
        
        loop {
            // 检查面部IP变化
            let current_face_ip = FACE_CONIG.functional.wifi_ip.clone();
            if current_face_ip != last_face_ip && !current_face_ip.is_empty() && current_face_ip != "0.0.0.0" {
                println!("检测到面部IP变更: {} -> {}", last_face_ip, current_face_ip);
                last_face_ip = current_face_ip.clone();
                let _ = update_stream_ip(DeviceType::Face, current_face_ip);
            }
            
            // 检查左眼IP变化
            let current_left_eye_ip = EYE_CONFIG.functional.left_ip.clone();
            if current_left_eye_ip != last_left_eye_ip && !current_left_eye_ip.is_empty() && current_left_eye_ip != "0.0.0.0" {
                println!("检测到左眼IP变更: {} -> {}", last_left_eye_ip, current_left_eye_ip);
                last_left_eye_ip = current_left_eye_ip.clone();
                let _ = update_stream_ip(DeviceType::LeftEye, current_left_eye_ip);
            }
            
            // 检查右眼IP变化
            let current_right_eye_ip = EYE_CONFIG.functional.right_ip.clone();
            if current_right_eye_ip != last_right_eye_ip && !current_right_eye_ip.is_empty() && current_right_eye_ip != "0.0.0.0" {
                println!("检测到右眼IP变更: {} -> {}", last_right_eye_ip, current_right_eye_ip);
                last_right_eye_ip = current_right_eye_ip.clone();
                let _ = update_stream_ip(DeviceType::RightEye, current_right_eye_ip);
            }
            
            // 等待下一次检查
            std::thread::sleep(check_interval);
        }
    });
}

// 获取流句柄的辅助函数
pub fn get_face_stream() -> Option<ESP32Stream> {
    let stream_guard = FACE_STREAM.lock().unwrap();
    stream_guard.as_ref().map(|controller| controller.stream_handle())
}

pub fn get_left_eye_stream() -> Option<ESP32Stream> {
    let stream_guard = LEFT_EYE_STREAM.lock().unwrap();
    stream_guard.as_ref().map(|controller| controller.stream_handle())
}

pub fn get_right_eye_stream() -> Option<ESP32Stream> {
    let stream_guard = RIGHT_EYE_STREAM.lock().unwrap();
    stream_guard.as_ref().map(|controller| controller.stream_handle())
}

// 清理所有流
pub fn cleanup_all_streams() {
    // 停止面部流
    {
        let mut stream_guard = FACE_STREAM.lock().unwrap();
        if let Some(controller) = &mut *stream_guard {
            controller.stop();
        }
        *stream_guard = None;
    }
    
    // 停止左眼流
    {
        let mut stream_guard = LEFT_EYE_STREAM.lock().unwrap();
        if let Some(controller) = &mut *stream_guard {
            controller.stop();
        }
        *stream_guard = None;
    }
    
    // 停止右眼流
    {
        let mut stream_guard = RIGHT_EYE_STREAM.lock().unwrap();
        if let Some(controller) = &mut *stream_guard {
            controller.stop();
        }
        *stream_guard = None;
    }
    
    println!("所有视频流已清理");
}