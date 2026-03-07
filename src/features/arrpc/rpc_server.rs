// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Local RPC server that mimics Discord's IPC interface
//!
//! Games/apps connect to this server thinking it's Discord, send
//! SET_ACTIVITY commands, and we forward them as gateway presence updates.

use serde::{Deserialize, Serialize};

/// RPC opcodes used in the IPC protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum RpcOpCode {
    Handshake = 0,
    Frame = 1,
    Close = 2,
    Ping = 3,
    Pong = 4,
}

impl TryFrom<u32> for RpcOpCode {
    type Error = ();
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Handshake),
            1 => Ok(Self::Frame),
            2 => Ok(Self::Close),
            3 => Ok(Self::Ping),
            4 => Ok(Self::Pong),
            _ => Err(()),
        }
    }
}

/// RPC Handshake request from a connecting application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcHandshake {
    pub v: u32,
    pub client_id: String,
}

/// RPC command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpcCommand {
    SetActivity,
    Subscribe,
    Unsubscribe,
    GetGuild,
    GetGuilds,
    GetChannel,
    GetChannels,
    Unknown,
}

impl RpcCommand {
    pub fn from_str(s: &str) -> Self {
        match s {
            "SET_ACTIVITY" => Self::SetActivity,
            "SUBSCRIBE" => Self::Subscribe,
            "UNSUBSCRIBE" => Self::Unsubscribe,
            "GET_GUILD" => Self::GetGuild,
            "GET_GUILDS" => Self::GetGuilds,
            "GET_CHANNEL" => Self::GetChannel,
            "GET_CHANNELS" => Self::GetChannels,
            _ => Self::Unknown,
        }
    }
}

/// Activity data received via RPC SET_ACTIVITY
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcActivity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamps: Option<RpcTimestamps>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<RpcAssets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party: Option<RpcParty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<RpcSecrets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<RpcButton>>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub activity_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcTimestamps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcAssets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcParty {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<[u32; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcSecrets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spectate: Option<String>,
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcButton {
    pub label: String,
    pub url: String,
}

/// IPC socket paths for different platforms
pub fn get_ipc_socket_paths() -> Vec<String> {
    let mut paths = Vec::new();

    #[cfg(target_os = "linux")]
    {
        // Try XDG_RUNTIME_DIR first, then /tmp
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            for i in 0..10 {
                paths.push(format!("{}/discord-ipc-{}", xdg, i));
            }
        }
        for i in 0..10 {
            paths.push(format!("/tmp/discord-ipc-{}", i));
        }
    }

    #[cfg(target_os = "macos")]
    {
        for i in 0..10 {
            paths.push(format!("/tmp/discord-ipc-{}", i));
        }
    }

    #[cfg(target_os = "windows")]
    {
        for i in 0..10 {
            paths.push(format!(r"\\?\pipe\discord-ipc-{}", i));
        }
    }

    paths
}

/// Convert an RPC activity to a gateway-compatible activity
pub fn rpc_to_gateway_activity(
    rpc: &RpcActivity,
    application_id: &str,
    app_name: &str,
) -> serde_json::Value {
    let mut activity = serde_json::json!({
        "name": rpc.name.as_deref().unwrap_or(app_name),
        "type": rpc.activity_type.unwrap_or(0),
        "application_id": application_id,
    });

    if let Some(ref state) = rpc.state {
        activity["state"] = serde_json::json!(state);
    }
    if let Some(ref details) = rpc.details {
        activity["details"] = serde_json::json!(details);
    }
    if let Some(ref ts) = rpc.timestamps {
        activity["timestamps"] = serde_json::to_value(ts).unwrap_or_default();
    }
    if let Some(ref assets) = rpc.assets {
        activity["assets"] = serde_json::to_value(assets).unwrap_or_default();
    }
    if let Some(ref party) = rpc.party {
        activity["party"] = serde_json::to_value(party).unwrap_or_default();
    }
    if let Some(ref buttons) = rpc.buttons {
        let labels: Vec<&str> = buttons.iter().map(|b| b.label.as_str()).collect();
        activity["buttons"] = serde_json::json!(labels);
        let metadata: Vec<serde_json::Value> = buttons
            .iter()
            .map(|b| serde_json::json!({"label": b.label, "url": b.url}))
            .collect();
        activity["metadata"] = serde_json::json!({"button_urls": metadata.iter().map(|m| m["url"].as_str().unwrap_or("")).collect::<Vec<_>>()});
    }

    activity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_opcode() {
        assert_eq!(RpcOpCode::try_from(0), Ok(RpcOpCode::Handshake));
        assert_eq!(RpcOpCode::try_from(1), Ok(RpcOpCode::Frame));
        assert_eq!(RpcOpCode::try_from(99), Err(()));
    }

    #[test]
    fn test_rpc_command() {
        assert_eq!(
            RpcCommand::from_str("SET_ACTIVITY"),
            RpcCommand::SetActivity
        );
        assert_eq!(RpcCommand::from_str("UNKNOWN_CMD"), RpcCommand::Unknown);
    }

    #[test]
    fn test_ipc_paths() {
        let paths = get_ipc_socket_paths();
        assert!(!paths.is_empty());
        assert!(paths[0].contains("discord-ipc-"));
    }

    #[test]
    fn test_rpc_to_gateway_activity() {
        let rpc = RpcActivity {
            state: Some("In Game".to_string()),
            details: Some("Playing level 5".to_string()),
            timestamps: Some(RpcTimestamps {
                start: Some(1700000000),
                end: None,
            }),
            assets: None,
            party: None,
            secrets: None,
            buttons: None,
            activity_type: Some(0),
            name: Some("Cool Game".to_string()),
        };

        let activity = rpc_to_gateway_activity(&rpc, "app123", "Cool Game");
        assert_eq!(activity["name"], "Cool Game");
        assert_eq!(activity["state"], "In Game");
        assert_eq!(activity["details"], "Playing level 5");
        assert_eq!(activity["application_id"], "app123");
    }
}
