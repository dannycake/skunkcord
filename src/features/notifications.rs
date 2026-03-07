// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Notification system — desktop notifications and sounds
//!
//! Handles notification display and sound playback for messages,
//! calls, and other Discord events. Respects per-channel/guild mute settings.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Notification event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationEvent {
    /// New message in a channel
    Message,
    /// Direct message
    DirectMessage,
    /// Mention (@user)
    Mention,
    /// Incoming voice/video call
    IncomingCall,
    /// User joined voice channel
    VoiceJoin,
    /// User left voice channel
    VoiceLeave,
    /// Someone started streaming
    StreamStart,
    /// Deafen toggle
    Deafen,
    /// Mute toggle
    Mute,
}

impl NotificationEvent {
    /// Get the default sound file name for this event
    pub fn default_sound(&self) -> &'static str {
        match self {
            Self::Message | Self::DirectMessage | Self::Mention => "message1",
            Self::IncomingCall => "call_ringing",
            Self::VoiceJoin => "user_join",
            Self::VoiceLeave => "user_leave",
            Self::StreamStart => "stream_started",
            Self::Deafen => "deafen",
            Self::Mute => "mute",
        }
    }
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Enable desktop notification popups
    pub enable_popups: bool,
    /// Enable notification sounds
    pub enable_sounds: bool,
    /// Sound volume (0.0 - 1.0)
    pub volume: f32,
    /// Muted channel IDs (no notifications)
    pub muted_channels: HashSet<String>,
    /// Muted guild IDs (no notifications)
    pub muted_guilds: HashSet<String>,
    /// Suppress @everyone and @here
    pub suppress_everyone: bool,
    /// Suppress role mentions
    pub suppress_roles: bool,
    /// Show message content in notification (vs just "New message")
    pub show_content_preview: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enable_popups: true,
            enable_sounds: true,
            volume: 0.5,
            muted_channels: HashSet::new(),
            muted_guilds: HashSet::new(),
            suppress_everyone: false,
            suppress_roles: false,
            show_content_preview: true,
        }
    }
}

impl NotificationConfig {
    /// Check if a channel should receive notifications
    pub fn should_notify_channel(&self, channel_id: &str, guild_id: Option<&str>) -> bool {
        if self.muted_channels.contains(channel_id) {
            return false;
        }
        if let Some(gid) = guild_id {
            if self.muted_guilds.contains(gid) {
                return false;
            }
        }
        true
    }

    /// Mute a channel
    pub fn mute_channel(&mut self, channel_id: &str) {
        self.muted_channels.insert(channel_id.to_string());
    }

    /// Unmute a channel
    pub fn unmute_channel(&mut self, channel_id: &str) {
        self.muted_channels.remove(channel_id);
    }

    /// Mute a guild
    pub fn mute_guild(&mut self, guild_id: &str) {
        self.muted_guilds.insert(guild_id.to_string());
    }

    /// Unmute a guild
    pub fn unmute_guild(&mut self, guild_id: &str) {
        self.muted_guilds.remove(guild_id);
    }

    /// Check if we should show content in the notification
    pub fn get_notification_body(&self, content: &str, channel_name: &str) -> String {
        if self.show_content_preview {
            // Truncate long messages
            if content.len() > 200 {
                format!("{}...", &content[..197])
            } else {
                content.to_string()
            }
        } else {
            format!("New message in #{}", channel_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_allows_all() {
        let config = NotificationConfig::default();
        assert!(config.should_notify_channel("any", Some("any_guild")));
    }

    #[test]
    fn test_mute_channel() {
        let mut config = NotificationConfig::default();
        config.mute_channel("ch1");
        assert!(!config.should_notify_channel("ch1", None));
        assert!(config.should_notify_channel("ch2", None));

        config.unmute_channel("ch1");
        assert!(config.should_notify_channel("ch1", None));
    }

    #[test]
    fn test_mute_guild() {
        let mut config = NotificationConfig::default();
        config.mute_guild("g1");
        assert!(!config.should_notify_channel("ch1", Some("g1")));
        assert!(config.should_notify_channel("ch1", Some("g2")));
        assert!(config.should_notify_channel("ch1", None)); // DMs unaffected
    }

    #[test]
    fn test_notification_body() {
        let config = NotificationConfig::default();
        assert_eq!(config.get_notification_body("Hello!", "general"), "Hello!");

        let mut no_preview = NotificationConfig::default();
        no_preview.show_content_preview = false;
        assert_eq!(
            no_preview.get_notification_body("Hello!", "general"),
            "New message in #general"
        );
    }

    #[test]
    fn test_long_content_truncated() {
        let config = NotificationConfig::default();
        let long = "a".repeat(300);
        let body = config.get_notification_body(&long, "ch");
        assert!(body.len() <= 203); // 197 + "..."
        assert!(body.ends_with("..."));
    }

    #[test]
    fn test_event_sounds() {
        assert_eq!(NotificationEvent::Message.default_sound(), "message1");
        assert_eq!(
            NotificationEvent::IncomingCall.default_sound(),
            "call_ringing"
        );
        assert_eq!(NotificationEvent::VoiceJoin.default_sound(), "user_join");
    }
}
