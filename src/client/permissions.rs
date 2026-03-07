// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord permission calculator
//!
//! Computes effective permissions for a user in a channel following
//! Discord's permission hierarchy:
//! 1. Start with @everyone role permissions
//! 2. Apply role permissions (OR all roles)
//! 3. Apply channel @everyone overwrites
//! 4. Apply channel role overwrites
//! 5. Apply channel member-specific overwrite
//! 6. Administrator bypasses everything

use crate::client::Permission;

/// Permission overwrite from channel data
#[derive(Debug, Clone)]
pub struct PermOverwrite {
    pub id: String,
    /// 0 = role, 1 = member
    pub overwrite_type: u8,
    pub allow: u64,
    pub deny: u64,
}

/// Compute the effective permissions for a member in a guild (no channel context)
pub fn compute_base_permissions(
    everyone_permissions: u64,
    member_role_permissions: &[u64],
    owner_id: &str,
    user_id: &str,
) -> u64 {
    // Owner has ALL permissions
    if owner_id == user_id {
        return u64::MAX;
    }

    let mut perms = everyone_permissions;

    // OR together all role permissions
    for role_perms in member_role_permissions {
        perms |= role_perms;
    }

    // Administrator grants all
    if perms & (Permission::Administrator as u64) != 0 {
        return u64::MAX;
    }

    perms
}

/// Compute the effective permissions for a member in a specific channel
pub fn compute_channel_permissions(
    base_permissions: u64,
    overwrites: &[PermOverwrite],
    member_role_ids: &[String],
    everyone_role_id: &str,
    user_id: &str,
) -> u64 {
    // Administrator bypasses all overwrites
    if base_permissions == u64::MAX || base_permissions & (Permission::Administrator as u64) != 0 {
        return u64::MAX;
    }

    let mut perms = base_permissions;

    // 1. Apply @everyone channel overwrite
    if let Some(ow) = overwrites.iter().find(|o| o.id == everyone_role_id) {
        perms &= !ow.deny;
        perms |= ow.allow;
    }

    // 2. Apply role overwrites (combined)
    let mut role_allow: u64 = 0;
    let mut role_deny: u64 = 0;
    for ow in overwrites.iter().filter(|o| {
        o.overwrite_type == 0 && o.id != everyone_role_id && member_role_ids.contains(&o.id)
    }) {
        role_allow |= ow.allow;
        role_deny |= ow.deny;
    }
    perms &= !role_deny;
    perms |= role_allow;

    // 3. Apply member-specific overwrite
    if let Some(ow) = overwrites
        .iter()
        .find(|o| o.overwrite_type == 1 && o.id == user_id)
    {
        perms &= !ow.deny;
        perms |= ow.allow;
    }

    perms
}

/// Check if computed permissions include a specific permission
pub fn has_permission(permissions: u64, perm: Permission) -> bool {
    permissions == u64::MAX || permissions & (perm as u64) != 0
}

/// Get a human-readable list of permission names from a bitfield
pub fn permission_names(permissions: u64) -> Vec<&'static str> {
    let mut names = Vec::new();
    let all_perms = [
        (Permission::CreateInstantInvite, "Create Invite"),
        (Permission::KickMembers, "Kick Members"),
        (Permission::BanMembers, "Ban Members"),
        (Permission::Administrator, "Administrator"),
        (Permission::ManageChannels, "Manage Channels"),
        (Permission::ManageGuild, "Manage Server"),
        (Permission::AddReactions, "Add Reactions"),
        (Permission::ViewAuditLog, "View Audit Log"),
        (Permission::ViewChannel, "View Channel"),
        (Permission::SendMessages, "Send Messages"),
        (Permission::ManageMessages, "Manage Messages"),
        (Permission::EmbedLinks, "Embed Links"),
        (Permission::AttachFiles, "Attach Files"),
        (Permission::ReadMessageHistory, "Read History"),
        (Permission::MentionEveryone, "Mention Everyone"),
        (Permission::Connect, "Connect"),
        (Permission::Speak, "Speak"),
        (Permission::MuteMembers, "Mute Members"),
        (Permission::DeafenMembers, "Deafen Members"),
        (Permission::MoveMembers, "Move Members"),
        (Permission::ManageRoles, "Manage Roles"),
        (Permission::ManageWebhooks, "Manage Webhooks"),
        (Permission::ManageEmojisAndStickers, "Manage Emoji"),
        (Permission::ModerateMembers, "Timeout Members"),
    ];

    for (perm, name) in &all_perms {
        if permissions & (*perm as u64) != 0 {
            names.push(*name);
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEW: u64 = Permission::ViewChannel as u64;
    const SEND: u64 = Permission::SendMessages as u64;
    const ADMIN: u64 = Permission::Administrator as u64;
    const MANAGE_MSG: u64 = Permission::ManageMessages as u64;

    #[test]
    fn test_owner_gets_all() {
        let perms = compute_base_permissions(0, &[], "owner1", "owner1");
        assert_eq!(perms, u64::MAX);
    }

    #[test]
    fn test_admin_gets_all() {
        let perms = compute_base_permissions(ADMIN, &[], "owner", "user1");
        assert_eq!(perms, u64::MAX);
    }

    #[test]
    fn test_role_permissions_combined() {
        let perms = compute_base_permissions(VIEW, &[SEND, MANAGE_MSG], "owner", "user1");
        assert!(perms & VIEW != 0);
        assert!(perms & SEND != 0);
        assert!(perms & MANAGE_MSG != 0);
    }

    #[test]
    fn test_channel_overwrite_deny() {
        let base = VIEW | SEND;
        let overwrites = vec![PermOverwrite {
            id: "everyone".to_string(),
            overwrite_type: 0,
            allow: 0,
            deny: SEND,
        }];
        let perms = compute_channel_permissions(base, &overwrites, &[], "everyone", "user1");
        assert!(perms & VIEW != 0);
        assert!(perms & SEND == 0); // denied by overwrite
    }

    #[test]
    fn test_role_overwrite_overrides_everyone() {
        let base = VIEW | SEND;
        let overwrites = vec![
            PermOverwrite {
                id: "everyone".to_string(),
                overwrite_type: 0,
                allow: 0,
                deny: SEND,
            },
            PermOverwrite {
                id: "mod_role".to_string(),
                overwrite_type: 0,
                allow: SEND,
                deny: 0,
            },
        ];
        let perms = compute_channel_permissions(
            base,
            &overwrites,
            &["mod_role".to_string()],
            "everyone",
            "user1",
        );
        assert!(perms & SEND != 0); // role overwrite restores it
    }

    #[test]
    fn test_member_overwrite_final() {
        let base = VIEW;
        let overwrites = vec![
            PermOverwrite {
                id: "everyone".to_string(),
                overwrite_type: 0,
                allow: 0,
                deny: SEND,
            },
            PermOverwrite {
                id: "user1".to_string(),
                overwrite_type: 1,
                allow: SEND,
                deny: 0,
            },
        ];
        let perms = compute_channel_permissions(base, &overwrites, &[], "everyone", "user1");
        assert!(perms & SEND != 0); // member overwrite grants it
    }

    #[test]
    fn test_admin_bypasses_overwrites() {
        let base = u64::MAX; // admin
        let overwrites = vec![PermOverwrite {
            id: "everyone".to_string(),
            overwrite_type: 0,
            allow: 0,
            deny: u64::MAX,
        }];
        let perms = compute_channel_permissions(base, &overwrites, &[], "everyone", "user1");
        assert_eq!(perms, u64::MAX); // admin ignores all overwrites
    }

    #[test]
    fn test_has_permission() {
        assert!(has_permission(VIEW | SEND, Permission::ViewChannel));
        assert!(!has_permission(VIEW, Permission::SendMessages));
        assert!(has_permission(u64::MAX, Permission::Administrator)); // admin = everything
    }

    #[test]
    fn test_permission_names() {
        let names = permission_names(VIEW | SEND);
        assert!(names.contains(&"View Channel"));
        assert!(names.contains(&"Send Messages"));
        assert!(!names.contains(&"Administrator"));
    }
}
