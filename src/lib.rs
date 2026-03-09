// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Skunkcord Client Library
//!
//! A Discord user-account client using Qt for UI and Rust for the backend,
//! with browser emulation to avoid detection.
//!
//! ## Features
//!
//! - Browser fingerprint emulation (Chrome)
//! - WebSocket gateway connection for real-time events
//! - Moderation actions (kick, ban, timeout, delete messages)
//! - User profiles and connections
//! - Guild and channel management
//! - SOCKS5 proxy support with Mullvad server selection
//! - Telemetry endpoint blocking (same as Vencord/BetterDiscord)
//! - Automatic rate-limit retry with backoff
//! - Feature flags with safety profiles

pub mod app_runner;
pub mod bridge;
pub mod build_number;
pub mod captcha;
pub mod client;
pub mod features;
pub mod fingerprint;
pub mod gateway;
pub mod input;
pub mod mobile_ffi;
pub mod proxy;
pub mod rendering;
pub mod security;
pub mod storage;
#[cfg(feature = "desktop")]
pub mod ui;
pub mod voice;

pub mod plugins;

use thiserror::Error;

/// Main error type for the Discord client
#[derive(Error, Debug)]
pub enum DiscordError {
    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Gateway error: {0}")]
    Gateway(String),

    #[error("Rate limited: retry after {0}ms")]
    RateLimited(u64),

    #[error("Session expired")]
    SessionExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Missing permissions: {0}")]
    MissingPermissions(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Proxy error: {0}")]
    Proxy(String),

    #[error("Telemetry endpoint blocked")]
    TelemetryBlocked,

    #[error("Captcha required")]
    CaptchaRequired(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DiscordError>;

/// Discord API version
pub const API_VERSION: u8 = 10;

/// Discord Gateway version
pub const GATEWAY_VERSION: u8 = 10;
