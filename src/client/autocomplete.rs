// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Slash command autocomplete handler
//!
//! When a user types a slash command, Discord returns autocomplete
//! suggestions for the focused option. This module handles sending
//! autocomplete interactions and parsing responses.

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Autocomplete interaction request
#[derive(Debug, Clone, Serialize)]
pub struct AutocompleteRequest {
    #[serde(rename = "type")]
    pub interaction_type: u8, // Always 4 (APPLICATION_COMMAND_AUTOCOMPLETE)
    pub application_id: String,
    pub channel_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
    pub session_id: String,
    pub data: AutocompleteData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

/// Autocomplete data payload
#[derive(Debug, Clone, Serialize)]
pub struct AutocompleteData {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub command_type: u8,
    pub version: String,
    pub options: Vec<AutocompleteOption>,
}

/// An option in the autocomplete request
#[derive(Debug, Clone, Serialize)]
pub struct AutocompleteOption {
    pub name: String,
    #[serde(rename = "type")]
    pub option_type: u8,
    pub value: serde_json::Value,
    /// Whether this is the option being autocompleted
    pub focused: bool,
}

/// Autocomplete choice returned by a bot
#[derive(Debug, Clone, Deserialize)]
pub struct AutocompleteChoice {
    pub name: String,
    pub value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_localizations: Option<serde_json::Value>,
}

impl DiscordClient {
    /// Send an autocomplete interaction
    pub async fn send_autocomplete(&self, request: &AutocompleteRequest) -> Result<()> {
        let response = self.post("/interactions", request).await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Autocomplete failed: {}",
                response.status()
            )))
        }
    }
}

/// Build an autocomplete request for a focused option
pub fn build_autocomplete_request(
    application_id: &str,
    channel_id: &str,
    guild_id: Option<&str>,
    session_id: &str,
    command_id: &str,
    command_name: &str,
    command_version: &str,
    focused_option: &str,
    focused_type: u8,
    current_value: &str,
) -> AutocompleteRequest {
    AutocompleteRequest {
        interaction_type: 4,
        application_id: application_id.to_string(),
        channel_id: channel_id.to_string(),
        guild_id: guild_id.map(|s| s.to_string()),
        session_id: session_id.to_string(),
        data: AutocompleteData {
            id: command_id.to_string(),
            name: command_name.to_string(),
            command_type: 1,
            version: command_version.to_string(),
            options: vec![AutocompleteOption {
                name: focused_option.to_string(),
                option_type: focused_type,
                value: serde_json::json!(current_value),
                focused: true,
            }],
        },
        nonce: Some(uuid::Uuid::new_v4().to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_autocomplete() {
        let req = build_autocomplete_request(
            "app123",
            "ch456",
            Some("g789"),
            "sess",
            "cmd1",
            "search",
            "1",
            "query",
            3, // STRING type
            "hel",
        );

        assert_eq!(req.interaction_type, 4);
        assert_eq!(req.application_id, "app123");
        assert_eq!(req.data.name, "search");
        assert_eq!(req.data.options.len(), 1);
        assert!(req.data.options[0].focused);
        assert_eq!(req.data.options[0].name, "query");
        assert_eq!(req.data.options[0].value, "hel");
    }

    #[test]
    fn test_autocomplete_serialization() {
        let req = build_autocomplete_request(
            "app", "ch", None, "sess", "cmd", "test", "1", "opt", 3, "value",
        );
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"type\":4"));
        assert!(json.contains("\"focused\":true"));
        assert!(!json.contains("guild_id")); // None should be skipped
    }
}
