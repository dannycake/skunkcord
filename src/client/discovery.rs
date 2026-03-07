// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Server discovery and guild widget endpoints

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Guild widget settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildWidget {
    pub enabled: bool,
    pub channel_id: Option<String>,
}

/// Guild widget data (public info)
#[derive(Debug, Clone, Deserialize)]
pub struct GuildWidgetData {
    pub id: String,
    pub name: String,
    pub instant_invite: Option<String>,
    pub channels: Vec<WidgetChannel>,
    pub members: Vec<WidgetMember>,
    pub presence_count: u32,
}

/// Channel in widget data
#[derive(Debug, Clone, Deserialize)]
pub struct WidgetChannel {
    pub id: String,
    pub name: String,
    pub position: i32,
}

/// Member in widget data
#[derive(Debug, Clone, Deserialize)]
pub struct WidgetMember {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub status: String,
    pub avatar_url: String,
}

/// Guild preview (for discoverable guilds)
#[derive(Debug, Clone, Deserialize)]
pub struct GuildPreview {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub splash: Option<String>,
    pub discovery_splash: Option<String>,
    pub description: Option<String>,
    pub features: Vec<String>,
    pub approximate_member_count: u32,
    pub approximate_presence_count: u32,
    pub emojis: Vec<serde_json::Value>,
    pub stickers: Vec<serde_json::Value>,
}

impl DiscordClient {
    /// Get guild widget settings
    pub async fn get_guild_widget_settings(&self, guild_id: &str) -> Result<GuildWidget> {
        let response = self.get(&format!("/guilds/{}/widget", guild_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let widget: GuildWidget = serde_json::from_str(&body)?;
            Ok(widget)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get widget settings: {}",
                response.status()
            )))
        }
    }

    /// Get guild widget data (public, no auth required)
    pub async fn get_guild_widget_data(&self, guild_id: &str) -> Result<GuildWidgetData> {
        let response = self
            .get(&format!("/guilds/{}/widget.json", guild_id))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let data: GuildWidgetData = serde_json::from_str(&body)?;
            Ok(data)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get widget data: {}",
                response.status()
            )))
        }
    }

    /// Get guild preview (for discoverable/lurkable guilds)
    pub async fn get_guild_preview(&self, guild_id: &str) -> Result<GuildPreview> {
        let response = self.get(&format!("/guilds/{}/preview", guild_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let preview: GuildPreview = serde_json::from_str(&body)?;
            Ok(preview)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get guild preview: {}",
                response.status()
            )))
        }
    }
}
