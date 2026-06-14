// Monadnock Cyber GPG — Phase 0 menu-bar app.
// Lives in the macOS menu bar / Windows system tray; a window provides the
// generate / encrypt / decrypt UI. All crypto is delegated to `mc-core`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

#[derive(Serialize)]
struct KeyOut {
    public: String,
    secret: String,
    fingerprint: String,
}

#[tauri::command]
fn generate_key(userid: String) -> Result<KeyOut, String> {
    mc_core::generate_key(&userid)
        .map(|k| KeyOut {
            public: k.public,
            secret: k.secret,
            fingerprint: k.fingerprint,
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn encrypt(plaintext: String, recipient_public: String) -> Result<String, String> {
    mc_core::encrypt(&plaintext, &recipient_public).map_err(|e| e.to_string())
}

#[tauri::command]
fn decrypt(ciphertext: String, secret: String) -> Result<String, String> {
    mc_core::decrypt(&ciphertext, &secret).map_err(|e| e.to_string())
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![generate_key, encrypt, decrypt])
        .setup(|app| {
            // True menu-bar app on macOS: no Dock icon.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let open = MenuItem::with_id(app, "open", "Open Monadnock Cyber GPG", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Monadnock Cyber GPG")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Monadnock Cyber GPG");
}
