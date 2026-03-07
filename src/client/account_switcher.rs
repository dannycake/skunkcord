// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Multi-account switching backend
//!
//! Manages multiple Discord user sessions and enables fast switching
//! between accounts. Each account has its own token, fingerprint,
//! cookies, proxy config, and feature flags.

use crate::features::FeatureFlags;
use crate::proxy::ProxyConfig;
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-account settings that differ between accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    /// User ID
    pub user_id: String,
    /// Display name (cached)
    pub display_name: String,
    /// Avatar URL (cached)
    pub avatar_url: Option<String>,
    /// Per-account feature flags (overrides global if set)
    pub feature_flags: Option<FeatureFlags>,
    /// Per-account proxy config (overrides global if set)
    pub proxy_config: Option<ProxyConfig>,
    /// Order in the account list (lower = higher in UI)
    pub sort_order: u32,
}

/// Account switcher that manages multiple accounts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountSwitcher {
    /// All registered accounts, keyed by user_id
    pub accounts: HashMap<String, AccountConfig>,
    /// Currently active account user_id
    pub active_account_id: Option<String>,
}

impl AccountSwitcher {
    /// Create a new account switcher
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update an account
    pub fn add_account(&mut self, config: AccountConfig) {
        let user_id = config.user_id.clone();
        self.accounts.insert(user_id, config);
    }

    /// Remove an account
    pub fn remove_account(&mut self, user_id: &str) {
        self.accounts.remove(user_id);
        if self.active_account_id.as_deref() == Some(user_id) {
            self.active_account_id = None;
        }
    }

    /// Set the active account
    pub fn set_active(&mut self, user_id: &str) -> Result<()> {
        if self.accounts.contains_key(user_id) {
            self.active_account_id = Some(user_id.to_string());
            Ok(())
        } else {
            Err(DiscordError::NotFound(format!(
                "Account {} not found",
                user_id
            )))
        }
    }

    /// Get the active account config
    pub fn active_account(&self) -> Option<&AccountConfig> {
        self.active_account_id
            .as_ref()
            .and_then(|id| self.accounts.get(id))
    }

    /// Get all accounts sorted by sort_order
    pub fn sorted_accounts(&self) -> Vec<&AccountConfig> {
        let mut accounts: Vec<&AccountConfig> = self.accounts.values().collect();
        accounts.sort_by_key(|a| a.sort_order);
        accounts
    }

    /// Get the number of registered accounts
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Check if a user_id is registered
    pub fn has_account(&self, user_id: &str) -> bool {
        self.accounts.contains_key(user_id)
    }

    /// Get the feature flags for a specific account (falls back to None if not set)
    pub fn get_account_flags(&self, user_id: &str) -> Option<&FeatureFlags> {
        self.accounts
            .get(user_id)
            .and_then(|a| a.feature_flags.as_ref())
    }

    /// Get the proxy config for a specific account (falls back to None if not set)
    pub fn get_account_proxy(&self, user_id: &str) -> Option<&ProxyConfig> {
        self.accounts
            .get(user_id)
            .and_then(|a| a.proxy_config.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_account(id: &str, name: &str, order: u32) -> AccountConfig {
        AccountConfig {
            user_id: id.to_string(),
            display_name: name.to_string(),
            avatar_url: None,
            feature_flags: None,
            proxy_config: None,
            sort_order: order,
        }
    }

    #[test]
    fn test_add_and_switch() {
        let mut switcher = AccountSwitcher::new();
        switcher.add_account(make_account("1", "Alice", 0));
        switcher.add_account(make_account("2", "Bob", 1));

        assert_eq!(switcher.account_count(), 2);
        assert!(switcher.active_account().is_none());

        switcher.set_active("1").unwrap();
        assert_eq!(switcher.active_account().unwrap().display_name, "Alice");

        switcher.set_active("2").unwrap();
        assert_eq!(switcher.active_account().unwrap().display_name, "Bob");
    }

    #[test]
    fn test_remove_account() {
        let mut switcher = AccountSwitcher::new();
        switcher.add_account(make_account("1", "Alice", 0));
        switcher.set_active("1").unwrap();

        switcher.remove_account("1");
        assert_eq!(switcher.account_count(), 0);
        assert!(switcher.active_account().is_none());
        assert!(switcher.active_account_id.is_none());
    }

    #[test]
    fn test_sorted_accounts() {
        let mut switcher = AccountSwitcher::new();
        switcher.add_account(make_account("3", "Charlie", 2));
        switcher.add_account(make_account("1", "Alice", 0));
        switcher.add_account(make_account("2", "Bob", 1));

        let sorted = switcher.sorted_accounts();
        assert_eq!(sorted[0].display_name, "Alice");
        assert_eq!(sorted[1].display_name, "Bob");
        assert_eq!(sorted[2].display_name, "Charlie");
    }

    #[test]
    fn test_set_active_nonexistent() {
        let mut switcher = AccountSwitcher::new();
        assert!(switcher.set_active("nonexistent").is_err());
    }

    #[test]
    fn test_per_account_flags() {
        let mut switcher = AccountSwitcher::new();
        let mut account = make_account("1", "Alice", 0);
        account.feature_flags = Some(FeatureFlags::paranoid());
        switcher.add_account(account);

        let flags = switcher.get_account_flags("1").unwrap();
        assert!(!flags.fake_mute);
        assert!(!flags.arrpc);

        assert!(switcher.get_account_flags("nonexistent").is_none());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut switcher = AccountSwitcher::new();
        switcher.add_account(make_account("1", "Alice", 0));
        switcher.add_account(make_account("2", "Bob", 1));
        switcher.set_active("1").unwrap();

        let json = serde_json::to_string(&switcher).unwrap();
        let deserialized: AccountSwitcher = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.account_count(), 2);
        assert_eq!(deserialized.active_account_id, Some("1".to_string()));
    }
}
