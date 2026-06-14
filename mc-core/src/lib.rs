//! Monadnock Cyber GPG — core OpenPGP operations over a simple on-disk keyring.
//!
//! UI-agnostic: the Tauri app (and later mobile) call into this crate so all
//! security-critical logic lives in one place. Keys are stored as armored
//! files (`<FINGERPRINT>.asc`) under a caller-provided keyring directory.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use sequoia_openpgp as openpgp;

use openpgp::cert::prelude::*;
use openpgp::crypto::SessionKey;
use openpgp::packet::{PKESK, SKESK};
use openpgp::parse::stream::{
    DecryptionHelper, DecryptorBuilder, MessageLayer, MessageStructure, VerificationHelper,
    VerifierBuilder,
};
use openpgp::parse::Parse;
use openpgp::policy::{Policy, StandardPolicy};
use openpgp::serialize::stream::{Armorer, Encryptor2, LiteralWriter, Message, Signer};
use openpgp::serialize::SerializeInto;
use openpgp::types::SymmetricAlgorithm;
use openpgp::Cert;

/// Summary of a key in the keyring (safe to hand to the UI).
#[derive(Debug, Clone, Serialize)]
pub struct CertInfo {
    /// Uppercase hex fingerprint, no spaces — also the stable id used by the UI.
    pub fingerprint: String,
    /// Primary user id, e.g. `"Alice <alice@example.org>"`.
    pub userid: String,
    /// Whether we hold the secret key (can decrypt / sign with it).
    pub has_secret: bool,
}

fn primary_userid(cert: &Cert) -> String {
    cert.userids()
        .next()
        .map(|u| String::from_utf8_lossy(u.userid().value()).to_string())
        .unwrap_or_else(|| "(no user id)".to_string())
}

fn info(cert: &Cert) -> CertInfo {
    CertInfo {
        fingerprint: cert.fingerprint().to_hex(),
        userid: primary_userid(cert),
        has_secret: cert.is_tsk(),
    }
}

fn cert_path(keyring: &Path, fingerprint: &str) -> PathBuf {
    keyring.join(format!("{}.asc", fingerprint.replace(' ', "").to_uppercase()))
}

fn save(keyring: &Path, cert: &Cert) -> Result<CertInfo> {
    fs::create_dir_all(keyring)?;
    let armored = if cert.is_tsk() {
        cert.as_tsk().armored().to_vec()?
    } else {
        cert.armored().to_vec()?
    };
    fs::write(cert_path(keyring, &cert.fingerprint().to_hex()), armored)?;
    Ok(info(cert))
}

fn load_cert(keyring: &Path, fingerprint: &str) -> Result<Cert> {
    let path = cert_path(keyring, fingerprint);
    let bytes = fs::read(&path).with_context(|| format!("no key {fingerprint} in keyring"))?;
    Cert::from_bytes(&bytes).map_err(Into::into)
}

/// Generate a general-purpose Curve25519 key for `userid` and store it.
pub fn generate_key(keyring: &Path, userid: &str) -> Result<CertInfo> {
    let (cert, _revocation) = CertBuilder::new()
        .add_userid(userid)
        .add_signing_subkey()
        .add_transport_encryption_subkey()
        .generate()?;
    save(keyring, &cert)
}

/// Import an armored public or secret key, merging with any existing copy.
pub fn import_cert(keyring: &Path, armored: &str) -> Result<CertInfo> {
    let cert = Cert::from_bytes(armored.as_bytes()).context("not a valid OpenPGP key")?;
    let path = cert_path(keyring, &cert.fingerprint().to_hex());
    let merged = if path.exists() {
        let existing = Cert::from_bytes(&fs::read(&path)?)?;
        existing.merge_public_and_secret(cert)?
    } else {
        cert
    };
    save(keyring, &merged)
}

/// List every key in the keyring, sorted by user id.
pub fn list_keys(keyring: &Path) -> Result<Vec<CertInfo>> {
    let mut out = Vec::new();
    if keyring.exists() {
        for entry in fs::read_dir(keyring)? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) == Some("asc") {
                if let Ok(cert) = Cert::from_bytes(&fs::read(&path)?) {
                    out.push(info(&cert));
                }
            }
        }
    }
    out.sort_by(|a, b| a.userid.to_lowercase().cmp(&b.userid.to_lowercase()));
    Ok(out)
}

/// Export the public certificate (never the secret) for sharing.
pub fn export_public(keyring: &Path, fingerprint: &str) -> Result<String> {
    let cert = load_cert(keyring, fingerprint)?;
    let armored = cert.armored().to_vec()?;
    Ok(String::from_utf8(armored)?)
}

/// Delete a key from the keyring.
pub fn delete_key(keyring: &Path, fingerprint: &str) -> Result<()> {
    fs::remove_file(cert_path(keyring, fingerprint))
        .with_context(|| format!("no key {fingerprint} in keyring"))?;
    Ok(())
}

/// Encrypt `plaintext` for the keyring key `recipient_fingerprint`.
pub fn encrypt(keyring: &Path, plaintext: &str, recipient_fingerprint: &str) -> Result<String> {
    let policy = &StandardPolicy::new();
    let cert = load_cert(keyring, recipient_fingerprint)?;

    let recipients = cert
        .keys()
        .with_policy(policy, None)
        .supported()
        .alive()
        .revoked(false)
        .for_transport_encryption();

    let mut sink: Vec<u8> = Vec::new();
    {
        let message = Message::new(&mut sink);
        let message = Armorer::new(message).build()?;
        let message = Encryptor2::for_recipients(message, recipients).build()?;
        let mut writer = LiteralWriter::new(message).build()?;
        writer.write_all(plaintext.as_bytes())?;
        writer.finalize()?;
    }
    Ok(String::from_utf8(sink)?)
}

/// Find a key in the keyring whose user id contains `email` (case-insensitive).
fn cert_for_email(keyring: &Path, email: &str) -> Option<Cert> {
    let needle = email.to_lowercase();
    let entries = fs::read_dir(keyring).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("asc") {
            continue;
        }
        let Ok(bytes) = fs::read(&path) else { continue };
        let Ok(cert) = Cert::from_bytes(&bytes) else { continue };
        for uid in cert.userids() {
            if String::from_utf8_lossy(uid.userid().value())
                .to_lowercase()
                .contains(&needle)
            {
                return Some(cert);
            }
        }
    }
    None
}

/// Of `emails`, which have NO key in the keyring (so we can't encrypt to them).
pub fn emails_without_keys(keyring: &Path, emails: &[String]) -> Vec<String> {
    emails
        .iter()
        .filter(|e| !e.is_empty() && cert_for_email(keyring, e).is_none())
        .cloned()
        .collect()
}

/// Encrypt `plaintext` to every recipient identified by `emails` (multi-recipient).
pub fn encrypt_to_emails(keyring: &Path, plaintext: &str, emails: &[String]) -> Result<String> {
    let policy = &StandardPolicy::new();
    let certs: Vec<Cert> = emails.iter().filter_map(|e| cert_for_email(keyring, e)).collect();
    if certs.is_empty() {
        return Err(anyhow!("no keys found for any recipient"));
    }
    let mut recipients = Vec::new();
    for cert in &certs {
        for ka in cert
            .keys()
            .with_policy(policy, None)
            .supported()
            .alive()
            .revoked(false)
            .for_transport_encryption()
        {
            recipients.push(ka);
        }
    }
    if recipients.is_empty() {
        return Err(anyhow!("recipient keys have no usable encryption subkey"));
    }

    let mut sink: Vec<u8> = Vec::new();
    {
        let message = Message::new(&mut sink);
        let message = Armorer::new(message).build()?;
        let message = Encryptor2::for_recipients(message, recipients).build()?;
        let mut writer = LiteralWriter::new(message).build()?;
        writer.write_all(plaintext.as_bytes())?;
        writer.finalize()?;
    }
    Ok(String::from_utf8(sink)?)
}

/// Decrypt armored `ciphertext` using whichever secret key in the keyring fits.
pub fn decrypt(keyring: &Path, ciphertext: &str) -> Result<String> {
    let policy = &StandardPolicy::new();

    let mut secret_certs = Vec::new();
    if keyring.exists() {
        for entry in fs::read_dir(keyring)? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) == Some("asc") {
                if let Ok(cert) = Cert::from_bytes(&fs::read(&path)?) {
                    if cert.is_tsk() {
                        secret_certs.push(cert);
                    }
                }
            }
        }
    }
    if secret_certs.is_empty() {
        return Err(anyhow!("no secret keys in the keyring to decrypt with"));
    }

    let helper = Helper { policy, certs: secret_certs };
    let mut decryptor =
        DecryptorBuilder::from_bytes(ciphertext.as_bytes())?.with_policy(policy, None, helper)?;

    let mut plaintext: Vec<u8> = Vec::new();
    std::io::copy(&mut decryptor, &mut plaintext)?;
    Ok(String::from_utf8(plaintext)?)
}

/// Outcome of verifying a signed message.
#[derive(Debug, Clone, Serialize)]
pub struct VerifyOutcome {
    pub valid: bool,
    pub signer: Option<String>,
    pub text: String,
}

fn all_certs(keyring: &Path) -> Result<Vec<Cert>> {
    let mut out = Vec::new();
    if keyring.exists() {
        for entry in fs::read_dir(keyring)? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) == Some("asc") {
                if let Ok(cert) = Cert::from_bytes(&fs::read(&path)?) {
                    out.push(cert);
                }
            }
        }
    }
    Ok(out)
}

/// Sign `text` with the keyring key `signer_fingerprint`; armored signed message.
pub fn sign(keyring: &Path, text: &str, signer_fingerprint: &str) -> Result<String> {
    let policy = &StandardPolicy::new();
    let cert = load_cert(keyring, signer_fingerprint)?;
    let keypair = cert
        .keys()
        .unencrypted_secret()
        .with_policy(policy, None)
        .for_signing()
        .next()
        .ok_or_else(|| anyhow!("no signing key for {signer_fingerprint}"))?
        .key()
        .clone()
        .into_keypair()?;

    let mut sink: Vec<u8> = Vec::new();
    {
        let message = Message::new(&mut sink);
        let message = Armorer::new(message).build()?;
        let message = Signer::new(message, keypair).build()?;
        let mut writer = LiteralWriter::new(message).build()?;
        writer.write_all(text.as_bytes())?;
        writer.finalize()?;
    }
    Ok(String::from_utf8(sink)?)
}

/// Verify an armored signed message against the keyring.
pub fn verify(keyring: &Path, signed: &str) -> Result<VerifyOutcome> {
    let policy = &StandardPolicy::new();
    let helper = VerifyHelper {
        certs: all_certs(keyring)?,
        signer: None,
    };
    let mut content: Vec<u8> = Vec::new();
    let mut verifier =
        VerifierBuilder::from_bytes(signed.as_bytes())?.with_policy(policy, None, helper)?;
    match std::io::copy(&mut verifier, &mut content) {
        Ok(_) => Ok(VerifyOutcome {
            valid: true,
            signer: verifier.helper_ref().signer.clone(),
            text: String::from_utf8(content)?,
        }),
        Err(_) => Ok(VerifyOutcome {
            valid: false,
            signer: None,
            text: String::new(),
        }),
    }
}

struct VerifyHelper {
    certs: Vec<Cert>,
    signer: Option<String>,
}

impl VerificationHelper for VerifyHelper {
    fn get_certs(&mut self, _ids: &[openpgp::KeyHandle]) -> Result<Vec<Cert>> {
        Ok(self.certs.clone())
    }
    fn check(&mut self, structure: MessageStructure) -> Result<()> {
        for layer in structure.into_iter() {
            if let MessageLayer::SignatureGroup { results } = layer {
                for result in results {
                    if let Ok(good) = result {
                        let fpr = good.ka.cert().fingerprint().to_hex();
                        self.signer = self
                            .certs
                            .iter()
                            .find(|c| c.fingerprint().to_hex() == fpr)
                            .map(primary_userid)
                            .or(Some(fpr));
                        return Ok(());
                    }
                }
            }
        }
        Err(anyhow!("no valid signature"))
    }
}

struct Helper<'a> {
    policy: &'a dyn Policy,
    certs: Vec<Cert>,
}

impl<'a> VerificationHelper for Helper<'a> {
    fn get_certs(&mut self, _ids: &[openpgp::KeyHandle]) -> Result<Vec<Cert>> {
        Ok(Vec::new())
    }
    fn check(&mut self, _structure: MessageStructure) -> Result<()> {
        Ok(())
    }
}

impl<'a> DecryptionHelper for Helper<'a> {
    fn decrypt<D>(
        &mut self,
        pkesks: &[PKESK],
        _skesks: &[SKESK],
        sym_algo: Option<SymmetricAlgorithm>,
        mut decrypt: D,
    ) -> Result<Option<openpgp::Fingerprint>>
    where
        D: FnMut(SymmetricAlgorithm, &SessionKey) -> bool,
    {
        for cert in &self.certs {
            for ka in cert
                .keys()
                .unencrypted_secret()
                .with_policy(self.policy, None)
                .for_transport_encryption()
            {
                let mut keypair = match ka.key().clone().into_keypair() {
                    Ok(kp) => kp,
                    Err(_) => continue,
                };
                for pkesk in pkesks {
                    if pkesk
                        .decrypt(&mut keypair, sym_algo)
                        .map(|(algo, session_key)| decrypt(algo, &session_key))
                        .unwrap_or(false)
                    {
                        return Ok(Some(cert.fingerprint()));
                    }
                }
            }
        }
        Err(anyhow!("no matching secret key in keyring"))
    }
}

/// Encrypt a file on disk for `recipient_fingerprint` (binary OpenPGP output).
pub fn encrypt_file(
    keyring: &Path,
    input: &Path,
    output: &Path,
    recipient_fingerprint: &str,
) -> Result<()> {
    let policy = &StandardPolicy::new();
    let cert = load_cert(keyring, recipient_fingerprint)?;
    let recipients = cert
        .keys()
        .with_policy(policy, None)
        .supported()
        .alive()
        .revoked(false)
        .for_transport_encryption();

    let mut sink = fs::File::create(output)?;
    let message = Message::new(&mut sink);
    let message = Encryptor2::for_recipients(message, recipients).build()?;
    let mut writer = LiteralWriter::new(message).build()?;
    let mut reader = fs::File::open(input)?;
    std::io::copy(&mut reader, &mut writer)?;
    writer.finalize()?;
    Ok(())
}

/// Decrypt an OpenPGP file using whichever secret key in the keyring fits.
pub fn decrypt_file(keyring: &Path, input: &Path, output: &Path) -> Result<()> {
    let policy = &StandardPolicy::new();
    let secret_certs: Vec<Cert> = all_certs(keyring)?.into_iter().filter(|c| c.is_tsk()).collect();
    if secret_certs.is_empty() {
        return Err(anyhow!("no secret keys in the keyring to decrypt with"));
    }
    let helper = Helper { policy, certs: secret_certs };
    let reader = fs::File::open(input)?;
    let mut decryptor =
        DecryptorBuilder::from_reader(reader)?.with_policy(policy, None, helper)?;
    let mut out = fs::File::create(output)?;
    std::io::copy(&mut decryptor, &mut out)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("mc-{}-{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn keyring_encrypt_decrypt_and_export() {
        let dir = tmp("kr");
        let _alice = generate_key(&dir, "Alice <alice@example.org>").unwrap();
        let bob = generate_key(&dir, "Bob <bob@example.org>").unwrap();

        let keys = list_keys(&dir).unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.iter().all(|k| k.has_secret));

        let ct = encrypt(&dir, "secret message", &bob.fingerprint).unwrap();
        assert!(ct.contains("BEGIN PGP MESSAGE"));
        let pt = decrypt(&dir, &ct).unwrap();
        assert_eq!(pt, "secret message");

        // Exported key is public-only and re-imports without secret.
        let pubkey = export_public(&dir, &bob.fingerprint).unwrap();
        assert!(pubkey.contains("PUBLIC KEY") && !pubkey.contains("PRIVATE KEY"));

        let dir2 = tmp("kr2");
        let imported = import_cert(&dir2, &pubkey).unwrap();
        assert_eq!(imported.fingerprint, bob.fingerprint);
        assert!(!imported.has_secret);

        fs::remove_dir_all(&dir).ok();
        fs::remove_dir_all(&dir2).ok();
    }

    #[test]
    fn sign_and_verify() {
        let dir = tmp("sv");
        let alice = generate_key(&dir, "Alice <alice@example.org>").unwrap();

        let signed = sign(&dir, "trust me", &alice.fingerprint).unwrap();
        assert!(signed.contains("BEGIN PGP MESSAGE"));

        let outcome = verify(&dir, &signed).unwrap();
        assert!(outcome.valid);
        assert_eq!(outcome.text, "trust me");
        assert!(outcome.signer.unwrap().contains("Alice"));

        // A tampered message should not verify.
        let tampered = signed.replace("trust me", "trust me ");
        let bad = verify(&dir, &tampered).unwrap();
        assert!(!bad.valid || bad.text != "trust me ");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn file_round_trip() {
        let dir = tmp("file");
        let bob = generate_key(&dir, "Bob <bob@example.org>").unwrap();

        let plain = dir.join("secret.txt");
        fs::write(&plain, b"file contents 123").unwrap();
        let enc = dir.join("secret.txt.pgp");
        encrypt_file(&dir, &plain, &enc, &bob.fingerprint).unwrap();
        let dec = dir.join("secret.out.txt");
        decrypt_file(&dir, &enc, &dec).unwrap();

        assert_eq!(fs::read(&dec).unwrap(), b"file contents 123");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn encrypt_to_emails_multi() {
        let dir = tmp("eml");
        generate_key(&dir, "Alice <alice@example.org>").unwrap();
        generate_key(&dir, "Bob <bob@example.org>").unwrap();

        let missing = emails_without_keys(
            &dir, &["alice@example.org".into(), "nobody@example.com".into()]);
        assert_eq!(missing, vec!["nobody@example.com".to_string()]);

        let ct = encrypt_to_emails(
            &dir, "multi-recipient", &["alice@example.org".into(), "bob@example.org".into()]).unwrap();
        assert!(ct.contains("BEGIN PGP MESSAGE"));
        assert_eq!(decrypt(&dir, &ct).unwrap(), "multi-recipient");

        fs::remove_dir_all(&dir).ok();
    }
}
