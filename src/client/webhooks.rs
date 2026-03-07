// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Webhook API endpoints
//!
//! Manage and execute webhooks for channels.

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Webhook object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: String,
    #[serde(rename = "type")]
    pub webhook_type: u8,
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub token: Option<String>,
    pub application_id: Option<String>,
    pub url: Option<String>,
}

/// Webhook types
pub mod webhook_type {
    /// Created by a guild member
    pub const INCOMING: u8 = 1;
    /// Channel Follower
    pub const CHANNEL_FOLLOWER: u8 = 2;
    /// Application (interactions)
    pub const APPLICATION: u8 = 3;
}

/// Create webhook request
#[derive(Debug, Clone, Serialize)]
pub struct CreateWebhook {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
}

/// Webhook message payload
#[derive(Debug, Clone, Serialize)]
pub struct WebhookMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<serde_json::Value>>,
}

impl DiscordClient {
    /// Get webhooks for a channel
    pub async fn get_channel_webhooks(&self, channel_id: &str) -> Result<Vec<Webhook>> {
        let response = self
            .get(&format!("/channels/{}/webhooks", channel_id))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let webhooks: Vec<Webhook> = serde_json::from_str(&body)?;
            Ok(webhooks)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get webhooks: {}",
                response.status()
            )))
        }
    }

    /// Get webhooks for a guild
    pub async fn get_guild_webhooks(&self, guild_id: &str) -> Result<Vec<Webhook>> {
        let response = self.get(&format!("/guilds/{}/webhooks", guild_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let webhooks: Vec<Webhook> = serde_json::from_str(&body)?;
            Ok(webhooks)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get guild webhooks: {}",
                response.status()
            )))
        }
    }

    /// Create a webhook
    pub async fn create_webhook(
        &self,
        channel_id: &str,
        request: &CreateWebhook,
    ) -> Result<Webhook> {
        let response = self
            .post(&format!("/channels/{}/webhooks", channel_id), request)
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let webhook: Webhook = serde_json::from_str(&body)?;
            Ok(webhook)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to create webhook: {}",
                response.status()
            )))
        }
    }

    /// Delete a webhook
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<()> {
        let response = self.delete(&format!("/webhooks/{}", webhook_id)).await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to delete webhook: {}",
                response.status()
            )))
        }
    }
}
