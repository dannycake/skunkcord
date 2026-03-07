// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord Interactions — slash commands, buttons, modals
//!
//! Handles fetching available slash commands, sending interactions,
//! and processing interaction responses (buttons, select menus, modals).

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Application command (slash command) object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationCommand {
    pub id: String,
    pub application_id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub command_type: Option<u8>,
    pub options: Option<Vec<CommandOption>>,
    pub dm_permission: Option<bool>,
    pub version: Option<String>,
}

/// Command option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOption {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub option_type: u8,
    pub required: Option<bool>,
    pub choices: Option<Vec<CommandChoice>>,
    pub options: Option<Vec<CommandOption>>,
    pub autocomplete: Option<bool>,
    pub min_value: Option<serde_json::Value>,
    pub max_value: Option<serde_json::Value>,
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
}

/// Command option choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandChoice {
    pub name: String,
    pub value: serde_json::Value,
}

/// Command option types
pub mod option_type {
    pub const SUB_COMMAND: u8 = 1;
    pub const SUB_COMMAND_GROUP: u8 = 2;
    pub const STRING: u8 = 3;
    pub const INTEGER: u8 = 4;
    pub const BOOLEAN: u8 = 5;
    pub const USER: u8 = 6;
    pub const CHANNEL: u8 = 7;
    pub const ROLE: u8 = 8;
    pub const MENTIONABLE: u8 = 9;
    pub const NUMBER: u8 = 10;
    pub const ATTACHMENT: u8 = 11;
}

/// Interaction types
pub mod interaction_type {
    pub const PING: u8 = 1;
    pub const APPLICATION_COMMAND: u8 = 2;
    pub const MESSAGE_COMPONENT: u8 = 3;
    pub const APPLICATION_COMMAND_AUTOCOMPLETE: u8 = 4;
    pub const MODAL_SUBMIT: u8 = 5;
}

/// Component types
pub mod component_type {
    pub const ACTION_ROW: u8 = 1;
    pub const BUTTON: u8 = 2;
    pub const STRING_SELECT: u8 = 3;
    pub const TEXT_INPUT: u8 = 4;
    pub const USER_SELECT: u8 = 5;
    pub const ROLE_SELECT: u8 = 6;
    pub const MENTIONABLE_SELECT: u8 = 7;
    pub const CHANNEL_SELECT: u8 = 8;
}

/// Message component (button, select menu, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageComponent {
    #[serde(rename = "type")]
    pub component_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<MessageComponent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<SelectOption>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_values: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_values: Option<u8>,
}

/// Select menu option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
    pub emoji: Option<serde_json::Value>,
    pub default: Option<bool>,
}

/// Interaction data for sending
#[derive(Debug, Clone, Serialize)]
pub struct InteractionRequest {
    #[serde(rename = "type")]
    pub interaction_type: u8,
    pub application_id: String,
    pub channel_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
    pub session_id: String,
    pub data: InteractionData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

/// Interaction data payload
#[derive(Debug, Clone, Serialize)]
pub struct InteractionData {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<InteractionOption>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
}

/// Interaction option value
#[derive(Debug, Clone, Serialize)]
pub struct InteractionOption {
    pub name: String,
    #[serde(rename = "type")]
    pub option_type: u8,
    pub value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<InteractionOption>>,
}

impl DiscordClient {
    /// Search for available slash commands in a channel
    pub async fn search_commands(
        &self,
        channel_id: &str,
        query: &str,
        command_type: Option<u8>,
        limit: Option<u8>,
    ) -> Result<Vec<ApplicationCommand>> {
        let cmd_type = command_type.unwrap_or(1);
        let limit = limit.unwrap_or(25);
        let encoded = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
        let response = self
            .get(&format!(
                "/channels/{}/application-commands/search?type={}&query={}&limit={}&include_applications=true",
                channel_id, cmd_type, encoded, limit
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let wrapper: serde_json::Value = serde_json::from_str(&body)?;
            if let Some(cmds) = wrapper.get("application_commands") {
                let commands: Vec<ApplicationCommand> = serde_json::from_value(cmds.clone())?;
                return Ok(commands);
            }
            Ok(vec![])
        } else {
            Err(DiscordError::Http(format!(
                "Failed to search commands: {}",
                response.status()
            )))
        }
    }

    /// Send an interaction (slash command, button click, etc.)
    pub async fn send_interaction(&self, interaction: &InteractionRequest) -> Result<()> {
        let response = self.post("/interactions", interaction).await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to send interaction: {}",
                response.status()
            )))
        }
    }
}
