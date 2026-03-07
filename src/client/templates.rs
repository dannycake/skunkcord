// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Guild template API
//!
//! Server templates allow sharing and cloning guild configurations.

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Guild template object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildTemplate {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub usage_count: u32,
    pub creator_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub source_guild_id: String,
    pub serialized_source_guild: serde_json::Value,
    pub is_dirty: Option<bool>,
}

impl DiscordClient {
    /// Get a guild template by code
    pub async fn get_template(&self, code: &str) -> Result<GuildTemplate> {
        let response = self.get(&format!("/guilds/templates/{}", code)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let template: GuildTemplate = serde_json::from_str(&body)?;
            Ok(template)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get template: {}",
                response.status()
            )))
        }
    }

    /// Get templates for a guild
    pub async fn get_guild_templates(&self, guild_id: &str) -> Result<Vec<GuildTemplate>> {
        let response = self.get(&format!("/guilds/{}/templates", guild_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let templates: Vec<GuildTemplate> = serde_json::from_str(&body)?;
            Ok(templates)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get guild templates: {}",
                response.status()
            )))
        }
    }
}
