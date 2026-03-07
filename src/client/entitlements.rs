// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Entitlements API (application subscriptions/purchases)
//!
//! Entitlements represent a user's access to premium features
//! of an application (e.g., Nitro perks, app-specific subscriptions).

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Entitlement object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entitlement {
    pub id: String,
    pub sku_id: String,
    pub application_id: String,
    pub user_id: Option<String>,
    pub guild_id: Option<String>,
    #[serde(rename = "type")]
    pub entitlement_type: u8,
    pub deleted: bool,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
}

/// Entitlement types
pub mod entitlement_type {
    pub const PURCHASE: u8 = 1;
    pub const PREMIUM_SUBSCRIPTION: u8 = 2;
    pub const DEVELOPER_GIFT: u8 = 3;
    pub const TEST_MODE_PURCHASE: u8 = 4;
    pub const FREE_PURCHASE: u8 = 5;
    pub const USER_GIFT: u8 = 6;
    pub const PREMIUM_PURCHASE: u8 = 7;
    pub const APPLICATION_SUBSCRIPTION: u8 = 8;
}

impl DiscordClient {
    /// List entitlements for the current user
    pub async fn list_entitlements(&self) -> Result<Vec<Entitlement>> {
        let response = self.get("/users/@me/entitlements").await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let entitlements: Vec<Entitlement> = serde_json::from_str(&body).unwrap_or_default();
            Ok(entitlements)
        } else {
            // May 404 if no entitlements
            Ok(vec![])
        }
    }
}
