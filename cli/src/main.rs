//! `mc` — command-line core for Monadnock Cyber GPG.
//!
//! Shares the desktop app's keyring (~/Library/Application Support/
//! com.monadnockcyber.gpg/keyring) so keys created in the app work here, and
//! powers macOS right-click Services / Quick Actions. Override the keyring with
//! the MC_KEYRING environment variable.
//!
//! Usage:
//!   mc gen "Name <email>"          create a key
//!   mc list                        list keys
//!   mc encrypt --to <FPR>          stdin -> encrypted (stdout)
//!   mc decrypt                     stdin -> plaintext (stdout)  (tolerates surrounding text)
//!   mc sign --as <FPR>             stdin -> signed (stdout)
//!   mc verify                      stdin -> prints signer + verified text

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;

use anyhow::{anyhow, Result};

fn keyring() -> PathBuf {
    if let Ok(p) = std::env::var("MC_KEYRING") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join("Library/Application Support/com.monadnockcyber.gpg/keyring")
}

fn read_stdin() -> Result<String> {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s)?;
    Ok(s)
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1).cloned())
}

fn extract_pgp(kind: &str, text: &str) -> String {
    let begin = format!("-----BEGIN PGP {kind}-----");
    let end = format!("-----END PGP {kind}-----");
    if let (Some(a), Some(b)) = (text.find(&begin), text.find(&end)) {
        return text[a..b + end.len()].to_string();
    }
    text.trim().to_string()
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let kr = keyring();
    match args.get(1).map(String::as_str) {
        Some("gen") => {
            let uid = args.get(2).ok_or_else(|| anyhow!("usage: mc gen \"Name <email>\""))?;
            let info = mc_core::generate_key(&kr, uid)?;
            println!("{}  {}", info.fingerprint, info.userid);
        }
        Some("list") => {
            for k in mc_core::list_keys(&kr)? {
                println!("{} {} {}", if k.has_secret { "sec" } else { "pub" }, k.fingerprint, k.userid);
            }
        }
        Some("encrypt") => {
            let to = flag(&args, "--to").ok_or_else(|| anyhow!("usage: mc encrypt --to <FPR>"))?;
            let input = read_stdin()?;
            print!("{}", mc_core::encrypt(&kr, &input, &to)?);
        }
        Some("decrypt") => {
            let input = read_stdin()?;
            let block = extract_pgp("MESSAGE", &input);
            print!("{}", mc_core::decrypt(&kr, &block)?);
        }
        Some("sign") => {
            let as_fpr = flag(&args, "--as").ok_or_else(|| anyhow!("usage: mc sign --as <FPR>"))?;
            let input = read_stdin()?;
            print!("{}", mc_core::sign(&kr, &input, &as_fpr)?);
        }
        Some("verify") => {
            let input = read_stdin()?;
            let block = extract_pgp("MESSAGE", &input);
            let r = mc_core::verify(&kr, &block)?;
            if r.valid {
                eprintln!("Good signature from {}", r.signer.unwrap_or_else(|| "?".into()));
                print!("{}", r.text);
            } else {
                return Err(anyhow!("signature could not be verified"));
            }
        }
        _ => {
            eprintln!("mc — Monadnock Cyber GPG\n\nCommands: gen, list, encrypt --to <FPR>, decrypt, sign --as <FPR>, verify");
            exit(2);
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        let _ = std::io::stderr().flush();
        exit(1);
    }
}
