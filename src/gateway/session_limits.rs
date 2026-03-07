// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Gateway session limit tracking
//!
//! Discord limits the number of gateway sessions and identifies per day.
//! Exceeding these limits results in connection rejection.
//! This module tracks usage to prevent hitting limits.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Session limit info from GET /gateway/bot (or /gateway for user accounts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartLimit {
    /// Total allowed session starts per day
    pub total: u32,
    /// Remaining session starts
    pub remaining: u32,
    /// Milliseconds until the limit resets
    pub reset_after: u64,
    /// Maximum allowed concurrent sessions (shards)
    pub max_concurrency: u32,
}

/// Local session tracking
#[derive(Debug, Clone)]
pub struct SessionTracker {
    /// Known limit info (from last API call)
    limit_info: Option<SessionStartLimit>,
    /// When we last fetched limit info
    last_fetched: Option<Instant>,
    /// Number of sessions we've started since last reset
    local_session_count: u32,
    /// When we started counting
    count_start: Instant,
}

impl SessionTracker {
    pub fn new() -> Self {
        Self {
            limit_info: None,
            last_fetched: None,
            local_session_count: 0,
            count_start: Instant::now(),
        }
    }

    /// Update with limit info from API
    pub fn update_limits(&mut self, limits: SessionStartLimit) {
        self.limit_info = Some(limits);
        self.last_fetched = Some(Instant::now());
    }

    /// Record a new session start
    pub fn on_session_start(&mut self) {
        self.local_session_count += 1;

        // Reset counter if >24 hours have passed
        if self.count_start.elapsed() > Duration::from_secs(86400) {
            self.local_session_count = 1;
            self.count_start = Instant::now();
        }
    }

    /// Check if we're safe to start a new session
    pub fn can_start_session(&self) -> bool {
        if let Some(ref limits) = self.limit_info {
            if limits.remaining == 0 {
                return false;
            }
        }

        // Conservative local check: don't exceed 900 sessions/day
        // (Discord's default limit is 1000)
        self.local_session_count < 900
    }

    /// Get the remaining session starts (from API info, if available)
    pub fn remaining(&self) -> Option<u32> {
        self.limit_info.as_ref().map(|l| l.remaining)
    }

    /// Get time until limit resets
    pub fn reset_after(&self) -> Option<Duration> {
        self.limit_info
            .as_ref()
            .map(|l| Duration::from_millis(l.reset_after))
    }

    /// Get the local session count since tracking started
    pub fn local_count(&self) -> u32 {
        self.local_session_count
    }

    /// Check if our limit info is stale (>5 minutes old)
    pub fn is_stale(&self) -> bool {
        self.last_fetched
            .map(|t| t.elapsed() > Duration::from_secs(300))
            .unwrap_or(true)
    }
}

impl Default for SessionTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tracker_allows_sessions() {
        let tracker = SessionTracker::new();
        assert!(tracker.can_start_session());
        assert_eq!(tracker.local_count(), 0);
        assert!(tracker.remaining().is_none());
    }

    #[test]
    fn test_session_counting() {
        let mut tracker = SessionTracker::new();
        tracker.on_session_start();
        tracker.on_session_start();
        assert_eq!(tracker.local_count(), 2);
        assert!(tracker.can_start_session());
    }

    #[test]
    fn test_limit_update() {
        let mut tracker = SessionTracker::new();
        tracker.update_limits(SessionStartLimit {
            total: 1000,
            remaining: 500,
            reset_after: 3600000,
            max_concurrency: 1,
        });

        assert_eq!(tracker.remaining(), Some(500));
        assert!(tracker.can_start_session());
        assert!(!tracker.is_stale());
    }

    #[test]
    fn test_zero_remaining_blocks() {
        let mut tracker = SessionTracker::new();
        tracker.update_limits(SessionStartLimit {
            total: 1000,
            remaining: 0,
            reset_after: 3600000,
            max_concurrency: 1,
        });

        assert!(!tracker.can_start_session());
    }

    #[test]
    fn test_stale_check() {
        let tracker = SessionTracker::new();
        assert!(tracker.is_stale()); // Never fetched = stale
    }
}
