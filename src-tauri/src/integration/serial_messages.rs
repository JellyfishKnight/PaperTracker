use std::sync::{Arc, Mutex};
use crate::serial::messages::SerialMessage;
use crate::serial::{SerialClient, SerialEvent};
use crate::websocket::{self, VideoRequest};
use crate::websocket::{manager::VideoStreamManager, DeviceType};
use crate::utils::consts::*;
use crossbeam::channel::{Receiver, Sender};

pub fn listen_for_serial_events(
    serial_rx: Receiver<SerialMessage>, 
    face_tx: Sender<VideoRequest>,
    left_eye_tx: Sender<VideoRequest>,
    right_eye_tx: Sender<VideoRequest>,) {
    std::thread::spawn(move || {
        println!("Listening for serial events...");
        while let Ok(event) = serial_rx.recv() {
            match event {
                SerialMessage::DeviceStatus(status) => {
                    // Only update if it's a valid device type and has a valid IP
                    if !status.ip.is_empty() {
                        println!("Updating stream IP for device type {:?} to {}", status.device_type, status.ip);
                        // Convert device_type from u32 to DeviceType enum
                        match status.device_type {
                            DEVICE_TYPE_FACE => {
                                face_tx.send(VideoRequest::Connect { url: status.ip, device_type: websocket::messages::DeviceType::Face }).ok();
                            },
                            DEVICE_TYPE_LEFT_EYE => {
                                left_eye_tx.send(VideoRequest::Connect { url: status.ip, device_type: websocket::messages::DeviceType::LeftEye }).ok();
                            },
                            DEVICE_TYPE_RIGHT_EYE => {
                                right_eye_tx.send(VideoRequest::Connect { url: status.ip, device_type: websocket::messages::DeviceType::RightEye }).ok();
                            }
                            _ => {
                                println!("Unknown device type: {:?}", status.device_type);
                            }
                        };
                    }

                },
                _ => {
                    println!("Received unknown event: {:?}", event);
                } // Ignore other events
            }
        }
    });
}