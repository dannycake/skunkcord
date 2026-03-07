// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord thread API endpoints

use super::{Channel, DiscordClient};
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Thread member object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMember {
    pub id: Option<String>,
    pub user_id: Option<String>,
    pub join_timestamp: String,
    pub flags: u32,
}

/// Thread list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadListResponse {
    pub threads: Vec<Channel>,
    pub members: Vec<ThreadMember>,
    pub has_more: Option<bool>,
}

/// Create thread request
#[derive(Debug, Clone, Serialize)]
pub struct CreateThreadRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_archive_duration: Option<u32>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub thread_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_user: Option<u32>,
}

impl DiscordClient {
    /// Create a thread from a message
    pub async fn create_thread_from_message(
        &self,
        channel_id: &str,
        message_id: &str,
        name: &str,
        auto_archive_duration: Option<u32>,
    ) -> Result<Channel> {
        let body = CreateThreadRequest {
            name: name.to_string(),
            auto_archive_duration,
            thread_type: None,
            invitable: None,
            rate_limit_per_user: None,
        };
        let response = self
            .post(
                &format!("/channels/{}/messages/{}/threads", channel_id, message_id),
                &body,
            )
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let channel: Channel = serde_json::from_str(&body)?;
            Ok(channel)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to create thread: {}",
                response.status()
            )))
        }
    }

    /// Create a thread without a message (standalone)
    pub async fn create_thread(
        &self,
        channel_id: &str,
        request: &CreateThreadRequest,
    ) -> Result<Channel> {
        let response = self
            .post(&format!("/channels/{}/threads", channel_id), request)
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let channel: Channel = serde_json::from_str(&body)?;
            Ok(channel)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to create thread: {}",
                response.status()
            )))
        }
    }

    /// Join a thread
    pub async fn join_thread(&self, channel_id: &str) -> Result<()> {
        let response = self
            .put(
                &format!("/channels/{}/thread-members/@me", channel_id),
                &serde_json::json!({}),
            )
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to join thread: {}",
                response.status()
            )))
        }
    }

    /// Leave a thread
    pub async fn leave_thread(&self, channel_id: &str) -> Result<()> {
        let response = self
            .delete(&format!("/channels/{}/thread-members/@me", channel_id))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to leave thread: {}",
                response.status()
            )))
        }
    }

    /// List thread members
    pub async fn list_thread_members(&self, channel_id: &str) -> Result<Vec<ThreadMember>> {
        let response = self
            .get(&format!("/channels/{}/thread-members", channel_id))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let members: Vec<ThreadMember> = serde_json::from_str(&body)?;
            Ok(members)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to list thread members: {}",
                response.status()
            )))
        }
    }

    /// List active threads in a guild
    pub async fn list_active_threads(&self, guild_id: &str) -> Result<ThreadListResponse> {
        let response = self
            .get(&format!("/guilds/{}/threads/active", guild_id))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let list: ThreadListResponse = serde_json::from_str(&body)?;
            Ok(list)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to list active threads: {}",
                response.status()
            )))
        }
    }

    /// List public archived threads in a channel
    pub async fn list_public_archived_threads(
        &self,
        channel_id: &str,
    ) -> Result<ThreadListResponse> {
        let response = self
            .get(&format!("/channels/{}/threads/archived/public", channel_id))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let list: ThreadListResponse = serde_json::from_str(&body)?;
            Ok(list)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to list archived threads: {}",
                response.status()
            )))
        }
    }

    /// List private archived threads in a channel
    pub async fn list_private_archived_threads(
        &self,
        channel_id: &str,
    ) -> Result<ThreadListResponse> {
        let response = self
            .get(&format!(
                "/channels/{}/threads/archived/private",
                channel_id
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let list: ThreadListResponse = serde_json::from_str(&body)?;
            Ok(list)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to list private archived threads: {}",
                response.status()
            )))
        }
    }
}
