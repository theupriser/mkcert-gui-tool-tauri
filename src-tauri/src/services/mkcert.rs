use std::process::Command;
use std::path::PathBuf;
use std::fs;
use tauri::Manager;

fn root_ca_dir() -> Result<PathBuf, String> {
    let ca_root_output = Command::new("mkcert")
        .arg("-CAROOT")
        .output()
        .map_err(|e| e.to_string())?;

    if !ca_root_output.status.success() {
        return Err(String::from_utf8_lossy(&ca_root_output.stderr).trim().to_string());
    }

    let ca_path_str = String::from_utf8_lossy(&ca_root_output.stdout).trim().to_string();
    if ca_path_str.is_empty() {
        return Err("mkcert did not return a root certificate directory.".to_string());
    }

    Ok(PathBuf::from(ca_path_str))
}

pub fn check_root_ca() -> serde_json::Value {
    let check_cmd = if cfg!(target_os = "windows") { "where" } else { "which" };
    let has_binary = Command::new(check_cmd).arg("mkcert").output().is_ok_and(|o| o.status.success());

    if !has_binary {
        return serde_json::json!({
            "installed": false,
            "missing_binary": true,
            "error": "❌ mkcert binary was not found! Please install mkcert (apt, brew, choco) and restart."
        });
    }

    if let Ok(ca_root_dir) = root_ca_dir() {
        let mut root_ca = ca_root_dir;
        root_ca.push("rootCA.pem");

        if root_ca.exists() {
            return serde_json::json!({ "installed": true, "missing_binary": false });
        }
    }

    serde_json::json!({ "installed": false, "missing_binary": false })
}

pub fn open_root_ca_folder() -> serde_json::Value {
    let ca_root_dir = match root_ca_dir() {
        Ok(path) => path,
        Err(e) => {
            return serde_json::json!({ "success": false, "message": e });
        }
    };

    let open_result = if cfg!(target_os = "windows") {
        Command::new("explorer").arg(&ca_root_dir).output()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(&ca_root_dir).output()
    } else {
        Command::new("xdg-open").arg(&ca_root_dir).output()
    };

    match open_result {
        Ok(output) if output.status.success() => {
            serde_json::json!({
                "success": true,
                "message": format!("Opened root certificate folder: {}", ca_root_dir.display())
            })
        }
        Ok(output) => serde_json::json!({
            "success": false,
            "message": String::from_utf8_lossy(&output.stderr).to_string()
        }),
        Err(e) => serde_json::json!({ "success": false, "message": e.to_string() }),
    }
}

pub fn install_root_ca() -> serde_json::Value {
    match Command::new("mkcert").arg("-install").output() {
        Ok(o) => serde_json::json!({ "success": true, "message": String::from_utf8_lossy(&o.stdout).to_string() }),
        Err(e) => serde_json::json!({ "success": false, "message": e.to_string() }),
    }
}

pub fn generate_certs(domains_string: String, target_dir: String, app_handle: &tauri::AppHandle) -> serde_json::Value {
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
