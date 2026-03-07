// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Voice connection manager
//!
//! Orchestrates the full voice connection lifecycle:
//! 1. Send VoiceStateUpdate on main gateway (join channel)
//! 2. Wait for VOICE_STATE_UPDATE + VOICE_SERVER_UPDATE events
//! 3. Connect to voice gateway WebSocket
//! 4. Identify, receive Ready (SSRC, modes)
//! 5. IP discovery via UDP
//! 6. Select protocol, receive session description (secret key)
//! 7. Send/receive encrypted Opus audio via UDP

use super::fake_mute::FakeMuteState;
use super::gateway::*;
use super::udp::RtpHeader;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Current state of a voice connection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceConnectionState {
    /// Not connected to any voice channel
    Disconnected,
    /// Waiting for VOICE_STATE_UPDATE and VOICE_SERVER_UPDATE from main gateway
    WaitingForEvents,
    /// Connecting to voice gateway WebSocket
    ConnectingGateway,
    /// Connected to voice gateway, performing handshake
    Handshaking,
    /// Performing UDP IP discovery
    Discovering,
    /// Selecting encryption protocol
    SelectingProtocol,
    /// Fully connected and ready for audio
    Connected,
    /// Connection failed
    Failed(String),
}

/// A managed voice connection
pub struct VoiceConnection {
    /// Current connection state
    pub state: Arc<RwLock<VoiceConnectionState>>,
    /// Guild ID (None for DM calls)
    pub guild_id: Option<String>,
    /// Channel ID
    pub channel_id: String,
    /// Our user ID
    pub user_id: String,
    /// Session ID from VOICE_STATE_UPDATE
    pub session_id: Arc<RwLock<Option<String>>>,
    /// Voice server token from VOICE_SERVER_UPDATE
    pub server_token: Arc<RwLock<Option<String>>>,
    /// Voice server endpoint from VOICE_SERVER_UPDATE
    pub server_endpoint: Arc<RwLock<Option<String>>>,
    /// Our SSRC (from voice gateway Ready)
    pub ssrc: Arc<RwLock<Option<u32>>>,
    /// Selected encryption mode
    pub encryption_mode: Arc<RwLock<Option<String>>>,
    /// Encryption secret key (from SessionDescription)
    pub secret_key: Arc<RwLock<Option<Vec<u8>>>>,
    /// Our external IP (from IP discovery)
    pub external_ip: Arc<RwLock<Option<String>>>,
    /// Our external port (from IP discovery)
    pub external_port: Arc<RwLock<Option<u16>>>,
    /// RTP header for outgoing packets
    pub rtp_header: Arc<RwLock<Option<RtpHeader>>>,
    /// SSRC to user mapping
    pub ssrc_map: Arc<RwLock<SsrcUserMap>>,
    /// Fake mute/deafen state
    pub mute_state: Arc<RwLock<FakeMuteState>>,
    /// Whether we're currently speaking
    pub is_speaking: Arc<RwLock<bool>>,
}

impl VoiceConnection {
    /// Create a new voice connection (not yet connected)
    pub fn new(guild_id: Option<String>, channel_id: String, user_id: String) -> Self {
        Self {
            state: Arc::new(RwLock::new(VoiceConnectionState::Disconnected)),
            guild_id,
            channel_id,
            user_id,
            session_id: Arc::new(RwLock::new(None)),
            server_token: Arc::new(RwLock::new(None)),
            server_endpoint: Arc::new(RwLock::new(None)),
            ssrc: Arc::new(RwLock::new(None)),
            encryption_mode: Arc::new(RwLock::new(None)),
            secret_key: Arc::new(RwLock::new(None)),
            external_ip: Arc::new(RwLock::new(None)),
            external_port: Arc::new(RwLock::new(None)),
            rtp_header: Arc::new(RwLock::new(None)),
            ssrc_map: Arc::new(RwLock::new(SsrcUserMap::new())),
            mute_state: Arc::new(RwLock::new(FakeMuteState::default())),
            is_speaking: Arc::new(RwLock::new(false)),
        }
    }

    /// Handle VOICE_STATE_UPDATE event from main gateway
    pub async fn on_voice_state_update(&self, session_id: &str) {
        *self.session_id.write().await = Some(session_id.to_string());
        self.check_ready_to_connect().await;
    }

    /// Handle VOICE_SERVER_UPDATE event from main gateway
    pub async fn on_voice_server_update(&self, token: &str, endpoint: Option<&str>) {
        *self.server_token.write().await = Some(token.to_string());
        if let Some(ep) = endpoint {
            *self.server_endpoint.write().await = Some(ep.to_string());
        }
        self.check_ready_to_connect().await;
    }

    /// Check if we have both events needed to connect to voice gateway
    async fn check_ready_to_connect(&self) {
        let has_session = self.session_id.read().await.is_some();
        let has_token = self.server_token.read().await.is_some();
        let has_endpoint = self.server_endpoint.read().await.is_some();

        if has_session && has_token && has_endpoint {
            *self.state.write().await = VoiceConnectionState::ConnectingGateway;
            tracing::info!("Voice: ready to connect to gateway");
        }
    }

    /// Handle voice gateway Ready event
    pub async fn on_voice_ready(&self, ready: &VoiceReady) {
        *self.ssrc.write().await = Some(ready.ssrc);
        *self.rtp_header.write().await = Some(RtpHeader::new(ready.ssrc));

        // Select best encryption mode
        if let Some(mode) = select_encryption_mode(&ready.modes) {
            *self.encryption_mode.write().await = Some(mode.clone());
            tracing::info!("Voice: selected encryption mode: {}", mode);
        }

        *self.state.write().await = VoiceConnectionState::Discovering;
    }

    /// Handle IP discovery result
    pub async fn on_ip_discovered(&self, ip: String, port: u16) {
        *self.external_ip.write().await = Some(ip.clone());
        *self.external_port.write().await = Some(port);
        *self.state.write().await = VoiceConnectionState::SelectingProtocol;
        tracing::info!("Voice: discovered external {}:{}", ip, port);
    }

    /// Handle session description (encryption key received)
    pub async fn on_session_description(&self, desc: &VoiceSessionDescription) {
        *self.secret_key.write().await = Some(desc.secret_key.clone());
        *self.state.write().await = VoiceConnectionState::Connected;
        tracing::info!("Voice: fully connected with mode {}", desc.mode);
    }

    /// Handle a client connecting to voice
    pub async fn on_client_connect(&self, user_id: &str, audio_ssrc: Option<u32>) {
        if let Some(ssrc) = audio_ssrc {
            self.ssrc_map
                .write()
                .await
                .register(ssrc, user_id.to_string());
        }
    }

    /// Handle a client disconnecting from voice
    pub async fn on_client_disconnect(&self, user_id: &str) {
        self.ssrc_map.write().await.remove_user(user_id);
    }

    /// Perform UDP IP discovery against the voice server.
    ///
    /// Sends our SSRC to the voice server's UDP endpoint and receives
    /// our external IP and port back. This is needed for Select Protocol.
    pub async fn perform_ip_discovery(
        &self,
        server_ip: &str,
        server_port: u16,
    ) -> Result<(String, u16), String> {
        let ssrc = self
            .ssrc
            .read()
            .await
            .ok_or_else(|| "No SSRC available".to_string())?;

        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("UDP bind failed: {}", e))?;

        let addr = format!("{}:{}", server_ip, server_port);
        socket
            .connect(&addr)
            .await
            .map_err(|e| format!("UDP connect to {} failed: {}", addr, e))?;

        // Send IP discovery packet
        let discovery_packet = super::udp::build_ip_discovery_packet(ssrc);
        socket
            .send(&discovery_packet)
            .await
            .map_err(|e| format!("UDP send failed: {}", e))?;

        // Receive response (with timeout)
        let mut buf = vec![0u8; 74];
        let recv_result =
            tokio::time::timeout(std::time::Duration::from_secs(5), socket.recv(&mut buf))
                .await
                .map_err(|_| "IP discovery timed out (5s)".to_string())?
                .map_err(|e| format!("UDP recv failed: {}", e))?;

        if recv_result < 74 {
            return Err(format!(
                "IP discovery response too short: {} bytes",
                recv_result
            ));
        }

        super::udp::parse_ip_discovery_response(&buf)
            .ok_or_else(|| "Failed to parse IP discovery response".to_string())
    }

    /// Check if fully connected and ready for audio
    pub async fn is_connected(&self) -> bool {
        *self.state.read().await == VoiceConnectionState::Connected
    }

    /// Get the VoiceStateUpdate payload for the main gateway
    pub fn voice_state_update_payload(&self) -> serde_json::Value {
        let mute = tokio::runtime::Handle::current()
            .block_on(async { self.mute_state.read().await.clone() });

        serde_json::json!({
            "guild_id": self.guild_id,
            "channel_id": self.channel_id,
            "self_mute": mute.gateway_self_mute(),
            "self_deaf": mute.gateway_self_deaf(),
        })
    }

    /// Disconnect from voice
    pub async fn disconnect(&self) {
        *self.state.write().await = VoiceConnectionState::Disconnected;
        *self.session_id.write().await = None;
        *self.server_token.write().await = None;
        *self.server_endpoint.write().await = None;
        *self.ssrc.write().await = None;
        *self.secret_key.write().await = None;
        *self.external_ip.write().await = None;
        *self.external_port.write().await = None;
        *self.rtp_header.write().await = None;
        *self.ssrc_map.write().await = SsrcUserMap::new();
        *self.is_speaking.write().await = false;
        tracing::info!("Voice: disconnected");
    }

    /// Get connection info summary
    pub async fn info(&self) -> VoiceConnectionInfo {
        VoiceConnectionInfo {
            guild_id: self.guild_id.clone(),
            channel_id: self.channel_id.clone(),
            session_id: self.session_id.read().await.clone().unwrap_or_default(),
            token: self.server_token.read().await.clone().unwrap_or_default(),
            endpoint: self
                .server_endpoint
                .read()
                .await
                .clone()
                .unwrap_or_default(),
            user_id: self.user_id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_connection_is_disconnected() {
        let conn = VoiceConnection::new(Some("g1".into()), "ch1".into(), "u1".into());
        assert_eq!(*conn.state.read().await, VoiceConnectionState::Disconnected);
        assert!(!conn.is_connected().await);
    }

    #[tokio::test]
    async fn test_voice_state_update_triggers_connect_check() {
        let conn = VoiceConnection::new(Some("g1".into()), "ch1".into(), "u1".into());
        // Only session — not enough
        conn.on_voice_state_update("sess123").await;
        assert_eq!(*conn.state.read().await, VoiceConnectionState::Disconnected);

        // Add token + endpoint — now ready
        *conn.server_token.write().await = Some("tok".into());
        conn.on_voice_server_update("tok", Some("endpoint.discord.gg"))
            .await;
        assert_eq!(
            *conn.state.read().await,
            VoiceConnectionState::ConnectingGateway
        );
    }

    #[tokio::test]
    async fn test_voice_ready_selects_encryption() {
        let conn = VoiceConnection::new(Some("g1".into()), "ch1".into(), "u1".into());
        let ready = VoiceReady {
            ssrc: 12345,
            ip: "1.2.3.4".into(),
            port: 5000,
            modes: vec![
                "xsalsa20_poly1305".into(),
                "aead_xchacha20_poly1305_rtpsize".into(),
            ],
            experiments: vec![],
        };
        conn.on_voice_ready(&ready).await;

        assert_eq!(*conn.ssrc.read().await, Some(12345));
        assert_eq!(
            *conn.encryption_mode.read().await,
            Some("aead_xchacha20_poly1305_rtpsize".into())
        );
        assert_eq!(*conn.state.read().await, VoiceConnectionState::Discovering);
    }

    #[tokio::test]
    async fn test_full_connection_flow() {
        let conn = VoiceConnection::new(Some("g1".into()), "ch1".into(), "u1".into());

        // 1. Receive events
        conn.on_voice_state_update("sess").await;
        conn.on_voice_server_update("tok", Some("ep.gg")).await;
        assert_eq!(
            *conn.state.read().await,
            VoiceConnectionState::ConnectingGateway
        );

        // 2. Voice ready
        conn.on_voice_ready(&VoiceReady {
            ssrc: 1,
            ip: "1.1.1.1".into(),
            port: 80,
            modes: vec!["xsalsa20_poly1305".into()],
            experiments: vec![],
        })
        .await;
        assert_eq!(*conn.state.read().await, VoiceConnectionState::Discovering);

        // 3. IP discovered
        conn.on_ip_discovered("2.2.2.2".into(), 9999).await;
        assert_eq!(
            *conn.state.read().await,
            VoiceConnectionState::SelectingProtocol
        );

        // 4. Session description
        conn.on_session_description(&VoiceSessionDescription {
            mode: "xsalsa20_poly1305".into(),
            secret_key: vec![0u8; 32],
        })
        .await;
        assert!(conn.is_connected().await);

        // 5. Disconnect
        conn.disconnect().await;
        assert!(!conn.is_connected().await);
    }

    #[tokio::test]
    async fn test_client_connect_disconnect() {
        let conn = VoiceConnection::new(None, "ch1".into(), "u1".into());
        conn.on_client_connect("u2", Some(999)).await;
        assert_eq!(conn.ssrc_map.read().await.get_user(999), Some("u2"));

        conn.on_client_disconnect("u2").await;
        assert_eq!(conn.ssrc_map.read().await.get_user(999), None);
    }
}
