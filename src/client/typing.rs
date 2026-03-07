// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Typing indicator throttling
//!
//! Discord typing indicators last 10 seconds and should only be sent
//! at most once per ~8 seconds per channel to avoid rate limiting and
//! appearing bot-like.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Typing indicator throttle — ensures we don't spam typing events
pub struct TypingThrottle {
    /// Last typing event sent per channel
    last_sent: HashMap<String, Instant>,
    /// Minimum interval between typing events (per channel)
    min_interval: Duration,
}

impl TypingThrottle {
    /// Create a new throttle with the default interval (8 seconds)
    pub fn new() -> Self {
        Self {
            last_sent: HashMap::new(),
            min_interval: Duration::from_secs(8),
        }
    }

    /// Create a throttle with a custom interval
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            last_sent: HashMap::new(),
            min_interval: interval,
        }
    }

    /// Check if we should send a typing indicator for this channel.
    /// Returns true and records the timestamp if enough time has passed.
    /// Returns false if we should skip (too recent).
    pub fn should_send(&mut self, channel_id: &str) -> bool {
        let now = Instant::now();
        if let Some(last) = self.last_sent.get(channel_id) {
            if now.duration_since(*last) < self.min_interval {
                return false;
            }
        }
        self.last_sent.insert(channel_id.to_string(), now);
        true
    }

    /// Clear typing state for a channel (e.g., after sending a message)
    pub fn clear(&mut self, channel_id: &str) {
        self.last_sent.remove(channel_id);
    }

    /// Clear all typing states
    pub fn clear_all(&mut self) {
        self.last_sent.clear();
    }
}

impl Default for TypingThrottle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_send_allowed() {
        let mut throttle = TypingThrottle::new();
        assert!(throttle.should_send("ch1"));
    }

    #[test]
    fn test_immediate_second_send_blocked() {
        let mut throttle = TypingThrottle::new();
        assert!(throttle.should_send("ch1"));
        assert!(!throttle.should_send("ch1")); // too soon
    }

    #[test]
    fn test_different_channels_independent() {
        let mut throttle = TypingThrottle::new();
        assert!(throttle.should_send("ch1"));
        assert!(throttle.should_send("ch2")); // different channel, allowed
        assert!(!throttle.should_send("ch1")); // same channel, blocked
    }

    #[test]
    fn test_clear_allows_resend() {
        let mut throttle = TypingThrottle::new();
        assert!(throttle.should_send("ch1"));
        assert!(!throttle.should_send("ch1"));
        throttle.clear("ch1");
        assert!(throttle.should_send("ch1")); // cleared, allowed again
    }

    #[test]
    fn test_short_interval() {
        let mut throttle = TypingThrottle::with_interval(Duration::from_millis(10));
        assert!(throttle.should_send("ch1"));
        assert!(!throttle.should_send("ch1"));
        std::thread::sleep(Duration::from_millis(15));
        assert!(throttle.should_send("ch1")); // interval passed
    }
}
