// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Session file encryption at rest.
//!
//! Uses AES-256-GCM with a key derived from machine ID + app identifier via Argon2id.
//! No user password; key is bound to the machine.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use std::fs;

/// Application identifier used in key derivation (must match between encrypt/decrypt).
const KEY_DOMAIN: &[u8] = b"discord-qt-sessions-v1";
/// Fixed salt for Argon2 (deterministic per machine; no per-file salt needed for this use case).
const ARGON2_SALT: &[u8] = b"discord-qt-sessions-salt";

/// Length of the nonce in bytes (96 bits for AES-GCM).
const NONCE_LEN: usize = 12;

/// Try to read machine-bound identifier. Used as input to key derivation.
fn machine_id() -> Vec<u8> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(data) = fs::read_to_string("/etc/machine-id") {
            return data.trim_end().as_bytes().to_vec();
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(out) = Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout);
                if let Some(start) = s.find("IOPlatformUUID") {
                    let rest = &s[start..];
                    if let Some(open) = rest.find('"') {
                        let after = &rest[open + 1..];
                        if let Some(close) = after.find('"') {
                            return after[..close].as_bytes().to_vec();
                        }
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        if let Ok(out) = Command::new("reg")
            .args([
                "query",
                "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Cryptography",
                "/v",
                "MachineGuid",
            ])
            .output()
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout);
                for line in s.lines() {
                    if line.trim().starts_with("MachineGuid") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            return parts[2].as_bytes().to_vec();
                        }
                        break;
                    }
                }
            }
        }
    }

    // Fallback: bind to user home so at least it's not the same on every machine
    let fallback = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "discord-qt-fallback".to_string());
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    fallback.hash(&mut hasher);
    hasher.finish().to_le_bytes().to_vec()
}

/// Derive a 32-byte key from machine ID using Argon2id.
fn derive_key() -> [u8; 32] {
    let machine = machine_id();
    let mut input = machine;
    input.extend_from_slice(KEY_DOMAIN);
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(&input, ARGON2_SALT, &mut key)
        .expect("Argon2 key derivation");
    key
}

/// Encrypt plaintext with AES-256-GCM. Returns nonce (12 bytes) || ciphertext (includes tag).
pub fn encrypt(plaintext: &[u8]) -> crate::Result<Vec<u8>> {
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| crate::DiscordError::Other(e.to_string()))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| crate::DiscordError::Other(e.to_string()))?;
    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt data produced by encrypt (nonce || ciphertext). Returns plaintext.
pub fn decrypt(data: &[u8]) -> crate::Result<Vec<u8>> {
    if data.len() < NONCE_LEN + 16 {
        return Err(crate::DiscordError::Other("encrypted data too short".to_string()));
    }
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| crate::DiscordError::Other(e.to_string()))?;
    let (nonce_slice, ciphertext) = data.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_slice);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| crate::DiscordError::Other(e.to_string()))?;
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn roundtrip() {
        let plain = b"hello world session data";
        let encrypted = encrypt(plain).unwrap();
        assert_ne!(&encrypted[..], plain);
        assert_eq!(encrypted.len(), NONCE_LEN + plain.len() + 16);
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(&decrypted[..], plain);
    }

    #[test]
    fn wrong_data_fails() {
        let mut bad = vec![0u8; 50];
        rand::rngs::OsRng.fill_bytes(&mut bad[..]);
        assert!(decrypt(&bad).is_err());
    }
}
