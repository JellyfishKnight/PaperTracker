use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Runtime};
use crate::paper_tracker_config::config::{FACE_CONFIG, EYE_CONFIG};
use crate::utils::consts::{DEVICE_TYPE_FACE, DEVICE_TYPE_LEFT_EYE, DEVICE_TYPE_RIGHT_EYE};
use crossbeam::channel::{Sender, Receiver};

pub struct ImageStreamState {
    pub face_stream_resp: Arc::<Mutex<bus::BusReader<crate::websocket::image_msg::ImageResponse>>>,
    pub left_eye_stream_resp: Arc::<Mutex<bus::BusReader<crate::websocket::image_msg::ImageResponse>>>,
    pub right_eye_stream_resp: Arc::<Mutex<bus::BusReader<crate::websocket::image_msg::ImageResponse>>>,
    pub face_stream_req: Sender<crate::websocket::image_msg::ImageRequest>,
    pub left_eye_stream_req: Sender<crate::websocket::image_msg::ImageRequest>,
    pub right_eye_stream_req: Sender<crate::websocket::image_msg::ImageRequest>,
}

pub fn init_device<R: Runtime>(app: &AppHandle<R>) {
    // init serial
    let mut serial = crate::serial::esp32_serial::Esp32Serial::new();
    let global_req_tx = serial.get_request_tx();
    let global_write_tx = serial.get_write_tx();
    let global_resp_rx = serial.get_response_rx();
    let global_msg_rx = serial.get_message_rx();
    // init face image stream
    let face_image_msg_rx = serial.get_message_rx();
    let mut face_image_stream = crate::websocket::image_stream::ImageStream::new(
        face_image_msg_rx, 
        FACE_CONFIG.functional.wifi_ip.clone(), 
        DEVICE_TYPE_FACE);
    let face_image_stream_request_tx = face_image_stream.get_request_tx();
    let face_image_stream_response_rx = face_image_stream.get_response_rx();
    std::thread::spawn(move || {
        face_image_stream.start();
    });
    // init left eye image stream
    let left_eye_image_msg_rx = serial.get_message_rx();
    let mut left_eye_image_stream = crate::websocket::image_stream::ImageStream::new(
        left_eye_image_msg_rx, 
        EYE_CONFIG.functional.left_ip.clone(),
        DEVICE_TYPE_LEFT_EYE);
    let left_eye_image_stream_request_tx = left_eye_image_stream.get_request_tx();
    let left_eye_image_stream_response_rx = left_eye_image_stream.get_response_rx();
    std::thread::spawn(move || {
        left_eye_image_stream.start();
    });
    // init right eye image stream
    let right_eye_image_msg_rx = serial.get_message_rx();
    let mut right_eye_image_stream = crate::websocket::image_stream::ImageStream::new(
        right_eye_image_msg_rx, 
        EYE_CONFIG.functional.right_ip.clone(),
        DEVICE_TYPE_RIGHT_EYE);
    let right_eye_image_stream_request_tx = right_eye_image_stream.get_request_tx();
    let right_eye_image_stream_response_rx = right_eye_image_stream.get_response_rx();
    std::thread::spawn(move || {
        right_eye_image_stream.start();
    });
    // start serial
    std::thread::spawn(move || {
        serial.start();
    });

    let image_stream_state = ImageStreamState {
        face_stream_resp: Arc::new(Mutex::new(face_image_stream_response_rx)),
        left_eye_stream_resp: Arc::new(Mutex::new(left_eye_image_stream_response_rx)),
        right_eye_stream_resp: Arc::new(Mutex::new(right_eye_image_stream_response_rx)),
        face_stream_req: face_image_stream_request_tx,
        left_eye_stream_req: left_eye_image_stream_request_tx,
        right_eye_stream_req: right_eye_image_stream_request_tx,
    };

    app.manage(image_stream_state);
    app.manage(global_req_tx);
    app.manage(global_write_tx);
    app.manage(Mutex::new(global_resp_rx));
    app.manage(Mutex::new(global_msg_rx));
}