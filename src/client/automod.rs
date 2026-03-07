// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Auto Moderation API
//!
//! Create and manage auto-moderation rules for guilds.

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Auto moderation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoModRule {
    pub id: String,
    pub guild_id: String,
    pub name: String,
    pub creator_id: String,
    pub event_type: u8,
    pub trigger_type: u8,
    pub trigger_metadata: TriggerMetadata,
    pub actions: Vec<AutoModAction>,
    pub enabled: bool,
    pub exempt_roles: Vec<String>,
    pub exempt_channels: Vec<String>,
}

/// Trigger metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerMetadata {
    #[serde(default)]
    pub keyword_filter: Vec<String>,
    #[serde(default)]
    pub regex_patterns: Vec<String>,
    #[serde(default)]
    pub presets: Vec<u8>,
    #[serde(default)]
    pub allow_list: Vec<String>,
    pub mention_total_limit: Option<u8>,
    pub mention_raid_protection_enabled: Option<bool>,
}

/// Auto moderation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoModAction {
    #[serde(rename = "type")]
    pub action_type: u8,
    pub metadata: Option<ActionMetadata>,
}

/// Action metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionMetadata {
    pub channel_id: Option<String>,
    pub duration_seconds: Option<u64>,
    pub custom_message: Option<String>,
}

/// Trigger types
pub mod trigger_type {
    pub const KEYWORD: u8 = 1;
    pub const SPAM: u8 = 3;
    pub const KEYWORD_PRESET: u8 = 4;
    pub const MENTION_SPAM: u8 = 5;
}

/// Action types
pub mod action_type {
    pub const BLOCK_MESSAGE: u8 = 1;
    pub const SEND_ALERT_MESSAGE: u8 = 2;
    pub const TIMEOUT: u8 = 3;
}

/// Event types
pub mod event_type {
    pub const MESSAGE_SEND: u8 = 1;
}

impl DiscordClient {
    /// List auto moderation rules for a guild
    pub async fn list_automod_rules(&self, guild_id: &str) -> Result<Vec<AutoModRule>> {
        let response = self
            .get(&format!("/guilds/{}/auto-moderation/rules", guild_id))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let rules: Vec<AutoModRule> = serde_json::from_str(&body)?;
            Ok(rules)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to list automod rules: {}",
                response.status()
            )))
        }
    }

    /// Get a specific auto moderation rule
    pub async fn get_automod_rule(&self, guild_id: &str, rule_id: &str) -> Result<AutoModRule> {
        let response = self
            .get(&format!(
                "/guilds/{}/auto-moderation/rules/{}",
                guild_id, rule_id
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let rule: AutoModRule = serde_json::from_str(&body)?;
            Ok(rule)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get automod rule: {}",
                response.status()
            )))
        }
    }

    /// Delete an auto moderation rule
    pub async fn delete_automod_rule(&self, guild_id: &str, rule_id: &str) -> Result<()> {
        let response = self
            .delete(&format!(
                "/guilds/{}/auto-moderation/rules/{}",
                guild_id, rule_id
            ))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to delete automod rule: {}",
                response.status()
            )))
        }
    }
}
