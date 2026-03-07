// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord Gateway WebSocket Module
//!
//! Handles the WebSocket connection to Discord's gateway for real-time events.

mod events;
pub mod health;
mod payloads;
pub mod session_limits;

pub use events::*;
pub use health::*;
pub use payloads::*;
pub use session_limits::*;

use crate::fingerprint::BrowserFingerprint;
use crate::{DiscordError, Result, GATEWAY_VERSION};
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use zlib_stream::{ZlibDecompressionError, ZlibStreamDecompressor};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

/// Discord Gateway URL
pub const GATEWAY_URL: &str = "wss://gateway.discord.gg";

/// Gateway connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayState {
    Disconnected,
    Connecting,
    Connected,
    Resuming,
    Reconnecting,
}

/// Gateway connection for Discord real-time events
pub struct Gateway {
    /// Connection state
    state: Arc<RwLock<GatewayState>>,
    /// Authentication token
    token: String,
    /// Browser fingerprint for identification
    fingerprint: BrowserFingerprint,
    /// Session ID for resuming
    session_id: Arc<RwLock<Option<String>>>,
    /// Resume gateway URL
    resume_url: Arc<RwLock<Option<String>>>,
    /// Last sequence number received
    sequence: Arc<RwLock<Option<u64>>>,
    /// Heartbeat interval in milliseconds
    heartbeat_interval: Arc<RwLock<Option<u64>>>,
    /// Event sender for broadcasting events
    event_tx: broadcast::Sender<GatewayEvent>,
    /// Command sender for sending commands to the gateway
    command_tx: Option<mpsc::Sender<GatewayCommand>>,
    /// Shared command sender that survives reconnects — cloneable for external use
    pub shared_cmd_tx: Arc<tokio::sync::Mutex<Option<mpsc::Sender<GatewayCommand>>>>,
    /// Last heartbeat acknowledgement time
    last_heartbeat_ack: Arc<RwLock<Option<Instant>>>,
}

impl Gateway {
    /// Create a new gateway connection
    pub fn new(token: String, fingerprint: BrowserFingerprint) -> Self {
        let (event_tx, _) = broadcast::channel(1024);

        Self {
            state: Arc::new(RwLock::new(GatewayState::Disconnected)),
            token,
            fingerprint,
            session_id: Arc::new(RwLock::new(None)),
            resume_url: Arc::new(RwLock::new(None)),
            sequence: Arc::new(RwLock::new(None)),
            heartbeat_interval: Arc::new(RwLock::new(None)),
            event_tx,
            command_tx: None,
            shared_cmd_tx: Arc::new(tokio::sync::Mutex::new(None)),
            last_heartbeat_ack: Arc::new(RwLock::new(None)),
        }
    }

    /// Subscribe to gateway events
    pub fn subscribe(&self) -> broadcast::Receiver<GatewayEvent> {
        self.event_tx.subscribe()
    }

    /// Get current connection state
    pub async fn state(&self) -> GatewayState {
        *self.state.read().await
    }

    /// Connect to the gateway (single attempt).
    /// If we have a valid session_id + sequence, attempts Resume (op 6).
    /// Otherwise does a fresh Identify (op 2).
    pub async fn connect(&mut self) -> Result<()> {
        *self.state.write().await = GatewayState::Connecting;

        // Use resume URL if we have one from a previous session, otherwise default
        let base_url = {
            let resume = self.resume_url.read().await;
            resume.clone().unwrap_or_else(|| GATEWAY_URL.to_string())
        };

        let url = format!(
            "{}/?v={}&encoding=json&compress=zlib-stream",
            base_url, GATEWAY_VERSION
        );

        // Check if we should resume
        let should_resume = {
            let sid = self.session_id.read().await;
            let seq = self.sequence.read().await;
            sid.is_some() && seq.is_some()
        };

        if should_resume {
            tracing::info!("Connecting to Gateway (will resume): {}", url);
        } else {
            tracing::info!("Connecting to Gateway (fresh identify): {}", url);
        }

        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| DiscordError::WebSocket(e.to_string()))?;

        *self.state.write().await = GatewayState::Connected;

        // Create command channel
        let (command_tx, command_rx) = mpsc::channel::<GatewayCommand>(100);
        self.command_tx = Some(command_tx.clone());
        // Update shared sender so external callers (e.g. voice) can reach the gateway
        *self.shared_cmd_tx.lock().await = Some(command_tx);

        // Start the connection handler (it will send Resume or Identify after Hello)
        self.run_connection(ws_stream, command_rx, should_resume)
            .await
    }

    /// Connect to the gateway with automatic reconnection.
    ///
    /// This will:
    /// 1. Connect to the gateway
    /// 2. If disconnected, wait with exponential backoff + jitter
    /// 3. Attempt to resume if we have a session_id and sequence
    /// 4. If resume fails (invalid session), do a fresh identify
    /// 5. Repeat until explicitly closed or a non-resumable error occurs
    pub async fn connect_with_reconnect(&mut self) -> Result<()> {
        let mut backoff_ms: u64 = 1000;
        const MAX_BACKOFF_MS: u64 = 60_000;
        const MAX_RECONNECT_ATTEMPTS: u32 = 50;
        const HEALTHY_CONNECTION_SECS: u64 = 30;

        for attempt in 0..MAX_RECONNECT_ATTEMPTS {
            let connected_at = Instant::now();

            match self.connect().await {
                Ok(()) => {
                    // Connection ended gracefully or was asked to reconnect
                    let state = *self.state.read().await;
                    match state {
                        GatewayState::Disconnected => {
                            // Explicitly disconnected (user called close)
                            tracing::info!("Gateway disconnected gracefully");
                            return Ok(());
                        }
                        GatewayState::Reconnecting => {
                            // Server asked us to reconnect or session invalidated
                            tracing::info!(
                                "Gateway reconnecting (attempt {}/{})",
                                attempt + 1,
                                MAX_RECONNECT_ATTEMPTS
                            );
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    tracing::warn!("Gateway connection error: {}", e);

                    // Check if this is a non-resumable error
                    if let DiscordError::WebSocket(ref msg) = e {
                        if msg.contains("4004")
                            || msg.contains("4010")
                            || msg.contains("4011")
                            || msg.contains("4012")
                            || msg.contains("4013")
                            || msg.contains("4014")
                        {
                            tracing::error!("Non-resumable gateway error: {}. Giving up.", msg);
                            return Err(e);
                        }
                    }
                }
            }

            // Reset backoff if the connection was healthy for > 30 seconds.
            // This prevents slow escalation when occasional reconnects happen
            // on an otherwise stable session.
            if connected_at.elapsed() > Duration::from_secs(HEALTHY_CONNECTION_SECS) {
                tracing::debug!(
                    "Connection lasted {:.0}s — resetting backoff",
                    connected_at.elapsed().as_secs_f64()
                );
                backoff_ms = 1000;
            }

            // Add jitter to backoff (±25%)
            let jitter = {
                let mut rng = rand::thread_rng();
                let range = (backoff_ms as f64 * 0.25) as u64;
                if range > 0 {
                    rng.gen_range(0..=range * 2) as i64 - range as i64
                } else {
                    0
                }
            };
            let wait = Duration::from_millis((backoff_ms as i64 + jitter).max(500) as u64);

            tracing::info!("Reconnecting in {:?}...", wait);
            tokio::time::sleep(wait).await;

            // Exponential backoff, capped
            backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
        }

        Err(DiscordError::Gateway(format!(
            "Max reconnection attempts ({}) exceeded",
            MAX_RECONNECT_ATTEMPTS
        )))
    }

    /// Run the connection loop.
    /// `attempt_resume`: if true, send Resume (op 6) after Hello instead of Identify (op 2).
    async fn run_connection(
        &self,
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        mut command_rx: mpsc::Receiver<GatewayCommand>,
        attempt_resume: bool,
    ) -> Result<()> {
        let (mut write, mut read) = ws_stream.split();

        let state = self.state.clone();
        let sequence = self.sequence.clone();
        let session_id = self.session_id.clone();
        let resume_url = self.resume_url.clone();
        let heartbeat_interval = self.heartbeat_interval.clone();
        let last_heartbeat_ack = self.last_heartbeat_ack.clone();
        let event_tx = self.event_tx.clone();
        let token = self.token.clone();
        let fingerprint = self.fingerprint.clone();

        // Zlib-stream decompressor (maintains state across chunks)
        let mut decompress = ZlibStreamDecompressor::new();

        // Heartbeat timer — None until we receive Hello with the interval
        let mut heartbeat_tick: Option<tokio::time::Interval> = None;

        // Track when we last sent a heartbeat so we can detect missing ACKs
        let mut last_heartbeat_sent: Option<Instant> = None;

        // Main event loop
        loop {
            tokio::select! {
                // === Heartbeat timer ===
                // This runs inside the main select loop so heartbeats are
                // sent directly on the write half — no channel indirection.
                _ = async {
                    if let Some(ref mut tick) = heartbeat_tick {
                        tick.tick().await;
                    } else {
                        // No interval yet — sleep forever (will be woken by other branches)
                        std::future::pending::<()>().await;
                    }
                } => {
                    // Check for zombie connection: if we sent a heartbeat but never
                    // got an ACK back, the connection is dead.
                    if let Some(sent_at) = last_heartbeat_sent {
                        let ack_time = *last_heartbeat_ack.read().await;
                        let got_ack = ack_time.map_or(false, |ack| ack > sent_at);
                        if !got_ack {
                            let elapsed = sent_at.elapsed();
                            tracing::warn!(
                                "No heartbeat ACK received for {:.1}s — zombie connection, reconnecting",
                                elapsed.as_secs_f64()
                            );
                            *state.write().await = GatewayState::Reconnecting;
                            break;
                        }
                    }

                    let seq = *sequence.read().await;
                    let heartbeat = GatewayPayload::heartbeat(seq);
                    if let Ok(msg) = serde_json::to_string(&heartbeat) {
                        tracing::trace!("Sending heartbeat (seq: {:?})", seq);
                        if write.send(Message::Text(msg)).await.is_err() {
                            tracing::warn!("Failed to send heartbeat — connection may be dead");
                            *state.write().await = GatewayState::Reconnecting;
                            break;
                        }
                        last_heartbeat_sent = Some(Instant::now());
                    }
                }

                // === Incoming WebSocket messages ===
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Binary(data))) => {
                            match decompress.decompress(data) {
                                Ok(decompressed_bytes) => {
                                    let decompressed = match String::from_utf8(decompressed_bytes) {
                                        Ok(s) => s,
                                        Err(e) => {
                                            tracing::warn!("Gateway: invalid UTF-8 in decompressed message ({} bytes): {}", e.as_bytes().len(), e);
                                            continue;
                                        }
                                    };
                                    if let Ok(payload) = serde_json::from_str::<GatewayPayload>(&decompressed) {
                                            tracing::debug!("Received op {} t={:?}", payload.op, payload.t);
                                            // Update sequence number
                                            if let Some(s) = payload.s {
                                                *sequence.write().await = Some(s);
                                            }

                                            match payload.op {
                                                // ---- Dispatch (op 0) ----
                                                0 => {
                                                    if let Some(event_name) = &payload.t {
                                                        if let Some(data) = &payload.d {
                                                            let event = Self::parse_dispatch_event(
                                                                event_name, data, &session_id, &resume_url
                                                            ).await;
                                                            let _ = event_tx.send(event);
                                                        }
                                                    }
                                                }
                                                // ---- Heartbeat request (op 1) ----
                                                1 => {
                                                    let seq = *sequence.read().await;
                                                    let hb = GatewayPayload::heartbeat(seq);
                                                    let msg = serde_json::to_string(&hb).unwrap();
                                                    let _ = write.send(Message::Text(msg)).await;
                                                }
                                                // ---- Reconnect (op 7) ----
                                                7 => {
                                                    tracing::info!("Gateway requested reconnect");
                                                    *state.write().await = GatewayState::Reconnecting;
                                                    break;
                                                }
                                                // ---- Invalid Session (op 9) ----
                                                9 => {
                                                    let resumable = payload.d
                                                        .as_ref()
                                                        .and_then(|d| d.as_bool())
                                                        .unwrap_or(false);

                                                    if !resumable {
                                                        tracing::warn!("Invalid session (not resumable) — clearing session");
                                                        *session_id.write().await = None;
                                                        *sequence.write().await = None;
                                                    } else {
                                                        tracing::warn!("Invalid session (resumable) — will retry");
                                                    }

                                                    // Discord says to wait 1-5 seconds before reconnecting
                                                    let wait_secs = {
                                                        let mut rng = rand::thread_rng();
                                                        rng.gen_range(1..=5)
                                                    };
                                                    tracing::info!("Waiting {}s before reconnect (per Discord guidelines)", wait_secs);
                                                    tokio::time::sleep(Duration::from_secs(wait_secs)).await;

                                                    *state.write().await = GatewayState::Reconnecting;
                                                    break;
                                                }
                                                // ---- Hello (op 10) ----
                                                10 => {
                                                    if let Some(data) = &payload.d {
                                                        if let Some(interval_ms) = data.get("heartbeat_interval").and_then(|v| v.as_u64()) {
                                                            *heartbeat_interval.write().await = Some(interval_ms);

                                                            // Start heartbeat timer with jitter for first beat
                                                            let jitter = {
                                                                let mut rng = rand::thread_rng();
                                                                rng.gen_range(0..interval_ms)
                                                            };
                                                            let mut tick = tokio::time::interval_at(
                                                                tokio::time::Instant::now() + Duration::from_millis(jitter),
                                                                Duration::from_millis(interval_ms),
                                                            );
                                                            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                                                            heartbeat_tick = Some(tick);

                                                            tracing::info!(
                                                                "Heartbeat interval: {}ms (first in {}ms)",
                                                                interval_ms, jitter
                                                            );
                                                        }
                                                    }

                                                    // Send Resume or Identify based on state
                                                    if attempt_resume {
                                                        let sid = session_id.read().await.clone();
                                                        let seq = *sequence.read().await;
                                                        if let (Some(sid), Some(seq)) = (sid, seq) {
                                                            tracing::info!("Sending Resume (session: {}, seq: {})", sid, seq);
                                                            let resume = GatewayPayload::resume(&token, &sid, seq);
                                                            let msg = serde_json::to_string(&resume).unwrap();
                                                            let _ = write.send(Message::Text(msg)).await;
                                                        } else {
                                                            tracing::info!("No valid session for resume, sending Identify");
                                                            let identify = GatewayPayload::identify(&token, &fingerprint);
                                                            let msg = serde_json::to_string(&identify).unwrap();
                                                            let _ = write.send(Message::Text(msg)).await;
                                                        }
                                                    } else {
                                                        tracing::info!("Sending Identify");
                                                        let identify = GatewayPayload::identify(&token, &fingerprint);
                                                        let msg = serde_json::to_string(&identify).unwrap();
                                                        let _ = write.send(Message::Text(msg)).await;
                                                    }
                                                }
                                                // ---- Heartbeat ACK (op 11) ----
                                                11 => {
                                                    *last_heartbeat_ack.write().await = Some(Instant::now());
                                                    tracing::trace!("Heartbeat ACK received");
                                                }
                                                _ => {
                                                    tracing::debug!("Unknown gateway opcode: {}", payload.op);
                                                }
                                            }
                                        }
                                    }
                                Err(ZlibDecompressionError::NeedMoreData) => {
                                    // Wait for more chunks
                                }
                                Err(e) => {
                                    tracing::error!("Zlib decompression error: {:?}", e);
                                    *state.write().await = GatewayState::Reconnecting;
                                    break;
                                }
                            }
                        }
                        Some(Ok(Message::Text(text))) => {
                            // Uncompressed text — handle all opcodes the same as binary
                            if let Ok(payload) = serde_json::from_str::<GatewayPayload>(&text) {
                                tracing::debug!("Received text op {} t={:?}", payload.op, payload.t);
                                if let Some(s) = payload.s {
                                    *sequence.write().await = Some(s);
                                }

                                match payload.op {
                                    0 => {
                                        if let Some(event_name) = &payload.t {
                                            if let Some(data) = &payload.d {
                                                let event = Self::parse_dispatch_event(
                                                    event_name, data, &session_id, &resume_url
                                                ).await;
                                                let _ = event_tx.send(event);
                                            }
                                        }
                                    }
                                    1 => {
                                        let seq = *sequence.read().await;
                                        let hb = GatewayPayload::heartbeat(seq);
                                        let msg = serde_json::to_string(&hb).unwrap();
                                        let _ = write.send(Message::Text(msg)).await;
                                    }
                                    7 => {
                                        tracing::info!("Gateway requested reconnect (text frame)");
                                        *state.write().await = GatewayState::Reconnecting;
                                        break;
                                    }
                                    9 => {
                                        let resumable = payload.d
                                            .as_ref()
                                            .and_then(|d| d.as_bool())
                                            .unwrap_or(false);
                                        if !resumable {
                                            tracing::warn!("Invalid session (not resumable) — clearing session");
                                            *session_id.write().await = None;
                                            *sequence.write().await = None;
                                        }
                                        let wait_secs = {
                                            let mut rng = rand::thread_rng();
                                            rng.gen_range(1..=5)
                                        };
                                        tokio::time::sleep(Duration::from_secs(wait_secs)).await;
                                        *state.write().await = GatewayState::Reconnecting;
                                        break;
                                    }
                                    11 => {
                                        *last_heartbeat_ack.write().await = Some(Instant::now());
                                        tracing::trace!("Heartbeat ACK received (text)");
                                    }
                                    _ => {
                                        tracing::debug!("Unhandled text opcode: {}", payload.op);
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Close(frame))) => {
                            tracing::info!("Gateway connection closed: {:?}", frame);
                            *state.write().await = GatewayState::Disconnected;
                            break;
                        }
                        Some(Err(e)) => {
                            tracing::error!("WebSocket error: {}", e);
                            *state.write().await = GatewayState::Reconnecting;
                            break;
                        }
                        None => {
                            tracing::warn!("WebSocket stream ended");
                            *state.write().await = GatewayState::Reconnecting;
                            break;
                        }
                        _ => {}
                    }
                }

                // === Outgoing commands ===
                Some(cmd) = command_rx.recv() => {
                    let payload_json = match cmd {
                        GatewayCommand::UpdatePresence(presence) => {
                            serde_json::to_string(&GatewayPayload::presence_update(presence)).ok()
                        }
                        GatewayCommand::RequestGuildMembers(request) => {
                            serde_json::to_string(&GatewayPayload {
                                op: 8,
                                d: Some(serde_json::to_value(&request).unwrap()),
                                s: None, t: None,
                            }).ok()
                        }
                        GatewayCommand::VoiceStateUpdate(update) => {
                            serde_json::to_string(&GatewayPayload {
                                op: 4,
                                d: Some(serde_json::to_value(&update).unwrap()),
                                s: None, t: None,
                            }).ok()
                        }
                        GatewayCommand::LazyGuild(request) => {
                            serde_json::to_string(&GatewayPayload {
                                op: 14,
                                d: Some(serde_json::to_value(&request).unwrap()),
                                s: None, t: None,
                            }).ok()
                        }
                        GatewayCommand::RequestSoundboardSounds(request) => {
                            serde_json::to_string(&GatewayPayload {
                                op: 31,
                                d: Some(serde_json::to_value(&request).unwrap()),
                                s: None, t: None,
                            }).ok()
                        }
                        GatewayCommand::Close => {
                            let _ = write.close().await;
                            *state.write().await = GatewayState::Disconnected;
                            break;
                        }
                    };
                    if let Some(json) = payload_json {
                        let _ = write.send(Message::Text(json)).await;
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse a dispatch event (op 0) into a typed GatewayEvent
    async fn parse_dispatch_event(
        event_name: &str,
        data: &serde_json::Value,
        session_id: &Arc<RwLock<Option<String>>>,
        resume_url: &Arc<RwLock<Option<String>>>,
    ) -> GatewayEvent {
        match event_name {
            // ── Connection lifecycle ──────────────────────────────────
            "READY" => {
                match serde_json::from_value::<ReadyEvent>(data.clone()) {
                    Ok(ready) => {
                        tracing::info!(
                            "Gateway READY: session={}, guilds={}, user={}",
                            ready.session_id,
                            ready.guilds.len(),
                            ready.user.display_name()
                        );
                        *session_id.write().await = Some(ready.session_id.clone());
                        if let Some(url) = &ready.resume_gateway_url {
                            *resume_url.write().await = Some(url.clone());
                        }
                        return GatewayEvent::Ready(ready);
                    }
                    Err(e) => {
                        tracing::warn!("READY struct parse failed: {}. Extracting session from raw JSON.", e);
                        if let Some(sid) = data.get("session_id").and_then(|v| v.as_str()) {
                            *session_id.write().await = Some(sid.to_string());
                        }
                        if let Some(url) = data.get("resume_gateway_url").and_then(|v| v.as_str()) {
                            *resume_url.write().await = Some(url.to_string());
                        }
                    }
                }
            }
            "READY_SUPPLEMENTAL" => {
                tracing::debug!("Received READY_SUPPLEMENTAL");
                return GatewayEvent::ReadySupplemental(data.clone());
            }
            "RESUMED" => {
                tracing::info!("Gateway session resumed successfully");
                return GatewayEvent::Raw {
                    event_type: "RESUMED".to_string(),
                    data: data.clone(),
                };
            }
            "SESSIONS_REPLACE" => {
                tracing::debug!("Sessions replaced");
                return GatewayEvent::SessionsReplace(data.clone());
            }

            // ── Messages ─────────────────────────────────────────────
            "MESSAGE_CREATE" => {
                if let Ok(msg) = serde_json::from_value::<MessageCreateEvent>(data.clone()) {
                    return GatewayEvent::MessageCreate(msg);
                }
            }
            "MESSAGE_UPDATE" => {
                if let Ok(msg) = serde_json::from_value::<MessageUpdateEvent>(data.clone()) {
                    return GatewayEvent::MessageUpdate(msg);
                }
            }
            "MESSAGE_DELETE" => {
                if let Ok(msg) = serde_json::from_value::<MessageDeleteEvent>(data.clone()) {
                    return GatewayEvent::MessageDelete(msg);
                }
            }
            "MESSAGE_DELETE_BULK" => {
                if let Ok(msg) = serde_json::from_value::<MessageDeleteBulkEvent>(data.clone()) {
                    return GatewayEvent::MessageDeleteBulk(msg);
                }
            }
            "MESSAGE_ACK" => {
                if let Ok(ack) = serde_json::from_value::<MessageAckEvent>(data.clone()) {
                    return GatewayEvent::MessageAck(ack);
                }
            }

            // ── Reactions ────────────────────────────────────────────
            "MESSAGE_REACTION_ADD" => {
                if let Ok(r) = serde_json::from_value::<MessageReactionAddEvent>(data.clone()) {
                    return GatewayEvent::MessageReactionAdd(r);
                }
            }
            "MESSAGE_REACTION_REMOVE" => {
                if let Ok(r) = serde_json::from_value::<MessageReactionRemoveEvent>(data.clone()) {
                    return GatewayEvent::MessageReactionRemove(r);
                }
            }
            "MESSAGE_REACTION_REMOVE_ALL" => {
                if let Ok(r) = serde_json::from_value::<MessageReactionRemoveAllEvent>(data.clone())
                {
                    return GatewayEvent::MessageReactionRemoveAll(r);
                }
            }
            "MESSAGE_REACTION_REMOVE_EMOJI" => {
                if let Ok(r) = serde_json::from_value::<MessageReactionRemoveEmojiEvent>(data.clone()) {
                    return GatewayEvent::MessageReactionRemoveEmoji(r);
                }
            }

            // ── Polls ────────────────────────────────────────────────
            "MESSAGE_POLL_VOTE_ADD" => {
                if let Ok(v) = serde_json::from_value::<MessagePollVoteEvent>(data.clone()) {
                    return GatewayEvent::MessagePollVoteAdd(v);
                }
            }
            "MESSAGE_POLL_VOTE_REMOVE" => {
                if let Ok(v) = serde_json::from_value::<MessagePollVoteEvent>(data.clone()) {
                    return GatewayEvent::MessagePollVoteRemove(v);
                }
            }

            // ── Presence & typing ────────────────────────────────────
            "PRESENCE_UPDATE" => {
                if let Ok(p) = serde_json::from_value::<PresenceUpdateEvent>(data.clone()) {
                    return GatewayEvent::PresenceUpdate(p);
                }
            }
            "TYPING_START" => {
                if let Ok(t) = serde_json::from_value::<TypingStartEvent>(data.clone()) {
                    return GatewayEvent::TypingStart(t);
                }
            }

            // ── Users ────────────────────────────────────────────────
            "USER_UPDATE" => {
                if let Ok(u) = serde_json::from_value::<crate::client::User>(data.clone()) {
                    return GatewayEvent::UserUpdate(u);
                }
            }
            "USER_SETTINGS_UPDATE" => {
                return GatewayEvent::UserSettingsUpdate(data.clone());
            }
            "USER_NOTE_UPDATE" => {
                if let Ok(n) = serde_json::from_value::<UserNoteUpdateEvent>(data.clone()) {
                    return GatewayEvent::UserNoteUpdate(n);
                }
            }

            // ── Guilds ───────────────────────────────────────────────
            "GUILD_CREATE" => {
                return GatewayEvent::GuildCreate(data.clone());
            }
            "GUILD_UPDATE" => {
                return GatewayEvent::GuildUpdate(data.clone());
            }
            "GUILD_DELETE" => {
                if let Ok(g) = serde_json::from_value::<GuildDeleteEvent>(data.clone()) {
                    return GatewayEvent::GuildDelete(g);
                }
            }

            // ── Guild members ────────────────────────────────────────
            "GUILD_MEMBER_ADD" => {
                if let Ok(m) = serde_json::from_value::<GuildMemberAddEvent>(data.clone()) {
                    return GatewayEvent::GuildMemberAdd(m);
                }
            }
            "GUILD_MEMBER_UPDATE" => {
                if let Ok(m) = serde_json::from_value::<GuildMemberUpdateEvent>(data.clone()) {
                    return GatewayEvent::GuildMemberUpdate(m);
                }
            }
            "GUILD_MEMBER_REMOVE" => {
                if let Ok(m) = serde_json::from_value::<GuildMemberRemoveEvent>(data.clone()) {
                    return GatewayEvent::GuildMemberRemove(m);
                }
            }
            "GUILD_MEMBERS_CHUNK" => {
                if let Ok(c) = serde_json::from_value::<GuildMembersChunkEvent>(data.clone()) {
                    return GatewayEvent::GuildMembersChunk(c);
                }
            }
            "GUILD_MEMBER_LIST_UPDATE" => {
                return GatewayEvent::GuildMemberListUpdate(data.clone());
            }

            // ── Guild moderation ─────────────────────────────────────
            "GUILD_BAN_ADD" => {
                if let Ok(b) = serde_json::from_value::<GuildBanEvent>(data.clone()) {
                    return GatewayEvent::GuildBanAdd(b);
                }
            }
            "GUILD_BAN_REMOVE" => {
                if let Ok(b) = serde_json::from_value::<GuildBanEvent>(data.clone()) {
                    return GatewayEvent::GuildBanRemove(b);
                }
            }
            "GUILD_AUDIT_LOG_ENTRY_CREATE" => {
                return GatewayEvent::GuildAuditLogEntryCreate(data.clone());
            }

            // ── Guild roles ──────────────────────────────────────────
            "GUILD_ROLE_CREATE" => {
                if let Ok(r) = serde_json::from_value::<GuildRoleEvent>(data.clone()) {
                    return GatewayEvent::GuildRoleCreate(r);
                }
            }
            "GUILD_ROLE_UPDATE" => {
                if let Ok(r) = serde_json::from_value::<GuildRoleEvent>(data.clone()) {
                    return GatewayEvent::GuildRoleUpdate(r);
                }
            }
            "GUILD_ROLE_DELETE" => {
                if let Ok(r) = serde_json::from_value::<GuildRoleDeleteEvent>(data.clone()) {
                    return GatewayEvent::GuildRoleDelete(r);
                }
            }

            // ── Guild customisation ──────────────────────────────────
            "GUILD_EMOJIS_UPDATE" => {
                if let Ok(e) = serde_json::from_value::<GuildEmojisUpdateEvent>(data.clone()) {
                    return GatewayEvent::GuildEmojisUpdate(e);
                }
            }
            "GUILD_STICKERS_UPDATE" => {
                return GatewayEvent::GuildStickersUpdate(data.clone());
            }
            "GUILD_INTEGRATIONS_UPDATE" => {
                return GatewayEvent::GuildIntegrationsUpdate(data.clone());
            }

            // ── Guild scheduled events ───────────────────────────────
            "GUILD_SCHEDULED_EVENT_CREATE" => {
                return GatewayEvent::GuildScheduledEventCreate(data.clone());
            }
            "GUILD_SCHEDULED_EVENT_UPDATE" => {
                return GatewayEvent::GuildScheduledEventUpdate(data.clone());
            }
            "GUILD_SCHEDULED_EVENT_DELETE" => {
                return GatewayEvent::GuildScheduledEventDelete(data.clone());
            }
            "GUILD_SCHEDULED_EVENT_USER_ADD" => {
                return GatewayEvent::GuildScheduledEventUserAdd(data.clone());
            }
            "GUILD_SCHEDULED_EVENT_USER_REMOVE" => {
                return GatewayEvent::GuildScheduledEventUserRemove(data.clone());
            }

            // ── Guild soundboard ─────────────────────────────────────
            "GUILD_SOUNDBOARD_SOUND_CREATE" => {
                return GatewayEvent::GuildSoundboardSoundCreate(data.clone());
            }
            "GUILD_SOUNDBOARD_SOUND_UPDATE" => {
                return GatewayEvent::GuildSoundboardSoundUpdate(data.clone());
            }
            "GUILD_SOUNDBOARD_SOUND_DELETE" => {
                return GatewayEvent::GuildSoundboardSoundDelete(data.clone());
            }
            "GUILD_SOUNDBOARD_SOUNDS_UPDATE" => {
                return GatewayEvent::GuildSoundboardSoundsUpdate(data.clone());
            }
            "SOUNDBOARD_SOUNDS" => {
                return GatewayEvent::SoundboardSounds(data.clone());
            }

            // ── Channels ─────────────────────────────────────────────
            "CHANNEL_CREATE" => {
                if let Ok(c) = serde_json::from_value::<crate::client::Channel>(data.clone()) {
                    return GatewayEvent::ChannelCreate(c);
                }
            }
            "CHANNEL_UPDATE" => {
                if let Ok(c) = serde_json::from_value::<crate::client::Channel>(data.clone()) {
                    return GatewayEvent::ChannelUpdate(c);
                }
            }
            "CHANNEL_DELETE" => {
                if let Ok(c) = serde_json::from_value::<crate::client::Channel>(data.clone()) {
                    return GatewayEvent::ChannelDelete(c);
                }
            }
            "CHANNEL_PINS_UPDATE" => {
                if let Ok(p) = serde_json::from_value::<ChannelPinsUpdateEvent>(data.clone()) {
                    return GatewayEvent::ChannelPinsUpdate(p);
                }
            }
            "CHANNEL_UNREAD_UPDATE" => {
                return GatewayEvent::ChannelUnreadUpdate(data.clone());
            }

            // ── Threads ──────────────────────────────────────────────
            "THREAD_CREATE" => {
                return GatewayEvent::ThreadCreate(data.clone());
            }
            "THREAD_UPDATE" => {
                return GatewayEvent::ThreadUpdate(data.clone());
            }
            "THREAD_DELETE" => {
                return GatewayEvent::ThreadDelete(data.clone());
            }
            "THREAD_LIST_SYNC" => {
                return GatewayEvent::ThreadListSync(data.clone());
            }
            "THREAD_MEMBER_UPDATE" => {
                return GatewayEvent::ThreadMemberUpdate(data.clone());
            }
            "THREAD_MEMBERS_UPDATE" => {
                return GatewayEvent::ThreadMembersUpdate(data.clone());
            }

            // ── Relationships (user-client only) ─────────────────────
            "RELATIONSHIP_ADD" => {
                if let Ok(r) = serde_json::from_value::<crate::client::Relationship>(data.clone()) {
                    return GatewayEvent::RelationshipAdd(r);
                }
            }
            "RELATIONSHIP_REMOVE" => {
                if let Ok(r) = serde_json::from_value::<RelationshipRemoveEvent>(data.clone()) {
                    return GatewayEvent::RelationshipRemove(r);
                }
            }

            // ── Voice ────────────────────────────────────────────────
            "VOICE_STATE_UPDATE" => {
                if let Ok(v) = serde_json::from_value::<VoiceStateUpdateEvent>(data.clone()) {
                    return GatewayEvent::VoiceStateUpdate(v);
                }
            }
            "VOICE_SERVER_UPDATE" => {
                if let Ok(v) = serde_json::from_value::<VoiceServerUpdateEvent>(data.clone()) {
                    return GatewayEvent::VoiceServerUpdate(v);
                }
            }
            "VOICE_CHANNEL_EFFECT_SEND" => {
                return GatewayEvent::VoiceChannelEffectSend(data.clone());
            }

            // ── Stage instances ──────────────────────────────────────
            "STAGE_INSTANCE_CREATE" => {
                return GatewayEvent::StageInstanceCreate(data.clone());
            }
            "STAGE_INSTANCE_UPDATE" => {
                return GatewayEvent::StageInstanceUpdate(data.clone());
            }
            "STAGE_INSTANCE_DELETE" => {
                return GatewayEvent::StageInstanceDelete(data.clone());
            }

            // ── Interactions ─────────────────────────────────────────
            "INTERACTION_CREATE" => {
                if let Ok(i) = serde_json::from_value::<InteractionCreateEvent>(data.clone()) {
                    return GatewayEvent::InteractionCreate(i);
                }
            }

            // ── Invites ──────────────────────────────────────────────
            "INVITE_CREATE" => {
                return GatewayEvent::InviteCreate(data.clone());
            }
            "INVITE_DELETE" => {
                return GatewayEvent::InviteDelete(data.clone());
            }

            // ── Integrations ─────────────────────────────────────────
            "INTEGRATION_CREATE" => {
                return GatewayEvent::IntegrationCreate(data.clone());
            }
            "INTEGRATION_UPDATE" => {
                return GatewayEvent::IntegrationUpdate(data.clone());
            }
            "INTEGRATION_DELETE" => {
                return GatewayEvent::IntegrationDelete(data.clone());
            }

            // ── Webhooks ─────────────────────────────────────────────
            "WEBHOOKS_UPDATE" => {
                return GatewayEvent::WebhooksUpdate(data.clone());
            }

            // ── Auto-moderation ──────────────────────────────────────
            "AUTO_MODERATION_RULE_CREATE" => {
                return GatewayEvent::AutoModerationRuleCreate(data.clone());
            }
            "AUTO_MODERATION_RULE_UPDATE" => {
                return GatewayEvent::AutoModerationRuleUpdate(data.clone());
            }
            "AUTO_MODERATION_RULE_DELETE" => {
                return GatewayEvent::AutoModerationRuleDelete(data.clone());
            }
            "AUTO_MODERATION_ACTION_EXECUTION" => {
                return GatewayEvent::AutoModerationActionExecution(data.clone());
            }

            // ── Application commands ─────────────────────────────────
            "APPLICATION_COMMAND_PERMISSIONS_UPDATE" => {
                return GatewayEvent::ApplicationCommandPermissionsUpdate(data.clone());
            }

            // ── Entitlements (monetisation) ──────────────────────────
            "ENTITLEMENT_CREATE" => {
                return GatewayEvent::EntitlementCreate(data.clone());
            }
            "ENTITLEMENT_UPDATE" => {
                return GatewayEvent::EntitlementUpdate(data.clone());
            }
            "ENTITLEMENT_DELETE" => {
                return GatewayEvent::EntitlementDelete(data.clone());
            }

            // ── Subscriptions (premium apps) ─────────────────────────
            "SUBSCRIPTION_CREATE" => {
                return GatewayEvent::SubscriptionCreate(data.clone());
            }
            "SUBSCRIPTION_UPDATE" => {
                return GatewayEvent::SubscriptionUpdate(data.clone());
            }
            "SUBSCRIPTION_DELETE" => {
                return GatewayEvent::SubscriptionDelete(data.clone());
            }

            // ── Catch-all ────────────────────────────────────────────
            _ => {
                tracing::debug!("Unhandled dispatch event: {}", event_name);
            }
        }

        // Fallback: return as raw event (typed parse failed or unknown event)
        GatewayEvent::Raw {
            event_type: event_name.to_string(),
            data: data.clone(),
        }
    }

    /// Send a command to the gateway
    pub async fn send_command(&self, command: GatewayCommand) -> Result<()> {
        if let Some(ref tx) = self.command_tx {
            tx.send(command)
                .await
                .map_err(|e| DiscordError::Gateway(e.to_string()))
        } else {
            Err(DiscordError::Gateway("Not connected".to_string()))
        }
    }

    /// Update presence/status
    pub async fn update_presence(&self, presence: PresenceUpdate) -> Result<()> {
        self.send_command(GatewayCommand::UpdatePresence(presence))
            .await
    }

    /// Request lazy guild data (member sidebar) — mimics real Discord client behavior
    pub async fn request_lazy_guild(&self, request: LazyGuildRequest) -> Result<()> {
        self.send_command(GatewayCommand::LazyGuild(request)).await
    }

    /// Request soundboard sounds for the given guild IDs (op 31)
    pub async fn request_soundboard_sounds(&self, guild_ids: Vec<String>) -> Result<()> {
        self.send_command(GatewayCommand::RequestSoundboardSounds(
            RequestSoundboardSounds { guild_ids },
        ))
        .await
    }

    /// Disconnect from the gateway
    pub async fn disconnect(&self) -> Result<()> {
        self.send_command(GatewayCommand::Close).await
    }
}

/// Commands that can be sent to the gateway
#[derive(Debug, Clone)]
pub enum GatewayCommand {
    /// Op 3: Update presence (status, activities)
    UpdatePresence(PresenceUpdate),
    /// Op 8: Request guild members (triggers GUILD_MEMBERS_CHUNK)
    RequestGuildMembers(GuildMembersRequest),
    /// Op 4: Voice state update (join/move/disconnect voice)
    VoiceStateUpdate(VoiceStateUpdate),
    /// Op 14: Request lazy guild — loads member sidebar data
    /// This is what the real Discord client sends to populate the member list
    LazyGuild(LazyGuildRequest),
    /// Op 31: Request soundboard sounds (triggers SOUNDBOARD_SOUNDS)
    RequestSoundboardSounds(RequestSoundboardSounds),
    /// Disconnect the gateway
    Close,
}

/// Op 14: Lazy guild request — populates member sidebar data
/// This matches what the real Discord client sends when viewing a guild
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LazyGuildRequest {
    pub guild_id: String,
    /// Channel ranges to load members for, e.g., [[0, 99], [100, 199]]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<std::collections::HashMap<String, Vec<[u32; 2]>>>,
    /// Whether to include typing status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typing: Option<bool>,
    /// Whether to include thread member counts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threads: Option<bool>,
    /// Whether to include activities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activities: Option<bool>,
    /// List of member IDs to specifically include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<String>>,
}

/// Op 31: Request soundboard sounds for a list of guilds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSoundboardSounds {
    pub guild_ids: Vec<String>,
}

/// Presence update payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdate {
    pub since: Option<u64>,
    pub activities: Vec<Activity>,
    pub status: String,
    pub afk: bool,
}

impl Default for PresenceUpdate {
    fn default() -> Self {
        Self {
            since: None,
            activities: vec![],
            status: "online".to_string(),
            afk: false,
        }
    }
}

/// Activity for presence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub name: String,
    #[serde(rename = "type")]
    pub activity_type: u8,
    pub url: Option<String>,
    pub state: Option<String>,
    pub details: Option<String>,
}

/// Guild members request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMembersRequest {
    pub guild_id: String,
    pub query: Option<String>,
    pub limit: u32,
    pub presences: Option<bool>,
    pub user_ids: Option<Vec<String>>,
    pub nonce: Option<String>,
}

/// Voice state update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceStateUpdate {
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub self_mute: bool,
    pub self_deaf: bool,
}
