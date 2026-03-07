// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Voice Gateway WebSocket connection
//!
//! Handles the WebSocket connection to Discord's voice server for
//! setting up audio channels. This is separate from the main gateway.

use serde::{Deserialize, Serialize};

/// Voice Gateway connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceGatewayState {
    Disconnected,
    Connecting,
    Identifying,
    Ready,
    SessionReady,
    Closed,
}

/// Voice connection info received from the main gateway
#[derive(Debug, Clone)]
pub struct VoiceConnectionInfo {
    /// Guild ID (None for DM calls)
    pub guild_id: Option<String>,
    /// Channel ID
    pub channel_id: String,
    /// Voice session ID (from VOICE_STATE_UPDATE)
    pub session_id: String,
    /// Voice server token (from VOICE_SERVER_UPDATE)
    pub token: String,
    /// Voice server endpoint (from VOICE_SERVER_UPDATE)
    pub endpoint: String,
    /// Our user ID
    pub user_id: String,
}

/// Voice Gateway opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VoiceOpCode {
    /// Start a new voice connection
    Identify = 0,
    /// Select the voice protocol and mode
    SelectProtocol = 1,
    /// Connection ready (SSRC, IP, port, modes)
    Ready = 2,
    /// Keep connection alive
    Heartbeat = 3,
    /// Session description (encryption key)
    SessionDescription = 4,
    /// Indicate speaking state
    Speaking = 5,
    /// Heartbeat ACK
    HeartbeatAck = 6,
    /// Resume connection
    Resume = 7,
    /// Hello (heartbeat interval)
    Hello = 8,
    /// Resume acknowledged
    Resumed = 9,
    /// Client connected to voice
    ClientConnect = 12,
    /// Client disconnected from voice
    ClientDisconnect = 13,
    /// Session update (codecs)
    SessionUpdate = 14,
}

/// Voice Identify payload
#[derive(Debug, Clone, Serialize)]
pub struct VoiceIdentify {
    pub server_id: String,
    pub user_id: String,
    pub session_id: String,
    pub token: String,
}

/// Voice Ready event data
#[derive(Debug, Clone, Deserialize)]
pub struct VoiceReady {
    pub ssrc: u32,
    pub ip: String,
    pub port: u16,
    pub modes: Vec<String>,
    #[serde(default)]
    pub experiments: Vec<String>,
}

/// Voice Select Protocol payload
#[derive(Debug, Clone, Serialize)]
pub struct VoiceSelectProtocol {
    pub protocol: String,
    pub data: VoiceProtocolData,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceProtocolData {
    pub address: String,
    pub port: u16,
    pub mode: String,
}

/// Voice Session Description (received after protocol selection)
#[derive(Debug, Clone, Deserialize)]
pub struct VoiceSessionDescription {
    pub mode: String,
    pub secret_key: Vec<u8>,
}

/// Voice Speaking payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSpeaking {
    pub speaking: u32,
    pub delay: u32,
    pub ssrc: u32,
}

/// Speaking flags
#[derive(Debug, Clone, Copy)]
pub enum SpeakingFlag {
    /// Normal voice audio
    Microphone = 1 << 0,
    /// Audio from screen share / soundboard
    Soundshare = 1 << 1,
    /// Priority speaker
    Priority = 1 << 2,
}

/// Client connect event
#[derive(Debug, Clone, Deserialize)]
pub struct VoiceClientConnect {
    pub user_id: String,
    #[serde(default)]
    pub audio_ssrc: Option<u32>,
    #[serde(default)]
    pub video_ssrc: Option<u32>,
}

/// Client disconnect event
#[derive(Debug, Clone, Deserialize)]
pub struct VoiceClientDisconnect {
    pub user_id: String,
}

/// Voice Hello payload
#[derive(Debug, Clone, Deserialize)]
pub struct VoiceHello {
    pub heartbeat_interval: f64,
}

/// Preferred encryption modes (in order of preference)
pub const PREFERRED_ENCRYPTION_MODES: &[&str] = &[
    "aead_xchacha20_poly1305_rtpsize",
    "aead_aes256_gcm_rtpsize",
    "xsalsa20_poly1305_lite",
    "xsalsa20_poly1305_suffix",
    "xsalsa20_poly1305",
];

/// Select the best encryption mode from the server's supported modes
pub fn select_encryption_mode(server_modes: &[String]) -> Option<String> {
    for preferred in PREFERRED_ENCRYPTION_MODES {
        if server_modes.iter().any(|m| m == preferred) {
            return Some(preferred.to_string());
        }
    }
    // Fallback: use first available
    server_modes.first().cloned()
}

/// SSRC to user mapping for identifying who is speaking
#[derive(Debug, Clone, Default)]
pub struct SsrcUserMap {
    map: std::collections::HashMap<u32, String>,
}

impl SsrcUserMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a user's SSRC
    pub fn register(&mut self, ssrc: u32, user_id: String) {
        self.map.insert(ssrc, user_id);
    }

    /// Look up user by SSRC
    pub fn get_user(&self, ssrc: u32) -> Option<&str> {
        self.map.get(&ssrc).map(|s| s.as_str())
    }

    /// Remove a user's mapping
    pub fn remove_user(&mut self, user_id: &str) {
        self.map.retain(|_, v| v != user_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_encryption_mode() {
        // Server supports our preferred mode
        let modes = vec![
            "xsalsa20_poly1305".to_string(),
            "aead_xchacha20_poly1305_rtpsize".to_string(),
        ];
        assert_eq!(
            select_encryption_mode(&modes),
            Some("aead_xchacha20_poly1305_rtpsize".to_string())
        );

        // Server only supports older mode
        let modes = vec!["xsalsa20_poly1305".to_string()];
        assert_eq!(
            select_encryption_mode(&modes),
            Some("xsalsa20_poly1305".to_string())
        );

        // Empty modes
        assert_eq!(select_encryption_mode(&[]), None);
    }

    #[test]
    fn test_ssrc_user_map() {
        let mut map = SsrcUserMap::new();
        map.register(12345, "user1".to_string());
        map.register(67890, "user2".to_string());

        assert_eq!(map.get_user(12345), Some("user1"));
        assert_eq!(map.get_user(67890), Some("user2"));
        assert_eq!(map.get_user(99999), None);

        map.remove_user("user1");
        assert_eq!(map.get_user(12345), None);
    }

    #[test]
    fn test_voice_opcodes() {
        assert_eq!(VoiceOpCode::Identify as u8, 0);
        assert_eq!(VoiceOpCode::Ready as u8, 2);
        assert_eq!(VoiceOpCode::Speaking as u8, 5);
        assert_eq!(VoiceOpCode::Hello as u8, 8);
    }
}
