// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord cookie management
//!
//! Manages Cloudflare and Discord-specific cookies that must be
//! present on API requests for proper browser emulation.
//! Missing cookies are a detection signal.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Important Discord/Cloudflare cookies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscordCookies {
    /// Cloudflare device fingerprint UUID
    pub dcfduid: Option<String>,
    /// Cloudflare signed device fingerprint UUID
    pub sdcfduid: Option<String>,
    /// Cloudflare request UID
    pub cfruid: Option<String>,
    /// Discord locale preference
    pub locale: Option<String>,
    /// Any additional cookies
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

impl DiscordCookies {
    /// Create from a cookie map (e.g., extracted from browser session)
    pub fn from_map(map: &HashMap<String, String>) -> Self {
        Self {
            dcfduid: map.get("__dcfduid").cloned(),
            sdcfduid: map.get("__sdcfduid").cloned(),
            cfruid: map.get("__cfruid").cloned(),
            locale: map.get("locale").cloned(),
            extra: map
                .iter()
                .filter(|(k, _)| {
                    !matches!(
                        k.as_str(),
                        "__dcfduid" | "__sdcfduid" | "__cfruid" | "locale"
                    )
                })
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }

    /// Format as a Cookie header string
    pub fn to_header_string(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref v) = self.dcfduid {
            parts.push(format!("__dcfduid={}", v));
        }
        if let Some(ref v) = self.sdcfduid {
            parts.push(format!("__sdcfduid={}", v));
        }
        if let Some(ref v) = self.cfruid {
            parts.push(format!("__cfruid={}", v));
        }
        if let Some(ref v) = self.locale {
            parts.push(format!("locale={}", v));
        }
        for (k, v) in &self.extra {
            parts.push(format!("{}={}", k, v));
        }

        parts.join("; ")
    }

    /// Check if we have the critical Cloudflare cookies
    pub fn has_cf_cookies(&self) -> bool {
        self.dcfduid.is_some() && self.sdcfduid.is_some()
    }

    /// Merge new cookies into existing ones (new values overwrite)
    pub fn merge(&mut self, other: &DiscordCookies) {
        if other.dcfduid.is_some() {
            self.dcfduid = other.dcfduid.clone();
        }
        if other.sdcfduid.is_some() {
            self.sdcfduid = other.sdcfduid.clone();
        }
        if other.cfruid.is_some() {
            self.cfruid = other.cfruid.clone();
        }
        if other.locale.is_some() {
            self.locale = other.locale.clone();
        }
        for (k, v) in &other.extra {
            self.extra.insert(k.clone(), v.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_header_string() {
        let cookies = DiscordCookies {
            dcfduid: Some("abc123".to_string()),
            sdcfduid: Some("def456".to_string()),
            cfruid: None,
            locale: Some("en-US".to_string()),
            extra: HashMap::new(),
        };
        let header = cookies.to_header_string();
        assert!(header.contains("__dcfduid=abc123"));
        assert!(header.contains("__sdcfduid=def456"));
        assert!(header.contains("locale=en-US"));
        assert!(!header.contains("__cfruid"));
    }

    #[test]
    fn test_from_map() {
        let mut map = HashMap::new();
        map.insert("__dcfduid".to_string(), "abc".to_string());
        map.insert("__sdcfduid".to_string(), "def".to_string());
        map.insert("custom".to_string(), "val".to_string());

        let cookies = DiscordCookies::from_map(&map);
        assert_eq!(cookies.dcfduid, Some("abc".to_string()));
        assert_eq!(cookies.sdcfduid, Some("def".to_string()));
        assert!(cookies.has_cf_cookies());
        assert_eq!(cookies.extra.get("custom"), Some(&"val".to_string()));
    }

    #[test]
    fn test_merge() {
        let mut base = DiscordCookies {
            dcfduid: Some("old".to_string()),
            sdcfduid: None,
            ..Default::default()
        };
        let update = DiscordCookies {
            dcfduid: Some("new".to_string()),
            sdcfduid: Some("new_sd".to_string()),
            ..Default::default()
        };
        base.merge(&update);
        assert_eq!(base.dcfduid, Some("new".to_string()));
        assert_eq!(base.sdcfduid, Some("new_sd".to_string()));
    }
}
