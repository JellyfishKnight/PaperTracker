use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use super::client::VideoStreamClient;
use super::messages::{DeviceType, VideoEvent};

// Manager for multiple video streams
pub struct VideoStreamManager {
    // Map of device type to client
    clients: Arc<Mutex<HashMap<DeviceType, VideoStreamClient>>>,
}

impl VideoStreamManager {
    // Create a new video stream manager
    pub fn new() -> Self {
        VideoStreamManager {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    // Get or create a client for the given device type
    pub fn get_client(&self, device_type: DeviceType) -> VideoStreamClient {
        let mut clients = self.clients.lock().unwrap();
        
        if !clients.contains_key(&device_type) {
            // Create a new client
            let client = VideoStreamClient::new();
            clients.insert(device_type, client);
        }
        
        // Clone is only cloning the channels, not the entire client
        clients.get(&device_type).unwrap().clone()
    }
    
    // Connect to a stream
    pub fn connect(&self, device_type: DeviceType, url: String) -> Result<(), String> {
        let client = self.get_client(device_type);
        client.connect(url, device_type)
    }
    
    // Disconnect from a stream
    pub fn disconnect(&self, device_type: DeviceType) -> Result<(), String> {
        let client = self.get_client(device_type);
        client.disconnect()
    }
    
    // Update device IP
    pub fn update_device_ip(&self, device_type: DeviceType, ip: String) -> Result<(), String> {
        // Disconnect and reconnect with the new IP
        let client = self.get_client(device_type);
        let _ = client.disconnect();
        client.connect(ip, device_type)
    }
    
    // Get the latest frame from a device
    pub fn get_frame(&self, device_type: DeviceType) -> Result<Option<opencv::core::Mat>, String> {
        let client = self.get_client(device_type);
        client.get_frame()
    }
    
    // Check if a device is connected
    pub fn is_connected(&self, device_type: DeviceType) -> bool {
        let client = self.get_client(device_type);
        match client.check_status() {
            Ok((connected, _, _, _)) => connected,
            Err(_) => false,
        }
    }
    
    // Close all streams
    pub fn close_all(&self) {
        let mut clients = self.clients.lock().unwrap();
        for (_, client) in clients.drain() {
            let _ = client.disconnect();
        }
    }
}

impl Clone for VideoStreamClient {
    fn clone(&self) -> Self {
        // Create new channels
        let (request_tx, request_rx) = crossbeam::channel::unbounded();
        let (response_tx, response_rx) = crossbeam::channel::unbounded();
        let (event_tx, event_rx) = crossbeam::channel::unbounded();
        
        // Forward messages between original and new channels
        let original_request_tx = self.request_tx.clone();
        std::thread::spawn(move || {
            while let Ok(request) = request_rx.recv() {
                if original_request_tx.send(request).is_err() {
                    break;
                }
            }
        });
        
        let original_response_rx = self.response_rx.clone();
        std::thread::spawn(move || {
            while let Ok(response) = original_response_rx.recv() {
                if response_tx.send(response).is_err() {
                    break;
                }
            }
        });
        
        let original_event_rx = self.event_rx.clone();
        std::thread::spawn(move || {
            while let Ok(event) = original_event_rx.recv() {
                if event_tx.send(event).is_err() {
                    break;
                }
            }
        });
        
        VideoStreamClient {
            request_tx,
            response_rx,
            event_rx,
            worker_handle: None, // No handle because we're sharing the original worker
        }
    }
}