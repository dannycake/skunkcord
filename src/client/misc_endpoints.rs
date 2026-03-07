// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Miscellaneous API endpoints
//!
//! Smaller endpoints that don't warrant their own module:
//! - Channel following (announcement subscriptions)
//! - Scheduled event users
//! - Application info
//! - Voice regions

use super::{DiscordClient, User};
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Voice region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceRegion {
    pub id: String,
    pub name: String,
    pub optimal: bool,
    pub deprecated: bool,
    pub custom: bool,
}

/// Application info (from /oauth2/@me or /applications/@me)
#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationInfo {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub description: String,
    pub bot_public: Option<bool>,
    pub bot_require_code_grant: Option<bool>,
    pub owner: Option<User>,
    pub flags: Option<u64>,
}

/// Followed channel
#[derive(Debug, Clone, Deserialize)]
pub struct FollowedChannel {
    pub channel_id: String,
    pub webhook_id: String,
}

/// Scheduled event user
#[derive(Debug, Clone, Deserialize)]
pub struct ScheduledEventUser {
    pub guild_scheduled_event_id: String,
    pub user: User,
    pub member: Option<serde_json::Value>,
}

impl DiscordClient {
    /// Get available voice regions
    pub async fn get_voice_regions(&self) -> Result<Vec<VoiceRegion>> {
        let response = self.get("/voice/regions").await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let regions: Vec<VoiceRegion> = serde_json::from_str(&body)?;
            Ok(regions)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get voice regions: {}",
                response.status()
            )))
        }
    }

    /// Follow an announcement channel (cross-post to another channel)
    pub async fn follow_channel(
        &self,
        channel_id: &str,
        target_channel_id: &str,
    ) -> Result<FollowedChannel> {
        let body = serde_json::json!({ "webhook_channel_id": target_channel_id });
        let response = self
            .post(&format!("/channels/{}/followers", channel_id), &body)
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let followed: FollowedChannel = serde_json::from_str(&body)?;
            Ok(followed)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to follow channel: {}",
                response.status()
            )))
        }
    }

    /// Crosspost a message (publish from announcement channel)
    pub async fn crosspost_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        let response = self
            .post(
                &format!("/channels/{}/messages/{}/crosspost", channel_id, message_id),
                &serde_json::json!({}),
            )
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to crosspost message: {}",
                response.status()
            )))
        }
    }

    /// Get users interested in a scheduled event
    pub async fn get_scheduled_event_users(
        &self,
        guild_id: &str,
        event_id: &str,
        limit: Option<u8>,
    ) -> Result<Vec<ScheduledEventUser>> {
        let limit = limit.unwrap_or(100);
        let response = self
            .get(&format!(
                "/guilds/{}/scheduled-events/{}/users?limit={}&with_member=true",
                guild_id, event_id, limit
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let users: Vec<ScheduledEventUser> = serde_json::from_str(&body)?;
            Ok(users)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get event users: {}",
                response.status()
            )))
        }
    }

    /// Get guild voice regions (may include VIP regions)
    pub async fn get_guild_voice_regions(&self, guild_id: &str) -> Result<Vec<VoiceRegion>> {
        let response = self.get(&format!("/guilds/{}/regions", guild_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let regions: Vec<VoiceRegion> = serde_json::from_str(&body)?;
            Ok(regions)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get guild voice regions: {}",
                response.status()
            )))
        }
    }

    /// Group DM: add a recipient
    pub async fn group_dm_add_recipient(&self, channel_id: &str, user_id: &str) -> Result<()> {
        let response = self
            .put(
                &format!("/channels/{}/recipients/{}", channel_id, user_id),
                &serde_json::json!({}),
            )
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to add DM recipient: {}",
                response.status()
            )))
        }
    }

    /// Group DM: remove a recipient
    pub async fn group_dm_remove_recipient(&self, channel_id: &str, user_id: &str) -> Result<()> {
        let response = self
            .delete(&format!("/channels/{}/recipients/{}", channel_id, user_id))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to remove DM recipient: {}",
                response.status()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_region_deserialize() {
        let json = r#"{"id": "us-east", "name": "US East", "optimal": true, "deprecated": false, "custom": false}"#;
        let region: VoiceRegion = serde_json::from_str(json).unwrap();
        assert_eq!(region.id, "us-east");
        assert!(region.optimal);
        assert!(!region.deprecated);
    }
}
