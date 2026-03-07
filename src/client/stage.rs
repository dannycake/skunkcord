// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Stage Channel (Stage Instance) API
//!
//! Stage channels are special voice channels for large audiences.

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Stage instance object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageInstance {
    pub id: String,
    pub guild_id: String,
    pub channel_id: String,
    pub topic: String,
    pub privacy_level: u8,
    pub discoverable_disabled: Option<bool>,
    pub guild_scheduled_event_id: Option<String>,
}

/// Stage privacy levels
pub mod stage_privacy {
    pub const PUBLIC: u8 = 1;
    pub const GUILD_ONLY: u8 = 2;
}

/// Create stage instance request
#[derive(Debug, Clone, Serialize)]
pub struct CreateStageInstance {
    pub channel_id: String,
    pub topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_level: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_start_notification: Option<bool>,
}

impl DiscordClient {
    /// Create a stage instance (go live in a stage channel)
    pub async fn create_stage_instance(
        &self,
        request: &CreateStageInstance,
    ) -> Result<StageInstance> {
        let response = self.post("/stage-instances", request).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let instance: StageInstance = serde_json::from_str(&body)?;
            Ok(instance)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to create stage instance: {}",
                response.status()
            )))
        }
    }

    /// Get a stage instance by channel ID
    pub async fn get_stage_instance(&self, channel_id: &str) -> Result<StageInstance> {
        let response = self
            .get(&format!("/stage-instances/{}", channel_id))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let instance: StageInstance = serde_json::from_str(&body)?;
            Ok(instance)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get stage instance: {}",
                response.status()
            )))
        }
    }

    /// Delete (end) a stage instance
    pub async fn delete_stage_instance(&self, channel_id: &str) -> Result<()> {
        let response = self
            .delete(&format!("/stage-instances/{}", channel_id))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to delete stage instance: {}",
                response.status()
            )))
        }
    }

    /// Request to speak in a stage channel
    /// Sets the user's voice state to request speaking permission
    pub async fn request_to_speak(&self, guild_id: &str) -> Result<()> {
        let body = serde_json::json!({
            "channel_id": null,
            "request_to_speak_timestamp": chrono::Utc::now().to_rfc3339(),
        });
        let response = self
            .patch(&format!("/guilds/{}/voice-states/@me", guild_id), &body)
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to request to speak: {}",
                response.status()
            )))
        }
    }
}
