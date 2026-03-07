// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Gateway payload structures

use crate::fingerprint::BrowserFingerprint;
use crate::gateway::PresenceUpdate;
use serde::{Deserialize, Serialize};

/// Gateway payload structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayPayload {
    /// Opcode
    pub op: u8,
    /// Event data
    pub d: Option<serde_json::Value>,
    /// Sequence number
    pub s: Option<u64>,
    /// Event name
    pub t: Option<String>,
}

impl GatewayPayload {
    /// Create a heartbeat payload
    pub fn heartbeat(sequence: Option<u64>) -> Self {
        Self {
            op: 1,
            d: sequence.map(|s| serde_json::Value::Number(s.into())),
            s: None,
            t: None,
        }
    }

    /// Create an identify payload
    pub fn identify(token: &str, fingerprint: &BrowserFingerprint) -> Self {
        let identify_data = IdentifyPayload {
            token: token.to_string(),
            capabilities: 1734653, // Standard Discord capabilities (matches reference)
            properties: IdentifyProperties::from_fingerprint(fingerprint),
            presence: IdentifyPresence::default(),
            compress: false,
            client_state: ClientState::default(),
        };

        Self {
            op: 2,
            d: Some(serde_json::to_value(&identify_data).unwrap()),
            s: None,
            t: None,
        }
    }

    /// Create a resume payload
    pub fn resume(token: &str, session_id: &str, sequence: u64) -> Self {
        let resume_data = ResumePayload {
            token: token.to_string(),
            session_id: session_id.to_string(),
            seq: sequence,
        };

        Self {
            op: 6,
            d: Some(serde_json::to_value(&resume_data).unwrap()),
            s: None,
            t: None,
        }
    }

    /// Create a presence update payload
    pub fn presence_update(presence: PresenceUpdate) -> Self {
        Self {
            op: 3,
            d: Some(serde_json::to_value(&presence).unwrap()),
            s: None,
            t: None,
        }
    }
}

/// Identify payload data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifyPayload {
    pub token: String,
    pub capabilities: u32,
    pub properties: IdentifyProperties,
    pub presence: IdentifyPresence,
    pub compress: bool,
    pub client_state: ClientState,
}

/// Identify properties (browser info)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifyProperties {
    pub os: String,
    pub browser: String,
    pub device: String,
    pub system_locale: String,
    pub browser_user_agent: String,
    pub browser_version: String,
    pub os_version: String,
    pub referrer: String,
    pub referring_domain: String,
    pub referrer_current: String,
    pub referring_domain_current: String,
    pub release_channel: String,
    pub client_build_number: u64,
    pub client_event_source: Option<String>,
    pub design_id: u32,
}

impl IdentifyProperties {
    pub fn from_fingerprint(fp: &BrowserFingerprint) -> Self {
        Self {
            os: fp.os.clone(),
            browser: fp.browser.clone(),
            device: fp.device.clone(),
            system_locale: fp.system_locale.clone(),
            browser_user_agent: fp.user_agent.clone(),
            browser_version: fp.browser_version.clone(),
            os_version: fp.os_version.clone(),
            referrer: "".to_string(),
            referring_domain: "".to_string(),
            referrer_current: "".to_string(),
            referring_domain_current: "".to_string(),
            release_channel: fp.release_channel.clone(),
            client_build_number: fp.client_build_number,
            client_event_source: None,
            design_id: 0,
        }
    }
}

/// Identify presence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifyPresence {
    pub status: String,
    pub since: u64,
    pub activities: Vec<serde_json::Value>,
    pub afk: bool,
}

impl Default for IdentifyPresence {
    fn default() -> Self {
        Self {
            status: "online".to_string(),
            since: 0,
            activities: vec![],
            afk: false,
        }
    }
}

/// Client state for identify
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientState {
    pub guild_versions: serde_json::Value,
    pub highest_last_message_id: String,
    pub read_state_version: u32,
    pub user_guild_settings_version: i32,
    pub user_settings_version: i32,
    pub private_channels_version: String,
    pub api_code_version: u32,
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            guild_versions: serde_json::json!({}),
            highest_last_message_id: "0".to_string(),
            read_state_version: 0,
            user_guild_settings_version: -1,
            user_settings_version: -1,
            private_channels_version: "0".to_string(),
            api_code_version: 0,
        }
    }
}

/// Resume payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumePayload {
    pub token: String,
    pub session_id: String,
    pub seq: u64,
}

/// Gateway opcodes
///
/// Covers all documented Discord Gateway v10 opcodes plus undocumented
/// user-client ones (Op 14 Lazy Guild, Op 31 Request Soundboard Sounds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    /// 0 — Dispatch (receive): an event was dispatched
    Dispatch = 0,
    /// 1 — Heartbeat (send/receive): keep the connection alive
    Heartbeat = 1,
    /// 2 — Identify (send): start a new session
    Identify = 2,
    /// 3 — Presence Update (send): update the client's presence
    PresenceUpdate = 3,
    /// 4 — Voice State Update (send): join/move/leave voice
    VoiceStateUpdate = 4,
    /// 6 — Resume (send): resume a previous session
    Resume = 6,
    /// 7 — Reconnect (receive): server asks client to reconnect
    Reconnect = 7,
    /// 8 — Request Guild Members (send): request members for a guild
    RequestGuildMembers = 8,
    /// 9 — Invalid Session (receive): session has been invalidated
    InvalidSession = 9,
    /// 10 — Hello (receive): sent after connecting, contains heartbeat_interval
    Hello = 10,
    /// 11 — Heartbeat ACK (receive): confirms heartbeat was received
    HeartbeatAck = 11,
    /// 14 — Lazy Guild (send): request lazy guild member list (user-client)
    LazyGuild = 14,
    /// 31 — Request Soundboard Sounds (send): request soundboard sounds for guilds
    RequestSoundboardSounds = 31,
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Dispatch),
            1 => Ok(Self::Heartbeat),
            2 => Ok(Self::Identify),
            3 => Ok(Self::PresenceUpdate),
            4 => Ok(Self::VoiceStateUpdate),
            6 => Ok(Self::Resume),
            7 => Ok(Self::Reconnect),
            8 => Ok(Self::RequestGuildMembers),
            9 => Ok(Self::InvalidSession),
            10 => Ok(Self::Hello),
            11 => Ok(Self::HeartbeatAck),
            14 => Ok(Self::LazyGuild),
            31 => Ok(Self::RequestSoundboardSounds),
            _ => Err(()),
        }
    }
}

/// Gateway close codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseCode {
    UnknownError = 4000,
    UnknownOpcode = 4001,
    DecodeError = 4002,
    NotAuthenticated = 4003,
    AuthenticationFailed = 4004,
    AlreadyAuthenticated = 4005,
    InvalidSeq = 4007,
    RateLimited = 4008,
    SessionTimedOut = 4009,
    InvalidShard = 4010,
    ShardingRequired = 4011,
    InvalidApiVersion = 4012,
    InvalidIntents = 4013,
    DisallowedIntents = 4014,
}

impl CloseCode {
    /// Check if this close code allows reconnection
    pub fn can_reconnect(&self) -> bool {
        !matches!(
            self,
            Self::AuthenticationFailed
                | Self::InvalidShard
                | Self::ShardingRequired
                | Self::InvalidApiVersion
                | Self::InvalidIntents
                | Self::DisallowedIntents
        )
    }
}

impl TryFrom<u16> for CloseCode {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            4000 => Ok(Self::UnknownError),
            4001 => Ok(Self::UnknownOpcode),
            4002 => Ok(Self::DecodeError),
            4003 => Ok(Self::NotAuthenticated),
            4004 => Ok(Self::AuthenticationFailed),
            4005 => Ok(Self::AlreadyAuthenticated),
            4007 => Ok(Self::InvalidSeq),
            4008 => Ok(Self::RateLimited),
            4009 => Ok(Self::SessionTimedOut),
            4010 => Ok(Self::InvalidShard),
            4011 => Ok(Self::ShardingRequired),
            4012 => Ok(Self::InvalidApiVersion),
            4013 => Ok(Self::InvalidIntents),
            4014 => Ok(Self::DisallowedIntents),
            _ => Err(()),
        }
    }
}
