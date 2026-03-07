// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Show Hidden Channels — display channels the user can't access
//!
//! Parses permission overwrites from guild data to identify channels
//! where the user lacks VIEW_CHANNEL permission. Instead of hiding them,
//! displays them with a lock icon.
//!
//! Accessing channel metadata for hidden channels could be logged server-side.

use crate::client::Permission;

/// Permission overwrite from guild data
#[derive(Debug, Clone)]
pub struct PermissionOverwrite {
    /// Role or user ID
    pub id: String,
    /// Type: 0 = role, 1 = member
    pub overwrite_type: u8,
    /// Allowed permissions
    pub allow: u64,
    /// Denied permissions
    pub deny: u64,
}

/// Result of checking channel visibility
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelVisibility {
    /// User can see this channel normally
    Visible,
    /// User cannot see this channel (VIEW_CHANNEL denied)
    Hidden,
    /// Visibility depends on other roles (uncertain)
    Unknown,
}

/// View channel permission flag
const VIEW_CHANNEL: u64 = 1 << 10;

/// Check if a user can view a channel based on permission overwrites
///
/// This replicates Discord's permission calculation:
/// 1. Start with @everyone role permissions
/// 2. Apply role-level overwrites (OR all allowed, OR all denied)
/// 3. Apply member-specific overwrite (if exists)
pub fn check_channel_visibility(
    everyone_role_permissions: u64,
    user_role_ids: &[String],
    overwrites: &[PermissionOverwrite],
    everyone_role_id: &str,
    user_id: &str,
) -> ChannelVisibility {
    // Start with base permissions from @everyone
    let mut permissions = everyone_role_permissions;

    // Administrator bypasses all
    if permissions & (Permission::Administrator as u64) != 0 {
        return ChannelVisibility::Visible;
    }

    // Apply @everyone channel overwrite
    let everyone_overwrite = overwrites.iter().find(|o| o.id == everyone_role_id);
    if let Some(ow) = everyone_overwrite {
        permissions &= !ow.deny;
        permissions |= ow.allow;
    }

    // Apply role overwrites (combined)
    let mut role_allow: u64 = 0;
    let mut role_deny: u64 = 0;
    for ow in overwrites.iter().filter(|o| {
        o.overwrite_type == 0 && o.id != everyone_role_id && user_role_ids.contains(&o.id)
    }) {
        role_allow |= ow.allow;
        role_deny |= ow.deny;
    }
    permissions &= !role_deny;
    permissions |= role_allow;

    // Apply member-specific overwrite
    let member_overwrite = overwrites
        .iter()
        .find(|o| o.overwrite_type == 1 && o.id == user_id);
    if let Some(ow) = member_overwrite {
        permissions &= !ow.deny;
        permissions |= ow.allow;
    }

    if permissions & VIEW_CHANNEL != 0 {
        ChannelVisibility::Visible
    } else {
        ChannelVisibility::Hidden
    }
}

/// Get the list of role IDs that have access to a hidden channel
pub fn roles_with_access(
    overwrites: &[PermissionOverwrite],
    everyone_permissions: u64,
) -> Vec<String> {
    let base_has_view = everyone_permissions & VIEW_CHANNEL != 0;

    overwrites
        .iter()
        .filter(|ow| {
            ow.overwrite_type == 0 // Role overwrites only
                && ((ow.allow & VIEW_CHANNEL != 0) // Explicitly allowed
                    || (base_has_view && ow.deny & VIEW_CHANNEL == 0)) // Not denied when base allows
        })
        .map(|ow| ow.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_with_base_permissions() {
        let result = check_channel_visibility(
            VIEW_CHANNEL, // @everyone has view
            &[],
            &[],
            "guild_id",
            "user_id",
        );
        assert_eq!(result, ChannelVisibility::Visible);
    }

    #[test]
    fn test_hidden_without_base_permissions() {
        let result = check_channel_visibility(
            0, // @everyone has no permissions
            &[],
            &[],
            "guild_id",
            "user_id",
        );
        assert_eq!(result, ChannelVisibility::Hidden);
    }

    #[test]
    fn test_hidden_by_everyone_deny() {
        let overwrites = vec![PermissionOverwrite {
            id: "guild_id".to_string(),
            overwrite_type: 0,
            allow: 0,
            deny: VIEW_CHANNEL,
        }];
        let result =
            check_channel_visibility(VIEW_CHANNEL, &[], &overwrites, "guild_id", "user_id");
        assert_eq!(result, ChannelVisibility::Hidden);
    }

    #[test]
    fn test_visible_by_role_allow() {
        let overwrites = vec![
            PermissionOverwrite {
                id: "guild_id".to_string(),
                overwrite_type: 0,
                allow: 0,
                deny: VIEW_CHANNEL,
            },
            PermissionOverwrite {
                id: "admin_role".to_string(),
                overwrite_type: 0,
                allow: VIEW_CHANNEL,
                deny: 0,
            },
        ];
        let result = check_channel_visibility(
            VIEW_CHANNEL,
            &["admin_role".to_string()],
            &overwrites,
            "guild_id",
            "user_id",
        );
        assert_eq!(result, ChannelVisibility::Visible);
    }

    #[test]
    fn test_member_override() {
        let overwrites = vec![
            PermissionOverwrite {
                id: "guild_id".to_string(),
                overwrite_type: 0,
                allow: 0,
                deny: VIEW_CHANNEL,
            },
            PermissionOverwrite {
                id: "user_id".to_string(),
                overwrite_type: 1,
                allow: VIEW_CHANNEL,
                deny: 0,
            },
        ];
        let result =
            check_channel_visibility(VIEW_CHANNEL, &[], &overwrites, "guild_id", "user_id");
        assert_eq!(result, ChannelVisibility::Visible);
    }
}
