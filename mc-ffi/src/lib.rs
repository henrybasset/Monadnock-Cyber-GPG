//! C ABI over `mc-core`, for the Swift / MailKit extension to link against.
//!
//! All functions take the keyring directory as a C string. String results are
//! heap-allocated C strings the caller must free with `mc_string_free`; a NULL
//! return means failure.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

unsafe fn as_str<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    CStr::from_ptr(p).to_str().ok()
}

fn to_c(s: String) -> *mut c_char {
    CString::new(s).map(CString::into_raw).unwrap_or(std::ptr::null_mut())
}

/// Decrypt armored `ciphertext` against the keyring. NULL on failure.
#[no_mangle]
pub extern "C" fn mc_decrypt(keyring: *const c_char, ciphertext: *const c_char) -> *mut c_char {
    let (Some(kr), Some(ct)) = (unsafe { as_str(keyring) }, unsafe { as_str(ciphertext) }) else {
        return std::ptr::null_mut();
    };
    match mc_core::decrypt(Path::new(kr), ct) {
        Ok(pt) => to_c(pt),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Encrypt `plaintext` to recipient fingerprint. NULL on failure.
#[no_mangle]
pub extern "C" fn mc_encrypt(
    keyring: *const c_char,
    plaintext: *const c_char,
    recipient: *const c_char,
) -> *mut c_char {
    let (Some(kr), Some(pt), Some(rcpt)) = (
        unsafe { as_str(keyring) },
        unsafe { as_str(plaintext) },
        unsafe { as_str(recipient) },
    ) else {
        return std::ptr::null_mut();
    };
    match mc_core::encrypt(Path::new(kr), pt, rcpt) {
        Ok(ct) => to_c(ct),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Sign `text` with the given signer fingerprint. NULL on failure.
#[no_mangle]
pub extern "C" fn mc_sign(
    keyring: *const c_char,
    text: *const c_char,
    signer: *const c_char,
) -> *mut c_char {
    let (Some(kr), Some(t), Some(s)) = (
        unsafe { as_str(keyring) },
        unsafe { as_str(text) },
        unsafe { as_str(signer) },
    ) else {
        return std::ptr::null_mut();
    };
    match mc_core::sign(Path::new(kr), t, s) {
        Ok(out) => to_c(out),
        Err(_) => std::ptr::null_mut(),
    }
}

/// JSON array of keys: `[{"fingerprint","userid","has_secret"}, ...]`. NULL on failure.
#[no_mangle]
pub extern "C" fn mc_list_json(keyring: *const c_char) -> *mut c_char {
    let Some(kr) = (unsafe { as_str(keyring) }) else {
        return std::ptr::null_mut();
    };
    match mc_core::list_keys(Path::new(kr)) {
        Ok(keys) => serde_json::to_string(&keys).map(to_c).unwrap_or(std::ptr::null_mut()),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Generate a key for `userid` and store it. Returns JSON key info. NULL on failure.
#[no_mangle]
pub extern "C" fn mc_generate(keyring: *const c_char, userid: *const c_char) -> *mut c_char {
    let (Some(kr), Some(uid)) = (unsafe { as_str(keyring) }, unsafe { as_str(userid) }) else {
        return std::ptr::null_mut();
    };
    match mc_core::generate_key(Path::new(kr), uid) {
        Ok(info) => serde_json::to_string(&info).map(to_c).unwrap_or(std::ptr::null_mut()),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Import an armored key. Returns JSON key info. NULL on failure.
#[no_mangle]
pub extern "C" fn mc_import(keyring: *const c_char, armored: *const c_char) -> *mut c_char {
    let (Some(kr), Some(a)) = (unsafe { as_str(keyring) }, unsafe { as_str(armored) }) else {
        return std::ptr::null_mut();
    };
    match mc_core::import_cert(Path::new(kr), a) {
        Ok(info) => serde_json::to_string(&info).map(to_c).unwrap_or(std::ptr::null_mut()),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a string returned by this library.
#[no_mangle]
pub extern "C" fn mc_string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)) };
    }
}
