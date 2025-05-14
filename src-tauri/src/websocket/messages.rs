use opencv::core::Mat;
use serde::{Deserialize, Serialize};
use std::time::Instant;

// Enum for device types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    Unknown,
    Face,
    LeftEye,
    RightEye,
}

// Request messages sent to the worker
#[derive(Debug, Clone)]
pub enum VideoRequest {
    // Connect to a video stream
    Connect {
        url: String,
        device_type: DeviceType,
    },
    // Disconnect from the current stream
    Disconnect,
    // Get the latest frame
    GetFrame,
    // Check connection status
    CheckStatus,
    // Stop the worker
    Shutdown,
}

// Response messages sent from the worker
#[derive(Debug, Clone)]
pub enum VideoResponse {
    // Connected successfully
    Connected {
        url: String,
    },
    // Failed to connect
    ConnectFailed {
        url: String,
        error: String,
    },
    // Disconnected
    Disconnected,
    // Frame data
    Frame(Option<Mat>),
    // Status info
    Status {
        connected: bool,
        url: String,
        battery: Option<f32>,
        brightness: Option<i32>,
    },
    // Error
    Error(String),
}

// Event messages sent from the worker
#[derive(Debug, Clone)]
pub enum VideoEvent {
    // New frame available
    NewFrame,
    // Connection status changed
    ConnectionChanged {
        connected: bool,
        url: String,
    },
    // Device status updated
    StatusUpdated {
        battery: Option<f32>,
        brightness: Option<i32>,
    },
    // Error occurred
    Error(String),
}

// Device status information
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceStatus {
    pub battery: Option<f32>,
    pub brightness: Option<i32>,
}

// Frame with metadata
#[derive(Debug, Clone)]
pub struct Frame {
    pub image: Mat,
    pub timestamp: Instant,
}