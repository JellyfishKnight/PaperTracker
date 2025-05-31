use ftlog::*;

pub fn normalize_windows_path(path_str: &str) -> String {
    // 移除Windows长路径前缀
    let cleaned = if path_str.starts_with("\\\\?\\") {
        &path_str[4..]
    } else {
        path_str
    };
    
    // 标准化路径分隔符（可选，通常不需要）
    #[cfg(windows)]
    let normalized = cleaned.replace('/', "\\");
    #[cfg(not(windows))]
    let normalized = cleaned.to_string();
    
    debug!("路径标准化: {} -> {}", path_str, normalized);
    normalized
}