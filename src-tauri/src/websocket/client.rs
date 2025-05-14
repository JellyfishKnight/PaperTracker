use crossbeam::channel::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use opencv::core::Mat;
use super::messages::{VideoRequest, VideoResponse, VideoEvent, DeviceType};
use super::worker::spawn_video_stream_worker;

// Client API for video streaming
pub struct VideoStreamClient {
    // Channel for sending requests to the worker
    pub request_tx: Sender<VideoRequest>,
    // Channel for receiving responses from the worker
    pub response_rx: Receiver<VideoResponse>,
    // Channel for receiving events from the worker
    pub event_rx: Receiver<VideoEvent>,
    // Handle to the worker thread
    pub worker_handle: Option<JoinHandle<()>>,
}

impl VideoStreamClient {
    // Create a new video stream client and start the worker thread
    pub fn new() -> Self {
        // Create channels
        let (request_tx, request_rx) = crossbeam::channel::unbounded();
        let (response_tx, response_rx) = crossbeam::channel::unbounded();
        let (event_tx, event_rx) = crossbeam::channel::unbounded();
        
        // Spawn worker thread
        let worker_handle = spawn_video_stream_worker(request_rx, response_tx, event_tx);
        
        VideoStreamClient {
            request_tx,
            response_rx,
            event_rx,
            worker_handle: Some(worker_handle),
        }
    }
    
    // Connect to a video stream
    pub fn connect(&self, url: String, device_type: DeviceType) -> Result<(), String> {
        self.request_tx.send(VideoRequest::Connect { url: url.clone(), device_type })
            .map_err(|e| format!("Failed to send connect request: {}", e))?;
        
        // Wait for response
        match self.response_rx.recv() {
            Ok(VideoResponse::Connected { url: _ }) => {
                Ok(())
            }
            Ok(VideoResponse::ConnectFailed { url, error }) => {
                Err(format!("Failed to connect to {}: {}", url, error))
            }
            _ => Err("Unexpected response from worker".to_string()),
        }
    }
    
    // Disconnect from the current stream
    pub fn disconnect(&self) -> Result<(), String> {
        self.request_tx.send(VideoRequest::Disconnect)
            .map_err(|e| format!("Failed to send disconnect request: {}", e))?;
        
        // Wait for response
        match self.response_rx.recv() {
            Ok(VideoResponse::Disconnected) => Ok(()),
            _ => Err("Unexpected response from worker".to_string()),
        }
    }
    
    // Get the latest frame
    pub fn get_frame(&self) -> Result<Option<Mat>, String> {
        self.request_tx.send(VideoRequest::GetFrame)
            .map_err(|e| format!("Failed to send get frame request: {}", e))?;
        
        // Wait for response
        match self.response_rx.recv() {
            Ok(VideoResponse::Frame(frame)) => Ok(frame),
            _ => Err("Unexpected response from worker".to_string()),
        }
    }
    
    // Check connection status
    pub fn check_status(&self) -> Result<(bool, String, Option<f32>, Option<i32>), String> {
        self.request_tx.send(VideoRequest::CheckStatus)
            .map_err(|e| format!("Failed to send check status request: {}", e))?;
        
        // Wait for response
        match self.response_rx.recv() {
            Ok(VideoResponse::Status { connected, url, battery, brightness }) => {
                Ok((connected, url, battery, brightness))
            }
            _ => Err("Unexpected response from worker".to_string()),
        }
    }
    
    // Try to receive an event, non-blocking
    pub fn try_recv_event(&self) -> Option<VideoEvent> {
        match self.event_rx.try_recv() {
            Ok(event) => Some(event),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                println!("Event channel disconnected");
                None
            }
        }
    }
    
    // Get the event receiver for external use
    pub fn get_event_receiver(&self) -> Receiver<VideoEvent> {
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
}

impl Drop for VideoStreamClient {
    fn drop(&mut self) {
        // Send shutdown request to worker
        let _ = self.request_tx.send(VideoRequest::Shutdown);
        
        // Wait for worker thread to finish
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}