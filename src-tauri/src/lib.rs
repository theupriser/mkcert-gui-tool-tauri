pub mod settings;
pub mod services;
pub mod commands;
pub mod utils;

use services::mkcert;

pub fn run_background_worker(app_handle: &tauri::AppHandle) {
    if let Some(settings) = settings::load(app_handle) {
        let _ = mkcert::generate_certs(settings.domains, settings.directory, app_handle);
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
            commands::load_settings,
            commands::save_settings,
            commands::check_root_ca,
            commands::install_root_ca,
            commands::select_directory,
            commands::generate_certs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
