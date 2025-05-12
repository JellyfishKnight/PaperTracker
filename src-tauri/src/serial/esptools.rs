use tauri::{Manager, AppHandle, Runtime};


#[tauri::command]
#[cfg(target_os = "macos")]
pub fn restart_esp32<R: Runtime>(app: AppHandle<R>) -> std::result::Result<(), String> {
    let esp_tool_path = app.path().resolve("assets/esptool", tauri::path::BaseDirectory::Resource);
    if esp_tool_path.is_err() {
        println!("无法解析配置文件资源路径");
        return std::result::Result::Err("无法解析ESP TOOL资源路径".to_string());
    }
    let esp_tool_path = esp_tool_path.unwrap();
    

    std::result::Result::Ok(())
}


#[tauri::command]
#[cfg(target_os = "macos")]
pub fn flash_esp32<R: Runtime>(app: AppHandle<R>) -> std::result::Result<(), String> {
    let esp_tool_path = app.path().resolve("assets/esptool", tauri::path::BaseDirectory::Resource);
    if esp_tool_path.is_err() {
        println!("无法解析配置文件资源路径");
        return std::result::Result::Err("无法解析ESP TOOL资源路径".to_string());
    }
    let esp_tool_path = esp_tool_path.unwrap();


    std::result::Result::Ok(())
}


#[tauri::command]
#[cfg(target_os = "windows")]
pub fn restart_esp32<R: Runtime>(app: &AppHandle<R>) -> Result<(), tauri::Error> {
    let esp_tool_path = app.path().resolve("assets/esptool", tauri::path::BaseDirectory::Resource);

    anyhow::Ok(())
}


#[tauri::command]
#[cfg(target_os = "windows")]
pub fn flash_esp32<R: Runtime>(app: &AppHandle<R>) -> Result<(), tauri::Error> {

    anyhow::Ok(())
}
