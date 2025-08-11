use std::path::PathBuf;
use crate::AppError;
use crate::config::app;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 設定保存・読み込みコマンド
#[tauri::command]
pub async fn save_output_folder(output_folder: String) -> Result<(), String> {
    let app_data_dir = dirs::data_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join(app::DATA_DIR_NAME);

    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| AppError::custom(format!("Failed to create app data directory: {}", e)).to_string())?;
    }

    let config_path = app_data_dir.join("config.json");
    let config = serde_json::json!({
        "output_folder": output_folder
    });

    std::fs::write(&config_path, config.to_string())
        .map_err(|e| AppError::custom(format!("Failed to save config: {}", e)).to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn load_output_folder() -> Result<Option<String>, String> {
    let app_data_dir = dirs::data_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join(app::DATA_DIR_NAME);

    let config_path = app_data_dir.join("config.json");
    
    if !config_path.exists() {
        return Ok(None);
    }

    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| AppError::custom(format!("Failed to read config: {}", e)).to_string())?;

    let config: serde_json::Value = serde_json::from_str(&config_content)
        .map_err(|e| AppError::custom(format!("Failed to parse config: {}", e)).to_string())?;

    Ok(config.get("output_folder")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string()))
}

// フォルダを開くコマンド
#[tauri::command]
pub async fn open_folder(folder_path: String) -> Result<(), String> {
    use std::process::Command;
    
    let path = std::path::Path::new(&folder_path);
    let folder_to_open = if path.is_file() {
        // ファイルの場合は親ディレクトリを開く
        path.parent().unwrap_or(path)
    } else {
        // ディレクトリの場合はそのまま
        path
    };

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(folder_to_open)
            .spawn()
            .map_err(|e| AppError::custom(format!("Failed to open folder: {}", e)).to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(folder_to_open)
            .spawn()
            .map_err(|e| AppError::custom(format!("Failed to open folder: {}", e)).to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(folder_to_open)
            .spawn()
            .map_err(|e| AppError::custom(format!("Failed to open folder: {}", e)).to_string())?;
    }

    Ok(())
}