use std::collections::VecDeque;
use crossbeam::channel::{self, Receiver, Sender};
use opencv::core::MatTraitConst;
use std::thread;
use std::time::{Duration, Instant};
use opencv::{core::Mat, imgcodecs};
use tungstenite::{connect, Message, WebSocket};
use url::Url;
use super::messages::{VideoRequest, VideoResponse, VideoEvent, DeviceType, DeviceStatus, Frame};

// Stream connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StreamStatus {
    Disconnected,
    Connecting,
    Connected,
}

// Worker that manages a video stream connection
pub struct VideoStreamWorker {
    // Channel for receiving requests
    request_rx: Receiver<VideoRequest>,
    // Channel for sending responses
    response_tx: Sender<VideoResponse>,
    // Channel for sending events
    event_tx: Sender<VideoEvent>,
    // Current stream status
    status: StreamStatus,
    // Current device type
    device_type: DeviceType,
    // Current URL
    current_url: String,
    // List of URLs to try
    connection_urls: Vec<String>,
    // WebSocket connection
    websocket: Option<WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>>,
    // Frame buffer
    image_buffer: VecDeque<Frame>,
    // Last time a frame or heartbeat was received
    last_activity_time: Instant,
    // Missed heartbeats counter
    missed_heartbeats: usize,
    // Device status info
    device_status: DeviceStatus,
    // Flag to indicate if the worker should keep running
    running: bool,
}

impl VideoStreamWorker {
    // Create a new video stream worker
    pub fn new(
        request_rx: Receiver<VideoRequest>,
        response_tx: Sender<VideoResponse>,
        event_tx: Sender<VideoEvent>,
    ) -> Self {
        VideoStreamWorker {
            request_rx,
            response_tx,
            event_tx,
            status: StreamStatus::Disconnected,
            device_type: DeviceType::Unknown,
            current_url: String::new(),
            connection_urls: Vec::new(),
            websocket: None,
            image_buffer: VecDeque::with_capacity(2),
            last_activity_time: Instant::now(),
            missed_heartbeats: 0,
            device_status: DeviceStatus {
                battery: None,
                brightness: None,
            },
            running: true,
        }
    }

    // Run the worker loop
    pub fn run(&mut self) {
        println!("Video stream worker started");

        while self.running {
            // Handle incoming requests
            while let Ok(request) = self.request_rx.try_recv() {
                self.handle_request(request);
            }

            // If connected, check for WebSocket messages
            if let StreamStatus::Connected = self.status {
                if let Some(ref mut socket) = self.websocket {
                    match socket.read_message() {
                        Ok(message) => {
                            self.handle_websocket_message(message);
                            self.last_activity_time = Instant::now();
                            self.missed_heartbeats = 0;
                        }
                        Err(e) => {
                            if !e.to_string().contains("would block") {
                                println!("WebSocket read error: {}", e);
                                self.disconnect();
                            }
                        }
                    }
                }

                // Check heartbeat timeout
                let elapsed = Instant::now().duration_since(self.last_activity_time);
                if elapsed > Duration::from_millis(2000) {  // 2 second timeout
                    self.missed_heartbeats += 1;
                    if self.missed_heartbeats > 3 {
                        println!("WebSocket heartbeat timeout, reconnecting");
                        self.reconnect();
                    }
                }
            }

            // If connecting, try the next URL
            if let StreamStatus::Connecting = self.status {
                if !self.connection_urls.is_empty() {
                    self.try_next_connection();
                } else {
                    // No more URLs to try, back to disconnected
                    self.status = StreamStatus::Disconnected;
                }
            }

            // Small sleep to avoid high CPU usage
            thread::sleep(Duration::from_millis(5));
        }

        // Clean up on exit
        self.disconnect();
        println!("Video stream worker stopped");
    }

    // Handle incoming requests
    fn handle_request(&mut self, request: VideoRequest) {
        match request {
            VideoRequest::Connect { url, device_type } => {
                self.connect(url, device_type);
            }
            VideoRequest::Disconnect => {
                self.disconnect();
                self.response_tx.send(VideoResponse::Disconnected).ok();
            }
            VideoRequest::GetFrame => {
                let frame = self.image_buffer.front().map(|f| f.image.clone());
                self.response_tx.send(VideoResponse::Frame(frame)).ok();
            }
            VideoRequest::CheckStatus => {
                self.response_tx.send(VideoResponse::Status {
                    connected: self.status == StreamStatus::Connected,
                    url: self.current_url.clone(),
                    battery: self.device_status.battery,
                    brightness: self.device_status.brightness,
                }).ok();
            }
            VideoRequest::Shutdown => {
                println!("Received shutdown request");
                self.running = false;
            }
        }
    }

    // Set up connection URLs and start connecting
    fn connect(&mut self, url: String, device_type: DeviceType) {
        // If already connected to the requested URL, do nothing
        if self.status == StreamStatus::Connected && self.current_url == url {
            self.response_tx.send(VideoResponse::Connected { url }).ok();
            return;
        }

        // Disconnect if already connected to a different URL
        if self.status == StreamStatus::Connected {
            self.disconnect();
        }

        // Set device type and URL
        self.device_type = device_type;
        self.current_url = url.clone();

        // Clear previous connection URLs
        self.connection_urls.clear();

        // Add WebSocket URL to the list
        let ws_url = if !url.starts_with("ws://") && !url.starts_with("wss://") {
            if url.starts_with("http://") {
                format!("ws://{}/ws", url.strip_prefix("http://").unwrap_or(&url))
            } else if url.starts_with("https://") {
                format!("wss://{}/ws", url.strip_prefix("https://").unwrap_or(&url))
            } else {
                // Assume host:port format
                let mut ws_url = format!("ws://{}", url);
                if !ws_url.contains(':') {
                    ws_url.push_str(":80");
                }
                if !ws_url.contains("/ws") {
                    ws_url.push_str("/ws");
                }
                ws_url
            }
        } else {
            url.clone()
        };

        self.connection_urls.push(ws_url);

        // Add device-specific URLs to try
        match device_type {
            DeviceType::Face => {
                self.add_unique_url("ws://paper1.local:80/ws");
            }
            DeviceType::LeftEye => {
                self.add_unique_url("ws://paper2.local:80/ws");
            }
            DeviceType::RightEye => {
                self.add_unique_url("ws://paper3.local:80/ws");
            }
            _ => {
                // Add nothing for unknown device types
            }
        }

        // Start connecting
        self.status = StreamStatus::Connecting;
        println!("Starting connection to: {:?}", self.connection_urls);
    }

    // Try the next URL in the list
    fn try_next_connection(&mut self) {
        if self.connection_urls.is_empty() {
            self.status = StreamStatus::Disconnected;
            return;
        }

        let url_to_try = self.connection_urls.remove(0);
        println!("Trying to connect to {}", url_to_try);

        // Try to parse and connect to the URL
        match Url::parse(&url_to_try) {
            Ok(parsed_url) => {
                match connect(parsed_url) {
                    Ok((socket, _)) => {
                        println!("Connected to WebSocket: {}", url_to_try);
                        self.websocket = Some(socket);
                        self.status = StreamStatus::Connected;
                        self.last_activity_time = Instant::now();
                        self.missed_heartbeats = 0;
                        
                        // Send connected response and event
                        self.response_tx.send(VideoResponse::Connected { 
                            url: url_to_try.clone() 
                        }).ok();
                        
                        self.event_tx.send(VideoEvent::ConnectionChanged { 
                            connected: true, 
                            url: url_to_try 
                        }).ok();
                    }
                    Err(e) => {
                        println!("Failed to connect to {}: {}", url_to_try, e);
                        // Try the next URL
                        if !self.connection_urls.is_empty() {
                            self.try_next_connection();
                        } else {
                            self.status = StreamStatus::Disconnected;
                            self.response_tx.send(VideoResponse::ConnectFailed { 
                                url: url_to_try, 
                                error: e.to_string() 
                            }).ok();
                        }
                    }
                }
            }
            Err(e) => {
                println!("Invalid URL {}: {}", url_to_try, e);
                // Try the next URL
                if !self.connection_urls.is_empty() {
                    self.try_next_connection();
                } else {
                    self.status = StreamStatus::Disconnected;
                    self.response_tx.send(VideoResponse::ConnectFailed { 
                        url: url_to_try, 
                        error: e.to_string() 
                    }).ok();
                }
            }
        }
    }

    // Disconnect from the current stream
    fn disconnect(&mut self) {
        if let Some(ref mut socket) = self.websocket {
            let _ = socket.close(None);
        }
        self.websocket = None;
        self.status = StreamStatus::Disconnected;
        
        // Send disconnected event
        self.event_tx.send(VideoEvent::ConnectionChanged { 
            connected: false, 
            url: self.current_url.clone() 
        }).ok();
        
        println!("Disconnected from WebSocket");
    }

    // Reconnect to the current URL
    fn reconnect(&mut self) {
        println!("Reconnecting to {}", self.current_url);
        let current_url = self.current_url.clone();
        let device_type = self.device_type;
        
        self.disconnect();
        self.connect(current_url, device_type);
    }

    // Handle a WebSocket message
    fn handle_websocket_message(&mut self, message: Message) {
        match message {
            Message::Binary(data) => {
                // Process image data
                if data.len() < 10 {
                    println!("Received binary data too small to be an image");
                    return;
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
                        
                        // Send new frame event
                        self.event_tx.send(VideoEvent::NewFrame).ok();
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
                // Process JSON status data
                match serde_json::from_str::<DeviceStatus>(&text) {
                    Ok(status) => {
                        self.device_status = status.clone();
                        
                        // Send status update event
                        self.event_tx.send(VideoEvent::StatusUpdated { 
                            battery: status.battery, 
                            brightness: status.brightness 
                        }).ok();
                    }
                    Err(e) => {
                        println!("Failed to parse status message: {}, message: {}", e, text);
                    }
                }
            }
            Message::Close(_) => {
                println!("Received WebSocket close frame");
                self.disconnect();
            }
            _ => {
                // Ignore other message types
            }
        }
    }

    // Add unique URL to the connection list
    fn add_unique_url(&mut self, url: &str) {
        if !self.connection_urls.contains(&url.to_string()) {
            self.connection_urls.push(url.to_string());
        }
    }
}

// Helper function to spawn a video stream worker in a new thread
pub fn spawn_video_stream_worker(
    request_rx: Receiver<VideoRequest>,
    response_tx: Sender<VideoResponse>,
    event_tx: Sender<VideoEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut worker = VideoStreamWorker::new(request_rx, response_tx, event_tx);
        worker.run();
    })
}