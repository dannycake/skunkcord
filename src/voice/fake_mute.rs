// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Fake Mute/Deafen feature
//!
//! Allows appearing muted/deafened to other users while still receiving
//! (and optionally recording) audio.
//!
//! ⚠️ HIGH RISK: Discord's server can see that self_mute is true in the
//! gateway VoiceStateUpdate while UDP audio packets are still flowing.
//! This is a detectable state contradiction.
//!
//! How it works:
//! 1. Send VoiceStateUpdate with self_mute: true (appears muted to others)
//! 2. Keep the UDP audio connection alive
//! 3. Continue receiving and decrypting incoming audio
//! 4. Optionally continue sending silence frames (keeps connection alive)
//! 5. Optionally record incoming audio to file

use serde::{Deserialize, Serialize};

/// Fake mute/deafen state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FakeMuteState {
    /// Show as muted to others (but still connected to voice UDP)
    pub fake_mute: bool,
    /// Show as deafened to others (but still receive audio)
    pub fake_deafen: bool,
    /// Actually muted (no audio sent or received)
    pub real_mute: bool,
    /// Actually deafened (no audio received)
    pub real_deafen: bool,
    /// Record incoming audio while fake muted/deafened
    pub record_audio: bool,
    /// Send silence frames to keep UDP connection alive while fake muted
    pub send_silence: bool,
}

impl Default for FakeMuteState {
    fn default() -> Self {
        Self {
            fake_mute: false,
            fake_deafen: false,
            real_mute: false,
            real_deafen: false,
            record_audio: false,
            send_silence: true, // Default to sending silence to keep connection alive
        }
    }
}

impl FakeMuteState {
    /// Get the self_mute value to send in VoiceStateUpdate
    /// Returns true if either really muted or fake muted
    pub fn gateway_self_mute(&self) -> bool {
        self.real_mute || self.fake_mute
    }

    /// Get the self_deaf value to send in VoiceStateUpdate
    /// Returns true if either really deafened or fake deafened
    pub fn gateway_self_deaf(&self) -> bool {
        self.real_deafen || self.fake_deafen
    }

    /// Should we actually send audio data?
    /// Only if not really muted (fake mute still allows sending if desired)
    pub fn should_send_audio(&self) -> bool {
        !self.real_mute
    }

    /// Should we actually receive/process audio data?
    /// Only if not really deafened (fake deafen still receives)
    pub fn should_receive_audio(&self) -> bool {
        !self.real_deafen
    }

    /// Should we send silence frames to keep the UDP connection alive?
    pub fn should_send_silence(&self) -> bool {
        self.send_silence && (self.fake_mute || self.real_mute)
    }

    /// Should we record incoming audio?
    pub fn should_record(&self) -> bool {
        self.record_audio && (self.fake_mute || self.fake_deafen)
    }

    /// Toggle fake mute
    pub fn toggle_fake_mute(&mut self) {
        self.fake_mute = !self.fake_mute;
        // Can't be fake muted and really muted at the same time
        if self.fake_mute {
            self.real_mute = false;
        }
    }

    /// Toggle fake deafen
    pub fn toggle_fake_deafen(&mut self) {
        self.fake_deafen = !self.fake_deafen;
        if self.fake_deafen {
            self.real_deafen = false;
            // Fake deafen implies fake mute
            self.fake_mute = true;
            self.real_mute = false;
        }
    }

    /// Toggle real mute
    pub fn toggle_real_mute(&mut self) {
        self.real_mute = !self.real_mute;
        if self.real_mute {
            self.fake_mute = false;
        }
    }

    /// Toggle real deafen
    pub fn toggle_real_deafen(&mut self) {
        self.real_deafen = !self.real_deafen;
        if self.real_deafen {
            self.fake_deafen = false;
            self.real_mute = true;
            self.fake_mute = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = FakeMuteState::default();
        assert!(!state.fake_mute);
        assert!(!state.fake_deafen);
        assert!(!state.real_mute);
        assert!(!state.real_deafen);
        assert!(!state.gateway_self_mute());
        assert!(!state.gateway_self_deaf());
        assert!(state.should_send_audio());
        assert!(state.should_receive_audio());
    }

    #[test]
    fn test_fake_mute() {
        let mut state = FakeMuteState::default();
        state.toggle_fake_mute();

        assert!(state.fake_mute);
        assert!(state.gateway_self_mute()); // Appears muted
        assert!(!state.gateway_self_deaf());
        assert!(state.should_send_audio()); // But can still send
        assert!(state.should_receive_audio()); // And receive
        assert!(state.should_send_silence()); // Sends silence frames
    }

    #[test]
    fn test_fake_deafen() {
        let mut state = FakeMuteState::default();
        state.toggle_fake_deafen();

        assert!(state.fake_deafen);
        assert!(state.fake_mute); // Deafen implies mute
        assert!(state.gateway_self_mute());
        assert!(state.gateway_self_deaf());
        assert!(state.should_send_audio()); // Can still send if wanted
        assert!(state.should_receive_audio()); // Still receives!
    }

    #[test]
    fn test_real_mute_overrides_fake() {
        let mut state = FakeMuteState::default();
        state.fake_mute = true;
        state.toggle_real_mute();

        assert!(state.real_mute);
        assert!(!state.fake_mute); // Real mute clears fake
        assert!(state.gateway_self_mute());
        assert!(!state.should_send_audio()); // Actually can't send
    }

    #[test]
    fn test_real_deafen_overrides_fake() {
        let mut state = FakeMuteState::default();
        state.fake_deafen = true;
        state.toggle_real_deafen();

        assert!(state.real_deafen);
        assert!(!state.fake_deafen);
        assert!(state.gateway_self_deaf());
        assert!(!state.should_receive_audio()); // Actually can't receive
    }

    #[test]
    fn test_recording() {
        let mut state = FakeMuteState::default();
        state.fake_mute = true;
        state.record_audio = true;

        assert!(state.should_record());

        state.fake_mute = false;
        assert!(!state.should_record()); // Only records during fake mute/deafen
    }
}
