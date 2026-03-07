// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Voice audio encryption/decryption
//!
//! Implements the encryption layer for Discord voice UDP packets.
//! Discord supports multiple encryption modes; we implement the most common ones.
//!
//! Packet structure (xsalsa20_poly1305):
//! [RTP header (12 bytes)] [encrypted Opus payload + 16-byte MAC]
//!
//! The RTP header is used as the nonce (padded to 24 bytes with zeros).

use super::udp::RtpHeader;
use xsalsa20poly1305::aead::{Aead, KeyInit};
use xsalsa20poly1305::{Nonce, XSalsa20Poly1305};

/// Encryption mode implementations
pub enum EncryptionMode {
    /// xsalsa20_poly1305 — nonce is the RTP header (padded to 24 bytes)
    XSalsa20Poly1305,
    /// xsalsa20_poly1305_lite — nonce is an incrementing 4-byte counter appended to packet
    XSalsa20Poly1305Lite,
    /// xsalsa20_poly1305_suffix — nonce is 24 random bytes appended to packet
    XSalsa20Poly1305Suffix,
}

impl EncryptionMode {
    /// Parse encryption mode from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "xsalsa20_poly1305" => Some(Self::XSalsa20Poly1305),
            "xsalsa20_poly1305_lite" => Some(Self::XSalsa20Poly1305Lite),
            "xsalsa20_poly1305_suffix" => Some(Self::XSalsa20Poly1305Suffix),
            _ => None,
        }
    }
}

/// Build the nonce for xsalsa20_poly1305 mode
/// Uses the 12-byte RTP header padded with zeros to 24 bytes
pub fn build_rtp_nonce(rtp_header: &[u8; 12]) -> [u8; 24] {
    let mut nonce = [0u8; 24];
    nonce[..12].copy_from_slice(rtp_header);
    nonce
}

/// Build the nonce for xsalsa20_poly1305_lite mode
/// Uses a 4-byte incrementing counter, padded to 24 bytes
pub fn build_lite_nonce(counter: u32) -> [u8; 24] {
    let mut nonce = [0u8; 24];
    nonce[..4].copy_from_slice(&counter.to_be_bytes());
    nonce
}

/// Build a complete voice UDP packet (unencrypted — encryption applied externally)
///
/// Structure: [RTP header 12 bytes] [Opus audio payload]
/// After encryption: [RTP header 12 bytes] [encrypted payload + 16-byte MAC]
pub fn build_voice_packet(header: &RtpHeader, opus_data: &[u8]) -> Vec<u8> {
    let header_bytes = header.to_bytes();
    let mut packet = Vec::with_capacity(12 + opus_data.len());
    packet.extend_from_slice(&header_bytes);
    packet.extend_from_slice(opus_data);
    packet
}

/// Extract the RTP header and encrypted payload from a received packet
pub fn split_voice_packet(packet: &[u8]) -> Option<([u8; 12], &[u8])> {
    if packet.len() < 12 {
        return None;
    }
    let mut header = [0u8; 12];
    header.copy_from_slice(&packet[..12]);
    Some((header, &packet[12..]))
}

/// Encrypt an Opus audio frame using xsalsa20_poly1305.
///
/// Takes the RTP header bytes (for nonce), the opus payload, and the secret key.
/// Returns the encrypted payload (original + 16-byte MAC tag).
pub fn encrypt_xsalsa20(
    rtp_header: &[u8; 12],
    opus_data: &[u8],
    secret_key: &[u8; 32],
) -> Result<Vec<u8>, String> {
    let cipher =
        XSalsa20Poly1305::new_from_slice(secret_key).map_err(|e| format!("Invalid key: {}", e))?;
    let nonce_bytes = build_rtp_nonce(rtp_header);
    let nonce = Nonce::from_slice(&nonce_bytes);
    cipher
        .encrypt(nonce, opus_data)
        .map_err(|e| format!("Encryption failed: {}", e))
}

/// Decrypt an encrypted audio payload using xsalsa20_poly1305.
///
/// Takes the RTP header bytes (for nonce), the encrypted payload, and the secret key.
/// Returns the decrypted Opus data.
pub fn decrypt_xsalsa20(
    rtp_header: &[u8; 12],
    encrypted_data: &[u8],
    secret_key: &[u8; 32],
) -> Result<Vec<u8>, String> {
    let cipher =
        XSalsa20Poly1305::new_from_slice(secret_key).map_err(|e| format!("Invalid key: {}", e))?;
    let nonce_bytes = build_rtp_nonce(rtp_header);
    let nonce = Nonce::from_slice(&nonce_bytes);
    cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))
}

/// Encrypt using xsalsa20_poly1305_lite mode.
///
/// The nonce is a 4-byte incrementing counter (extended to 24 bytes).
/// The counter bytes are appended to the packet after encryption.
pub fn encrypt_xsalsa20_lite(
    opus_data: &[u8],
    secret_key: &[u8; 32],
    counter: u32,
) -> Result<(Vec<u8>, [u8; 4]), String> {
    let cipher =
        XSalsa20Poly1305::new_from_slice(secret_key).map_err(|e| format!("Invalid key: {}", e))?;
    let nonce_bytes = build_lite_nonce(counter);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let encrypted = cipher
        .encrypt(nonce, opus_data)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    Ok((encrypted, counter.to_be_bytes()))
}

/// Build a complete encrypted voice packet ready to send via UDP.
///
/// Returns: [RTP header (12 bytes)] [encrypted opus + MAC (payload_len + 16)]
pub fn build_encrypted_packet(
    header: &RtpHeader,
    opus_data: &[u8],
    secret_key: &[u8; 32],
    mode: &EncryptionMode,
    lite_counter: u32,
) -> Result<Vec<u8>, String> {
    let header_bytes = header.to_bytes();
    let mut packet = Vec::with_capacity(12 + opus_data.len() + 16 + 4);
    packet.extend_from_slice(&header_bytes);

    match mode {
        EncryptionMode::XSalsa20Poly1305 => {
            let encrypted = encrypt_xsalsa20(&header_bytes, opus_data, secret_key)?;
            packet.extend_from_slice(&encrypted);
        }
        EncryptionMode::XSalsa20Poly1305Lite => {
            let (encrypted, counter_bytes) =
                encrypt_xsalsa20_lite(opus_data, secret_key, lite_counter)?;
            packet.extend_from_slice(&encrypted);
            packet.extend_from_slice(&counter_bytes);
        }
        EncryptionMode::XSalsa20Poly1305Suffix => {
            // Generate 24 random bytes as nonce, append after ciphertext
            let mut nonce_bytes = [0u8; 24];
            use rand::RngCore;
            rand::thread_rng().fill_bytes(&mut nonce_bytes);
            let cipher = XSalsa20Poly1305::new_from_slice(secret_key)
                .map_err(|e| format!("Invalid key: {}", e))?;
            let nonce = Nonce::from_slice(&nonce_bytes);
            let encrypted = cipher
                .encrypt(nonce, opus_data)
                .map_err(|e| format!("Encryption failed: {}", e))?;
            packet.extend_from_slice(&encrypted);
            packet.extend_from_slice(&nonce_bytes);
        }
    }

    Ok(packet)
}

/// Audio frame timing constants
pub mod timing {
    use std::time::Duration;

    /// Duration of a single Opus frame (20ms)
    pub const FRAME_DURATION: Duration = Duration::from_millis(20);

    /// Number of frames per second (50)
    pub const FRAMES_PER_SECOND: u32 = 50;

    /// Timestamp increment per frame (960 samples at 48kHz)
    pub const TIMESTAMP_INCREMENT: u32 = 960;

    /// Number of silence frames to send after stopping speech
    /// Discord expects ~5 silence frames to properly end speaking
    pub const SILENCE_FRAME_COUNT: u32 = 5;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtp_nonce() {
        let header = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let nonce = build_rtp_nonce(&header);
        assert_eq!(&nonce[..12], &header);
        assert_eq!(&nonce[12..], &[0u8; 12]);
        assert_eq!(nonce.len(), 24);
    }

    #[test]
    fn test_lite_nonce() {
        let nonce = build_lite_nonce(42);
        assert_eq!(nonce[..4], 42u32.to_be_bytes());
        assert_eq!(&nonce[4..], &[0u8; 20]);
    }

    #[test]
    fn test_build_voice_packet() {
        let header = RtpHeader::new(12345);
        let opus = [0xF8, 0xFF, 0xFE]; // silence-ish
        let packet = build_voice_packet(&header, &opus);
        assert_eq!(packet.len(), 12 + 3);
        // First 12 bytes are header
        assert_eq!(&packet[..2], &[0x80, 0x78]); // version=2, PT=120
    }

    #[test]
    fn test_split_voice_packet() {
        let header = RtpHeader::new(99);
        let opus = vec![1, 2, 3, 4, 5];
        let packet = build_voice_packet(&header, &opus);

        let (hdr, payload) = split_voice_packet(&packet).unwrap();
        assert_eq!(payload, &[1, 2, 3, 4, 5]);

        // Parse header back
        let parsed = RtpHeader::from_bytes(&hdr).unwrap();
        assert_eq!(parsed.ssrc, 99);
    }

    #[test]
    fn test_split_too_short() {
        assert!(split_voice_packet(&[0u8; 11]).is_none());
        assert!(split_voice_packet(&[]).is_none());
    }

    #[test]
    fn test_encryption_mode_parse() {
        assert!(matches!(
            EncryptionMode::from_str("xsalsa20_poly1305"),
            Some(EncryptionMode::XSalsa20Poly1305)
        ));
        assert!(matches!(
            EncryptionMode::from_str("xsalsa20_poly1305_lite"),
            Some(EncryptionMode::XSalsa20Poly1305Lite)
        ));
        assert!(EncryptionMode::from_str("unknown").is_none());
    }

    #[test]
    fn test_timing_constants() {
        assert_eq!(timing::FRAME_DURATION.as_millis(), 20);
        assert_eq!(timing::FRAMES_PER_SECOND, 50);
        assert_eq!(timing::TIMESTAMP_INCREMENT, 960);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let header = RtpHeader::new(12345);
        let header_bytes = header.to_bytes();
        let opus_data = vec![0xF8, 0xFF, 0xFE, 0x00, 0x00]; // silence frame
        let secret_key = [0x42u8; 32];

        let encrypted = encrypt_xsalsa20(&header_bytes, &opus_data, &secret_key).unwrap();
        assert_ne!(encrypted, opus_data); // Should be different (encrypted)
        assert_eq!(encrypted.len(), opus_data.len() + 16); // 16 byte MAC

        let decrypted = decrypt_xsalsa20(&header_bytes, &encrypted, &secret_key).unwrap();
        assert_eq!(decrypted, opus_data);
    }

    #[test]
    fn test_encrypt_wrong_key_fails() {
        let header = RtpHeader::new(1);
        let header_bytes = header.to_bytes();
        let opus = vec![1, 2, 3, 4];
        let key1 = [0x11u8; 32];
        let key2 = [0x22u8; 32];

        let encrypted = encrypt_xsalsa20(&header_bytes, &opus, &key1).unwrap();
        let result = decrypt_xsalsa20(&header_bytes, &encrypted, &key2);
        assert!(result.is_err()); // Wrong key should fail
    }

    #[test]
    fn test_encrypt_lite_roundtrip() {
        let opus = vec![10, 20, 30, 40, 50];
        let key = [0x55u8; 32];
        let counter = 42u32;

        let (encrypted, counter_bytes) = encrypt_xsalsa20_lite(&opus, &key, counter).unwrap();
        assert_eq!(counter_bytes, 42u32.to_be_bytes());

        // Decrypt
        let nonce_bytes = build_lite_nonce(counter);
        let cipher = XSalsa20Poly1305::new_from_slice(&key).unwrap();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let decrypted = cipher.decrypt(nonce, encrypted.as_slice()).unwrap();
        assert_eq!(decrypted, opus);
    }

    #[test]
    fn test_build_encrypted_packet() {
        let header = RtpHeader::new(99);
        let opus = vec![0xF8, 0xFF, 0xFE];
        let key = [0xAA; 32];

        let packet =
            build_encrypted_packet(&header, &opus, &key, &EncryptionMode::XSalsa20Poly1305, 0)
                .unwrap();

        // Should be: 12 (RTP) + 3 (opus) + 16 (MAC) = 31
        assert_eq!(packet.len(), 31);
        // First 2 bytes are RTP header
        assert_eq!(&packet[..2], &[0x80, 0x78]);
    }

    #[test]
    fn test_build_encrypted_packet_lite() {
        let header = RtpHeader::new(99);
        let opus = vec![0xF8, 0xFF, 0xFE];
        let key = [0xBB; 32];

        let packet = build_encrypted_packet(
            &header,
            &opus,
            &key,
            &EncryptionMode::XSalsa20Poly1305Lite,
            7,
        )
        .unwrap();

        // Should be: 12 (RTP) + 3+16 (encrypted) + 4 (counter) = 35
        assert_eq!(packet.len(), 35);
        // Last 4 bytes are the counter
        assert_eq!(&packet[31..], &7u32.to_be_bytes());
    }
}
