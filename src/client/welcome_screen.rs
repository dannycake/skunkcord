// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Guild welcome screen API
//!
//! Welcome screens are shown to new members when they join a server
//! that has the feature enabled.

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Guild welcome screen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomeScreen {
    pub description: Option<String>,
    pub welcome_channels: Vec<WelcomeScreenChannel>,
}

/// A channel shown on the welcome screen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomeScreenChannel {
    pub channel_id: String,
    pub description: String,
    pub emoji_id: Option<String>,
    pub emoji_name: Option<String>,
}

impl DiscordClient {
    /// Get guild welcome screen
    pub async fn get_welcome_screen(&self, guild_id: &str) -> Result<WelcomeScreen> {
        let response = self
            .get(&format!("/guilds/{}/welcome-screen", guild_id))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let screen: WelcomeScreen = serde_json::from_str(&body)?;
            Ok(screen)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get welcome screen: {}",
                response.status()
            )))
        }
    }

    /// Get guild prune count (how many members would be pruned)
    pub async fn get_prune_count(&self, guild_id: &str, days: u8) -> Result<u32> {
        let response = self
            .get(&format!("/guilds/{}/prune?days={}", guild_id, days))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let data: serde_json::Value = serde_json::from_str(&body)?;
            Ok(data.get("pruned").and_then(|v| v.as_u64()).unwrap_or(0) as u32)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get prune count: {}",
                response.status()
            )))
        }
    }

    /// Begin guild prune (remove inactive members)
    pub async fn begin_prune(
        &self,
        guild_id: &str,
        days: u8,
        compute_prune_count: bool,
        reason: Option<&str>,
    ) -> Result<Option<u32>> {
        let body = serde_json::json!({
            "days": days,
            "compute_prune_count": compute_prune_count,
        });

        let endpoint = format!("/guilds/{}/prune", guild_id);
        let url = format!("{}/v{}{}", self.api_base, crate::API_VERSION, endpoint);
        let client = self.inner.read().await;
        let mut request = self
            .prepare_request(client.post(&url))
            .await
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&body)?);

        if let Some(r) = reason {
            request = request.header("X-Audit-Log-Reason", r);
        }

        let response = request
            .send()
            .await
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let data: serde_json::Value = serde_json::from_str(&body)?;
            Ok(data
                .get("pruned")
                .and_then(|v| v.as_u64())
                .map(|n| n as u32))
        } else {
            Err(DiscordError::Http(format!(
                "Failed to begin prune: {}",
                response.status()
            )))
        }
    }
}
