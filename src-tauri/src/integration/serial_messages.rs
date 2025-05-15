use std::sync::{Arc, Mutex};
use crate::serial::messages::SerialMessage;
use crate::serial::{SerialClient, SerialEvent};
use crate::websocket::{manager::VideoStreamManager, DeviceType};
use crate::utils::consts::*;
use crossbeam::channel::Receiver;

pub fn listen_for_serial_events(
    serial_rx: Receiver<SerialMessage>, 
    video_manager: Arc<Mutex<VideoStreamManager>>) {
    let video_manager = video_manager.clone();
    
    std::thread::spawn(move || {
        while let Ok(event) = serial_rx.recv() {
            match event {
                SerialMessage::DeviceStatus(status) => {
                    // Convert device_type from u32 to DeviceType enum
                    let device_type = match status.device_type {
                        DEVICE_TYPE_FACE => DeviceType::Face,
                        DEVICE_TYPE_LEFT_EYE => DeviceType::LeftEye,
                        DEVICE_TYPE_RIGHT_EYE => DeviceType::RightEye,
                        _ => DeviceType::Unknown,
                    };
                    
                    // Only update if it's a valid device type and has a valid IP
                    if device_type != DeviceType::Unknown && !status.ip.is_empty() {
                        println!("Updating stream IP for device type {:?} to {}", device_type, status.ip);
                        let _ = video_manager.lock().unwrap().update_device_ip(device_type, status.ip);
                    }
                },
                _ => {
                
                } // Ignore other events
            }
        }
    });
}