// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Browser Handoff RPC Server
//!
//! Listens on localhost ports 6463–6472 (Discord's desktop RPC ports) for
//! WebSocket connections from the browser. When the browser sends an invite
//! via the "InviteBrowser" command (or similar), we extract the invite code
//! and forward it to the UI via `UiUpdate::RpcInviteReceived`.
//!
//! This allows users to click "Open in App" on a discord.gg invite in the
//! browser and have this client handle it.

use crate::bridge::UiUpdate;
use std::sync::mpsc;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::StreamExt;

/// Discord desktop RPC port range
const RPC_PORT_START: u16 = 6463;
const RPC_PORT_END: u16 = 6472;

/// Start the browser handoff server. Tries to bind to the first available
/// port in 6463–6472 and listens for WebSocket connections.
///
/// Runs forever (until the task is cancelled). Should be spawned as a
/// background task.
pub async fn run_browser_handoff_server(update_tx: mpsc::Sender<UiUpdate>) {
    // Try to bind on the first available port
    let listener = {
        let mut bound = None;
        for port in RPC_PORT_START..=RPC_PORT_END {
            match TcpListener::bind(("127.0.0.1", port)).await {
                Ok(l) => {
                    tracing::info!("Browser handoff RPC server listening on 127.0.0.1:{}", port);
                    bound = Some(l);
                    break;
                }
                Err(e) => {
                    tracing::debug!("Could not bind RPC port {}: {}", port, e);
                }
            }
        }
        match bound {
            Some(l) => l,
            None => {
                tracing::warn!(
                    "Browser handoff: could not bind any port in {}-{}",
                    RPC_PORT_START,
                    RPC_PORT_END
                );
                return;
            }
        }
    };

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                tracing::debug!("Browser handoff: connection from {}", addr);
                let tx = update_tx.clone();
                tokio::spawn(async move {
                    match accept_async(stream).await {
                        Ok(ws) => {
                            handle_ws_connection(ws, tx).await;
                        }
                        Err(e) => {
                            tracing::debug!("Browser handoff: WS handshake failed: {}", e);
                        }
                    }
                });
            }
            Err(e) => {
                tracing::warn!("Browser handoff: accept error: {}", e);
            }
        }
    }
}

/// Handle a single WebSocket connection from the browser.
async fn handle_ws_connection(
    mut ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    update_tx: mpsc::Sender<UiUpdate>,
) {
    while let Some(msg) = ws.next().await {
        match msg {
            Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                if let Some(invite) = extract_invite_from_payload(&text) {
                    tracing::info!("Browser handoff: received invite '{}'", invite);
                    let _ = update_tx.send(UiUpdate::RpcInviteReceived(invite));
                }
            }
            Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => break,
            Err(e) => {
                tracing::debug!("Browser handoff: WS read error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

/// Try to extract an invite code from a browser handoff JSON payload.
///
/// Discord's browser client may send payloads like:
/// ```json
/// { "cmd": "INVITE_BROWSER", "args": { "code": "abc123" } }
/// ```
/// or include the invite code directly in various fields. We search
/// defensively for anything that looks like an invite code or URL.
fn extract_invite_from_payload(text: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(text).ok()?;

    // Try cmd=INVITE_BROWSER with args.code
    if let Some(cmd) = json.get("cmd").and_then(|v| v.as_str()) {
        let cmd_upper = cmd.to_uppercase();
        if cmd_upper == "INVITE_BROWSER" || cmd_upper == "BROWSER_HANDOFF" {
            // Try args.code first
            if let Some(code) = json
                .get("args")
                .and_then(|a| a.get("code"))
                .and_then(|v| v.as_str())
            {
                let code = code.trim();
                if !code.is_empty() {
                    return Some(code.to_string());
                }
            }
            // Try args.inviteCode
            if let Some(code) = json
                .get("args")
                .and_then(|a| a.get("inviteCode"))
                .and_then(|v| v.as_str())
            {
                let code = code.trim();
                if !code.is_empty() {
                    return Some(code.to_string());
                }
            }
        }
    }

    // Fallback: look for any field called "code" or "invite" at the top level or in "args"/"data"
    for container in &[json.get("args"), json.get("data"), Some(&json)] {
        if let Some(obj) = container {
            for key in &["code", "invite", "inviteCode", "invite_code"] {
                if let Some(val) = obj.get(*key).and_then(|v| v.as_str()) {
                    let val = val.trim();
                    if !val.is_empty() && val.len() < 100 {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_invite_browser() {
        let payload = r#"{"cmd":"INVITE_BROWSER","args":{"code":"abc123"}}"#;
        assert_eq!(
            extract_invite_from_payload(payload),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_extract_browser_handoff() {
        let payload = r#"{"cmd":"BROWSER_HANDOFF","args":{"inviteCode":"xyz789"}}"#;
        assert_eq!(
            extract_invite_from_payload(payload),
            Some("xyz789".to_string())
        );
    }

    #[test]
    fn test_extract_fallback_code() {
        let payload = r#"{"data":{"code":"test123"}}"#;
        assert_eq!(
            extract_invite_from_payload(payload),
            Some("test123".to_string())
        );
    }

    #[test]
    fn test_extract_no_invite() {
        let payload = r#"{"cmd":"SET_ACTIVITY","args":{"name":"Game"}}"#;
        assert_eq!(extract_invite_from_payload(payload), None);
    }
}
