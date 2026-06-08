use std::process::Command;
use std::fs::File;
use std::io::Write;

pub fn toggle(enable: bool) {
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
