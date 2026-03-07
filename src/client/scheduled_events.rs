// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Guild Scheduled Events API

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Scheduled event object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub id: String,
    pub guild_id: String,
    pub channel_id: Option<String>,
    pub creator_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub scheduled_start_time: String,
    pub scheduled_end_time: Option<String>,
    pub privacy_level: u8,
    pub status: u8,
    pub entity_type: u8,
    pub entity_id: Option<String>,
    pub entity_metadata: Option<EventEntityMetadata>,
    pub creator: Option<super::User>,
    pub user_count: Option<u32>,
    pub image: Option<String>,
}

/// Entity metadata for external events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEntityMetadata {
    pub location: Option<String>,
}

/// Event privacy levels
pub mod privacy_level {
    pub const GUILD_ONLY: u8 = 2;
}

/// Event status
pub mod event_status {
    pub const SCHEDULED: u8 = 1;
    pub const ACTIVE: u8 = 2;
    pub const COMPLETED: u8 = 3;
    pub const CANCELED: u8 = 4;
}

/// Event entity type
pub mod entity_type {
    pub const STAGE_INSTANCE: u8 = 1;
    pub const VOICE: u8 = 2;
    pub const EXTERNAL: u8 = 3;
}

impl DiscordClient {
    /// List scheduled events for a guild
    pub async fn list_scheduled_events(&self, guild_id: &str) -> Result<Vec<ScheduledEvent>> {
        let response = self
            .get(&format!(
                "/guilds/{}/scheduled-events?with_user_count=true",
                guild_id
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let events: Vec<ScheduledEvent> = serde_json::from_str(&body)?;
            Ok(events)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to list scheduled events: {}",
                response.status()
            )))
        }
    }

    /// Get a specific scheduled event
    pub async fn get_scheduled_event(
        &self,
        guild_id: &str,
        event_id: &str,
    ) -> Result<ScheduledEvent> {
        let response = self
            .get(&format!(
                "/guilds/{}/scheduled-events/{}?with_user_count=true",
                guild_id, event_id
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let event: ScheduledEvent = serde_json::from_str(&body)?;
            Ok(event)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get scheduled event: {}",
                response.status()
            )))
        }
    }
}
