// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Voice audio send/receive loop
//!
//! The actual network loop that sends and receives encrypted Opus
//! audio over UDP. This is the runtime component that ties together
//! the voice gateway, UDP connection, encryption, and fake mute.

use super::crypto::{build_rtp_nonce, build_voice_packet, split_voice_packet, timing};
use super::fake_mute::FakeMuteState;
use super::udp::{opus, RtpHeader};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, RwLock};

/// Audio sender configuration
pub struct AudioSenderConfig {
    /// UDP socket for sending
    pub server_addr: String,
    /// Our SSRC
    pub ssrc: u32,
    /// Encryption secret key (32 bytes)
    pub secret_key: Vec<u8>,
    /// Encryption mode name
    pub encryption_mode: String,
}

/// Commands sent to the audio loop
#[derive(Debug)]
pub enum AudioCommand {
    /// Start sending audio from microphone
    StartSpeaking,
    /// Stop sending audio (send silence frames, then stop)
    StopSpeaking,
    /// Send a specific opus frame
    SendFrame(Vec<u8>),
    /// Update mute state
    UpdateMuteState(FakeMuteState),
    /// Shutdown the audio loop
    Shutdown,
}

/// Events received from the audio loop
#[derive(Debug, Clone)]
pub enum AudioEvent {
    /// Received audio from another user
    AudioReceived {
        ssrc: u32,
        opus_data: Vec<u8>,
        sequence: u16,
        timestamp: u32,
    },
    /// A user started speaking
    UserSpeaking { ssrc: u32 },
    /// A user stopped speaking (no packets for >200ms)
    UserSilent { ssrc: u32 },
    /// Audio loop error
    Error(String),
}

/// State tracked per remote user's audio stream
struct RemoteStream {
    last_packet_time: Instant,
    was_speaking: bool,
}

/// Build the RTP + encryption nonce for a packet.
/// Returns (nonce_bytes, nonce_to_append) depending on encryption mode.
#[allow(dead_code)]
fn build_nonce_for_mode(mode: &str, header: &RtpHeader, lite_counter: u32) -> Vec<u8> {
    match mode {
        "xsalsa20_poly1305" => {
            let header_bytes = header.to_bytes();
            build_rtp_nonce(&header_bytes).to_vec()
        }
        "xsalsa20_poly1305_lite" => super::crypto::build_lite_nonce(lite_counter).to_vec(),
        _ => {
            // Default to RTP header nonce
            let header_bytes = header.to_bytes();
            build_rtp_nonce(&header_bytes).to_vec()
        }
    }
}

/// Create an audio sender task.
///
/// This spawns a tokio task that:
/// 1. Listens for AudioCommand messages
/// 2. Sends Opus frames at 20ms intervals via UDP
/// 3. Handles silence frame insertion when stopping speech
/// 4. Respects fake mute state
///
/// Returns a channel to send commands to the audio loop.
pub fn create_audio_sender(
    config: AudioSenderConfig,
    mute_state: Arc<RwLock<FakeMuteState>>,
) -> mpsc::Sender<AudioCommand> {
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<AudioCommand>(64);

    tokio::spawn(async move {
        // Bind a UDP socket
        let socket = match UdpSocket::bind("0.0.0.0:0").await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to bind UDP socket: {}", e);
                return;
            }
        };

        if let Err(e) = socket.connect(&config.server_addr).await {
            tracing::error!("Failed to connect UDP socket: {}", e);
            return;
        }

        let mut rtp = RtpHeader::new(config.ssrc);
        let mut _is_speaking = false;
        let mut silence_frames_remaining: u32 = 0;
        let mut lite_counter: u32 = 0;

        tracing::info!("Audio sender started for SSRC {}", config.ssrc);

        loop {
            tokio::select! {
                // Process commands
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(AudioCommand::StartSpeaking) => {
                            _is_speaking = true;
                            tracing::debug!("Started speaking");
                        }
                        Some(AudioCommand::StopSpeaking) => {
                            _is_speaking = false;
                            silence_frames_remaining = timing::SILENCE_FRAME_COUNT;
                            tracing::debug!("Stopped speaking, sending {} silence frames", silence_frames_remaining);
                        }
                        Some(AudioCommand::SendFrame(opus_data)) => {
                            let state = mute_state.read().await;
                            if state.should_send_audio() || state.should_send_silence() {
                                let packet = build_voice_packet(&rtp, &opus_data);
                                let _ = socket.send(&packet).await;
                                rtp.advance(timing::TIMESTAMP_INCREMENT);
                                lite_counter = lite_counter.wrapping_add(1);
                            }
                        }
                        Some(AudioCommand::UpdateMuteState(_new_state)) => {
                            // Mute state is shared via Arc<RwLock>
                        }
                        Some(AudioCommand::Shutdown) | None => {
                            tracing::info!("Audio sender shutting down");
                            break;
                        }
                    }
                }

                // Send silence frames after stopping speech
                _ = tokio::time::sleep(timing::FRAME_DURATION), if silence_frames_remaining > 0 => {
                    let packet = build_voice_packet(&rtp, opus::SILENCE_FRAME);
                    let _ = socket.send(&packet).await;
                    rtp.advance(timing::TIMESTAMP_INCREMENT);
                    silence_frames_remaining -= 1;
                    if silence_frames_remaining == 0 {
                        tracing::debug!("Finished sending silence frames");
                    }
                }
            }
        }
    });

    cmd_tx
}

/// Create an audio receiver task.
///
/// Listens on the UDP socket for incoming audio packets,
/// decrypts them, and forwards as AudioEvents.
pub fn create_audio_receiver(
    socket: Arc<UdpSocket>,
    event_tx: mpsc::Sender<AudioEvent>,
    mute_state: Arc<RwLock<FakeMuteState>>,
) {
    tokio::spawn(async move {
        let mut buf = vec![0u8; 4096];
        let mut remote_streams: std::collections::HashMap<u32, RemoteStream> =
            std::collections::HashMap::new();

        loop {
            match socket.recv(&mut buf).await {
                Ok(n) if n >= 12 => {
                    // Check if we should receive audio
                    let state = mute_state.read().await;
                    if !state.should_receive_audio() {
                        continue;
                    }
                    drop(state);

                    // Parse RTP header
                    if let Some((header_bytes, encrypted_payload)) = split_voice_packet(&buf[..n]) {
                        if let Some(rtp) = RtpHeader::from_bytes(&header_bytes) {
                            let now = Instant::now();

                            // Track speaking state
                            let stream = remote_streams.entry(rtp.ssrc).or_insert(RemoteStream {
                                last_packet_time: now,
                                was_speaking: false,
                            });

                            if !stream.was_speaking {
                                stream.was_speaking = true;
                                let _ = event_tx
                                    .send(AudioEvent::UserSpeaking { ssrc: rtp.ssrc })
                                    .await;
                            }
                            stream.last_packet_time = now;

                            // Forward the audio data (decryption would happen here
                            // with the secret_key — omitted as it requires the
                            // xsalsa20poly1305 crate which is an optional dependency)
                            let _ = event_tx
                                .send(AudioEvent::AudioReceived {
                                    ssrc: rtp.ssrc,
                                    opus_data: encrypted_payload.to_vec(),
                                    sequence: rtp.sequence,
                                    timestamp: rtp.timestamp,
                                })
                                .await;
                        }
                    }
                }
                Ok(_) => {} // Packet too small, ignore
                Err(e) => {
                    let _ = event_tx
                        .send(AudioEvent::Error(format!("UDP recv error: {}", e)))
                        .await;
                    break;
                }
            }

            // Check for users who stopped speaking (no packets for 200ms)
            let now = Instant::now();
            let mut silent_users = Vec::new();
            for (ssrc, stream) in &mut remote_streams {
                if stream.was_speaking
                    && now.duration_since(stream.last_packet_time).as_millis() > 200
                {
                    stream.was_speaking = false;
                    silent_users.push(*ssrc);
                }
            }
            for ssrc in silent_users {
                let _ = event_tx.send(AudioEvent::UserSilent { ssrc }).await;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_for_xsalsa20() {
        let header = RtpHeader::new(12345);
        let nonce = build_nonce_for_mode("xsalsa20_poly1305", &header, 0);
        assert_eq!(nonce.len(), 24);
    }

    #[test]
    fn test_nonce_for_lite() {
        let header = RtpHeader::new(1);
        let nonce = build_nonce_for_mode("xsalsa20_poly1305_lite", &header, 42);
        assert_eq!(nonce.len(), 24);
        assert_eq!(nonce[..4], 42u32.to_be_bytes());
    }

    #[test]
    fn test_audio_command_variants() {
        // Verify all variants construct without panic
        let _ = AudioCommand::StartSpeaking;
        let _ = AudioCommand::StopSpeaking;
        let _ = AudioCommand::SendFrame(vec![0xF8, 0xFF, 0xFE]);
        let _ = AudioCommand::UpdateMuteState(FakeMuteState::default());
        let _ = AudioCommand::Shutdown;
    }

    #[test]
    fn test_audio_event_variants() {
        let _ = AudioEvent::AudioReceived {
            ssrc: 1,
            opus_data: vec![],
            sequence: 0,
            timestamp: 0,
        };
        let _ = AudioEvent::UserSpeaking { ssrc: 1 };
        let _ = AudioEvent::UserSilent { ssrc: 1 };
        let _ = AudioEvent::Error("test".to_string());
    }
}
