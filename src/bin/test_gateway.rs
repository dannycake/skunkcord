// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Gateway connection diagnostic - connects to Discord gateway with real token.
//! Logs all raw messages to diagnose connection issues.
//! Run: DISCORD_TOKEN=your_token cargo run --bin test_gateway

use skunkcord::fingerprint::BrowserFingerprint;
use skunkcord::gateway::{GatewayPayload, ReadyEvent};
use skunkcord::GATEWAY_VERSION;
use futures_util::{SinkExt, StreamExt};
use std::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use zlib_stream::ZlibStreamDecompressor;

const GATEWAY_URL: &str = "wss://gateway.discord.gg";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");
    println!("=== Discord Gateway Diagnostic ===");
    println!("Token prefix: {}...", &token[..20.min(token.len())]);

    let fingerprint = BrowserFingerprint::new_chrome();
    println!("Fingerprint: build={}, browser={} {}, os={} {}",
        fingerprint.client_build_number,
        fingerprint.browser,
        fingerprint.browser_version,
        fingerprint.os,
        fingerprint.os_version,
    );

    let url = format!(
        "{}/?v={}&encoding=json&compress=zlib-stream",
        GATEWAY_URL, GATEWAY_VERSION
    );
    println!("\nConnecting to: {}", url);

    let (ws_stream, response) = connect_async(&url).await?;
    println!("WebSocket connected! HTTP status: {}", response.status());
    for (key, value) in response.headers() {
        println!("  {}: {}", key, value.to_str().unwrap_or("?"));
    }

    let (mut write, mut read) = ws_stream.split();
    let mut decompress = ZlibStreamDecompressor::new();
    let start = Instant::now();
    let mut msg_count = 0u32;
    let mut identified = false;

    println!("\n--- Listening for messages (60s timeout) ---\n");

    loop {
        if start.elapsed() > Duration::from_secs(60) {
            println!("\n[TIMEOUT] 60 seconds elapsed without completing handshake");
            break;
        }

        let msg = tokio::time::timeout(Duration::from_secs(15), read.next()).await;

        match msg {
            Err(_) => {
                println!("[TIMEOUT] No message received in 15 seconds (total elapsed: {:?})", start.elapsed());
                if identified {
                    println!("[DIAG] We sent Identify but got no response — likely rejected or payload issue");
                }
                continue;
            }
            Ok(None) => {
                println!("[CLOSED] WebSocket stream ended");
                break;
            }
            Ok(Some(Err(e))) => {
                println!("[ERROR] WebSocket error: {}", e);
                break;
            }
            Ok(Some(Ok(message))) => {
                msg_count += 1;
                let elapsed = start.elapsed();

                match &message {
                    Message::Binary(data) => {
                        println!("[MSG #{} @ {:?}] Binary frame, {} bytes", msg_count, elapsed, data.len());

                        // Show raw suffix to verify zlib-stream format
                        if data.len() >= 4 {
                            let suffix = &data[data.len()-4..];
                            let is_zlib_end = suffix == [0u8, 0, 255, 255].as_slice();
                            println!("  Suffix: {:02x?} (zlib_end={})", suffix, is_zlib_end);
                        }

                        match decompress.decompress(&data[..]) {
                            Ok(decompressed) => {
                                let text = String::from_utf8_lossy(&decompressed);
                                println!("  Decompressed: {} bytes", decompressed.len());

                                // Truncate for display but show enough context
                                let display = if text.len() > 2000 {
                                    format!("{}... [truncated, total {} chars]", &text[..2000], text.len())
                                } else {
                                    text.to_string()
                                };
                                println!("  JSON: {}", display);

                                // Parse and handle
                                match serde_json::from_str::<GatewayPayload>(&text) {
                                    Ok(payload) => {
                                        println!("  Parsed: op={} t={:?} s={:?}", payload.op, payload.t, payload.s);

                                        match payload.op {
                                            10 => {
                                                // Hello
                                                println!("\n  >>> Received Hello (op 10)");
                                                if let Some(ref d) = payload.d {
                                                    if let Some(interval) = d.get("heartbeat_interval").and_then(|v| v.as_u64()) {
                                                        println!("  Heartbeat interval: {}ms", interval);
                                                    }
                                                }

                                                // Send Identify
                                                let identify = GatewayPayload::identify(&token, &fingerprint);
                                                let identify_json = serde_json::to_string(&identify).unwrap();
                                                println!("\n  >>> Sending Identify (op 2)");
                                                println!("  Identify payload ({} bytes):", identify_json.len());
                                                // Print pretty but truncate token
                                                let identify_display = identify_json.replace(&token, "<TOKEN>");
                                                if identify_display.len() > 3000 {
                                                    println!("  {}", &identify_display[..3000]);
                                                } else {
                                                    println!("  {}", identify_display);
                                                }

                                                match write.send(Message::Text(identify_json)).await {
                                                    Ok(()) => {
                                                        println!("  >>> Identify SENT successfully");
                                                        identified = true;
                                                    }
                                                    Err(e) => {
                                                        println!("  >>> Identify SEND FAILED: {}", e);
                                                    }
                                                }
                                            }
                                            0 => {
                                                // Dispatch
                                                if let Some(event_name) = &payload.t {
                                                    println!("  >>> Dispatch event: {}", event_name);
                                                    if event_name == "READY" {
                                                        println!("\n  *** SUCCESS: Received READY event! ***");
                                                        if let Some(ref d) = payload.d {
                                                            let session = d.get("session_id").and_then(|v| v.as_str()).unwrap_or("?");
                                                            let guilds = d.get("guilds").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                                                            let user = d.get("user").and_then(|u| u.get("username")).and_then(|v| v.as_str()).unwrap_or("?");
                                                            println!("  Session: {}", session);
                                                            println!("  User: {}", user);
                                                            println!("  Guilds: {}", guilds);

                                                            // Test ReadyEvent parsing (same as Gateway struct)
                                                            println!("\n  --- Testing ReadyEvent struct parsing ---");
                                                            match serde_json::from_value::<ReadyEvent>(d.clone()) {
                                                                Ok(ready) => {
                                                                    println!("  ReadyEvent parsed OK! session={}, guilds={}", ready.session_id, ready.guilds.len());
                                                                }
                                                                Err(e) => {
                                                                    println!("  *** ReadyEvent PARSE FAILED: {} ***", e);
                                                                    println!("  This is the bug! The Gateway falls back to Raw event at debug level.");
                                                                    // Try to identify which field fails
                                                                    for field in ["v", "user", "private_channels", "guilds", "session_id",
                                                                                  "resume_gateway_url", "relationships", "user_settings",
                                                                                  "user_guild_settings", "read_state", "connected_accounts",
                                                                                  "session_type", "auth_session_id_hash"] {
                                                                        if let Some(val) = d.get(field) {
                                                                            let type_str = match val {
                                                                                serde_json::Value::Null => "null",
                                                                                serde_json::Value::Bool(_) => "bool",
                                                                                serde_json::Value::Number(_) => "number",
                                                                                serde_json::Value::String(_) => "string",
                                                                                serde_json::Value::Array(a) => {
                                                                                    println!("    {}: array[{}]", field, a.len());
                                                                                    continue;
                                                                                }
                                                                                serde_json::Value::Object(o) => {
                                                                                    println!("    {}: object with keys {:?}", field, o.keys().take(10).collect::<Vec<_>>());
                                                                                    continue;
                                                                                }
                                                                            };
                                                                            println!("    {}: {}", field, type_str);
                                                                        } else {
                                                                            println!("    {}: MISSING", field);
                                                                        }
                                                                    }
                                                                    // Show all top-level keys
                                                                    if let Some(obj) = d.as_object() {
                                                                        println!("  All top-level keys: {:?}", obj.keys().collect::<Vec<_>>());
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        // Wait a moment for any additional messages then exit
                                                        println!("\n  Listening for 5 more seconds...");
                                                        tokio::time::sleep(Duration::from_secs(5)).await;
    // Now test through the actual Gateway struct (same code path as main app)
    println!("\n  --- Testing Gateway struct integration ---");
    {
        use skunkcord::gateway::{Gateway, GatewayEvent};
        let fingerprint2 = BrowserFingerprint::new_chrome();
        let mut gw = Gateway::new(token.clone(), fingerprint2);
        let mut rx = gw.subscribe();
        let gw_handle = tokio::spawn(async move {
            if let Err(e) = gw.connect().await {
                eprintln!("  Gateway struct error: {}", e);
            }
        });
        let mut gw_ready = false;
        let gw_start = Instant::now();
        while gw_start.elapsed() < Duration::from_secs(30) {
            match tokio::time::timeout(Duration::from_secs(3), rx.recv()).await {
                Ok(Ok(GatewayEvent::Ready(ready))) => {
                    println!("  Gateway struct: READY! session={}, guilds={}, user={}",
                        ready.session_id, ready.guilds.len(), ready.user.display_name());
                    gw_ready = true;
                    break;
                }
                Ok(Ok(GatewayEvent::Raw { event_type, .. })) if event_type == "READY" => {
                    println!("  Gateway struct: READY (raw fallback)");
                    gw_ready = true;
                    break;
                }
                Ok(Ok(_)) => continue,
                Ok(Err(_)) => break,
                Err(_) => continue,
            }
        }
        gw_handle.abort();
        let _ = gw_handle.await;
        if gw_ready {
            println!("  Gateway struct: SUCCESS");
        } else {
            println!("  Gateway struct: FAILED - did not receive Ready event");
        }
    }

    println!("\n=== Gateway connection SUCCESSFUL ===");
    return Ok(());
                                                    }
                                                    if event_name == "READY_SUPPLEMENTAL" {
                                                        println!("  >>> READY_SUPPLEMENTAL received");
                                                    }
                                                }
                                            }
                                            1 => println!("  >>> Heartbeat request from server"),
                                            7 => println!("  >>> Reconnect requested by server"),
                                            9 => {
                                                let resumable = payload.d.as_ref().and_then(|d| d.as_bool()).unwrap_or(false);
                                                println!("  >>> Invalid Session (resumable={})", resumable);
                                                println!("  [DIAG] Discord rejected our session!");
                                            }
                                            11 => println!("  >>> Heartbeat ACK"),
                                            _ => println!("  >>> Unknown opcode: {}", payload.op),
                                        }
                                    }
                                    Err(e) => {
                                        println!("  [PARSE ERROR] Failed to parse payload: {}", e);
                                        println!("  Raw text: {}", &text[..500.min(text.len())]);
                                    }
                                }
                            }
                            Err(zlib_stream::ZlibDecompressionError::NeedMoreData) => {
                                println!("  [ZLIB] NeedMoreData — waiting for more chunks");
                            }
                            Err(e) => {
                                println!("  [ZLIB ERROR] Decompression failed: {:?}", e);
                            }
                        }
                    }
                    Message::Text(text) => {
                        println!("[MSG #{} @ {:?}] Text frame, {} bytes", msg_count, elapsed, text.len());
                        let display = if text.len() > 2000 {
                            format!("{}... [truncated]", &text[..2000])
                        } else {
                            text.to_string()
                        };
                        println!("  Content: {}", display);

                        // Try to parse as gateway payload
                        if let Ok(payload) = serde_json::from_str::<GatewayPayload>(text) {
                            println!("  Parsed: op={} t={:?} s={:?}", payload.op, payload.t, payload.s);
                            if payload.op == 9 {
                                let resumable = payload.d.as_ref().and_then(|d| d.as_bool()).unwrap_or(false);
                                println!("  >>> Invalid Session (resumable={}) — Discord rejected identify!", resumable);
                            }
                        }
                    }
                    Message::Close(frame) => {
                        println!("[MSG #{} @ {:?}] Close frame: {:?}", msg_count, elapsed, frame);
                        if let Some(f) = frame {
                            println!("  Close code: {}, reason: {}", f.code, f.reason);
                            match f.code.into() {
                                4004u16 => println!("  [DIAG] 4004 = Authentication failed (invalid token)"),
                                4013 => println!("  [DIAG] 4013 = Invalid intents"),
                                4014 => println!("  [DIAG] 4014 = Disallowed intents"),
                                code => println!("  [DIAG] Close code: {}", code),
                            }
                        }
                        break;
                    }
                    Message::Ping(data) => {
                        println!("[MSG #{} @ {:?}] Ping, {} bytes", msg_count, elapsed, data.len());
                    }
                    Message::Pong(data) => {
                        println!("[MSG #{} @ {:?}] Pong, {} bytes", msg_count, elapsed, data.len());
                    }
                    _ => {
                        println!("[MSG #{} @ {:?}] Other message type", msg_count, elapsed);
                    }
                }
            }
        }
    }

    println!("\n=== Diagnostic complete. {} messages received ===", msg_count);
    if !identified {
        println!("[DIAG] Never sent Identify — Hello was never received or decompression failed");
    } else {
        println!("[DIAG] Identify was sent but READY was never received");
    }
    Ok(())
}
