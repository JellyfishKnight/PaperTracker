use serde::{Deserialize, Serialize};

// Request messages sent to the worker
#[derive(Debug, Clone)]
pub enum SerialRequest {
    // Open a serial port with the given name
    OpenPort {
        port: String,
        baud_rate: u32,
    },
    // Close the current serial port
    ClosePort,
    // Send data to the serial port
    SendData(Vec<u8>),
    // Send SSID and password for WiFi configuration
    SendWifiConfig {
        ssid: String,
        password: String,
    },
    // Set the brightness level
    SetBrightness(u32),
    // Restart the ESP32 device
    RestartDevice,
    // Flash firmware to the ESP32 device
    FlashFirmware {
        device_type: String,
        firmware_type: String,
        firmware_path: Option<String>,
    },
    // Shutdown the worker thread
    Shutdown,
}

// Response messages sent from the worker
#[derive(Debug, Clone)]
pub enum SerialResponse {
    // Port opened successfully
    PortOpened {
        port: String,
    },
    // Failed to open port
    PortOpenFailed {
        port: String,
        error: String,
    },
    // Port closed
    PortClosed,
    // Data sent successfully
    DataSent,
    // Failed to send data
    SendFailed(String),
    // WiFi configuration sent
    WifiConfigSent,
    // WiFi configuration failed
    WifiConfigFailed(String),
    // Brightness set
    BrightnessSet,
    // Failed to set brightness
    BrightnessSetFailed(String),
    // Device restarted
    DeviceRestarted,
    // Failed to restart device
    RestartFailed(String),
    // Firmware flashed
    FirmwareFlashed,
    // Failed to flash firmware
    FlashFailed(String),
}

// Device status information from ESP32
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub ip: String,
    pub brightness: u32,
    pub power: u32,
    pub version: u32,
    pub device_type: u32,
}

// Event messages sent from the worker
#[derive(Debug, Clone)]
pub enum SerialEvent {
    // Device status update
    DeviceStatus(DeviceStatus),
    // Device connected
    DeviceConnected {
        port: String,
    },
    // Device disconnected
    DeviceDisconnected,
    // Firmware flash progress
    FlashProgress {
        progress: f32,
        message: String,
    },
    // Restart progress
    RestartProgress {
        progress: f32,
        message: String,
    },
    // Error message
    Error(String),
}