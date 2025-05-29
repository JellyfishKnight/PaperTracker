use std::{collections::VecDeque, sync::mpsc::TryRecvError, time::Instant};
use crossbeam::channel::{Sender, Receiver};
use opencv::{core::{Mat, MatTraitConst, Vector}, imgcodecs};
use tauri::{App, AppHandle, Emitter, Runtime};
use crate::{serial::serial_msg::SerialMessage, utils::consts::{DEVICE_TYPE_FACE, DEVICE_TYPE_LEFT_EYE, DEVICE_TYPE_RIGHT_EYE}};
use super::image_msg::{DeviceStatus, Frame, ImageRequest, ImageResponse, PortState, StreamSettingRequest, StreamSettingResponse};
use url::Url;
use tungstenite::{connect, Message, WebSocket};
use base64::{Engine as _, engine::general_purpose};
use ftlog::*;

pub struct ImageStream<R: Runtime> {
    request_rx: Receiver<ImageRequest>,
    request_tx: Sender<ImageRequest>,
    img_response_tx: bus::Bus<ImageResponse>,
    setting_response_tx: bus::Bus<StreamSettingResponse>,
    settings_rx: Receiver<StreamSettingRequest>,
    settings_tx: Sender<StreamSettingRequest>,
    serial_msg_rx: bus::BusReader<SerialMessage>,

    device_type: i32,
    port_state: PortState,
    ip: String,
    run: bool,
    device_status: DeviceStatus,
    image_buffer: VecDeque<Frame>,
    rotate_angle: f64,

    app_handle: AppHandle<R>,
}

impl<R: Runtime> ImageStream<R> {
    pub fn new(serial_msg_rx: bus::BusReader<SerialMessage>, ip: String, device_type: i32, app: AppHandle<R>) -> Self {
        let (request_tx, request_rx) = crossbeam::channel::unbounded();
        let img_response_tx = bus::Bus::<ImageResponse>::new(1);
        let (settings_tx, settings_rx) = crossbeam::channel::unbounded();
        let setting_response_tx = bus::Bus::<StreamSettingResponse>::new(1);
        ImageStream {
            request_rx,
            request_tx,
            img_response_tx,
            setting_response_tx,
            settings_rx,
            settings_tx,
            serial_msg_rx,
            device_type,
            port_state: PortState::Disconnected,
            ip,
            run: false,
            device_status: DeviceStatus { battery: 0, brightness: 0, wifi: String::new() },
            image_buffer: VecDeque::new(),
            rotate_angle: 0.0,
            app_handle: app
        }
    }

    pub fn get_request_tx(&self) -> Sender<ImageRequest> {
        self.request_tx.clone()
    }

    pub fn get_response_rx(&mut self) -> bus::BusReader<ImageResponse> {
        self.img_response_tx.add_rx()
    }

    pub fn get_setting_response_rx(&mut self) -> bus::BusReader<StreamSettingResponse> {
        self.setting_response_tx.add_rx()
    }

    pub fn get_settings_tx(&self) -> Sender<StreamSettingRequest> {
        self.settings_tx.clone()
    }

    pub fn start(&mut self) {
        let mut port: Option<WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>> = None;
        self.run = true;
        loop {
            if let PortState::Connected = self.port_state {
                match self.device_type {
                    DEVICE_TYPE_FACE => {
                        let _ = self.app_handle.emit("face_image_stream_status", "面捕WIFI已连接");
                    }
                    DEVICE_TYPE_LEFT_EYE => {
                        let _ = self.app_handle.emit("left_eye_image_stream_status", "左眼WIFI已连接");
                    }
                    DEVICE_TYPE_RIGHT_EYE => {
                        let _ = self.app_handle.emit("right_eye_image_stream_status", "右眼WIFI已连接");
                    }
                    _ => ()
                }
            }
            if let PortState::Disconnected = self.port_state {
                match self.device_type {
                    DEVICE_TYPE_FACE => {
                        let _ = self.app_handle.emit("face_image_stream_status", "面捕WIFI未连接");
                    }
                    DEVICE_TYPE_LEFT_EYE => {
                        let _ = self.app_handle.emit("left_eye_image_stream_status", "左眼WIFI未连接");
                    }
                    DEVICE_TYPE_RIGHT_EYE => {
                        let _ = self.app_handle.emit("right_eye_image_stream_status", "右眼WIFI未连接");
                    }
                    _ => ()
                }
            }
            match self.settings_rx.try_recv() {
                Ok(request) => {
                    match request {
                        StreamSettingRequest::GetDeviceStatus => {
                            self.get_device_status();
                        }
                        StreamSettingRequest::SetRotateAngle(angle) => {
                            self.rotate_angle = angle;
                            info!("Set rotate angle to: {}", angle);
                        }
                    }
                }
                Err(crossbeam::channel::TryRecvError::Disconnected) => {
                    // Handle error in receiving settings requests
                    error!("disconnected settings request in image stream");
                }
                _ => ()
            }
            // Handle settings requests
            match self.request_rx.try_recv() {
                Ok(request) => {
                    self.handle_request(request);
                }
                Err(crossbeam::channel::TryRecvError::Disconnected) => {
                    // Handle error in receiving requests
                    error!("disconnected request in serial");
                }
                _ => ()
            }
            match self.serial_msg_rx.try_recv() {
                Ok(msg) => {
                    self.handle_serial_message(msg);
                }
                Err(TryRecvError::Disconnected) => {
                    // Handle error in receiving serial messages
                    error!("disconnected serial message in image stream");
                }
                _ => ()
            }
            if let PortState::Disconnected = self.port_state {
                if self.run {
                    if self.connect(&mut port) {
                        info!("Stream {} Connected to: {}", self.device_type, self.ip);
                        match self.device_type {
                            DEVICE_TYPE_FACE => {
                                let _ = self.app_handle.emit("face_ip", self.ip.as_str());
                            }
                            DEVICE_TYPE_LEFT_EYE => {
                                let _ = self.app_handle.emit("left_eye_ip", self.ip.as_str());
                            }
                            DEVICE_TYPE_RIGHT_EYE => {
                                let _ = self.app_handle.emit("right_eye_ip", self.ip.as_str());
                            }
                            _ => ()
                        }
                        self.device_status.wifi = self.ip.clone();
                        self.port_state = PortState::Connected;
                    } else {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                } else {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    continue;
                }
            }
            if !self.run {
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
            if let Some(ref mut port) = port {
                match port.read() {
                    Ok(msg) => {
                        self.handle_websocket_message(msg);
                    }
                    Err(e) => {
                        error!("Stream {} Error reading from port: {}", self.device_type, e);
                        self.port_state = PortState::Disconnected;
                    }
                }
            }
        }
    }

    fn handle_websocket_message(&mut self, msg: Message) {
        match msg {
            Message::Binary(data) => {
                // Process image data
                if data.len() < 10 {
                    warn!("Received binary data too small to be an image");
                    return ;
                }
                // Decode image with OpenCV
                match imgcodecs::imdecode(&Mat::from_slice(&data).unwrap(), imgcodecs::IMREAD_COLOR) {
                    Ok(image) if !image.empty() => {
                        // Create a new frame and add it to the buffer
                        let mut frame = Frame {
                            image,
                            timestamp: Instant::now(),
                        };
                        // Keep only the most recent frame
                        self.image_buffer.clear();
                        let center = opencv::core::Point2f::new((frame.image.cols() / 2) as f32, (frame.image.rows() / 2) as f32);
                        let rotation_matrix = opencv::imgproc::get_rotation_matrix_2d(center, self.rotate_angle, 1.0).unwrap();
                        let mut rotated_image = opencv::core::Mat::default();
                        opencv::imgproc::warp_affine(
                            &frame.image,
                            &mut rotated_image,
                            &rotation_matrix,
                            frame.image.size().unwrap(),
                            opencv::imgproc::INTER_LINEAR,
                            opencv::core::BORDER_CONSTANT,
                            opencv::core::Scalar::default(),
                        ).unwrap();
                        frame.image = rotated_image;
                        self.image_buffer.push_back(frame);
                    }
                    Ok(_) => {
                        println!("Decoded image is empty");
                    }
                    Err(e) => {
                        println!("Failed to decode image: {}", e);
                    }
                }            
            }
            Message::Text(text) => {
                match serde_json::from_str::<DeviceStatus>(&text) {
                    Ok(status) => {
                        self.device_status = status.clone();
                    }
                    Err(e) => {
                        println!("Failed to parse status message: {}, message: {}", e, text);
                    }
                }
            }
            Message::Close(_) => {
                // Handle close message
                info!("Stream {} Connection closed", self.device_type);
                self.port_state = PortState::Disconnected;
            }
            _ => ()
        }
    }

    fn handle_request(&mut self, request: ImageRequest) {
        match request {
            ImageRequest::GetImageBase64 => {
                self.get_image_base64();
            }
            ImageRequest::GetImageOpenCV => {
                self.get_image_opencv();
            }
        }
    }

    fn connect(&mut self, port: &mut Option<WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>>) -> bool {
        let mut ip_list = Vec::new();
        match self.device_type {
            DEVICE_TYPE_FACE => {
                ip_list.push("ws://paper1.local:80/ws");
            },
            DEVICE_TYPE_LEFT_EYE => {
                ip_list.push("ws://paper2.local:80/ws");
            }
            DEVICE_TYPE_RIGHT_EYE => {
                ip_list.push("ws://paper3.local:80/ws");
            }
            _ => {
                error!("Unknown device type: {}", self.device_type);
            }
        }
        // make ip live longer
        let ip_formated = self.get_connect_url_from_self_ip();
        if !self.ip.is_empty() {
            ip_list.push(ip_formated.as_str());
        }

        for url_to_try in ip_list {
            info!("Stream {} Trying to connect to: {}", self.device_type, url_to_try);
            match Url::parse(url_to_try) {
                Ok(url) => {
                    match connect(url) {
                        Ok((mut ws, _)) => {
                            info!("Stream {} Connected to: {}", self.device_type, url_to_try);

                            if let tungstenite::stream::MaybeTlsStream::Plain(stream) = ws.get_mut() {
                                if let Err(e) = stream.set_read_timeout(Some(std::time::Duration::from_secs(2))) {
                                    warn!("Stream {} Failed to set read timeout: {}", self.device_type, e);
                                }
                            }
                            *port = Some(ws);
                            return true;
                        }
                        Err(e) => {
                            error!("Stream {} Failed to connect to {}: {}", self.device_type, url_to_try, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Stream {} Invalid URL {}: {}", self.device_type, url_to_try, e);
                }
            }
        }

        port.is_some()
    }

    fn get_connect_url_from_self_ip(&self) -> String {
        if !self.ip.starts_with("ws://") && !self.ip.starts_with("wss://") {
            if self.ip.starts_with("http://") {
                format!("ws://{}/ws", self.ip.strip_prefix("http://").unwrap_or(&self.ip))
            } else if self.ip.starts_with("https://") {
                format!("wss://{}/ws", self.ip.strip_prefix("https://").unwrap_or(&self.ip))
            } else {
                // Assume host:port format
                let mut ws_url = format!("ws://{}", self.ip);
                if !ws_url.contains(':') {
                    ws_url.push_str(":80");
                }
                if !ws_url.contains("/ws") {
                    ws_url.push_str("/ws");
                }
                ws_url
            }    
        } else {
            self.ip.clone()
        }
    }

    fn handle_serial_message(&mut self, msg: SerialMessage) {
        if let SerialMessage::DeviceStatus(status) = msg {
            if self.device_type == status.device_type {
                self.ip = status.ip.clone();
            }
        }
    }

    fn get_image_base64(&mut self) {
        if let Some(frame) = self.image_buffer.front() {
            // Convert image to base64
            let mut encoded_data = Vector::<u8>::new();
            
            // 设置 JPEG 编码参数
            let mut params = Vector::<i32>::new();
            params.push(imgcodecs::IMWRITE_JPEG_QUALITY);
            params.push(90); // JPEG 质量 (0-100)
            
            // 将 Mat 编码为 JPEG 格式
            match imgcodecs::imencode(".jpg", &frame.image, &mut encoded_data, &params) {
                Ok(_) => {
                    // 将编码后的数据转换为 Vec<u8>
                    let vec_data: Vec<u8> = encoded_data.into();
                    
                    // 使用 base64 编码
                    let base64_string = general_purpose::STANDARD.encode(&vec_data);
                    // 广播 base64 响应
                    self.img_response_tx.broadcast(ImageResponse::Base64ImageData(base64_string.into_bytes()));
                }
                Err(e) => {
                    error!("Failed to encode image to JPEG: {}", e);
                }
            }
        } else {
            debug!("No image available in buffer");
        }
    }
    
    fn get_image_opencv(&mut self) {
        // Implement the logic to get image in OpenCV format
        // This is a placeholder implementation
        if let Some(frame) = self.image_buffer.front() {
            self.img_response_tx.broadcast(ImageResponse::OpenCVImageData(frame.image.clone()));
        } else {
            debug!("No image available in buffer");
        }
    }

    fn get_device_status(&mut self) {
        // Implement the logic to get device status
        // This is a placeholder implementation
        self.setting_response_tx.broadcast(StreamSettingResponse::DeviceStatus(self.device_status.clone()));
    }


}