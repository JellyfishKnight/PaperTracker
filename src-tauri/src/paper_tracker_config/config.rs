use std::sync::Once;

use serde::{Deserialize, Serialize};
use crate::utils::roi::Roi;
use config;
use once_cell::sync::{Lazy, OnceCell};
use anyhow::{Ok, Result};

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
    pub modified: bool,
}

impl EyeConfig {
    pub fn new(config_path: &str) -> Result<Self> {
        let mut settigns = config::Config::default();
        settigns.merge(config::File::with_name(config_path))?;
        let conf: EyeConfig = settigns.try_into()?;
        Ok(conf)
    }

    pub fn new_args() -> Result<Self> {
        let config = EyeConfig::new(EYE_CONFIG_PATH.get().unwrap().as_str())?;
        Ok(config)
    }

    pub fn write(config_path: &str) -> Result<()> {


        Ok(())
    }
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
    pub cheek_puff_left_offset: f64,
    pub cheek_puff_right_offset: f64,
    pub jaw_open_offset: f64,
    pub tongue_out_offset: f64,
    pub mouth_close_offset: f64,
    pub mouth_funnel_offset: f64,
    pub mouth_pucker_offset: f64,
    pub mouth_roll_upper_offset: f64,
    pub mouth_roll_lower_offset: f64,
    pub mouth_shrug_upper_offset: f64,
    pub mouth_shrug_lower_offset: f64,
    pub cheek_puff_left_gain: f64,
    pub cheek_puff_right_gain: f64,
    pub jaw_open_gain: f64,
    pub tongue_out_gain: f64,
    pub mouth_close_gain: f64,
    pub mouth_funnel_gain: f64,
    pub mouth_pucker_gain: f64,
    pub mouth_roll_upper_gain: f64,
    pub mouth_roll_lower_gain: f64,
    pub mouth_shrug_upper_gain: f64,
    pub mouth_shrug_lower_gain: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceFilterConfig {
    pub kalman_dt: f64,
    pub kalman_q_factor: f64,
    pub r_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceConfig {
    pub functional: FaceFunctionalConfig,
    pub params: FaceParamsConfig,
    pub filter: FaceFilterConfig,
    pub modified: bool,
}

impl FaceConfig {
    pub fn new(config_path: &str) -> Result<Self> {
        let mut settigns = config::Config::default();
        settigns.merge(config::File::with_name(config_path))?;
        let conf: FaceConfig = settigns.try_into()?;
        Ok(conf)
    }

    pub fn new_args() -> Result<Self> {
        let config = FaceConfig::new(FACE_CONFIG_PATH.get().unwrap().as_str())?;
        Ok(config)
    }

    pub fn write(config_path: &str) -> Result<()> {


        Ok(())
    }
}


pub static EYE_CONFIG_PATH: OnceCell<String> = OnceCell::new();

pub static FACE_CONFIG_PATH: OnceCell<String> = OnceCell::new();

// 配置文件加载失败可直接panic

pub static EYE_CONFIG: Lazy<EyeConfig> = Lazy::new(|| EyeConfig::new_args().unwrap());

pub static FACE_CONIG: Lazy<FaceConfig> = Lazy::new(|| FaceConfig::new_args().unwrap());