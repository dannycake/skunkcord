// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Role Connections Metadata API
//!
//! Role connections allow apps to set metadata on a user that
//! guilds can use to gate roles (e.g., linked role requirements).

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Role connection object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConnection {
    pub platform_name: Option<String>,
    pub platform_username: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Role connection metadata record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConnectionMetadata {
    #[serde(rename = "type")]
    pub metadata_type: u8,
    pub key: String,
    pub name: String,
    pub description: String,
    pub name_localizations: Option<std::collections::HashMap<String, String>>,
    pub description_localizations: Option<std::collections::HashMap<String, String>>,
}

/// Metadata types
pub mod metadata_type {
    pub const INTEGER_LESS_THAN_OR_EQUAL: u8 = 1;
    pub const INTEGER_GREATER_THAN_OR_EQUAL: u8 = 2;
    pub const INTEGER_EQUAL: u8 = 3;
    pub const INTEGER_NOT_EQUAL: u8 = 4;
    pub const DATETIME_LESS_THAN_OR_EQUAL: u8 = 5;
    pub const DATETIME_GREATER_THAN_OR_EQUAL: u8 = 6;
    pub const BOOLEAN_EQUAL: u8 = 7;
    pub const BOOLEAN_NOT_EQUAL: u8 = 8;
}

impl DiscordClient {
    /// Get the current user's role connection for an application
    pub async fn get_role_connection(&self, application_id: &str) -> Result<RoleConnection> {
        let response = self
            .get(&format!(
                "/users/@me/applications/{}/role-connection",
                application_id
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let conn: RoleConnection = serde_json::from_str(&body)?;
            Ok(conn)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get role connection: {}",
                response.status()
            )))
        }
    }

    /// Update the current user's role connection for an application
    pub async fn update_role_connection(
        &self,
        application_id: &str,
        connection: &RoleConnection,
    ) -> Result<RoleConnection> {
        let response = self
            .put(
                &format!("/users/@me/applications/{}/role-connection", application_id),
                connection,
            )
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let conn: RoleConnection = serde_json::from_str(&body)?;
            Ok(conn)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to update role connection: {}",
                response.status()
            )))
        }
    }
}
