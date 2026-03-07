// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Session management for Discord authentication

use crate::fingerprint::BrowserFingerprint;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete session data extracted from browser login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Discord authentication token
    pub token: String,
    /// User ID associated with the token
    pub user_id: String,
    /// Display name for account switcher UI (optional; set after login)
    #[serde(default)]
    pub username: Option<String>,
    /// Avatar URL for account switcher UI (optional; set after login)
    #[serde(default)]
    pub avatar_url: Option<String>,
    /// Cookies extracted from the browser
    pub cookies: HashMap<String, String>,
    /// Local storage data
    pub local_storage: HashMap<String, String>,
    /// Browser fingerprint used for this session
    pub fingerprint: BrowserFingerprint,
    /// Session creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last used timestamp
    pub last_used: chrono::DateTime<chrono::Utc>,
}

impl Session {
    /// Create a new session from extracted data
    pub fn new(
        token: String,
        user_id: String,
        cookies: HashMap<String, String>,
        local_storage: HashMap<String, String>,
        fingerprint: BrowserFingerprint,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            token,
            user_id,
            username: None,
            avatar_url: None,
            cookies,
            local_storage,
            fingerprint,
            created_at: now,
            last_used: now,
        }
    }

    /// Update the last used timestamp
    pub fn touch(&mut self) {
        self.last_used = chrono::Utc::now();
    }

    /// Check if the session is stale (not used in 7 days)
    pub fn is_stale(&self) -> bool {
        let now = chrono::Utc::now();
        let diff = now - self.last_used;
        diff.num_days() > 7
    }

    /// Check if the fingerprint's build number is still plausible.
    /// Discord updates the build number frequently — an old one is a detection signal.
    /// Returns true if the build number is within a reasonable range of current.
    pub fn is_fingerprint_plausible(&self) -> bool {
        let build = self.fingerprint.client_build_number;
        // Build numbers increase by ~100-500 per week
        // Allow up to ~5000 behind current (roughly 2-3 weeks)
        let min_plausible = 340000u64; // anything below this is ancient
        build >= min_plausible
    }

    /// Check if this session should have its fingerprint refreshed.
    /// Returns true if the session is older than 3 days (fingerprint may be stale).
    pub fn needs_fingerprint_refresh(&self) -> bool {
        let now = chrono::Utc::now();
        let age = now - self.created_at;
        age.num_days() > 3 || !self.is_fingerprint_plausible()
    }

    /// Get the __dcfduid cookie if present
    pub fn get_dcfduid(&self) -> Option<&str> {
        self.cookies.get("__dcfduid").map(|s| s.as_str())
    }

    /// Get the __sdcfduid cookie if present
    pub fn get_sdcfduid(&self) -> Option<&str> {
        self.cookies.get("__sdcfduid").map(|s| s.as_str())
    }

    /// Get the __cfruid cookie if present
    pub fn get_cfruid(&self) -> Option<&str> {
        self.cookies.get("__cfruid").map(|s| s.as_str())
    }
}

/// Credentials for email/password login (not recommended)
#[derive(Debug, Clone, Serialize)]
pub struct LoginCredentials {
    pub login: String,
    pub password: String,
    pub undelete: bool,
    pub login_source: Option<String>,
    pub gift_code_sku_id: Option<String>,
}

/// Response from login endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    pub token: Option<String>,
    pub user_id: Option<String>,
    pub mfa: Option<bool>,
    pub sms: Option<bool>,
    pub totp: Option<bool>,
    pub backup: Option<bool>,
    pub webauthn: Option<String>,
    pub ticket: Option<String>,
    pub login_instance_id: Option<String>,
    #[serde(default)]
    pub captcha_key: Option<Vec<String>>,
    pub captcha_sitekey: Option<String>,
    pub captcha_service: Option<String>,
    #[serde(default)]
    pub captcha_rqdata: Option<String>,
    #[serde(default)]
    pub captcha_rqtoken: Option<String>,
    #[serde(default)]
    pub captcha_session_id: Option<String>,
    pub user_settings: Option<serde_json::Value>,
}

/// Response from MFA verification (totp/sms/backup)
#[derive(Debug, Clone, Deserialize)]
pub struct MfaResponse {
    pub token: String,
    #[serde(default)]
    pub user_settings: Option<serde_json::Value>,
}

/// MFA verification request
#[derive(Debug, Clone, Serialize)]
pub struct MfaRequest {
    pub code: String,
    pub ticket: String,
    pub login_source: Option<String>,
    pub gift_code_sku_id: Option<String>,
    pub login_instance_id: Option<String>,
}

/// Token extracted from various sources
#[derive(Debug, Clone)]
pub struct ExtractedToken {
    pub token: String,
    pub source: TokenSource,
}

/// Source of the extracted token
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenSource {
    /// Extracted from localStorage
    LocalStorage,
    /// Extracted from network request intercept
    NetworkIntercept,
    /// Extracted from IndexedDB
    IndexedDB,
    /// Manually provided
    Manual,
}

impl ExtractedToken {
    pub fn new(token: String, source: TokenSource) -> Self {
        Self { token, source }
    }

    /// Validate token format
    pub fn is_valid_format(&self) -> bool {
        // Discord tokens have specific formats
        // User tokens: base64.timestamp.hmac (3 parts separated by dots)
        // Bot tokens: different format
        let parts: Vec<&str> = self.token.split('.').collect();
        if parts.len() != 3 {
            return false;
        }

        // First part should be base64 encoded user ID
        if base64::Engine::decode(&base64::engine::general_purpose::STANDARD, parts[0]).is_err() {
            return false;
        }

        true
    }

    /// Extract user ID from token
    pub fn extract_user_id(&self) -> Option<String> {
        let parts: Vec<&str> = self.token.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, parts[0])
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_staleness() {
        let fingerprint = BrowserFingerprint::default();
        let mut session = Session::new(
            "test_token".to_string(),
            "123456".to_string(),
            HashMap::new(),
            HashMap::new(),
            fingerprint,
        );

        assert!(!session.is_stale());
        session.touch();
        assert!(!session.is_stale());
    }
}
