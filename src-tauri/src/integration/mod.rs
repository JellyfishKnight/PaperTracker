pub mod serial_commands;
pub mod video_commands;

// Function to initialize all the services
pub fn init_services() -> (serial_commands::SerialState, video_commands::VideoState) {
    let serial_state = serial_commands::SerialState::new();
    let video_state = video_commands::VideoState::new();
    
    // Auto-connect to ESP32 device if available
    {
        let client = serial_state.client.lock().unwrap();
        if let Ok(port) = client.auto_connect() {
            println!("Auto-connected to ESP32 device at {}", port);
        } else {
            println!("No ESP32 device found for auto-connection");
        }
    }
    
    (serial_state, video_state)
}