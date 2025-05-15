use crossbeam::channel::{self, Receiver, RecvError, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use super::messages::{SerialEvent, SerialMessage, SerialRequest, SerialResponse};
use super::worker::{spawn_serial_worker, find_esp32_port};

// Client API for serial communication
pub struct SerialClient {
    // Channel for sending requests to the worker
    request_tx: Sender<SerialRequest>,
    // Channel for receiving responses from the worker
    response_rx: Receiver<SerialResponse>,
    // Channel for receiving events from the worker
    event_rx: Receiver<SerialEvent>,
    //
    message_rx: Receiver<SerialMessage>,
    // Handle to the worker thread
    worker_handle: Option<JoinHandle<()>>,
}

impl SerialClient {
    // Create a new serial client and start the worker thread
    pub fn new() -> Self {
        // Create channels
        let (request_tx, request_rx) = crossbeam::channel::unbounded();
        let (response_tx, response_rx) = crossbeam::channel::unbounded();
        let (event_tx, event_rx) = crossbeam::channel::unbounded();
        let (message_tx, message_rx) = crossbeam::channel::unbounded();
        
        // Spawn worker thread
        let worker_handle = spawn_serial_worker(request_rx, response_tx, event_tx, message_tx);
        
        SerialClient {
            request_tx,
            response_rx,
            event_rx,
            message_rx,
            worker_handle: Some(worker_handle),
        }
    }
    
    pub fn auto_connect(&self) -> Result<String, String> {
        // First check if we're already connected
        let is_connected = false;
        
        if is_connected {
            // If already connected, return success
            return Ok("Already connected".to_string());
        }
        
        // Otherwise try to find and connect to ESP32
        if let Some(port) = find_esp32_port() {
            self.open_port(port.clone())?;
            Ok(port)
        } else {
            Err("No ESP32 device found".to_string())
        }
    }
        
    // Open a serial port
    pub fn open_port(&self, port: String) -> Result<(), String> {
        self.request_tx.send(SerialRequest::OpenPort {
            port: port.clone(),
            baud_rate: 115200,
        }).map_err(|e| format!("Failed to send open port request: {}", e))?;
        
        // Wait for response
        match self.response_rx.recv() {
            Ok(SerialResponse::PortOpened { port: _ }) => {
                Ok(())
            }
            Ok(SerialResponse::PortOpenFailed { port, error }) => {
                Err(format!("Failed to open port {}: {}", port, error))
            }
            _ => Err("Unexpected response from worker".to_string()),
        }
    }
    
    // Close the current port
    pub fn close_port(&self) -> Result<(), String> {
        self.request_tx.send(SerialRequest::ClosePort)
            .map_err(|e| format!("Failed to send close port request: {}", e))?;
        
        // Wait for response
        match self.response_rx.recv() {
            Ok(SerialResponse::PortClosed) => Ok(()),
            _ => Err("Unexpected response from worker".to_string()),
        }
    }
    
    // Send WiFi configuration
    pub fn send_wifi_config(&self, ssid: String, password: String) -> Result<(), String> {
        self.request_tx.send(SerialRequest::SendWifiConfig { ssid, password })
            .map_err(|e| format!("Failed to send WiFi config request: {}", e))?;
        
        // Wait for response
        match self.response_rx.recv() {
            Ok(SerialResponse::WifiConfigSent) => Ok(()),
            Ok(SerialResponse::WifiConfigFailed(error)) => Err(error),
            _ => Err("Unexpected response from worker".to_string()),
        }
    }
    
    // Set brightness
    pub fn set_brightness(&self, brightness: u32) -> Result<(), String> {
        self.request_tx.send(SerialRequest::SetBrightness(brightness))
            .map_err(|e| format!("Failed to send brightness request: {}", e))?;
        
        // Wait for response
        match self.response_rx.recv() {
            Ok(SerialResponse::BrightnessSet) => Ok(()),
            Ok(SerialResponse::BrightnessSetFailed(error)) => Err(error),
            _ => Err("Unexpected response from worker".to_string()),
        }
    }
    
    // Restart the device
    pub fn restart_device(&self) -> Result<(), String> {
        self.request_tx.send(SerialRequest::RestartDevice)
            .map_err(|e| format!("Failed to send restart request: {}", e))?;
        
        // For restart, we don't wait for completion as it's asynchronous
        Ok(())
    }
    
    // Flash firmware
    pub fn flash_firmware(&self, device_type: String, firmware_type: String, firmware_path: Option<String>) -> Result<(), String> {
        self.request_tx.send(SerialRequest::FlashFirmware {
            device_type,
            firmware_type,
            firmware_path,
        }).map_err(|e| format!("Failed to send flash firmware request: {}", e))?;
        
        // For flashing, we don't wait for completion as it's asynchronous
        Ok(())
    }
    
    // Try to receive an event, non-blocking
    pub fn try_recv_event(&self) -> Option<SerialEvent> {
        match self.event_rx.try_recv() {
            Ok(event) => Some(event),
            Err(TryRecvError::Empty) => {
                println!("No event available");
                None
            }
            Err(TryRecvError::Disconnected) => {
                println!("Event channel disconnected");
                None
            }
        }
    }
    
    // Get the event receiver for external use
    pub fn get_event_receiver(&self) -> Receiver<SerialEvent> {
        let (tx, rx) = crossbeam::channel::unbounded();
        let event_rx = self.event_rx.clone();
        
        thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                if tx.send(event).is_err() {
                    break;
                }
            }
        });
        
        rx
    }

    pub fn try_recv_message(&self) -> Option<SerialMessage> {
        match self.message_rx.try_recv() {
            Ok(message) => Some(message),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                println!("Message channel disconnected");
                None
            }
        }
    }

    pub fn get_message_receiver(&self) -> Receiver<SerialMessage> {
        let (tx, rx) = crossbeam::channel::unbounded();
        let message_rx = self.message_rx.clone();
        
        thread::spawn(move || {
            while let Ok(message) = message_rx.recv() {
                if tx.send(message).is_err() {
                    break;
                }
            }
        });
        
        rx
    }
}

impl Drop for SerialClient {
    fn drop(&mut self) {
        // Send shutdown request to worker
        let _ = self.request_tx.send(SerialRequest::Shutdown);
        
        // Wait for worker thread to finish
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}