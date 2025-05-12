// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod serial;
mod paper_tracker_config;
mod utils;
mod websocket;

use std::time::Duration;

use serial::esp32::start_serial_mod;
use websocket::esp32_video_stream::*;
use opencv::{imgproc, highgui, core};

fn main() {
    // start_serial_mod();

    // let config = ESP32Config {
    //     connection_timeout_ms: 5000,  // 给予更长的连接时间
    //     heartbeat_timeout_ms: 2000,   // 每2秒检查心跳
    //     max_reconnect_attempts: 5,    // 最多尝试5次重连
    //     enable_mdns: true,           // 启用mDNS查询
    // };
    // // 创建控制器
    // let mut controller = ESP32StreamController::new(config);

    // // 初始化连接
    // println!("初始化连接...");
    // if let Err(e) = controller.init("192.168.1.221", DeviceType::Face) {
    //     eprintln!("初始化失败: {}", e);
    //     return ;
    // }
    // // 启动流
    // println!("开始连接...");
    // if let Err(e) = controller.start() {
    //     eprintln!("启动流失败: {}", e);
    //     return ;
    // }
    // // 获取流句柄，可以在其他线程中使用
    // let stream = controller.stream_handle();

    // // 创建窗口显示视频
    // highgui::named_window("ESP32 Stream", highgui::WINDOW_NORMAL).unwrap();
    // // 等待连接建立
    // println!("等待连接建立...");
    // let mut connect_attempts = 0;
    // while connect_attempts < 20 {
    //     if stream.status() == StreamStatus::Connected {
    //         println!("连接成功！");
    //         break;
    //     }
        
    //     println!("连接中... 尝试 {}/20", connect_attempts + 1);
    //     std::thread::sleep(Duration::from_millis(1000));
    //     connect_attempts += 1;
    // }
    
    // if stream.status() != StreamStatus::Connected {
    //     eprintln!("无法建立连接，退出程序");
    //     controller.stop();
    //     return ;
    // }
    // println!("开始接收视频流和设备状态信息");
    
    // // 主循环 - 显示视频
    // let mut last_fps_time = std::time::Instant::now();
    // let mut frames = 0;
    
    // loop {
    //     // 获取最新帧
    //     if let Some(frame) = stream.get_latest_frame() {
    //         // 获取设备状态
    //         let battery = stream.battery_percentage().unwrap_or(0.0);
    //         let brightness = stream.brightness_value().unwrap_or(0);
            
    //         // 在帧上显示状态信息
    //         let mut display_frame = frame.clone();
    //         let status_text = format!("电池: {:.1}%  亮度: {}", battery, brightness);
            
    //         imgproc::put_text(
    //             &mut display_frame,
    //             &status_text,
    //             core::Point::new(10, 30),
    //             imgproc::FONT_HERSHEY_SIMPLEX,
    //             0.8,
    //             core::Scalar::new(0.0, 255.0, 0.0, 0.0),
    //             2,
    //             imgproc::LINE_AA,
    //             false,
    //         ).unwrap();
            
    //         // 显示帧
    //         highgui::imshow("ESP32 Stream", &display_frame).unwrap();
            
    //         // 计算并显示帧率
    //         frames += 1;
    //         let elapsed = std::time::Instant::now().duration_since(last_fps_time);
    //         if elapsed.as_secs() >= 1 {
    //             println!("显示帧率: {:.1} FPS", frames as f64 / elapsed.as_secs_f64());
    //             frames = 0;
    //             last_fps_time = std::time::Instant::now();
    //         }
    //     }
        
    //     // 检查键盘输入，ESC键退出
    //     let key = highgui::wait_key(1).unwrap();
    //     if key == 27 {
    //         break;
    //     }
    // }
    
    // println!("停止视频流...");
    // controller.stop();
    papertracker_lib::run();
}
