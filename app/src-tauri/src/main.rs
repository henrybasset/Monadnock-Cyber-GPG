// Monadnock Cyber GPG — desktop + menu-bar app.
// Full-window desktop UI plus a macOS menu-bar / Windows system-tray presence.
// All crypto is delegated to `mc-core`, which persists keys under the app data dir.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_dialog::DialogExt;

fn keyring(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|d| d.join("keyring"))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_keys(app: tauri::AppHandle) -> Result<Vec<mc_core::CertInfo>, String> {
    mc_core::list_keys(&keyring(&app)?).map_err(|e| e.to_string())
}

#[tauri::command]
fn generate_key(app: tauri::AppHandle, userid: String) -> Result<mc_core::CertInfo, String> {
    mc_core::generate_key(&keyring(&app)?, &userid).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_key(app: tauri::AppHandle, armored: String) -> Result<mc_core::CertInfo, String> {
    mc_core::import_cert(&keyring(&app)?, &armored).map_err(|e| e.to_string())
}

#[tauri::command]
fn export_public(app: tauri::AppHandle, fingerprint: String) -> Result<String, String> {
    mc_core::export_public(&keyring(&app)?, &fingerprint).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_key(app: tauri::AppHandle, fingerprint: String) -> Result<(), String> {
    mc_core::delete_key(&keyring(&app)?, &fingerprint).map_err(|e| e.to_string())
}

#[tauri::command]
fn encrypt(app: tauri::AppHandle, plaintext: String, recipient: String) -> Result<String, String> {
    mc_core::encrypt(&keyring(&app)?, &plaintext, &recipient).map_err(|e| e.to_string())
}

#[tauri::command]
fn decrypt(app: tauri::AppHandle, ciphertext: String) -> Result<String, String> {
    mc_core::decrypt(&keyring(&app)?, &ciphertext).map_err(|e| e.to_string())
}

#[tauri::command]
fn sign(app: tauri::AppHandle, text: String, signer: String) -> Result<String, String> {
    mc_core::sign(&keyring(&app)?, &text, &signer).map_err(|e| e.to_string())
}

#[tauri::command]
fn verify(app: tauri::AppHandle, signed: String) -> Result<mc_core::VerifyOutcome, String> {
    mc_core::verify(&keyring(&app)?, &signed).map_err(|e| e.to_string())
}

#[tauri::command]
fn encrypt_file(app: tauri::AppHandle, recipient: String) -> Result<Option<String>, String> {
    let Some(picked) = app.dialog().file().blocking_pick_file() else {
        return Ok(None);
    };
    let input = picked.into_path().map_err(|e| e.to_string())?;
    let mut out = input.clone().into_os_string();
    out.push(".pgp");
    let output = PathBuf::from(out);
    mc_core::encrypt_file(&keyring(&app)?, &input, &output, &recipient).map_err(|e| e.to_string())?;
    Ok(Some(output.to_string_lossy().to_string()))
}

#[tauri::command]
fn decrypt_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let Some(picked) = app.dialog().file().blocking_pick_file() else {
        return Ok(None);
    };
    let input = picked.into_path().map_err(|e| e.to_string())?;
    let output = if input.extension().and_then(|e| e.to_str()) == Some("pgp") {
        input.with_extension("")
    } else {
        let mut o = input.clone().into_os_string();
        o.push(".decrypted");
        PathBuf::from(o)
    };
    mc_core::decrypt_file(&keyring(&app)?, &input, &output).map_err(|e| e.to_string())?;
    Ok(Some(output.to_string_lossy().to_string()))
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            list_keys,
            generate_key,
            import_key,
            export_public,
            delete_key,
            encrypt,
            decrypt,
            sign,
            verify,
            encrypt_file,
            decrypt_file
        ])
        .setup(|app| {
            // Live in the menu bar without a Dock icon, but the window is still a
            // full desktop app the user can keep open.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Regular);

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
