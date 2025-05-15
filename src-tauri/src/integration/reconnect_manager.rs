// integration/reconnect_manager.rs

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use crate::serial::{SerialClient, SerialEvent};
use crate::websocket::{VideoStreamClient, DeviceType, VideoEvent};
use crate::paper_tracker_config::config::{EYE_CONFIG, FACE_CONIG};

pub struct ReconnectManager {
    serial_client: Arc<Mutex<SerialClient>>,
    video_clients: Arc<Mutex<Vec<(DeviceType, VideoStreamClient)>>>,
    handles: Vec<JoinHandle<()>>,
    running: Arc<Mutex<bool>>,
}

impl ReconnectManager {
    pub fn new(serial_client: Arc<Mutex<SerialClient>>, 
               left_eye_client: VideoStreamClient, 
               right_eye_client: VideoStreamClient, 
               face_client: VideoStreamClient) -> Self {
        let video_clients = Arc::new(Mutex::new(vec![
            (DeviceType::LeftEye, left_eye_client),
            (DeviceType::RightEye, right_eye_client),
            (DeviceType::Face, face_client),
        ]));

        ReconnectManager {
            serial_client,
            video_clients,
            handles: Vec::new(),
            running: Arc::new(Mutex::new(true)),
        }
    }

    pub fn start(&mut self) {
        // Start serial reconnect guardian
        self.start_serial_guardian();
        
        // Start video stream guardians
        self.start_video_guardians();
    }

    fn start_serial_guardian(&mut self) {
        let client = self.serial_client.clone();
        let running = self.running.clone();
        
        let handle = thread::spawn(move || {
            println!("Serial connection guardian started");
            
            // Track connection state locally
            let mut connected = false;  
            while *running.lock().unwrap() {
                if !connected {
                    println!("Serial connection not established");
                    // Try to connect if not connected
                    println!("Attempting to connect to ESP32 device...");
                    let serial_client = client.lock().unwrap();
                    match serial_client.auto_connect() {
                        Ok(port) => {
                            println!("Successfully connected to ESP32 at port {}", port);
                            connected = true;
                        },
                        Err(e) => {
                            println!("Failed to connect to ESP32: {}", e);
                            // Still not connected, will retry on next loop
                        }
                    }
                } else {
                    // Check if still connected
                    let mut disconnect_detected = false;
                    
                    {
                        let serial_client = client.lock().unwrap();
                        // Check for disconnect events
                        if let Some(SerialEvent::Error(e)) = serial_client.try_recv_event() {
                            println!("Detected serial device error: {}", e);
                            disconnect_detected = true;
                        }
                    }
                    
                    // Also check for port existence as a backup method
                    if !disconnect_detected {
                        let port_exists = {
                            // Try to find ESP32 port
                            use crate::serial::worker::find_esp32_port;
                            find_esp32_port().is_some()
                        };
                        
                        if !port_exists {
                            println!("ESP32 device no longer detected");
                            disconnect_detected = true;
                        }
                    }
                    
                    if disconnect_detected {
                        println!("Serial connection lost");
                        connected = false; // Will trigger reconnection on next loop
                    }
                }
                
                // Sleep to avoid consuming too much CPU
                thread::sleep(Duration::from_secs(2));
            }
            
            println!("Serial connection guardian stopped");
        });
        
        self.handles.push(handle);
    }
    
    fn start_video_guardians(&mut self) {
        // Get IP addresses from config
        let left_eye_ip = EYE_CONFIG.functional.left_ip.clone();
        let right_eye_ip = EYE_CONFIG.functional.right_ip.clone();
        let face_ip = FACE_CONIG.functional.wifi_ip.clone();
        
        let video_clients = self.video_clients.clone();
        let running = self.running.clone();
        
        // Track connection states
        let connection_states = Arc::new(Mutex::new(vec![
            (DeviceType::LeftEye, false),
            (DeviceType::RightEye, false),
            (DeviceType::Face, false),
        ]));
        
        // Create guardian thread
        let handle = thread::spawn(move || {
            println!("Video stream connection guardian started");
            
            // Helper function to get device IP
            let get_device_ip = |device_type: &DeviceType| -> String {
                match device_type {
                    DeviceType::LeftEye => left_eye_ip.clone(),
                    DeviceType::RightEye => right_eye_ip.clone(),
                    DeviceType::Face => face_ip.clone(),
                    _ => String::new()
                }
            };
            
            while *running.lock().unwrap() {
                let mut states = connection_states.lock().unwrap();
                let clients = video_clients.lock().unwrap();
                
                for (idx, (device_type, client)) in clients.iter().enumerate() {
                    let (_, is_connected) = states[idx];
                    
                    if !is_connected {
                        // Try to connect if not connected
                        let device_ip = get_device_ip(device_type);
                        if !device_ip.is_empty() {
                            println!("Attempting to connect to {:?} at {}", device_type, device_ip);
                            match client.connect(device_ip, *device_type) {
                                Ok(_) => {
                                    println!("Connection request sent for {:?}", device_type);
                                    // Note: we'll verify the connection status on the next loop
                                },
                                Err(e) => {
                                    println!("Failed to initiate connection to {:?}: {}", device_type, e);
                                }
                            }
                        }
                    }
                    
                    // Check current connection status
                    let current_status = match client.check_status() {
                        Ok((connected, url, _, _)) => {
                            if connected != is_connected {
                                if connected {
                                    println!("{:?} connected to {}", device_type, url);
                                } else {
                                    println!("{:?} disconnected", device_type);
                                }
                            }
                            connected
                        },
                        Err(e) => {
                            println!("Failed to check {:?} status: {}", device_type, e);
                            false
                        }
                    };
                    
                    // Update connection state
                    states[idx] = (*device_type, current_status);
                }
                
                drop(clients);
                drop(states);
                
                // Sleep to avoid consuming too much CPU
                thread::sleep(Duration::from_secs(3));
            }
            
            println!("Video stream connection guardian stopped");
        });
        
        self.handles.push(handle);
    }
    
    pub fn stop(&mut self) {
        // Signal threads to stop
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }
        
        // Wait for threads to finish
        while let Some(handle) = self.handles.pop() {
            let _ = handle.join();
        }
    }
}

impl Drop for ReconnectManager {
    fn drop(&mut self) {
        self.stop();
    }
}