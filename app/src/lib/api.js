// Thin wrappers over the Rust (mc-core) commands.
import { invoke } from "@tauri-apps/api/core";

export const listKeys = () => invoke("list_keys");
export const generateKey = (userid) => invoke("generate_key", { userid });
export const importKey = (armored) => invoke("import_key", { armored });
export const exportPublic = (fingerprint) => invoke("export_public", { fingerprint });
export const deleteKey = (fingerprint) => invoke("delete_key", { fingerprint });
export const encryptText = (plaintext, recipient) =>
  invoke("encrypt", { plaintext, recipient });
export const decryptText = (ciphertext) => invoke("decrypt", { ciphertext });
export const signText = (text, signer) => invoke("sign", { text, signer });
export const verifyText = (signed) => invoke("verify", { signed });
export const encryptFile = (recipient) => invoke("encrypt_file", { recipient });
export const decryptFile = () => invoke("decrypt_file");

// Pretty-print a hex fingerprint in groups of four.
export const prettyFpr = (fpr) => (fpr.match(/.{1,4}/g) || []).join(" ");
