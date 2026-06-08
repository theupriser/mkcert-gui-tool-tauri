use std::path::PathBuf;
use tauri::Manager;
use std::fs;

pub fn get_config_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let mut path = app_handle.path().app_config_dir().unwrap_or_else(|_| {
        let mut home = app_handle.path().home_dir().unwrap_or_else(|_| PathBuf::from("."));
        home.push(".config");
        home.push("mkcert-gui-tool");
        home
    });
    let _ = fs::create_dir_all(&path);
    path.push("config.json");
    path
}
