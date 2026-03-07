// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord Soundboard API
//!
//! Guild soundboard sounds that can be played in voice channels.

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Soundboard sound object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundboardSound {
    pub name: String,
    pub sound_id: String,
    pub volume: f64,
    pub emoji_id: Option<String>,
    pub emoji_name: Option<String>,
    pub guild_id: Option<String>,
    pub available: bool,
    pub user: Option<super::User>,
}

/// Default soundboard sounds (built into Discord)
#[derive(Debug, Clone, Deserialize)]
pub struct DefaultSoundsResponse {
    #[serde(default)]
    pub items: Vec<SoundboardSound>,
}

impl DiscordClient {
    /// List guild soundboard sounds
    pub async fn list_guild_sounds(&self, guild_id: &str) -> Result<Vec<SoundboardSound>> {
        let response = self
            .get(&format!("/guilds/{}/soundboard-sounds", guild_id))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let wrapper: serde_json::Value = serde_json::from_str(&body)?;
            if let Some(items) = wrapper.get("items") {
                let sounds: Vec<SoundboardSound> = serde_json::from_value(items.clone())?;
                return Ok(sounds);
            }
            Ok(vec![])
        } else {
            Err(DiscordError::Http(format!(
                "Failed to list sounds: {}",
                response.status()
            )))
        }
    }

    /// Get default (built-in) soundboard sounds
    pub async fn get_default_sounds(&self) -> Result<Vec<SoundboardSound>> {
        let response = self.get("/soundboard-default-sounds").await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let sounds: Vec<SoundboardSound> = serde_json::from_str(&body)?;
            Ok(sounds)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get default sounds: {}",
                response.status()
            )))
        }
    }

    /// Send a soundboard sound in a voice channel
    pub async fn send_soundboard_sound(
        &self,
        channel_id: &str,
        sound_id: &str,
        source_guild_id: Option<&str>,
    ) -> Result<()> {
        let mut body = serde_json::json!({ "sound_id": sound_id });
        if let Some(gid) = source_guild_id {
            body["source_guild_id"] = serde_json::json!(gid);
        }

        let response = self
            .post(
                &format!("/channels/{}/send-soundboard-sound", channel_id),
                &body,
            )
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to send sound: {}",
                response.status()
            )))
        }
    }
}
