use crate::settings::{AppSettings, self};
use crate::services::{mkcert, scheduler};

#[tauri::command]
pub fn load_settings(app_handle: tauri::AppHandle) -> Option<AppSettings> {
    settings::load(&app_handle)
}

#[tauri::command]
pub fn save_settings(settings: AppSettings, app_handle: tauri::AppHandle) -> Result<(), String> {
    settings::save(&settings, &app_handle)?;
    scheduler::toggle(settings.auto_schedule);
    Ok(())
}

#[tauri::command]
pub fn check_root_ca() -> serde_json::Value {
    mkcert::check_root_ca()
}

#[tauri::command]
pub fn install_root_ca() -> serde_json::Value {
    mkcert::install_root_ca()
}

#[tauri::command]
pub async fn select_directory(app_handle: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    
    let (tx, rx) = tokio::sync::oneshot::channel();
    app_handle.dialog().file().pick_folder(move |folder| {
        let _ = tx.send(folder.and_then(|f| f.into_path().ok()));
    });

    if let Ok(Some(path)) = rx.await {
        return Some(path.to_string_lossy().to_string());
    }
    None
}

#[tauri::command]
pub fn generate_certs(domains_string: String, target_dir: String, app_handle: tauri::AppHandle) -> serde_json::Value {
    mkcert::generate_certs(domains_string, target_dir, &app_handle)
}
