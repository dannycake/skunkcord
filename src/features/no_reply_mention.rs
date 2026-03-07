// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! No Reply Mention — default replies don't ping the original author
//!
//! Sets `allowed_mentions.replied_user: false` on reply message payloads.
//! Uses official API — completely valid usage, just an unusual default.

use serde::Serialize;

/// Create allowed_mentions with replied_user set to false
pub fn no_ping_allowed_mentions() -> AllowedMentionsOverride {
    AllowedMentionsOverride {
        replied_user: false,
    }
}

/// Create allowed_mentions with replied_user set to true (normal behavior)
pub fn ping_allowed_mentions() -> AllowedMentionsOverride {
    AllowedMentionsOverride { replied_user: true }
}

/// Minimal allowed_mentions override for reply ping control
#[derive(Debug, Clone, Serialize)]
pub struct AllowedMentionsOverride {
    pub replied_user: bool,
}
