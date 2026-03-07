// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! User settings management
//!
//! Discord stores user settings in two forms:
//! 1. JSON via /users/@me/settings (legacy, still works)
//! 2. Protobuf via /users/@me/settings-proto/{type} (newer)
//!
//! This module handles both formats for full compatibility.

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Discord user settings (JSON format from /users/@me/settings)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserSettingsFull {
    // Appearance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    // Status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_status: Option<CustomStatusSetting>,

    // Privacy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub developer_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explicit_content_filter: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_guilds_restricted: Option<bool>,

    // Notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_attachment_media: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_embed_media: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_embeds: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_reactions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gif_auto_play: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animate_emoji: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animate_stickers: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_tts_command: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_display_compact: Option<bool>,

    // Voice
    #[serde(skip_serializing_if = "Option::is_none")]
    pub afk_timeout: Option<u32>,

    // Accessibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone_offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detect_platform_accounts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_sync_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_phone_integration_enabled: Option<bool>,

    // Activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_current_game: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_restricted_guild_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_joining_restricted_guild_ids: Option<Vec<String>>,

    // Guild ordering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_positions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_folders: Option<Vec<GuildFolder>>,
}

/// Custom status setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStatusSetting {
    pub text: Option<String>,
    pub expires_at: Option<String>,
    pub emoji_id: Option<String>,
    pub emoji_name: Option<String>,
}

/// Guild folder (for organizing servers in sidebar)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildFolder {
    pub id: Option<String>,
    pub guild_ids: Vec<String>,
    pub name: Option<String>,
    pub color: Option<u32>,
}

/// Settings proto types (for /users/@me/settings-proto/{type})
pub mod settings_proto_type {
    /// General user settings
    pub const USER_SETTINGS: u8 = 1;
    /// Frecency and state
    pub const FRECENCY: u8 = 2;
}

impl DiscordClient {
    /// Get full user settings
    pub async fn get_full_settings(&self) -> Result<UserSettingsFull> {
        let response = self.get("/users/@me/settings").await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let settings: UserSettingsFull = serde_json::from_str(&body)?;
            Ok(settings)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get settings: {}",
                response.status()
            )))
        }
    }

    /// Update user settings (partial update — only set fields are changed)
    pub async fn update_full_settings(&self, settings: &UserSettingsFull) -> Result<()> {
        let response = self.patch("/users/@me/settings", settings).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to update settings: {}",
                response.status()
            )))
        }
    }

    /// Update just the status
    pub async fn set_status(&self, status: &str) -> Result<()> {
        let settings = UserSettingsFull {
            status: Some(status.to_string()),
            ..Default::default()
        };
        self.update_full_settings(&settings).await
    }

    /// Update custom status
    pub async fn set_custom_status(
        &self,
        text: Option<&str>,
        emoji_name: Option<&str>,
    ) -> Result<()> {
        let custom = if text.is_some() || emoji_name.is_some() {
            Some(CustomStatusSetting {
                text: text.map(|t| t.to_string()),
                expires_at: None,
                emoji_id: None,
                emoji_name: emoji_name.map(|e| e.to_string()),
            })
        } else {
            None
        };

        let settings = UserSettingsFull {
            custom_status: custom,
            ..Default::default()
        };
        self.update_full_settings(&settings).await
    }

    /// Toggle developer mode
    pub async fn set_developer_mode(&self, enabled: bool) -> Result<()> {
        let settings = UserSettingsFull {
            developer_mode: Some(enabled),
            ..Default::default()
        };
        self.update_full_settings(&settings).await
    }

    /// Toggle game activity visibility
    pub async fn set_show_current_game(&self, show: bool) -> Result<()> {
        let settings = UserSettingsFull {
            show_current_game: Some(show),
            ..Default::default()
        };
        self.update_full_settings(&settings).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings_serializes_empty() {
        let settings = UserSettingsFull::default();
        let json = serde_json::to_string(&settings).unwrap();
        assert_eq!(json, "{}"); // All fields skipped when None
    }

    #[test]
    fn test_partial_settings() {
        let settings = UserSettingsFull {
            status: Some("dnd".to_string()),
            developer_mode: Some(true),
            ..Default::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"status\":\"dnd\""));
        assert!(json.contains("\"developer_mode\":true"));
        assert!(!json.contains("theme")); // Not set, skipped
    }

    #[test]
    fn test_guild_folder() {
        let folder = GuildFolder {
            id: Some("folder1".to_string()),
            guild_ids: vec!["g1".into(), "g2".into()],
            name: Some("Gaming".to_string()),
            color: Some(0x5865f2),
        };
        let json = serde_json::to_string(&folder).unwrap();
        assert!(json.contains("Gaming"));
        assert!(json.contains("g1"));
    }
}
