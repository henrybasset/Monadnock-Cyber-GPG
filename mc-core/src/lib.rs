//! Monadnock Cyber GPG — core OpenPGP operations.
//!
//! Thin, UI-agnostic wrapper over Sequoia-PGP: generate an OpenPGP key, and
//! encrypt/decrypt text. The Tauri app (and later mobile) call into this crate
//! so all security-critical logic lives in one place.

use std::io::Write;

use anyhow::{anyhow, Result};
use sequoia_openpgp as openpgp;

use openpgp::cert::prelude::*;
use openpgp::crypto::SessionKey;
use openpgp::packet::{PKESK, SKESK};
use openpgp::parse::stream::{
    DecryptionHelper, DecryptorBuilder, MessageStructure, VerificationHelper,
};
use openpgp::parse::Parse;
use openpgp::policy::{Policy, StandardPolicy};
use openpgp::serialize::stream::{Armorer, Encryptor2, LiteralWriter, Message};
use openpgp::serialize::SerializeInto;
use openpgp::types::SymmetricAlgorithm;

/// A freshly generated OpenPGP key, ASCII-armored.
#[derive(Debug, Clone)]
pub struct GeneratedKey {
    /// Public certificate (shareable).
    pub public: String,
    /// Secret key (the Transferable Secret Key) — keep private.
    pub secret: String,
    /// Primary key fingerprint.
    pub fingerprint: String,
}

/// Generate a general-purpose Curve25519 key for `userid`
/// (e.g. `"Alice <alice@example.org>"`).
pub fn generate_key(userid: &str) -> Result<GeneratedKey> {
    let (cert, _revocation) = CertBuilder::new()
        .add_userid(userid)
        .add_signing_subkey()
        .add_transport_encryption_subkey()
        .generate()?;

    // Bind to locals first so the armored temporaries (which borrow `cert`)
    // drop before `cert` does.
    let public = String::from_utf8(cert.armored().to_vec()?)?;
    let secret = String::from_utf8(cert.as_tsk().armored().to_vec()?)?;
    let fingerprint = cert.fingerprint().to_string();

    Ok(GeneratedKey { public, secret, fingerprint })
}

/// Encrypt `plaintext` for the holder of `recipient_public` (armored cert).
/// Returns ASCII-armored ciphertext.
pub fn encrypt(plaintext: &str, recipient_public: &str) -> Result<String> {
    let policy = &StandardPolicy::new();
    let cert = Cert::from_bytes(recipient_public.as_bytes())?;

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

/// Decrypt armored `ciphertext` with `secret` (armored TSK). Returns plaintext.
pub fn decrypt(ciphertext: &str, secret: &str) -> Result<String> {
    let policy = &StandardPolicy::new();
    let cert = Cert::from_bytes(secret.as_bytes())?;

    let helper = Helper { policy, secret: &cert };
    let mut decryptor =
        DecryptorBuilder::from_bytes(ciphertext.as_bytes())?.with_policy(policy, None, helper)?;

    let mut plaintext: Vec<u8> = Vec::new();
    std::io::copy(&mut decryptor, &mut plaintext)?;
    Ok(String::from_utf8(plaintext)?)
}

struct Helper<'a> {
    policy: &'a dyn Policy,
    secret: &'a Cert,
}

impl<'a> VerificationHelper for Helper<'a> {
    fn get_certs(&mut self, _ids: &[openpgp::KeyHandle]) -> Result<Vec<Cert>> {
        Ok(Vec::new()) // Phase 0: not verifying signatures yet.
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
        let keypair = self
            .secret
            .keys()
            .unencrypted_secret()
            .with_policy(self.policy, None)
            .for_transport_encryption()
            .next()
            .ok_or_else(|| anyhow!("no usable decryption subkey"))?
            .key()
            .clone()
            .into_keypair()?;

        let mut keypair = keypair;
        for pkesk in pkesks {
            if pkesk
                .decrypt(&mut keypair, sym_algo)
                .map(|(algo, session_key)| decrypt(algo, &session_key))
                .unwrap_or(false)
            {
                return Ok(Some(self.secret.fingerprint()));
            }
        }
        Err(anyhow!("could not decrypt with this key"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let key = generate_key("Test User <test@example.org>").expect("keygen");
        assert!(key.public.contains("BEGIN PGP PUBLIC KEY BLOCK"));
        assert!(key.secret.contains("BEGIN PGP PRIVATE KEY BLOCK"));

        let secret_msg = "Hello from Monadnock Cyber GPG!";
        let ct = encrypt(secret_msg, &key.public).expect("encrypt");
        assert!(ct.contains("BEGIN PGP MESSAGE"));

        let pt = decrypt(&ct, &key.secret).expect("decrypt");
        assert_eq!(pt, secret_msg);
    }
}
