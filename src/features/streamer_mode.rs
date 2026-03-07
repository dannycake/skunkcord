// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Streamer Mode — hide sensitive info when streaming
//!
//! Detects streaming software (OBS, Streamlabs) and automatically
//! hides personal information to prevent accidental leaks on stream.
//! Can also be manually toggled.

use serde::{Deserialize, Serialize};

/// Streamer mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamerModeConfig {
    /// Auto-detect streaming software
    pub auto_detect: bool,
    /// Manually enabled (overrides auto-detect)
    pub manually_enabled: bool,
    /// Hide personal info (email, phone, connected accounts)
    pub hide_personal_info: bool,
    /// Hide invite links
    pub hide_invite_links: bool,
    /// Disable notification popups
    pub disable_notifications: bool,
    /// Disable notification sounds
    pub disable_sounds: bool,
    /// Hide DM message content in notifications
    pub hide_dm_content: bool,
}

impl Default for StreamerModeConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            manually_enabled: false,
            hide_personal_info: true,
            hide_invite_links: true,
            disable_notifications: true,
            disable_sounds: true,
            hide_dm_content: true,
        }
    }
}

impl StreamerModeConfig {
    /// Check if streamer mode is currently active
    /// (either manually enabled or auto-detected)
    pub fn is_active(&self, streaming_detected: bool) -> bool {
        self.manually_enabled || (self.auto_detect && streaming_detected)
    }

    /// Process names to look for when auto-detecting streaming
    pub fn streaming_process_names() -> &'static [&'static str] {
        &[
            "obs",
            "obs64",
            "obs-studio",
            "streamlabs",
            "streamlabs obs",
            "xsplit",
            "xsplit.core",
            "wirecast",
            "twitchstudio",
            "twitch studio",
            "nvidia broadcast",
            "prism live studio",
        ]
    }

    /// Sanitize a string by replacing sensitive info with asterisks
    pub fn redact_email(email: &str) -> String {
        if let Some(at_pos) = email.find('@') {
            let local = &email[..at_pos];
            let domain = &email[at_pos..];
            if local.len() <= 2 {
                format!("**{}", domain)
            } else {
                format!("{}***{}", &local[..1], domain)
            }
        } else {
            "***".to_string()
        }
    }

    /// Redact an invite link
    pub fn redact_invite(text: &str) -> String {
        let re =
            regex::Regex::new(r"(?:https?://)?(?:discord\.gg|discord\.com/invite)/\S+").unwrap();
        re.replace_all(text, "[invite hidden]").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_manual() {
        let config = StreamerModeConfig {
            manually_enabled: true,
            ..Default::default()
        };
        assert!(config.is_active(false));
    }

    #[test]
    fn test_active_auto_detect() {
        let config = StreamerModeConfig::default();
        assert!(!config.is_active(false));
        assert!(config.is_active(true));
    }

    #[test]
    fn test_redact_email() {
        assert_eq!(
            StreamerModeConfig::redact_email("john@example.com"),
            "j***@example.com"
        );
        assert_eq!(
            StreamerModeConfig::redact_email("ab@test.com"),
            "**@test.com"
        );
    }

    #[test]
    fn test_redact_invite() {
        let text = "Join us at https://discord.gg/abc123 and discord.com/invite/xyz";
        let redacted = StreamerModeConfig::redact_invite(text);
        assert!(!redacted.contains("abc123"));
        assert!(!redacted.contains("xyz"));
        assert!(redacted.contains("[invite hidden]"));
    }
}
