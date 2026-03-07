// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Forum Channel API
//!
//! Forum channels are special channels where each "message" creates a thread.
//! They have tags, default sort order, and post creation with initial message.

use super::{Channel, DiscordClient, Message};
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Forum channel types
pub const CHANNEL_TYPE_GUILD_FORUM: u8 = 15;
pub const CHANNEL_TYPE_GUILD_MEDIA: u8 = 16;

/// Forum tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumTag {
    pub id: String,
    pub name: String,
    pub moderated: bool,
    pub emoji_id: Option<String>,
    pub emoji_name: Option<String>,
}

/// Default sort order for forum posts
pub mod sort_order {
    /// Sort by latest activity
    pub const LATEST_ACTIVITY: u8 = 0;
    /// Sort by creation date
    pub const CREATION_DATE: u8 = 1;
}

/// Forum layout type
pub mod layout_type {
    /// Not set
    pub const NOT_SET: u8 = 0;
    /// List view
    pub const LIST_VIEW: u8 = 1;
    /// Gallery view
    pub const GALLERY_VIEW: u8 = 2;
}

/// Create a forum post (thread with initial message)
#[derive(Debug, Clone, Serialize)]
pub struct CreateForumPost {
    /// Post title (thread name)
    pub name: String,
    /// Auto-archive duration in minutes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_archive_duration: Option<u32>,
    /// Rate limit per user (slow mode) in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_user: Option<u32>,
    /// Initial message content
    pub message: ForumPostMessage,
    /// Applied tag IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_tags: Option<Vec<String>>,
}

/// The initial message of a forum post
#[derive(Debug, Clone, Serialize)]
pub struct ForumPostMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
}

/// Forum post (thread) with its initial message
#[derive(Debug, Clone, Deserialize)]
pub struct ForumPostResponse {
    #[serde(flatten)]
    pub channel: Channel,
    pub message: Option<Message>,
}

impl DiscordClient {
    /// Create a forum post (starts a thread with an initial message)
    pub async fn create_forum_post(
        &self,
        channel_id: &str,
        post: &CreateForumPost,
    ) -> Result<Channel> {
        let response = self
            .post(&format!("/channels/{}/threads", channel_id), post)
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
                "Failed to create forum post: {}",
                response.status()
            )))
        }
    }

    /// Check if a channel is a forum channel
    pub fn is_forum_channel(channel_type: u8) -> bool {
        channel_type == CHANNEL_TYPE_GUILD_FORUM || channel_type == CHANNEL_TYPE_GUILD_MEDIA
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_forum_channel() {
        assert!(DiscordClient::is_forum_channel(CHANNEL_TYPE_GUILD_FORUM));
        assert!(DiscordClient::is_forum_channel(CHANNEL_TYPE_GUILD_MEDIA));
        assert!(!DiscordClient::is_forum_channel(0)); // text
        assert!(!DiscordClient::is_forum_channel(2)); // voice
    }
}
