use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use crate::serial::esp32::{Esp32Serial, serial_watchdog};

// Define the global static variable for ESP32 serial
pub static ESP32_SERIAL: Lazy<Arc<Mutex<Option<Esp32Serial>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

// Initialize the serial connection and start the watchdog
pub fn init_global_serial() {
    // Start the serial watchdog thread
    let esp32_serial_clone = ESP32_SERIAL.clone();
    std::thread::spawn(move || {
        serial_watchdog(esp32_serial_clone);
    });
}