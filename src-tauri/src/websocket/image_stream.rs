use std::{collections::VecDeque, os::unix::net::SocketAddr, time::Instant};

use crossbeam::channel::{Sender, Receiver};
use opencv::{core::{Mat, MatTraitConst}, imgcodecs};
use crate::{serial::serial_msg::SerialMessage, utils::consts::{DEVICE_TYPE_FACE, DEVICE_TYPE_LEFT_EYE, DEVICE_TYPE_RIGHT_EYE}};
use super::image_msg::{DeviceStatus, Frame, ImageRequest, ImageResponse, PortState};
use url::Url;
use tungstenite::{connect, Message, WebSocket};

use ftlog::*;

pub struct ImageStream {
    request_rx: Receiver<ImageRequest>,
    request_tx: Sender<ImageRequest>,
    response_tx: bus::Bus<ImageResponse>,
    serial_msg_rx: Receiver<SerialMessage>,

    device_type: i32,
    port_state: PortState,
    ip: String,
    run: bool,
    device_status: DeviceStatus,
    image_buffer: VecDeque<Frame>
}

impl ImageStream {
    pub fn new(serial_msg_rx: Receiver<SerialMessage>, ip: String, device_type: i32) -> Self {
        let (request_tx, request_rx) = crossbeam::channel::unbounded();
        let response_tx = bus::Bus::<ImageResponse>::new(1000);

        ImageStream {
            request_rx,
            request_tx,
            response_tx,
            serial_msg_rx,
            device_type,
            port_state: PortState::Disconnected,
            ip,
            run: false,
            device_status: DeviceStatus { battery: 0, brightness: 0 },
            image_buffer: VecDeque::new(),
        }
    }

    pub fn get_request_tx(&self) -> Sender<ImageRequest> {
        self.request_tx.clone()
    }

    pub fn get_response_rx(&mut self) -> bus::BusReader<ImageResponse> {
        self.response_tx.add_rx()
    }

    pub fn start(&mut self) {
        let mut port: Option<WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>> = None;
        self.run = true;
        loop {
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
                Err(crossbeam::channel::TryRecvError::Disconnected) => {
                    // Handle error in receiving serial messages
                    error!("disconnected serial message in image stream");
                }
                _ => ()
            }
            if let PortState::Disconnected = self.port_state {
                if self.run {
                    if self.connect(&mut port) {
                        info!("Stream {} Connected to: {}", self.device_type, self.ip);
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
                        let frame = Frame {
                            image,
                            timestamp: Instant::now(),
                        };
                        
                        // Keep only the most recent frame
                        self.image_buffer.clear();
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
            ImageRequest::GetDeviceStatus => {
                self.get_device_status();
            }
            ImageRequest::Stop => {
                self.run = false;
            }
            _ => {}
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
                        Ok((ws, _)) => {
                            info!("Stream {} Connected to: {}", self.device_type, url_to_try);
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
        // Implement the logic to get image in base64 format
        // This is a placeholder implementation
        let image_data = vec![0; 1024]; // Replace with actual image data
        self.response_tx.broadcast(ImageResponse::Base64ImageData(image_data));
    }

    fn get_image_opencv(&mut self) {
        // Implement the logic to get image in OpenCV format
        // This is a placeholder implementation
        if let Some(frame) = self.image_buffer.front() {
            self.response_tx.broadcast(ImageResponse::OpenCVImageData(frame.image.clone()));
        } else {
            debug!("No image available in buffer");
        }
    }

    fn get_device_status(&mut self) {
        // Implement the logic to get device status
        // This is a placeholder implementation
        self.response_tx.broadcast(ImageResponse::DeviceStatus(self.device_status.clone()));
    }


}