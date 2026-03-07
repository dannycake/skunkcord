// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Guild Onboarding API
//!
//! Guild onboarding is the flow new members see when joining a server.
//! It includes prompts for selecting channels, roles, and answering questions.

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Guild onboarding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildOnboarding {
    pub guild_id: String,
    pub prompts: Vec<OnboardingPrompt>,
    pub default_channel_ids: Vec<String>,
    pub enabled: bool,
    pub mode: u8,
}

/// An onboarding prompt (question/selection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingPrompt {
    pub id: String,
    #[serde(rename = "type")]
    pub prompt_type: u8,
    pub options: Vec<OnboardingOption>,
    pub title: String,
    pub single_select: bool,
    pub required: bool,
    pub in_onboarding: bool,
}

/// An option within a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingOption {
    pub id: String,
    pub channel_ids: Vec<String>,
    pub role_ids: Vec<String>,
    pub title: String,
    pub description: Option<String>,
    pub emoji: Option<OnboardingEmoji>,
}

/// Emoji for onboarding options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingEmoji {
    pub id: Option<String>,
    pub name: Option<String>,
    pub animated: Option<bool>,
}

/// Onboarding mode
pub mod onboarding_mode {
    /// Default: onboarding only runs for new members
    pub const ONBOARDING_DEFAULT: u8 = 0;
    /// Advanced: onboarding runs for all members who haven't completed it
    pub const ONBOARDING_ADVANCED: u8 = 1;
}

impl DiscordClient {
    /// Get guild onboarding configuration
    pub async fn get_guild_onboarding(&self, guild_id: &str) -> Result<GuildOnboarding> {
        let response = self
            .get(&format!("/guilds/{}/onboarding", guild_id))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let onboarding: GuildOnboarding = serde_json::from_str(&body)?;
            Ok(onboarding)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get onboarding: {}",
                response.status()
            )))
        }
    }
}
