// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Silent message toggle
//!
//! Adds the ability to send messages without triggering notifications
//! for recipients. Uses Discord's official SUPPRESS_NOTIFICATIONS flag (4096).

/// Message flag for suppressing notifications
pub const SUPPRESS_NOTIFICATIONS_FLAG: u32 = 1 << 12; // 4096

/// Apply the silent flag to a message flags value
pub fn apply_silent_flag(current_flags: Option<u32>) -> u32 {
    current_flags.unwrap_or(0) | SUPPRESS_NOTIFICATIONS_FLAG
}

/// Remove the silent flag from a message flags value
pub fn remove_silent_flag(current_flags: Option<u32>) -> u32 {
    current_flags.unwrap_or(0) & !SUPPRESS_NOTIFICATIONS_FLAG
}

/// Check if a flags value has the silent flag set
pub fn is_silent(flags: u32) -> bool {
    flags & SUPPRESS_NOTIFICATIONS_FLAG != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_silent() {
        assert_eq!(apply_silent_flag(None), 4096);
        assert_eq!(apply_silent_flag(Some(0)), 4096);
        assert_eq!(apply_silent_flag(Some(4096)), 4096); // idempotent
        assert_eq!(apply_silent_flag(Some(1)), 4097); // preserves other flags
    }

    #[test]
    fn test_remove_silent() {
        assert_eq!(remove_silent_flag(Some(4096)), 0);
        assert_eq!(remove_silent_flag(Some(4097)), 1);
        assert_eq!(remove_silent_flag(None), 0);
    }

    #[test]
    fn test_is_silent() {
        assert!(is_silent(4096));
        assert!(is_silent(4097));
        assert!(!is_silent(0));
        assert!(!is_silent(1));
    }
}
