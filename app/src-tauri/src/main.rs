// Monadnock Cyber GPG — desktop + menu-bar app.
// Full-window desktop UI plus a macOS menu-bar / Windows system-tray presence.
// All crypto is delegated to `mc-core`, which persists keys under the app data dir.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
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

// Async so Tauri runs these off the main thread — a blocking native file panel
// on the main thread deadlocks the UI.
#[tauri::command]
async fn encrypt_file(app: tauri::AppHandle, recipient: String) -> Result<Option<String>, String> {
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
async fn decrypt_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
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

/// Reveal a file in Finder (macOS) / Explorer (Windows), highlighting it.
#[tauri::command]
fn reveal(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = std::process::Command::new("open");
        c.arg("-R").arg(&path);
        c
    };
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("explorer");
        c.arg(format!("/select,{}", path));
        c
    };
    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut c = std::process::Command::new("xdg-open");
        let parent = std::path::Path::new(&path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        c.arg(parent);
        c
    };
    cmd.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

// Tray quick actions: transform the clipboard in place. Clipboard + crypto are
// fast and need no event loop, so running on the tray (main) thread is fine.
fn clip_encrypt(app: &tauri::AppHandle) {
    let Ok(kr) = keyring(app) else { return };
    let Ok(text) = app.clipboard().read_text() else {
        let _ = app.emit("tray-toast", "Clipboard has no text");
        return;
    };
    let keys = mc_core::list_keys(&kr).unwrap_or_default();
    let Some(first) = keys.first() else {
        let _ = app.emit("tray-toast", "No keys yet — create one first");
        return;
    };
    match mc_core::encrypt(&kr, &text, &first.fingerprint) {
        Ok(ct) => {
            let _ = app.clipboard().write_text(ct);
            let _ = app.emit("tray-toast", format!("Clipboard encrypted to {} ✓", first.userid));
        }
        Err(e) => {
            let _ = app.emit("tray-toast", format!("Encrypt failed: {e}"));
        }
    }
}

fn clip_decrypt(app: &tauri::AppHandle) {
    let Ok(kr) = keyring(app) else { return };
    let Ok(text) = app.clipboard().read_text() else { return };
    match mc_core::decrypt(&kr, &text) {
        Ok(pt) => {
            let _ = app.clipboard().write_text(pt);
            let _ = app.emit("tray-toast", "Clipboard decrypted ✓");
        }
        Err(e) => {
            let _ = app.emit("tray-toast", format!("Decrypt failed: {e}"));
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
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
            decrypt_file,
            reveal
        ])
        .setup(|app| {
            // Live in the menu bar without a Dock icon, but the window is still a
            // full desktop app the user can keep open.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Regular);

            let enc_clip =
                MenuItem::with_id(app, "enc_clip", "Encrypt Clipboard", true, None::<&str>)?;
            let dec_clip =
                MenuItem::with_id(app, "dec_clip", "Decrypt Clipboard", true, None::<&str>)?;
            let open =
                MenuItem::with_id(app, "open", "Open Monadnock Cyber GPG", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&enc_clip, &dec_clip, &open, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Monadnock Cyber GPG")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "enc_clip" => clip_encrypt(app),
                    "dec_clip" => clip_decrypt(app),
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
