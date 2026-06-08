use serde::{Serialize, Deserialize};
use std::fs;
use crate::utils::get_config_path;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub domains: String,
    pub directory: String,
    pub auto_schedule: bool,
}

pub fn load(app_handle: &tauri::AppHandle) -> Option<AppSettings> {
    let config_path = get_config_path(app_handle);
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(config_path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                return Some(settings);
            }
        }
    }
    None
}

pub fn save(settings: &AppSettings, app_handle: &tauri::AppHandle) -> Result<(), String> {
    let config_path = get_config_path(app_handle);
    let content = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(config_path, content).map_err(|e| e.to_string())?;
    Ok(())
}
