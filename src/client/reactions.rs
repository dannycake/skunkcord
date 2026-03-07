// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord reaction API endpoints

use super::{DiscordClient, User};
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Reaction object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub count: u32,
    pub me: bool,
    pub emoji: ReactionEmoji,
}

/// Emoji used in reactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionEmoji {
    pub id: Option<String>,
    pub name: Option<String>,
    pub animated: Option<bool>,
}

impl ReactionEmoji {
    /// Format emoji for use in API URL paths
    /// Unicode emoji: URL-encoded directly
    /// Custom emoji: name:id
    pub fn to_api_string(&self) -> String {
        if let Some(ref id) = self.id {
            format!("{}:{}", self.name.as_deref().unwrap_or("_"), id)
        } else {
            // Unicode emoji — needs URL encoding
            url::form_urlencoded::byte_serialize(self.name.as_deref().unwrap_or("").as_bytes())
                .collect()
        }
    }

    /// Display string for UI: unicode emoji as-is (name is the character), custom as :name:id
    pub fn display_string(&self) -> String {
        if let Some(ref id) = self.id {
            format!(
                "{}:{}",
                self.name.as_deref().unwrap_or("_"),
                id
            )
        } else {
            self.name.as_deref().unwrap_or("").to_string()
        }
    }
}

impl DiscordClient {
    /// Add a reaction to a message
    pub async fn add_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> Result<()> {
        let encoded_emoji =
            url::form_urlencoded::byte_serialize(emoji.as_bytes()).collect::<String>();
        let response = self
            .put(
                &format!(
                    "/channels/{}/messages/{}/reactions/{}/@me",
                    channel_id, message_id, encoded_emoji
                ),
                &serde_json::json!({}),
            )
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to add reaction: {}",
                response.status()
            )))
        }
    }

    /// Remove own reaction from a message
    pub async fn remove_own_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> Result<()> {
        let encoded_emoji =
            url::form_urlencoded::byte_serialize(emoji.as_bytes()).collect::<String>();
        let response = self
            .delete(&format!(
                "/channels/{}/messages/{}/reactions/{}/@me",
                channel_id, message_id, encoded_emoji
            ))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to remove reaction: {}",
                response.status()
            )))
        }
    }

    /// Remove another user's reaction from a message
    pub async fn remove_user_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
        user_id: &str,
    ) -> Result<()> {
        let encoded_emoji =
            url::form_urlencoded::byte_serialize(emoji.as_bytes()).collect::<String>();
        let response = self
            .delete(&format!(
                "/channels/{}/messages/{}/reactions/{}/{}",
                channel_id, message_id, encoded_emoji, user_id
            ))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to remove user reaction: {}",
                response.status()
            )))
        }
    }

    /// Get users who reacted with a specific emoji
    pub async fn get_reactions(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
        limit: Option<u8>,
    ) -> Result<Vec<User>> {
        let encoded_emoji =
            url::form_urlencoded::byte_serialize(emoji.as_bytes()).collect::<String>();
        let limit = limit.unwrap_or(25).min(100);
        let response = self
            .get(&format!(
                "/channels/{}/messages/{}/reactions/{}?limit={}",
                channel_id, message_id, encoded_emoji, limit
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let users: Vec<User> = serde_json::from_str(&body)?;
            Ok(users)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get reactions: {}",
                response.status()
            )))
        }
    }

    /// Delete all reactions on a message
    pub async fn delete_all_reactions(&self, channel_id: &str, message_id: &str) -> Result<()> {
        let response = self
            .delete(&format!(
                "/channels/{}/messages/{}/reactions",
                channel_id, message_id
            ))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to delete all reactions: {}",
                response.status()
            )))
        }
    }

    /// Delete all reactions for a specific emoji on a message
    pub async fn delete_all_reactions_for_emoji(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> Result<()> {
        let encoded_emoji =
            url::form_urlencoded::byte_serialize(emoji.as_bytes()).collect::<String>();
        let response = self
            .delete(&format!(
                "/channels/{}/messages/{}/reactions/{}",
                channel_id, message_id, encoded_emoji
            ))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to delete reactions for emoji: {}",
                response.status()
            )))
        }
    }
}
