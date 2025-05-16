use tauri::Manager;
// image_ui_updater.rs
use tauri::{AppHandle, Runtime, State, Window, ipc::Channel};
use crossbeam::channel::Receiver;
use crate::websocket::{self, DeviceType};
use crate::integration::VideoState;
use opencv::core::Mat;
use opencv::imgcodecs;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use base64::Engine;
use base64;

// Store active image streaming threads
pub struct ImageStreamState {
    pub face_stream_active: Arc<Mutex<bool>>,
    pub left_eye_stream_active: Arc<Mutex<bool>>,
    pub right_eye_stream_active: Arc<Mutex<bool>>,
}

impl Default for ImageStreamState {
    fn default() -> Self {
        Self {
            face_stream_active: Arc::new(Mutex::new(false)),
            left_eye_stream_active: Arc::new(Mutex::new(false)),
            right_eye_stream_active: Arc::new(Mutex::new(false)),
        }
    }
}

// Start streaming face camera images
#[tauri::command]
pub fn start_face_stream<R: Runtime>(
    app: AppHandle<R>,
    channel: Channel<Vec<u8>>,
) -> Result<(), String> {
    println!("--------------Starting face stream-------------");
    let image_state = app.state::<ImageStreamState>();
    let video_state = app.state::<VideoState>();

    // Check if already streaming
    {
        let mut active = image_state.face_stream_active.lock()
            .map_err(|e| format!("Failed to lock face stream state: {}", e))?;
        if *active {
            return Ok(());  // Already streaming
        }
        *active = true;  // Mark as active
    }
    
    // Clone what we need for the thread
    let video_manager = video_state.manager.clone();
    let active_flag = image_state.face_stream_active.clone();
    
    // Start a thread to listen for video frames and send them to the channel
    thread::spawn(move || {
        println!("Starting face image stream");
        
        // Loop while the stream is active
        while *active_flag.lock().unwrap() {
            // Get the face camera frame
            if let Ok(frame_option) = video_manager.lock().unwrap().get_frame(DeviceType::Face) {
                if let Some(frame) = frame_option {
                    // Convert Mat to JPEG bytes
                    let mut buf = opencv::core::Vector::new();
                    let params = opencv::core::Vector::new();
                    if let Ok(true) = imgcodecs::imencode(".jpg", &frame, &mut buf, &params) {
                        // Convert to base64
                        let base64_img = base64::engine::general_purpose::STANDARD.encode(&buf);
                        
                        // Send to frontend
                        let _ = channel.send(format!("data:image/jpeg;base64,{}", base64_img).into_bytes());
                    }
                }
            }
            
            // Sleep to avoid high CPU usage
            thread::sleep(Duration::from_millis(100));
        }
        
        println!("Face image stream stopped");
    });
    
    Ok(())
}

// Start streaming left eye camera images
#[tauri::command]
pub fn start_left_eye_stream<R: Runtime>(
    app: AppHandle<R>,
    channel: Channel<Vec<u8>>,
) -> Result<(), String> {
    let image_state = app.state::<ImageStreamState>();
    let video_state = app.state::<VideoState>();
    // Check if already streaming
    {
        let mut active = image_state.left_eye_stream_active.lock()
            .map_err(|e| format!("Failed to lock left eye stream state: {}", e))?;
        if *active {
            return Ok(());  // Already streaming
        }
        *active = true;  // Mark as active
    }
        
    // Clone what we need for the thread
    let video_manager = video_state.manager.clone();
    let active_flag = image_state.left_eye_stream_active.clone();
    
    // Start a thread to listen for video frames and send them to the channel
    thread::spawn(move || {
        println!("Starting left eye image stream");
        
        // Loop while the stream is active
        while *active_flag.lock().unwrap() {
            // Get the left eye camera frame
            if let Ok(frame_option) = video_manager.lock().unwrap().get_frame(DeviceType::LeftEye) {
                if let Some(frame) = frame_option {
                    // Convert Mat to JPEG bytes
                    let mut buf = opencv::core::Vector::new();
                    let params = opencv::core::Vector::new();
                    if let Ok(true) = imgcodecs::imencode(".jpg", &frame, &mut buf, &params) {
                        // Convert to base64
                        let base64_img = base64::engine::general_purpose::STANDARD.encode(&buf);
                        
                        // Send to frontend
                        let _ = channel.send(format!("data:image/jpeg;base64,{}", base64_img).into_bytes());
                    }
                }
            }
        }
        
        println!("Left eye image stream stopped");
    });
    
    Ok(())
}


// Start streaming right eye camera images
#[tauri::command]
pub fn start_right_eye_stream<R: Runtime>(
    app: AppHandle<R>,
    channel: Channel<Vec<u8>>,
) -> Result<(), String> {
    let image_state = app.state::<ImageStreamState>();
    let video_state = app.state::<VideoState>();
    // Check if already streaming
    {
        let mut active = image_state.right_eye_stream_active.lock()
            .map_err(|e| format!("Failed to lock right eye stream state: {}", e))?;
        if *active {
            return Ok(());  // Already streaming
        }
        *active = true;  // Mark as active
    }
        
    // Clone what we need for the thread
    let video_manager = video_state.manager.clone();
    let active_flag = image_state.right_eye_stream_active.clone();
    
    // Start a thread to listen for video frames and send them to the channel
    thread::spawn(move || {
        println!("Starting right eye image stream");
        
        // Loop while the stream is active
        while *active_flag.lock().unwrap() {
            // Get the right eye camera frame
            if let Ok(frame_option) = video_manager.lock().unwrap().get_frame(DeviceType::RightEye) {
                if let Some(frame) = frame_option {
                    // Convert Mat to JPEG bytes
                    let mut buf = opencv::core::Vector::new();
                    let params = opencv::core::Vector::new();
                    if let Ok(true) = imgcodecs::imencode(".jpg", &frame, &mut buf, &params) {
                        // Convert to base64
                        let base64_img = base64::engine::general_purpose::STANDARD.encode(&buf);
                        
                        // Send to frontend
                        let _ = channel.send(format!("data:image/jpeg;base64,{}", base64_img).into_bytes());
                    }
                }
            }
            
            // Sleep to avoid high CPU usage
            thread::sleep(Duration::from_millis(100));
        }
        
        println!("Right eye image stream stopped");
    });
    
    Ok(())
}
