#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // === SNAP PURGE LOGICA ===
    let vars: Vec<(String, String)> = std::env::vars().collect();
    for (key, value) in vars {
        if key.contains("SNAP") || value.contains("/snap/") || key == "GTK_PATH" || key == "GTK_MODULES" {
            std::env::remove_var(&key);
        }
    }

    // Forceer WebKit om de schone, native systeembibliotheken van Ubuntu te gebruiken
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

    // === FIX: MODERN FILE BROWSER (ELECTRON LOOK) ===
    // Dit dwingt Linux om de moderne Gnome/Ubuntu systeem-dialogen te gebruiken
    std::env::set_var("GTK_USE_PORTAL", "1");

    // Start de Tauri bibliotheek op
    app_lib::run();
}
