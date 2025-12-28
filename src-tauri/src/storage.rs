use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

pub fn get_app_data_dir(app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)?;
    }
    Ok(app_data_dir)
}

pub fn save_json<T: Serialize>(
    app: &AppHandle,
    filename: &str,
    data: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_data_dir = get_app_data_dir(app)?;
    let file_path = app_data_dir.join(filename);
    let json_string = serde_json::to_string_pretty(data)?;
    fs::write(file_path, json_string)?;
    Ok(())
}

pub fn load_json<T: for<'de> Deserialize<'de>>(
    app: &AppHandle,
    filename: &str,
) -> Result<T, Box<dyn std::error::Error>> {
    let app_data_dir = get_app_data_dir(app)?;
    let file_path = app_data_dir.join(filename);
    
    if !file_path.exists() {
        return Err("File not found".into());
    }
    
    let json_string = fs::read_to_string(file_path)?;
    let data: T = serde_json::from_str(&json_string)?;
    Ok(data)
}