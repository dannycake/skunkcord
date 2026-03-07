// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Voice Gateway WebSocket connection
//!
//! Actual WebSocket connection to Discord's voice server.
//! Handles identify, heartbeat, and event dispatching for voice.

use super::gateway::*;
use crate::{DiscordError, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

/// Events emitted by the voice gateway
#[derive(Debug, Clone)]
pub enum VoiceGatewayEvent {
    /// Voice Ready — SSRC, IP, port, encryption modes
    Ready(VoiceReady),
    /// Session description — encryption key
    SessionDescription(VoiceSessionDescription),
    /// Speaking state changed
    Speaking(VoiceSpeaking),
    /// Client connected
    ClientConnect(VoiceClientConnect),
    /// Client disconnected
    ClientDisconnect(VoiceClientDisconnect),
    /// Hello received (heartbeat interval)
    Hello(f64),
    /// Heartbeat ACK
    HeartbeatAck,
    /// Resumed successfully
    Resumed,
    /// Connection closed
    Closed,
}

/// Voice gateway payload
#[derive(Debug, Serialize, Deserialize)]
struct VoicePayload {
    op: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    d: Option<serde_json::Value>,
}

/// Connect to a voice gateway and run the event loop.
///
/// Returns a channel for receiving voice events and a channel for sending
/// voice gateway commands (like Speaking).
pub async fn connect_voice_gateway(
    info: &VoiceConnectionInfo,
) -> Result<(
    broadcast::Receiver<VoiceGatewayEvent>,
    mpsc::Sender<VoiceGatewayCommand>,
)> {
    let endpoint = if info.endpoint.starts_with("wss://") {
        info.endpoint.clone()
    } else {
        format!("wss://{}/?v=4", info.endpoint.trim_end_matches(":443"))
    };

    tracing::info!("Connecting to voice gateway: {}", endpoint);

    let (ws_stream, _) = connect_async(&endpoint)
        .await
        .map_err(|e| DiscordError::WebSocket(format!("Voice WS connect failed: {}", e)))?;

    let (event_tx, event_rx) = broadcast::channel(64);
    let (cmd_tx, cmd_rx) = mpsc::channel(32);

    let info_clone = info.clone();

    // Spawn the voice gateway loop
    tokio::spawn(async move {
        if let Err(e) = run_voice_gateway(ws_stream, &info_clone, event_tx, cmd_rx).await {
            tracing::error!("Voice gateway error: {}", e);
        }
    });

    Ok((event_rx, cmd_tx))
}

/// Commands we can send to the voice gateway
#[derive(Debug)]
pub enum VoiceGatewayCommand {
    /// Send Speaking (op 5)
    SetSpeaking { speaking: u32, ssrc: u32 },
    /// Select protocol (op 1)
    SelectProtocol {
        address: String,
        port: u16,
        mode: String,
    },
    /// Close connection
    Close,
}

async fn run_voice_gateway(
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    info: &VoiceConnectionInfo,
    event_tx: broadcast::Sender<VoiceGatewayEvent>,
    mut cmd_rx: mpsc::Receiver<VoiceGatewayCommand>,
) -> Result<()> {
    let (mut write, mut read) = ws_stream.split();

    let mut heartbeat_tick: Option<tokio::time::Interval> = None;
    let mut heartbeat_nonce: u64 = 0;

    loop {
        tokio::select! {
            // Heartbeat timer
            _ = async {
                if let Some(ref mut tick) = heartbeat_tick {
                    tick.tick().await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => {
                heartbeat_nonce += 1;
                let payload = VoicePayload {
                    op: VoiceOpCode::Heartbeat as u8,
                    d: Some(serde_json::json!(heartbeat_nonce)),
                };
                if let Ok(msg) = serde_json::to_string(&payload) {
                    if write.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
            }

            // Incoming messages
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(payload) = serde_json::from_str::<VoicePayload>(&text) {
                            match payload.op {
                                // Ready (op 2)
                                2 => {
                                    if let Some(data) = payload.d {
                                        if let Ok(ready) = serde_json::from_value::<VoiceReady>(data) {
                                            tracing::info!(
                                                "Voice ready: SSRC={}, {}:{}, {} modes",
                                                ready.ssrc, ready.ip, ready.port, ready.modes.len()
                                            );
                                            let _ = event_tx.send(VoiceGatewayEvent::Ready(ready));
                                        }
                                    }
                                }
                                // Session Description (op 4)
                                4 => {
                                    if let Some(data) = payload.d {
                                        if let Ok(desc) = serde_json::from_value::<VoiceSessionDescription>(data) {
                                            tracing::info!("Voice session: mode={}", desc.mode);
                                            let _ = event_tx.send(VoiceGatewayEvent::SessionDescription(desc));
                                        }
                                    }
                                }
                                // Speaking (op 5)
                                5 => {
                                    if let Some(data) = payload.d {
                                        if let Ok(speaking) = serde_json::from_value::<VoiceSpeaking>(data) {
                                            let _ = event_tx.send(VoiceGatewayEvent::Speaking(speaking));
                                        }
                                    }
                                }
                                // Heartbeat ACK (op 6)
                                6 => {
                                    let _ = event_tx.send(VoiceGatewayEvent::HeartbeatAck);
                                }
                                // Hello (op 8)
                                8 => {
                                    if let Some(data) = &payload.d {
                                        let interval = data.get("heartbeat_interval")
                                            .and_then(|v| v.as_f64())
                                            .unwrap_or(41250.0);

                                        // Start heartbeat
                                        let interval_ms = interval as u64;
                                        let mut tick = tokio::time::interval(Duration::from_millis(interval_ms));
                                        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                                        heartbeat_tick = Some(tick);

                                        let _ = event_tx.send(VoiceGatewayEvent::Hello(interval));

                                        // Send Identify (op 0)
                                        let identify = VoicePayload {
                                            op: VoiceOpCode::Identify as u8,
                                            d: Some(serde_json::to_value(&VoiceIdentify {
                                                server_id: info.guild_id.clone().unwrap_or_default(),
                                                user_id: info.user_id.clone(),
                                                session_id: info.session_id.clone(),
                                                token: info.token.clone(),
                                            }).unwrap()),
                                        };
                                        if let Ok(msg) = serde_json::to_string(&identify) {
                                            let _ = write.send(Message::Text(msg)).await;
                                        }
                                    }
                                }
                                // Resumed (op 9)
                                9 => {
                                    let _ = event_tx.send(VoiceGatewayEvent::Resumed);
                                }
                                // Client Connect (op 12)
                                12 => {
                                    if let Some(data) = payload.d {
                                        if let Ok(cc) = serde_json::from_value::<VoiceClientConnect>(data) {
                                            let _ = event_tx.send(VoiceGatewayEvent::ClientConnect(cc));
                                        }
                                    }
                                }
                                // Client Disconnect (op 13)
                                13 => {
                                    if let Some(data) = payload.d {
                                        if let Ok(cd) = serde_json::from_value::<VoiceClientDisconnect>(data) {
                                            let _ = event_tx.send(VoiceGatewayEvent::ClientDisconnect(cd));
                                        }
                                    }
                                }
                                _ => {
                                    tracing::debug!("Voice gateway unknown op: {}", payload.op);
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        let _ = event_tx.send(VoiceGatewayEvent::Closed);
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::error!("Voice WS error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }

            // Outgoing commands
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    VoiceGatewayCommand::SetSpeaking { speaking, ssrc } => {
                        let payload = VoicePayload {
                            op: VoiceOpCode::Speaking as u8,
                            d: Some(serde_json::json!({
                                "speaking": speaking,
                                "delay": 0,
                                "ssrc": ssrc,
                            })),
                        };
                        if let Ok(msg) = serde_json::to_string(&payload) {
                            let _ = write.send(Message::Text(msg)).await;
                        }
                    }
                    VoiceGatewayCommand::SelectProtocol { address, port, mode } => {
                        let payload = VoicePayload {
                            op: VoiceOpCode::SelectProtocol as u8,
                            d: Some(serde_json::json!({
                                "protocol": "udp",
                                "data": {
                                    "address": address,
                                    "port": port,
                                    "mode": mode,
                                }
                            })),
                        };
                        if let Ok(msg) = serde_json::to_string(&payload) {
                            let _ = write.send(Message::Text(msg)).await;
                        }
                    }
                    VoiceGatewayCommand::Close => {
                        let _ = write.close().await;
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
