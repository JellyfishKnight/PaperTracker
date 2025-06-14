use tauri::{AppHandle, Manager, Runtime};
use ftlog::*;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct UpdateInfo {
    pub remote_version: String,
    pub local_version: String,
    pub release_notes: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct VersionStruct {
    #[serde(rename = "tag")]
    pub version: String,
    #[serde(rename = "firmware")]
    pub firmware: String,
    #[serde(rename = "description")]
    pub description: String
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct VersionInfo {
    #[serde(rename = "version")]
    pub value: VersionStruct
}

#[tauri::command]
pub fn check_for_updates<R: Runtime>(app: AppHandle<R>) -> Result<UpdateInfo, String>{
    let local_version = match get_local_version("assets/version.json", app) {
        Some(content) => content,
        None => {
            error!("读取本地配置文件失败");
            return Err("读取本地配置文件失败".to_string());
        }
    };
    info!("本地版本: {}", local_version.value.version);
    let remote_version = match get_remote_version("http://47.116.163.1/version.json") {
        Some(content) => content,
        None => {
            error!("读取远程配置文件失败");
            return Err("读取远程配置文件失败".to_string());
        }
    };    
    info!("远程版本: {}", remote_version.value.version);
    Ok(UpdateInfo {
        remote_version: remote_version.value.version,
        local_version: local_version.value.version,
        release_notes: remote_version.value.description
    })
}

pub fn get_local_version<R: Runtime>(version_path: &str, app: AppHandle<R>) -> Option<VersionInfo> {
    let resource_path = match app.path().resolve(version_path, tauri::path::BaseDirectory::Resource) {
        Ok(path) => {
            path
        },
        Err(_) => {
            error!("无法解析资源路径");
            return None;
        }
    };
    if let Ok(version_file) = std::fs::File::open(resource_path) {
        match serde_json::from_reader::<std::fs::File, VersionInfo>(version_file) {
            Ok(version_info) => {
                Some(version_info)
            }
            Err(e) => {
                error!("解析本地版本文件失败: {}", e);
                None
            }
        }
    } else {
        error!("没有找到版本文件");
        None
    }
}

pub fn get_remote_version(url: &str) -> Option<VersionInfo> {
    match reqwest::blocking::get(url) {
        Ok(res) => match res.json::<VersionInfo>() {
            Ok(version_info) => Some(version_info),
            Err(e) => {
                error!("解析远程版本文件失败: {}", e);
                None
            }
        },
        Err(e) => {
            error!("请求远程版本文件失败: {}", e);
            None
        }
    }
}