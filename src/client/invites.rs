// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord invite API endpoints

use super::{DiscordClient, User};
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Invite object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invite {
    pub code: String,
    pub guild: Option<InviteGuild>,
    pub channel: Option<InviteChannel>,
    pub inviter: Option<User>,
    pub target_type: Option<u8>,
    pub target_user: Option<User>,
    pub approximate_presence_count: Option<u32>,
    pub approximate_member_count: Option<u32>,
    pub expires_at: Option<String>,
    /// Only present on detailed invites
    pub uses: Option<u32>,
    pub max_uses: Option<u32>,
    pub max_age: Option<u32>,
    pub temporary: Option<bool>,
    pub created_at: Option<String>,
}

/// Simplified guild in invite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteGuild {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub splash: Option<String>,
    pub banner: Option<String>,
    pub description: Option<String>,
    pub features: Vec<String>,
    pub verification_level: Option<u8>,
    pub vanity_url_code: Option<String>,
}

/// Simplified channel in invite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteChannel {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub channel_type: u8,
}

/// Create invite request
#[derive(Debug, Clone, Serialize)]
pub struct CreateInviteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique: Option<bool>,
}

impl Default for CreateInviteRequest {
    fn default() -> Self {
        Self {
            max_age: Some(86400), // 24 hours
            max_uses: Some(0),    // unlimited
            temporary: Some(false),
            unique: Some(false),
        }
    }
}

impl DiscordClient {
    /// Get an invite by code
    pub async fn get_invite(&self, code: &str) -> Result<Invite> {
        let response = self
            .get(&format!(
                "/invites/{}?with_counts=true&with_expiration=true",
                code
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let invite: Invite = serde_json::from_str(&body)?;
            Ok(invite)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get invite: {}",
                response.status()
            )))
        }
    }

    /// Accept/use an invite (join a guild) — user account only
    pub async fn accept_invite(&self, code: &str) -> Result<Invite> {
        let response = self
            .post(&format!("/invites/{}", code), &serde_json::json!({}))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let invite: Invite = serde_json::from_str(&body)?;
            Ok(invite)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to accept invite: {}",
                response.status()
            )))
        }
    }

    /// Delete an invite
    pub async fn delete_invite(&self, code: &str) -> Result<Invite> {
        let response = self.delete(&format!("/invites/{}", code)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let invite: Invite = serde_json::from_str(&body)?;
            Ok(invite)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to delete invite: {}",
                response.status()
            )))
        }
    }

    /// Get invites for a channel
    pub async fn get_channel_invites(&self, channel_id: &str) -> Result<Vec<Invite>> {
        let response = self
            .get(&format!("/channels/{}/invites", channel_id))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let invites: Vec<Invite> = serde_json::from_str(&body)?;
            Ok(invites)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get channel invites: {}",
                response.status()
            )))
        }
    }

    /// Create an invite for a channel
    pub async fn create_invite(
        &self,
        channel_id: &str,
        request: &CreateInviteRequest,
    ) -> Result<Invite> {
        let response = self
            .post(&format!("/channels/{}/invites", channel_id), request)
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let invite: Invite = serde_json::from_str(&body)?;
            Ok(invite)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to create invite: {}",
                response.status()
            )))
        }
    }

    /// Get all invites for a guild
    pub async fn get_guild_invites(&self, guild_id: &str) -> Result<Vec<Invite>> {
        let response = self.get(&format!("/guilds/{}/invites", guild_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let invites: Vec<Invite> = serde_json::from_str(&body)?;
            Ok(invites)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get guild invites: {}",
                response.status()
            )))
        }
    }
}
