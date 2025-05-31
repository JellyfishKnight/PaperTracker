use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PortState {
    Disconnected,
    Connected
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashCommand {
    pub tool_path: String,
    pub boot_loader_path: String,
    pub partition_path: String,
    pub firmware_path: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SerialRequest {
    // tool path
    Restart(String),
    // tool path, firmware path
    Flash(FlashCommand),
    GetStatus,
    Stop,
    Start
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SerialResponse {
    // restart result
    Restart(bool, String),
    // flash result, progress
    Flash((
        String,
        i32,
    )),
    // is connected, device type
    Status((
        PortState,
        i32
    )),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceStatus {
    pub ip: String,
    pub brightness: i32,
    pub power: f32,
    pub device_type: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WifiError {
    pub ssid: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SerialMessage {
    DeviceStatus(DeviceStatus),
    WifiError(WifiError),
    GeneralMessage(String),
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WifiConfig {
    pub ssid: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SerialSendPacket {
    WifiConfig(WifiConfig),
    Brightness(i32),
}

