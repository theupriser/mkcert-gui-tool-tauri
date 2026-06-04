use std::process::Command;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppSettings {
    domains: String,
    directory: String,
    auto_schedule: bool,
}

fn get_config_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let mut path = app_handle.path().app_config_dir().unwrap_or_else(|_| {
        let mut home = app_handle.path().home_dir().unwrap_or_else(|_| PathBuf::from("."));
        home.push(".config");
        home.push("mkcert-gui-tool");
        home
    });
    fs::create_dir_all(&path).unwrap_or_default();
    path.push("config.json");
    path
}

#[tauri::command]
fn load_settings(app_handle: tauri::AppHandle) -> Option<AppSettings> {
    let config_path = get_config_path(&app_handle);
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(config_path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                return Some(settings);
            }
        }
    }
    None
}

fn toggle_scheduler(enable: bool, _app_handle: &tauri::AppHandle) {
    let current_exe = std::env::current_exe().unwrap_or_default();
    let app_str = current_exe.to_string_lossy().to_string();
    let is_dev = app_str.contains("target");
    
    let worker_command = if is_dev {
        format!("{} -- --worker", app_str)
    } else {
        format!("{} --worker", app_str)
    };

    if cfg!(target_os = "windows") {
        let task_name = "MkcertAutoRenewalTaskTauri";
        let _ = Command::new("schtasks").args(&["/delete", "/tn", task_name, "/f"]).output();
        if enable {
            let cmd_str = format!("schtasks /create /tn \"{}\" /tr \"\\\"{}\\\"\" /sc daily /st 00:00 /f", task_name, worker_command);
            let _ = Command::new("cmd").args(&["/C", &cmd_str]).output();
        }
    } else {
        let current_cron = Command::new("crontab").arg("-l").output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        let mut lines: Vec<String> = current_cron.split('\n')
            .map(|s| s.to_string())
            .filter(|line| !line.contains("mkcert-gui-tool") && !line.trim().is_empty())
            .collect();

        if enable {
            lines.push(format!("0 0 * * * {} # mkcert-gui-tool", worker_command));
        }

        let new_cron = lines.join("\n") + "\n";
        let tmp_path = std::env::temp_dir().join("cron_tmp");
        if let Ok(mut file) = File::create(&tmp_path) {
            let _ = file.write_all(new_cron.as_bytes());
            let _ = Command::new("crontab").arg(tmp_path.to_str().unwrap()).output();
        }
    }
}

#[tauri::command]
fn save_settings(settings: AppSettings, app_handle: tauri::AppHandle) -> Result<(), String> {
    let config_path = get_config_path(&app_handle);
    let content = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(config_path, content).map_err(|e| e.to_string())?;
    
    toggle_scheduler(settings.auto_schedule, &app_handle);
    Ok(())
}

#[tauri::command]
fn check_root_ca(_app_handle: tauri::AppHandle) -> serde_json::Value {
    let check_cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
    let has_binary = Command::new(check_cmd).arg("mkcert").output().is_ok_and(|o| o.status.success());

    if !has_binary {
        return serde_json::json!({
            "installed": false,
            "missing_binary": true,
            "error": "❌ mkcert binary was not found! Please install mkcert (apt, brew, choco) and restart."
        });
    }

    let ca_root_output = Command::new("mkcert").arg("-CAROOT").output();
    if let Ok(output) = ca_root_output {
        let ca_path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let mut root_ca = PathBuf::from(ca_path_str);
        root_ca.push("rootCA.pem");

        if root_ca.exists() {
            return serde_json::json!({ "installed": true, "missing_binary": false });
        }
    }

    serde_json::json!({ "installed": false, "missing_binary": false })
}

#[tauri::command]
fn install_root_ca() -> serde_json::Value {
    match Command::new("mkcert").arg("-install").output() {
        Ok(o) => serde_json::json!({ "success": true, "message": String::from_utf8_lossy(&o.stdout).to_string() }),
        Err(e) => serde_json::json!({ "success": false, "message": e.to_string() }),
    }
}

#[tauri::command]
async fn select_directory(app_handle: tauri::AppHandle) -> Option<String> {
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
fn generate_certs(domains_string: String, target_dir: String, app_handle: tauri::AppHandle) -> serde_json::Value {
    let final_dir = if target_dir.is_empty() {
        let mut home = app_handle.path().home_dir().unwrap_or_else(|_| PathBuf::from("."));
        home.push(".local"); home.push("share"); home.push("mkcert-certs");
        home
    } else {
        PathBuf::from(target_dir)
    };

    let _ = fs::create_dir_all(&final_dir);
    
    let domains: Vec<&str> = domains_string.split(',').map(|d| d.trim()).filter(|d| !d.is_empty()).collect();
    if domains.is_empty() {
        return serde_json::json!({ "success": false, "message": "No valid domains entered." });
    }

    let cert_file = final_dir.join("local.pem");
    let key_file = final_dir.join("local-key.pem");

    let mut cmd = Command::new("mkcert");
    cmd.arg("-cert-file").arg(&cert_file)
       .arg("-key-file").arg(&key_file);
    
    for domain in domains {
        cmd.arg(domain);
    }

    match cmd.output() {
        Ok(o) => if o.status.success() {
            serde_json::json!({ "success": true, "message": format!("Certificates generated in: {}", final_dir.display()) })
        } else {
            serde_json::json!({ "success": false, "message": String::from_utf8_lossy(&o.stderr).to_string() })
        },
        Err(e) => serde_json::json!({ "success": false, "message": e.to_string() }),
    }
}

pub fn run_background_worker(app_handle: &tauri::AppHandle) {
    if let Some(settings) = load_settings(app_handle.clone()) {
        let _ = generate_certs(settings.domains, settings.directory, app_handle.clone());
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let args: Vec<String> = std::env::args().collect();
            
            if args.contains(&"--worker".to_string()) {
                run_background_worker(&app.handle());
                std::process::exit(0);
            }

            let _window = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::App("index.html".into())
            )
            .title("MKCert Auto-Generator")
            .inner_size(600.0, 720.0)
            .decorations(true)
            .build();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            check_root_ca,
            install_root_ca,
            select_directory,
            generate_certs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
