// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Unread tracking and read state management
//!
//! Tracks which channels have unread messages and mention counts.
//! Populated from the READY event's read_state field and updated
//! via MESSAGE_CREATE and MESSAGE_ACK gateway events.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Read state for a single channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelReadState {
    /// Channel ID
    pub channel_id: String,
    /// Last read message ID
    pub last_read_message_id: Option<String>,
    /// Number of unread mentions (@user)
    pub mention_count: u32,
    /// Last message ID in the channel (to compare with last_read)
    pub last_message_id: Option<String>,
}

impl ChannelReadState {
    /// Check if the channel has unread messages
    pub fn has_unreads(&self) -> bool {
        match (&self.last_read_message_id, &self.last_message_id) {
            (Some(read), Some(last)) => read != last,
            (None, Some(_)) => true,
            _ => false,
        }
    }

    /// Check if there are unread mentions
    pub fn has_mentions(&self) -> bool {
        self.mention_count > 0
    }
}

/// Manager for all channel read states
#[derive(Debug, Clone, Default)]
pub struct ReadStateManager {
    /// Read states keyed by channel ID
    states: HashMap<String, ChannelReadState>,
}

impl ReadStateManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize from READY event read_state data
    pub fn init_from_ready(&mut self, entries: &[ReadStateEntry]) {
        for entry in entries {
            self.states.insert(
                entry.id.clone(),
                ChannelReadState {
                    channel_id: entry.id.clone(),
                    last_read_message_id: entry.last_message_id.clone(),
                    mention_count: entry.mention_count.unwrap_or(0),
                    last_message_id: None, // Will be populated from channel data
                },
            );
        }
    }

    /// Update when a new message arrives in a channel
    pub fn on_message_create(&mut self, channel_id: &str, message_id: &str, mentions_me: bool) {
        let state = self
            .states
            .entry(channel_id.to_string())
            .or_insert_with(|| ChannelReadState {
                channel_id: channel_id.to_string(),
                last_read_message_id: None,
                mention_count: 0,
                last_message_id: None,
            });

        state.last_message_id = Some(message_id.to_string());
        if mentions_me {
            state.mention_count += 1;
        }
    }

    /// Update when a message is acknowledged (marked as read)
    pub fn on_message_ack(&mut self, channel_id: &str, message_id: &str) {
        if let Some(state) = self.states.get_mut(channel_id) {
            state.last_read_message_id = Some(message_id.to_string());
            // Reset mention count when we read up to this point
            if state.last_message_id.as_deref() == Some(message_id) {
                state.mention_count = 0;
            }
        }
    }

    /// Set the last message ID for a channel (from channel data)
    pub fn set_last_message_id(&mut self, channel_id: &str, message_id: &str) {
        let state = self
            .states
            .entry(channel_id.to_string())
            .or_insert_with(|| ChannelReadState {
                channel_id: channel_id.to_string(),
                last_read_message_id: None,
                mention_count: 0,
                last_message_id: None,
            });
        state.last_message_id = Some(message_id.to_string());
    }

    /// Get the read state for a channel
    pub fn get(&self, channel_id: &str) -> Option<&ChannelReadState> {
        self.states.get(channel_id)
    }

    /// Check if a channel has unreads
    pub fn has_unreads(&self, channel_id: &str) -> bool {
        self.states
            .get(channel_id)
            .map(|s| s.has_unreads())
            .unwrap_or(false)
    }

    /// Get mention count for a channel
    pub fn mention_count(&self, channel_id: &str) -> u32 {
        self.states
            .get(channel_id)
            .map(|s| s.mention_count)
            .unwrap_or(0)
    }

    /// Get all channels with unreads
    pub fn unread_channels(&self) -> Vec<&ChannelReadState> {
        self.states.values().filter(|s| s.has_unreads()).collect()
    }

    /// Get total mention count across all channels
    pub fn total_mentions(&self) -> u32 {
        self.states.values().map(|s| s.mention_count).sum()
    }

    /// Mark all channels as read (returns the list of channel/message pairs for bulk ACK)
    pub fn mark_all_read(&mut self) -> Vec<(String, String)> {
        let mut acks = Vec::new();
        for state in self.states.values_mut() {
            if state.has_unreads() {
                if let Some(ref last_msg) = state.last_message_id {
                    acks.push((state.channel_id.clone(), last_msg.clone()));
                    state.last_read_message_id = Some(last_msg.clone());
                    state.mention_count = 0;
                }
            }
        }
        acks
    }
}

/// Read state entry from READY event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadStateEntry {
    pub id: String,
    pub last_message_id: Option<String>,
    pub mention_count: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_unreads_initially() {
        let mgr = ReadStateManager::new();
        assert!(!mgr.has_unreads("ch1"));
        assert_eq!(mgr.mention_count("ch1"), 0);
    }

    #[test]
    fn test_message_creates_unread() {
        let mut mgr = ReadStateManager::new();
        mgr.on_message_create("ch1", "msg1", false);
        assert!(mgr.has_unreads("ch1"));
        assert_eq!(mgr.mention_count("ch1"), 0);
    }

    #[test]
    fn test_mention_tracking() {
        let mut mgr = ReadStateManager::new();
        mgr.on_message_create("ch1", "msg1", true);
        mgr.on_message_create("ch1", "msg2", true);
        assert_eq!(mgr.mention_count("ch1"), 2);
        assert_eq!(mgr.total_mentions(), 2);
    }

    #[test]
    fn test_ack_clears_unreads() {
        let mut mgr = ReadStateManager::new();
        mgr.on_message_create("ch1", "msg1", true);
        mgr.on_message_create("ch1", "msg2", false);

        mgr.on_message_ack("ch1", "msg2");
        let state = mgr.get("ch1").unwrap();
        assert_eq!(state.last_read_message_id, Some("msg2".to_string()));
        assert!(!state.has_unreads());
        assert_eq!(state.mention_count, 0);
    }

    #[test]
    fn test_mark_all_read() {
        let mut mgr = ReadStateManager::new();
        mgr.on_message_create("ch1", "msg1", false);
        mgr.on_message_create("ch2", "msg2", true);

        let acks = mgr.mark_all_read();
        assert_eq!(acks.len(), 2);
        assert!(!mgr.has_unreads("ch1"));
        assert!(!mgr.has_unreads("ch2"));
        assert_eq!(mgr.total_mentions(), 0);
    }

    #[test]
    fn test_init_from_ready() {
        let mut mgr = ReadStateManager::new();
        mgr.init_from_ready(&[ReadStateEntry {
            id: "ch1".to_string(),
            last_message_id: Some("msg5".to_string()),
            mention_count: Some(3),
        }]);

        let state = mgr.get("ch1").unwrap();
        assert_eq!(state.mention_count, 3);
        assert_eq!(state.last_read_message_id, Some("msg5".to_string()));
    }
}
