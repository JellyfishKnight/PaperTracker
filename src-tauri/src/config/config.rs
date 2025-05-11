use serde::{Deserialize, Serialize};
use crate::utils::roi::Roi;

/*************************************************************/
/***************************眼追参数****************************/
/*************************************************************/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EyeFunctionalConfig {
    pub left_ip: String,
    pub right_ip: String,
    pub left_brightness: i32,
    pub right_brightness: i32,
    pub energy_mode: i32,
    pub left_rect: Roi,
    pub right_rect: Roi,

    // 轴翻转设置
    pub left_flip_x: bool,
    pub right_flip_x: bool,
    pub flip_y: bool,
    pub left_rotate_angle: i32,
    pub right_rotate_angle: i32,    

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EyeParamsConfig {
    // 左眼校准数据
    pub left_calib_xmin: f64,
    pub left_calib_xmax: f64,
    pub left_calib_ymin: f64,
    pub left_calib_ymax: f64,
    pub left_calib_xoff: f64,
    pub left_calib_yoff: f64,
    pub left_has_calibration: bool,
    // 右眼校准数据
    pub right_calib_xmin: f64,
    pub right_calib_xmax: f64,
    pub right_calib_ymin: f64,
    pub right_calib_ymax: f64,
    pub right_calib_xoff: f64,
    pub right_calib_yoff: f64,
    pub right_has_calibration: bool,

    pub left_eye_fully_open: f64,
    pub left_eye_fully_closed: f64,
    pub right_eye_fully_open: f64,
    pub right_eye_fully_closed: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EyeConfig {
    pub functional: EyeFunctionalConfig,
    pub params: EyeParamsConfig,
}

/*************************************************************/
/***************************面捕参数****************************/
/*************************************************************/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceFunctionalConfig {
    pub brightness: i32,
    pub rotate_angle: i32,
    pub energy_mode: i32,
    pub rect: Roi,
    pub use_filter: bool,
    pub wifi_ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceParamsConfig {
    pub cheek_puff_left_offset: f32,
    pub cheek_puff_right_offset: f32,
    pub jaw_open_offset: f32,
    pub tongue_out_offset: f32,
    pub mouth_close_offset: f32,
    pub mouth_funnel_offset: f32,
    pub mouth_pucker_offset: f32,
    pub mouth_roll_upper_offset: f32,
    pub mouth_roll_lower_offset: f32,
    pub mouth_shrug_upper_offset: f32,
    pub mouth_shrug_lower_offset: f32,
    pub cheek_puff_left_gain: f32,
    pub cheek_puff_right_gain: f32,
    pub jaw_open_gain: f32,
    pub tongue_out_gain: f32,
    pub mouth_close_gain: f32,
    pub mouth_funnel_gain: f32,
    pub mouth_pucker_gain: f32,
    pub mouth_roll_upper_gain: f32,
    pub mouth_roll_lower_gain: f32,
    pub mouth_shrug_upper_gain: f32,
    pub mouth_shrug_lower_gain: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceFilterConfig {
    kalman_dt: f32,
    kalman_q_factor: f32,
    r_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceConfig {
    pub functional: FaceFunctionalConfig,
    pub params: FaceParamsConfig,
    pub filter: FaceFilterConfig,
}