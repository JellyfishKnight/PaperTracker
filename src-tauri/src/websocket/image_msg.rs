use serde::{Deserialize, Serialize};
use opencv::core::Mat;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortState {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub battery: f32,
    pub brightness: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageRequest {
    GetImageBase64,
    GetImageOpenCV,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamSettingRequest {
    GetDeviceStatus,
    SetRotateAngle(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamSettingResponse {
    DeviceStatus(DeviceStatus),
}

#[derive(Debug, Clone)]
pub enum ImageResponse {
    Base64ImageData(Vec<u8>),
    OpenCVImageData(Mat),
}

// Frame with metadata
#[derive(Debug, Clone)]
pub struct Frame {
    pub image: Mat,
    pub timestamp: Instant,
}