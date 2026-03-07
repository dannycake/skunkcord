// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord Voice Chat Module
//!
//! Implements voice channel connections including:
//! - Voice Gateway WebSocket (separate from main gateway)
//! - UDP audio connection with encryption
//! - Opus codec encode/decode
//! - Speaking state management
//! - Fake mute/deafen features
//!
//! Voice connection flow:
//! 1. Send op 4 (VoiceStateUpdate) on main gateway to join a voice channel
//! 2. Receive VOICE_STATE_UPDATE event (our voice state with session_id)
//! 3. Receive VOICE_SERVER_UPDATE event (voice server token + endpoint)
//! 4. Connect to voice gateway WebSocket at the endpoint
//! 5. Send Identify with server_id, user_id, session_id, token
//! 6. Receive Ready (SSRC, IP, port, supported encryption modes)
//! 7. Perform IP Discovery via UDP
//! 8. Send Select Protocol with discovered IP/port and encryption mode
//! 9. Receive Session Description with encryption key
//! 10. Begin sending/receiving encrypted Opus audio via UDP

pub mod audio_loop;
pub mod connection;
pub mod crypto;
pub mod fake_mute;
pub mod gateway;
pub mod udp;
pub mod voice_ws;

pub use connection::*;
pub use crypto::*;
pub use fake_mute::*;
pub use gateway::*;
pub use udp::*;
