// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Premium/Nitro and subscription endpoints

use super::DiscordClient;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Nitro subscription types
pub mod subscription_type {
    pub const NONE: u8 = 0;
    pub const NITRO_CLASSIC: u8 = 1;
    pub const NITRO: u8 = 2;
    pub const NITRO_BASIC: u8 = 3;
}

/// User subscription/billing info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub subscription_type: Option<u8>,
    pub payment_source_id: Option<String>,
    pub payment_gateway: Option<String>,
    pub status: Option<u8>,
    pub current_period_start: Option<String>,
    pub current_period_end: Option<String>,
    pub canceled_at: Option<String>,
}

/// Server boost status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildBoostStatus {
    pub premium_tier: u8,
    pub premium_subscription_count: u32,
    pub premium_progress_bar_enabled: Option<bool>,
}

/// Boost tier levels
pub mod boost_tier {
    pub const NONE: u8 = 0;
    pub const TIER_1: u8 = 1; // 2 boosts
    pub const TIER_2: u8 = 2; // 7 boosts
    pub const TIER_3: u8 = 3; // 14 boosts
}

/// Boost perks per tier
pub fn boost_perks(tier: u8) -> BoostPerks {
    match tier {
        1 => BoostPerks {
            emoji_limit: 100,
            sticker_limit: 15,
            bitrate_limit: 128000,
            upload_limit: 25 * 1024 * 1024,
            stream_quality: "720p60",
            vanity_url: false,
            banner: false,
            animated_icon: true,
        },
        2 => BoostPerks {
            emoji_limit: 150,
            sticker_limit: 30,
            bitrate_limit: 256000,
            upload_limit: 50 * 1024 * 1024,
            stream_quality: "1080p60",
            vanity_url: false,
            banner: true,
            animated_icon: true,
        },
        3 => BoostPerks {
            emoji_limit: 250,
            sticker_limit: 60,
            bitrate_limit: 384000,
            upload_limit: 100 * 1024 * 1024,
            stream_quality: "1080p60",
            vanity_url: true,
            banner: true,
            animated_icon: true,
        },
        _ => BoostPerks {
            emoji_limit: 50,
            sticker_limit: 5,
            bitrate_limit: 96000,
            upload_limit: 25 * 1024 * 1024,
            stream_quality: "720p30",
            vanity_url: false,
            banner: false,
            animated_icon: false,
        },
    }
}

/// Perks available at a boost tier
pub struct BoostPerks {
    pub emoji_limit: u32,
    pub sticker_limit: u32,
    pub bitrate_limit: u32,
    pub upload_limit: u64,
    pub stream_quality: &'static str,
    pub vanity_url: bool,
    pub banner: bool,
    pub animated_icon: bool,
}

/// Vanity URL info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VanityUrl {
    pub code: Option<String>,
    pub uses: Option<u32>,
}

impl DiscordClient {
    /// Get the current user's Nitro subscription info
    pub async fn get_subscriptions(&self) -> Result<Vec<SubscriptionInfo>> {
        let response = self.get("/users/@me/billing/subscriptions").await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let subs: Vec<SubscriptionInfo> = serde_json::from_str(&body)?;
            Ok(subs)
        } else {
            // 404 = no subscriptions
            Ok(vec![])
        }
    }

    /// Get a guild's vanity URL
    pub async fn get_vanity_url(&self, guild_id: &str) -> Result<VanityUrl> {
        let response = self
            .get(&format!("/guilds/{}/vanity-url", guild_id))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let vanity: VanityUrl = serde_json::from_str(&body)?;
            Ok(vanity)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get vanity URL: {}",
                response.status()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boost_perks_tiers() {
        let t0 = boost_perks(0);
        assert_eq!(t0.emoji_limit, 50);
        assert!(!t0.vanity_url);

        let t1 = boost_perks(1);
        assert_eq!(t1.emoji_limit, 100);
        assert!(t1.animated_icon);

        let t3 = boost_perks(3);
        assert_eq!(t3.emoji_limit, 250);
        assert!(t3.vanity_url);
        assert!(t3.banner);
    }

    #[test]
    fn test_subscription_types() {
        assert_eq!(subscription_type::NONE, 0);
        assert_eq!(subscription_type::NITRO, 2);
        assert_eq!(subscription_type::NITRO_BASIC, 3);
    }
}
