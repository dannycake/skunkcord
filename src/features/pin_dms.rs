// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Pin DMs — pin DM conversations to the top of the list
//!
//! Entirely local feature — stores pinned DM channel IDs in settings.
//! Zero API interaction, completely undetectable.

use serde::{Deserialize, Serialize};

/// Pinned DMs configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PinnedDms {
    /// Ordered list of pinned DM channel IDs
    pub pinned_channel_ids: Vec<String>,
}

impl PinnedDms {
    /// Pin a DM channel
    pub fn pin(&mut self, channel_id: &str) {
        if !self.is_pinned(channel_id) {
            self.pinned_channel_ids.push(channel_id.to_string());
        }
    }

    /// Unpin a DM channel
    pub fn unpin(&mut self, channel_id: &str) {
        self.pinned_channel_ids.retain(|id| id != channel_id);
    }

    /// Check if a DM is pinned
    pub fn is_pinned(&self, channel_id: &str) -> bool {
        self.pinned_channel_ids.iter().any(|id| id == channel_id)
    }

    /// Move a pinned DM to a new position
    pub fn reorder(&mut self, channel_id: &str, new_index: usize) {
        if let Some(pos) = self
            .pinned_channel_ids
            .iter()
            .position(|id| id == channel_id)
        {
            let id = self.pinned_channel_ids.remove(pos);
            let insert_at = new_index.min(self.pinned_channel_ids.len());
            self.pinned_channel_ids.insert(insert_at, id);
        }
    }

    /// Get the pin order index for sorting
    pub fn pin_order(&self, channel_id: &str) -> Option<usize> {
        self.pinned_channel_ids
            .iter()
            .position(|id| id == channel_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_unpin() {
        let mut pins = PinnedDms::default();
        pins.pin("123");
        pins.pin("456");
        assert!(pins.is_pinned("123"));
        assert!(pins.is_pinned("456"));
        assert!(!pins.is_pinned("789"));

        pins.unpin("123");
        assert!(!pins.is_pinned("123"));
        assert!(pins.is_pinned("456"));
    }

    #[test]
    fn test_pin_idempotent() {
        let mut pins = PinnedDms::default();
        pins.pin("123");
        pins.pin("123");
        assert_eq!(pins.pinned_channel_ids.len(), 1);
    }

    #[test]
    fn test_reorder() {
        let mut pins = PinnedDms::default();
        pins.pin("a");
        pins.pin("b");
        pins.pin("c");
        pins.reorder("c", 0);
        assert_eq!(pins.pinned_channel_ids, vec!["c", "a", "b"]);
    }

    #[test]
    fn test_pin_order() {
        let mut pins = PinnedDms::default();
        pins.pin("a");
        pins.pin("b");
        pins.pin("c");
        assert_eq!(pins.pin_order("a"), Some(0));
        assert_eq!(pins.pin_order("c"), Some(2));
        assert_eq!(pins.pin_order("x"), None);
    }
}
