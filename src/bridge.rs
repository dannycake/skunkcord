// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Backend-to-UI bridge
//!
//! Connects the Rust async backend (gateway events, API responses)
//! to the Qt/QML UI via channels. This is the integration layer that
//! makes the backend and UI work together.

use crate::client::attachments::{self, mime_from_extension};
use crate::client::{DiscordClient, Permission, Relationship};
use chrono::TimeZone;
use crate::features::FeatureFlags;
use crate::plugins::hooks::{GatewayEventHooks, MessageDeleteResult};
use crate::gateway::{GatewayCommand, GatewayEvent, LazyGuildRequest};
use crate::client::permissions::{
    compute_base_permissions, compute_channel_permissions, has_permission, permission_names,
    PermOverwrite,
};
use crate::storage::Storage;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

/// Login request sent from UI to worker thread (pre-auth flow).
#[derive(Debug, Clone)]
pub enum LoginRequest {
    /// Direct token login (existing flow)
    Token(String),
    /// Email/password credential login
    Credentials { email: String, password: String },
    /// Captcha solution (retry the pending login)
    CaptchaSolution {
        captcha_key: String,
        rqtoken: Option<String>,
    },
    /// MFA TOTP code
    MfaCode {
        ticket: String,
        code: String,
        login_instance_id: Option<String>,
    },
    /// User cancelled MFA (e.g. closed the overlay)
    CancelMfa,
    /// Switch to an existing account by user ID (load session from storage)
    SwitchAccount(String),
}

/// UI actions sent from QML to the backend
#[derive(Debug, Clone)]
pub enum UiAction {
    /// User wants to log in with a token
    Login(String),
    /// User wants to log out
    Logout,
    /// User selected a guild (empty string = DMs / Home)
    SelectGuild(String),
    /// User selected a channel — triggers message fetch
    /// Carries (channel_id, channel_type)
    SelectChannel(String, u8),
    /// User wants to send a message
    SendMessage {
        channel_id: String,
        content: String,
        silent: bool,
    },
    /// User started typing
    StartTyping(String),
    /// User wants to edit a message
    EditMessage {
        channel_id: String,
        message_id: String,
        content: String,
    },
    /// User wants to delete a message
    DeleteMessage {
        channel_id: String,
        message_id: String,
    },
    /// User wants to pin a message
    PinMessage {
        channel_id: String,
        message_id: String,
    },
    /// User wants to unpin a message
    UnpinMessage {
        channel_id: String,
        message_id: String,
    },
    /// User wants to view pinned messages for a channel
    OpenPins(String),
    /// User wants to add a reaction
    AddReaction {
        channel_id: String,
        message_id: String,
        emoji: String,
    },
    /// User wants to remove their reaction
    RemoveReaction {
        channel_id: String,
        message_id: String,
        emoji: String,
    },
    /// User changed status
    SetStatus(String),
    /// User changed custom status
    SetCustomStatus(Option<String>),
    /// User wants to switch account
    SwitchAccount(String),
    /// User wants to mark all as read
    MarkAllRead,
    /// Captcha was solved — token from hCaptcha widget
    CaptchaSolved(String),
    /// Open a DM with a specific user
    OpenDm(String),
    /// Join a guild by invite code or URL (discord.gg/... or discord.com/invite/... or raw code)
    JoinGuildByInvite { invite_code_or_url: String },
    /// Send a friend request to a user by username
    SendFriendRequest { username: String },
    /// Accept an incoming friend request
    AcceptFriendRequest { user_id: String },
    /// Remove a relationship (unfriend, reject request, or unblock)
    RemoveRelationship { user_id: String },
    /// Block a user
    BlockUser { user_id: String },
    /// Fetch full user profile for the profile popup (user_id, optional guild_id for guild context)
    FetchUserProfile {
        user_id: String,
        guild_id: Option<String>,
    },
    /// Load more (older) messages for pagination
    LoadMoreMessages {
        channel_id: String,
        before_message_id: String,
    },
    /// Send a message with extended options (silent flag, reply, stickers, file attachments)
    SendMessageEx {
        channel_id: String,
        content: String,
        silent: bool,
        reply_to_message_id: Option<String>,
        /// Sticker IDs to attach (max 3 per message)
        sticker_ids: Option<Vec<String>>,
        /// File attachment paths; read on worker thread.
        attachment_paths: Option<Vec<String>>,
    },
    /// Search GIFs via Tenor (empty string = trending)
    SearchGifs(String),
    /// Load sticker packs for the sticker picker
    LoadStickerPacks,
    /// Load guild emojis for the current guild (for emoji picker "Server" section)
    LoadGuildEmojis(String),
    /// Set plugin enabled state
    SetPluginEnabled { plugin_id: String, enabled: bool },
    /// Install plugin from git repository URL
    InstallPlugin(String),
    /// Refresh plugin list from disk (re-scan plugins directory)
    RefreshPlugins,
    /// Check for updates on plugins with git repos
    CheckPluginUpdates,
    /// Plugin button was clicked
    PluginButtonClicked {
        plugin_id: String,
        button_id: String,
    },
    /// Plugin modal was submitted with field values
    PluginModalSubmitted {
        plugin_id: String,
        modal_id: String,
        fields: std::collections::HashMap<String, String>,
    },
    /// Set proxy configuration
    SetProxySettings {
        enabled: bool,
        host: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
    },
    /// User wants to leave a guild
    LeaveGuild(String),
    /// User wants to mute notifications for a guild
    MuteGuild(String),
    /// User wants to unmute notifications for a guild
    UnmuteGuild(String),
}

/// UI updates sent from the backend to QML
#[derive(Debug, Clone, serde::Serialize)]
pub enum UiUpdate {
    /// Login succeeded
    LoginSuccess {
        user_id: String,
        username: String,
        avatar_url: Option<String>,
    },
    /// Login failed
    LoginFailed(String),
    /// Guild list loaded
    GuildsLoaded(Vec<GuildInfo>),
    /// Channel list loaded for a guild
    ChannelsLoaded(Vec<ChannelInfo>),
    /// DM channel list loaded
    DmChannelsLoaded(Vec<DmChannelInfo>),
    /// Messages loaded for a channel (replaces current message list)
    MessagesLoaded(Vec<MessageInfo>),
    /// New message received (append to current list)
    NewMessage(MessageInfo),
    /// Message was deleted — remove from UI
    MessageDeleted {
        channel_id: String,
        message_id: String,
    },
    /// Message was deleted but we have cached content (from plugin) — show as deleted
    MessageDeletedWithContent {
        channel_id: String,
        message_id: String,
        content: String,
        author_name: String,
        author_id: String,
        timestamp: String,
        author_avatar_url: Option<String>,
    },
    /// Message was edited
    MessageEdited {
        channel_id: String,
        message_id: String,
        new_content: String,
    },
    /// Unread count changed for a channel (guild_id empty for DMs)
    UnreadUpdate {
        channel_id: String,
        guild_id: Option<String>,
        has_unread: bool,
        mention_count: u32,
    },
    /// Captcha required — show the widget
    CaptchaRequired {
        sitekey: String,
        rqdata: Option<String>,
        rqtoken: Option<String>,
        captcha_session_id: Option<String>,
    },
    /// MFA required — show code input (ticket stored for submit_mfa_code)
    MfaRequired {
        ticket: String,
        login_instance_id: Option<String>,
        sms: bool,
        totp: bool,
        backup: bool,
    },
    /// Gateway connected
    Connected,
    /// Gateway disconnected
    Disconnected,
    /// Gateway reconnecting
    Reconnecting,
    /// More (older) messages loaded for pagination
    MoreMessagesLoaded {
        channel_id: String,
        messages: Vec<MessageInfo>,
        has_more: bool,
    },
    /// Pinned messages loaded for a channel
    PinsLoaded {
        channel_id: String,
        messages: Vec<MessageInfo>,
    },
    /// Someone started typing in a channel
    TypingStarted {
        channel_id: String,
        user_name: String,
        /// Role color (hex) for the typing user in this guild, if any
        role_color: Option<String>,
    },
    /// Message reactions updated (add/remove/replace) — UI should update the message's reactions
    MessageReactionsUpdated {
        channel_id: String,
        message_id: String,
        reactions: Vec<ReactionInfo>,
    },
    /// GIF search results loaded
    GifsLoaded(Vec<crate::features::gif_picker::GifResult>),
    /// Sticker packs loaded for picker (list_sticker_packs)
    StickerPacksLoaded(Vec<crate::client::StickerPack>),
    /// Guild emojis loaded for picker (list_guild_emojis)
    GuildEmojisLoaded(Vec<crate::client::GuildEmoji>),
    /// Member list loaded for a guild (from Op 14 Lazy Guild or Op 8 chunk)
    MembersLoaded {
        guild_id: String,
        members: Vec<MemberInfo>,
    },
    /// Error to display
    Error(String),
    /// Current user's profile in the selected guild (roles, nick)
    MyGuildProfile {
        guild_id: String,
        nick: Option<String>,
        roles: Vec<RoleDisplayInfo>,
    },
    /// User profile loaded (curated profile_json for UI, raw_json for developer copy)
    UserProfileLoaded {
        user_id: String,
        profile_json: String,
        raw_json: String,
    },
    /// User presence/status changed (online, idle, dnd, offline)
    PresenceUpdated {
        user_id: String,
        status: String,
    },
    /// Plugin UI elements added/updated (buttons, modals). Sent when plugin enabled.
    PluginUiUpdated {
        plugin_id: String,
        buttons: Vec<crate::plugins::manifest::PluginUiButton>,
        modals: Vec<crate::plugins::manifest::PluginUiModal>,
    },
    /// Plugin UI removed (plugin disabled)
    PluginUiRemoved { plugin_id: String },
    /// Plugins refreshed from disk — UI should call get_plugin_list again
    PluginsRefreshed,
    /// Plugin update check completed — JSON array of { plugin_id, has_update, current_version }
    PluginUpdatesAvailable(String),
    /// Relationships (friends, pending, blocked) loaded or updated
    RelationshipsLoaded(Vec<RelationshipInfo>),
    /// One relationship added (from gateway)
    RelationshipAdded(RelationshipInfo),
    /// One relationship removed (from gateway)
    RelationshipRemoved { user_id: String },
    /// User successfully joined a guild via invite — append to guild list
    JoinGuildSuccess(GuildInfo),
    /// Failed to join a guild via invite — show error in join popup (not login error_message)
    JoinGuildFailed(String),
    /// Guild mute state changed (notifications) — UI should update mutedGuildIds
    GuildMuteStateChanged {
        guild_id: String,
        muted: bool,
    },
    /// An invite was received from the browser via RPC handoff — show join prompt
    RpcInviteReceived(String),
    /// List of saved accounts for switcher (user_id, display_name); display_name may be empty
    AccountsList(Vec<(String, String)>),
}

/// Simplified guild info for UI
#[derive(Debug, Clone, serde::Serialize)]
pub struct GuildInfo {
    pub id: String,
    pub name: String,
    pub icon_url: Option<String>,
    pub has_unread: bool,
    pub mention_count: u32,
}

/// Simplified channel info for UI
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChannelInfo {
    pub id: String,
    pub guild_id: Option<String>,
    pub name: String,
    pub channel_type: u8,
    pub position: i32,
    pub parent_id: Option<String>,
    pub has_unread: bool,
    pub mention_count: u32,
    /// True if user lacks VIEW_CHANNEL permission
    pub is_hidden: bool,
}

/// Role display info for profile popup (name, color, position)
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoleDisplayInfo {
    pub id: String,
    pub name: String,
    pub color: String,
    pub position: i32,
}

/// Simplified member info for the guild member list (Op 14 / Op 8)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemberInfo {
    pub user_id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_flags: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premium_type: Option<u8>,
}

/// Simplified DM channel info for UI
#[derive(Debug, Clone, serde::Serialize)]
pub struct DmChannelInfo {
    pub id: String,
    pub recipient_name: String,
    pub recipient_id: String,
    pub recipient_avatar_url: Option<String>,
    pub channel_type: u8,
    /// Used to sort DM list by recency (Discord snowflake — higher = more recent)
    pub last_message_id: Option<String>,
}

/// Relationship info for Friends tab (friend, incoming/outgoing request, blocked)
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct RelationshipInfo {
    pub user_id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// 1 = friend, 2 = blocked, 3 = incoming request, 4 = outgoing request
    pub relationship_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
}

fn relationship_info_from(r: &Relationship) -> RelationshipInfo {
    RelationshipInfo {
        user_id: r.user.id.clone(),
        username: r.user.display_name().to_string(),
        avatar_url: Some(r.user.avatar_url(128)),
        relationship_type: r.relationship_type,
        nickname: r.nickname.clone(),
    }
}

/// Parse a Discord member object (user + optional nick) into MemberInfo.
/// Used for GUILD_MEMBER_LIST_UPDATE items and GUILD_MEMBERS_CHUNK members.
/// If `guild_roles` is provided, computes role_name and role_color from the member's roles.
fn member_info_from_member_json(
    member: &serde_json::Value,
    guild_roles: Option<&[crate::client::Role]>,
) -> Option<MemberInfo> {
    let user = member.get("user").and_then(|v| v.as_object()).or_else(|| member.as_object())?;
    let user_id = user.get("id").and_then(|v| v.as_str())?.to_string();
    let username = user
        .get("username")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let display_name = member
        .get("nick")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| user.get("global_name").and_then(|v| v.as_str()).map(|s| s.to_string()));
    let avatar_url = user.get("avatar").and_then(|v| v.as_str()).map(|hash| {
        let ext = if hash.starts_with("a_") { "gif" } else { "png" };
        format!(
            "https://cdn.discordapp.com/avatars/{}/{}.{}?size=64",
            user_id, hash, ext
        )
    });
    let public_flags = user.get("public_flags").and_then(|v| v.as_u64());
    let bot = user.get("bot").and_then(|v| v.as_bool());
    let premium_type = user.get("premium_type").and_then(|v| v.as_u64()).map(|v| v as u8);

    let (role_name, role_color) = if let Some(roles) = guild_roles {
        let member_role_ids: Vec<&str> = member
            .get("roles")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        roles
            .iter()
            .filter(|r| member_role_ids.contains(&r.id.as_str()))
            .filter(|r| r.color != 0)
            .max_by_key(|r| r.position)
            .map(|r| (Some(r.name.clone()), Some(r.color_hex())))
            .unwrap_or((None, None))
    } else {
        (None, None)
    };

    Some(MemberInfo {
        user_id,
        username: if username.is_empty() { "Unknown".to_string() } else { username },
        display_name,
        avatar_url,
        role_name,
        role_color,
        public_flags,
        bot,
        premium_type,
    })
}

/// Helper: look up a user ID in a map of user JSON objects, extracting name + avatar.
fn resolve_user_from_map(
    uid: &str,
    user_map: &HashMap<&str, &serde_json::Value>,
) -> (String, String, Option<String>) {
    if let Some(user) = user_map.get(uid) {
        let uname = user
            .get("global_name")
            .and_then(|v| v.as_str())
            .or_else(|| user.get("username").and_then(|v| v.as_str()))
            .unwrap_or("Unknown");
        let avatar_url = user.get("avatar").and_then(|v| v.as_str()).map(|hash| {
            let ext = if hash.starts_with("a_") { "gif" } else { "png" };
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.{}?size=64",
                uid, hash, ext
            )
        });
        (uname.to_string(), uid.to_string(), avatar_url)
    } else {
        ("Unknown User".to_string(), uid.to_string(), None)
    }
}

/// Extract a `DmChannelInfo` from a raw JSON private_channel object.
///
/// Tries `recipients` (array of user objects) first, then falls back to
/// `recipient_ids` (array of user-ID strings) resolved via `user_map`.
fn extract_dm_from_json(
    c: &serde_json::Value,
    user_map: &HashMap<&str, &serde_json::Value>,
) -> Option<DmChannelInfo> {
    let id = c.get("id").and_then(|v| v.as_str())?;
    let channel_type = c.get("type").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
    let last_message_id = c
        .get("last_message_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let recipients = c.get("recipients").and_then(|v| v.as_array());
    let recipient_ids = c.get("recipient_ids").and_then(|v| v.as_array());

    if channel_type == 1 {
        // 1:1 DM — try full recipients first, then recipient_ids
        let (name, rid, avatar) = if let Some(r) = recipients.and_then(|arr| arr.first()) {
            let uname = r
                .get("global_name")
                .and_then(|v| v.as_str())
                .or_else(|| r.get("username").and_then(|v| v.as_str()))
                .unwrap_or("Unknown");
            let uid = r.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let avatar_url = r.get("avatar").and_then(|v| v.as_str()).map(|hash| {
                let ext = if hash.starts_with("a_") { "gif" } else { "png" };
                format!(
                    "https://cdn.discordapp.com/avatars/{}/{}.{}?size=64",
                    uid, hash, ext
                )
            });
            (uname.to_string(), uid.to_string(), avatar_url)
        } else if let Some(uid_str) = recipient_ids
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
        {
            resolve_user_from_map(uid_str, user_map)
        } else {
            ("Unknown User".to_string(), String::new(), None)
        };
        Some(DmChannelInfo {
            id: id.to_string(),
            recipient_name: name,
            recipient_id: rid,
            recipient_avatar_url: avatar,
            channel_type,
            last_message_id,
        })
    } else if channel_type == 3 {
        // Group DM
        let name = c
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                // Try full recipients first
                if let Some(arr) = recipients {
                    let names: Vec<&str> = arr
                        .iter()
                        .filter_map(|u| {
                            u.get("global_name")
                                .and_then(|v| v.as_str())
                                .or_else(|| u.get("username").and_then(|v| v.as_str()))
                        })
                        .collect();
                    if !names.is_empty() {
                        return names.join(", ");
                    }
                }
                // Fall back to recipient_ids
                if let Some(ids) = recipient_ids {
                    let names: Vec<String> = ids
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|uid| resolve_user_from_map(uid, user_map).0)
                        .collect();
                    if !names.is_empty() {
                        return names.join(", ");
                    }
                }
                "Group DM".to_string()
            });
        let first_rid = recipients
            .and_then(|arr| arr.first())
            .and_then(|u| u.get("id").and_then(|v| v.as_str()))
            .map(|s| s.to_string())
            .or_else(|| {
                recipient_ids
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_default();
        Some(DmChannelInfo {
            id: id.to_string(),
            recipient_name: name,
            recipient_id: first_rid,
            recipient_avatar_url: None,
            channel_type,
            last_message_id,
        })
    } else {
        None
    }
}

/// Parse permission_overwrites from a channel JSON object into PermOverwrite (allow/deny as u64).
fn overwrites_from_channel_json(c: &serde_json::Value) -> Vec<PermOverwrite> {
    let arr = match c.get("permission_overwrites").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return vec![],
    };
    arr.iter()
        .filter_map(|o| {
            let id = o.get("id").and_then(|v| v.as_str())?.to_string();
            let overwrite_type = o.get("type").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let allow = o
                .get("allow")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .parse::<u64>()
                .unwrap_or(0);
            let deny = o
                .get("deny")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .parse::<u64>()
                .unwrap_or(0);
            Some(PermOverwrite {
                id,
                overwrite_type,
                allow,
                deny,
            })
        })
        .collect()
}

/// Compute whether the current user can view a channel (VIEW_CHANNEL). Returns true if hidden.
fn channel_is_hidden(
    guild_id: &str,
    my_user_id: &str,
    owner_id: &str,
    everyone_permissions: u64,
    member_role_permissions: &[u64],
    my_role_ids: &[String],
    overwrites: &[PermOverwrite],
) -> bool {
    let base = compute_base_permissions(
        everyone_permissions,
        member_role_permissions,
        owner_id,
        my_user_id,
    );
    let effective = compute_channel_permissions(
        base,
        overwrites,
        my_role_ids,
        guild_id,
        my_user_id,
    );
    !has_permission(effective, Permission::ViewChannel)
}

/// Reaction info for UI (one emoji + count + whether current user reacted)
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReactionInfo {
    /// Display string: unicode emoji as-is, or ":name:id" for custom
    pub emoji_display: String,
    pub count: u32,
    pub me: bool,
}

/// Simplified message info for UI
#[derive(Debug, Clone, serde::Serialize)]
pub struct MessageInfo {
    pub id: String,
    pub channel_id: String,
    pub author_name: String,
    pub author_id: String,
    pub author_avatar_url: Option<String>,
    pub content: String,
    pub timestamp: String,
    pub is_deleted: bool,
    pub edit_count: usize,
    /// Discord message type (0 = default, 7 = member join, 8 = boost, 19 = reply, etc.)
    pub message_type: u8,
    /// Reply: name of the author of the referenced message
    pub reply_author_name: Option<String>,
    /// Reply: content preview of the referenced message
    pub reply_content: Option<String>,
    /// Reply: author ID of the referenced message
    pub reply_author_id: Option<String>,
    /// Reply: author's highest role color in this guild (hex). None for DMs or no role.
    pub reply_author_role_color: Option<String>,
    /// Whether this message mentions the current user (via @user, @role, @everyone, @here)
    pub mentions_me: bool,
    /// Whether this message mentions @everyone or @here
    pub mention_everyone: bool,
    /// Author's highest role color in this guild (hex e.g. "#5865f2"). None = use default accent in UI (DMs or no colored role).
    pub author_role_color: Option<String>,
    /// Author's highest role name in this guild (e.g. "Moderator"). None = no role pill.
    pub author_role_name: Option<String>,
    /// User's public_flags bitfield for badges (Staff, Partner, HypeSquad, etc.)
    pub author_public_flags: u64,
    /// Whether the author is a bot
    pub author_bot: bool,
    /// Nitro tier: 0=none, 1=classic, 2=nitro, 3=basic
    pub author_premium_type: u8,
    /// JSON array of attachment objects for the UI: {id, filename, url, proxy_url, width?, height?, content_type?}
    pub attachments_json: String,
    /// JSON array of sticker objects for the UI: {id, name, url} where url is the CDN image URL
    pub stickers_json: String,
    /// JSON array of embed objects for the UI: {title, description, url, color, author, thumbnail, image, fields, footer}
    pub embeds_json: String,
    /// HTML-rendered content (Discord markdown → HTML) for RichText display; empty for system messages
    pub content_html: String,
    /// Reactions on this message (emoji display, count, me)
    pub reactions: Vec<ReactionInfo>,
}

/// Discord message type constants
pub const MSG_TYPE_DEFAULT: u8 = 0;
pub const MSG_TYPE_RECIPIENT_ADD: u8 = 1;
pub const MSG_TYPE_RECIPIENT_REMOVE: u8 = 2;
pub const MSG_TYPE_CALL: u8 = 3;
pub const MSG_TYPE_CHANNEL_NAME_CHANGE: u8 = 4;
pub const MSG_TYPE_CHANNEL_ICON_CHANGE: u8 = 5;
pub const MSG_TYPE_CHANNEL_PINNED: u8 = 6;
pub const MSG_TYPE_MEMBER_JOIN: u8 = 7;
pub const MSG_TYPE_BOOST: u8 = 8;
pub const MSG_TYPE_BOOST_TIER_1: u8 = 9;
pub const MSG_TYPE_BOOST_TIER_2: u8 = 10;
pub const MSG_TYPE_BOOST_TIER_3: u8 = 11;
pub const MSG_TYPE_CHANNEL_FOLLOW_ADD: u8 = 12;
pub const MSG_TYPE_GUILD_DISCOVERY_DISQUALIFIED: u8 = 14;
pub const MSG_TYPE_GUILD_DISCOVERY_REQUALIFIED: u8 = 15;
pub const MSG_TYPE_GUILD_DISCOVERY_GRACE_PERIOD_INITIAL_WARNING: u8 = 16;
pub const MSG_TYPE_GUILD_DISCOVERY_GRACE_PERIOD_FINAL_WARNING: u8 = 17;
pub const MSG_TYPE_THREAD_CREATED: u8 = 18;
pub const MSG_TYPE_REPLY: u8 = 19;
pub const MSG_TYPE_CHAT_INPUT_COMMAND: u8 = 20;
pub const MSG_TYPE_THREAD_STARTER: u8 = 21;
pub const MSG_TYPE_GUILD_INVITE_REMINDER: u8 = 22;
pub const MSG_TYPE_CONTEXT_MENU_COMMAND: u8 = 23;
pub const MSG_TYPE_AUTO_MODERATION: u8 = 24;
pub const MSG_TYPE_ROLE_SUBSCRIPTION: u8 = 25;
pub const MSG_TYPE_INTERACTION_PREMIUM_UPSELL: u8 = 26;
pub const MSG_TYPE_STAGE_START: u8 = 27;
pub const MSG_TYPE_STAGE_END: u8 = 28;
pub const MSG_TYPE_STAGE_SPEAKER: u8 = 29;
pub const MSG_TYPE_STAGE_TOPIC: u8 = 31;
pub const MSG_TYPE_APPLICATION_PREMIUM_SUBSCRIPTION: u8 = 32;
pub const MSG_TYPE_GUILD_INCIDENT_ALERT_MODE_ENABLED: u8 = 36;
pub const MSG_TYPE_GUILD_INCIDENT_ALERT_MODE_DISABLED: u8 = 37;
pub const MSG_TYPE_GUILD_INCIDENT_REPORT_RAID: u8 = 38;
pub const MSG_TYPE_GUILD_INCIDENT_REPORT_FALSE_ALARM: u8 = 39;
pub const MSG_TYPE_PURCHASE_NOTIFICATION: u8 = 44;
pub const MSG_TYPE_POLL_RESULT: u8 = 46;

/// Check if a message type has normal user content to display
fn message_type_has_content(t: u8) -> bool {
    matches!(t, MSG_TYPE_DEFAULT | MSG_TYPE_REPLY | MSG_TYPE_CHAT_INPUT_COMMAND | MSG_TYPE_CONTEXT_MENU_COMMAND)
}

/// Get a human-readable system message description for non-content message types
fn system_message_text(t: u8, author: &str) -> String {
    match t {
        MSG_TYPE_RECIPIENT_ADD => format!("{} added someone to the group.", author),
        MSG_TYPE_RECIPIENT_REMOVE => format!("{} removed someone from the group.", author),
        MSG_TYPE_CALL => format!("{} started a call.", author),
        MSG_TYPE_CHANNEL_NAME_CHANGE => format!("{} changed the channel name.", author),
        MSG_TYPE_CHANNEL_ICON_CHANGE => format!("{} changed the channel icon.", author),
        MSG_TYPE_CHANNEL_PINNED => format!("{} pinned a message to this channel.", author),
        MSG_TYPE_MEMBER_JOIN => format!("Welcome, {}! Enjoy your stay.", author),
        MSG_TYPE_BOOST => format!("{} just boosted the server!", author),
        MSG_TYPE_BOOST_TIER_1 => format!("{} just boosted the server! This server has achieved Level 1!", author),
        MSG_TYPE_BOOST_TIER_2 => format!("{} just boosted the server! This server has achieved Level 2!", author),
        MSG_TYPE_BOOST_TIER_3 => format!("{} just boosted the server! This server has achieved Level 3!", author),
        MSG_TYPE_CHANNEL_FOLLOW_ADD => format!("{} has added a channel to follow.", author),
        MSG_TYPE_GUILD_DISCOVERY_DISQUALIFIED => "This server has been removed from Server Discovery.".to_string(),
        MSG_TYPE_GUILD_DISCOVERY_REQUALIFIED => "This server is eligible for Server Discovery again.".to_string(),
        MSG_TYPE_GUILD_DISCOVERY_GRACE_PERIOD_INITIAL_WARNING => {
            "This server is in danger of being removed from Server Discovery.".to_string()
        }
        MSG_TYPE_GUILD_DISCOVERY_GRACE_PERIOD_FINAL_WARNING => {
            "This server has been removed from Server Discovery for not meeting the minimum activity requirement.".to_string()
        }
        MSG_TYPE_THREAD_CREATED => format!("{} started a thread.", author),
        MSG_TYPE_THREAD_STARTER => format!("{} started a thread.", author),
        MSG_TYPE_GUILD_INVITE_REMINDER => "Don't forget to invite your friends!".to_string(),
        MSG_TYPE_AUTO_MODERATION => "AutoMod has blocked a message.".to_string(),
        MSG_TYPE_ROLE_SUBSCRIPTION => format!("{} subscribed to a role.", author),
        MSG_TYPE_INTERACTION_PREMIUM_UPSELL => "This interaction requires a premium subscription.".to_string(),
        MSG_TYPE_STAGE_START => format!("{} started a Stage.", author),
        MSG_TYPE_STAGE_END => format!("{} ended the Stage.", author),
        MSG_TYPE_STAGE_SPEAKER => format!("{} is now a speaker.", author),
        MSG_TYPE_STAGE_TOPIC => format!("{} changed the Stage topic.", author),
        MSG_TYPE_APPLICATION_PREMIUM_SUBSCRIPTION => format!("{} subscribed to a premium app.", author),
        MSG_TYPE_GUILD_INCIDENT_ALERT_MODE_ENABLED => {
            "Incident alerts have been enabled for this server.".to_string()
        }
        MSG_TYPE_GUILD_INCIDENT_ALERT_MODE_DISABLED => {
            "Incident alerts have been disabled for this server.".to_string()
        }
        MSG_TYPE_GUILD_INCIDENT_REPORT_RAID => "A raid was reported and is being reviewed.".to_string(),
        MSG_TYPE_GUILD_INCIDENT_REPORT_FALSE_ALARM => {
            "The reported raid was marked as a false alarm.".to_string()
        }
        MSG_TYPE_PURCHASE_NOTIFICATION => format!("{} made a purchase.", author),
        MSG_TYPE_POLL_RESULT => "Poll has ended. See results above.".to_string(),
        _ => format!("[Unsupported message type {}]", t),
    }
}

impl MessageInfo {
    /// Convert a Discord API `Message` into a `MessageInfo` for the UI.
    /// `my_user_id` is the current user's ID, used to detect mentions.
    /// `my_role_ids` are the role IDs the current user has in this guild.
    pub fn from_message_with_context(
        m: &crate::client::Message,
        my_user_id: &str,
        my_role_ids: &[String],
    ) -> Self {
        // Extract reply info
        let (reply_author_name, reply_content, reply_author_id) =
            if let Some(ref referenced) = m.referenced_message {
                let rname = referenced
                    .author
                    .as_ref()
                    .map(|a| a.display_name().to_string());
                let rcontent = Some(if referenced.content.len() > 120 {
                    format!("{}...", &referenced.content[..120])
                } else {
                    referenced.content.clone()
                });
                let rid = referenced.author.as_ref().map(|a| a.id.clone());
                (rname, rcontent, rid)
            } else {
                (None, None, None)
            };

        // Detect if this message mentions the current user
        let mentioned_by_user = m.mentions.iter().any(|u| u.id == my_user_id);
        let mentioned_by_role = m
            .mention_roles
            .iter()
            .any(|role_id| my_role_ids.contains(role_id));
        let mentions_me = mentioned_by_user || mentioned_by_role || m.mention_everyone;

        // For system messages, generate a descriptive content string
        let content = if message_type_has_content(m.message_type) {
            m.content.clone()
        } else {
            let author_name = m
                .author
                .as_ref()
                .map(|a| a.display_name().to_string())
                .unwrap_or_else(|| "Someone".to_string());
            system_message_text(m.message_type, &author_name)
        };

        let attachments_json = serde_json::to_string(&m.attachments).unwrap_or_else(|_| "[]".to_string());
        let embeds_json = serde_json::to_string(&m.embeds).unwrap_or_else(|_| "[]".to_string());
        let stickers_json = serde_json::to_string(
            &m.sticker_items
                .iter()
                .map(|s| {
                    let url = crate::client::sticker_cdn_url(&s.id, s.format_type);
                    serde_json::json!({ "id": s.id, "name": s.name, "url": url })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap_or_else(|_| "[]".to_string());
        let content_html = if message_type_has_content(m.message_type) && !content.is_empty() {
            crate::rendering::markdown::parse_markdown(&content)
        } else {
            String::new()
        };

        let reactions: Vec<ReactionInfo> = m
            .reactions
            .iter()
            .map(|r| ReactionInfo {
                emoji_display: r.emoji.display_string(),
                count: r.count,
                me: r.me,
            })
            .collect();

        Self {
            id: m.id.clone(),
            channel_id: m.channel_id.clone(),
            author_name: m
                .author
                .as_ref()
                .map(|a| a.display_name().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            author_id: m
                .author
                .as_ref()
                .map(|a| a.id.clone())
                .unwrap_or_default(),
            author_avatar_url: m.author.as_ref().map(|a| a.avatar_url(64)),
            content,
            timestamp: m.timestamp.clone(),
            is_deleted: false,
            edit_count: 0,
            message_type: m.message_type,
            reply_author_name,
            reply_content,
            reply_author_id,
            reply_author_role_color: None,
            mentions_me,
            mention_everyone: m.mention_everyone,
            author_role_color: None,
            author_role_name: None,
            author_public_flags: m
                .author
                .as_ref()
                .and_then(|a| a.public_flags)
                .unwrap_or(0),
            author_bot: m.author.as_ref().and_then(|a| a.bot).unwrap_or(false),
            author_premium_type: m
                .author
                .as_ref()
                .and_then(|a| a.premium_type)
                .unwrap_or(0),
            attachments_json,
            stickers_json,
            embeds_json,
            content_html,
            reactions,
        }
    }

    /// Simple conversion without user context (for tests / fallback)
    pub fn from_message(m: &crate::client::Message) -> Self {
        Self::from_message_with_context(m, "", &[])
    }
}

/// The backend bridge that processes UI actions and emits UI updates
pub struct BackendBridge {
    /// Channel to receive UI actions
    pub action_rx: mpsc::Receiver<UiAction>,
    /// Channel to send UI updates
    pub update_tx: broadcast::Sender<UiUpdate>,
    /// Discord client
    pub client: Arc<DiscordClient>,
    /// Feature flags
    pub flags: Arc<RwLock<FeatureFlags>>,
    /// Plugin enable overrides (plugin_id -> enabled). Takes precedence over flags.
    pub plugin_enabled: Arc<RwLock<std::collections::HashMap<String, bool>>>,
    /// Gateway event hooks (plugins intercept events here — bridge never imports plugin code)
    pub gateway_hooks: Option<Arc<dyn GatewayEventHooks>>,
    /// In-memory bridge cache for channels/messages/DMs
    pub bridge_cache: SharedBridgeCache,
}

impl BackendBridge {
    /// Create a new bridge with channels for bidirectional communication
    pub fn new(
        client: Arc<DiscordClient>,
        flags: FeatureFlags,
        plugin_enabled: std::collections::HashMap<String, bool>,
        gateway_hooks: Option<Arc<dyn GatewayEventHooks>>,
    ) -> (Self, mpsc::Sender<UiAction>, broadcast::Receiver<UiUpdate>) {
        let (action_tx, action_rx) = mpsc::channel(256);
        let (update_tx, update_rx) = broadcast::channel(256);

        let bridge = Self {
            action_rx,
            update_tx: update_tx.clone(),
            client,
            flags: Arc::new(RwLock::new(flags)),
            plugin_enabled: Arc::new(RwLock::new(plugin_enabled)),
            gateway_hooks,
            bridge_cache: Arc::new(tokio::sync::Mutex::new(BridgeCache::default())),
        };

        (bridge, action_tx, update_rx)
    }

    /// Process a UI action with feature flag awareness.
    /// This is where feature flags are enforced on user-initiated actions.
    pub async fn handle_action(&self, action: &UiAction) {
        let flags = self.flags.read().await;

        match action {
            UiAction::SendMessage {
                channel_id,
                content,
                silent,
            } => {
                let mut final_content = content.clone();

                // ClearURLs: strip tracking params from outgoing URLs
                if flags.clear_urls {
                    final_content = crate::security::content::clean_message_urls(&final_content);
                }

                // Build message payload
                let mut msg = crate::client::CreateMessage::text(&final_content);

                // Silent message toggle
                if *silent && flags.silent_message_toggle {
                    msg.flags = Some(crate::features::silent_messages::apply_silent_flag(
                        msg.flags,
                    ));
                }

                match self.client.send_message(channel_id, msg).await {
                    Ok(_) => tracing::debug!("Message sent to {}", channel_id),
                    Err(e) => {
                        let _ = self
                            .update_tx
                            .send(UiUpdate::Error(format!("Send failed: {}", e)));
                    }
                }
            }
            UiAction::SetStatus(status) => {
                if let Err(e) = self.client.update_presence(status, None).await {
                    tracing::warn!("Failed to update status: {}", e);
                }
            }
            UiAction::MarkAllRead => {
                if flags.read_all_button {
                    // Use the read_states manager to generate ACK list
                    // For now, just log — in full integration this would
                    // iterate the ReadStateManager
                    tracing::info!("Mark all as read requested");
                }
            }
            _ => {
                tracing::debug!("Unhandled UI action: {:?}", action);
            }
        }
    }

    /// Process a gateway event and convert to UI updates
    pub async fn handle_gateway_event(&self, event: &GatewayEvent) {
        match event {
            GatewayEvent::MessageCreate(msg) => {
                if let Some(ref hooks) = self.gateway_hooks {
                    hooks.on_message_create(msg);
                }
                let my_uid = self.bridge_cache.lock().await.my_user_id.clone();
                let mut info = MessageInfo::from_message_with_context(&msg.message, &my_uid, &[]);
                let channel_id = msg.message.channel_id.clone();
                let mut guild_id = self.bridge_cache.lock().await.channel_guild.get(&channel_id).cloned();
                if guild_id.is_none() {
                    if let Ok(ch) = self.client.get_channel(&channel_id).await {
                        if let Some(ref gid) = ch.guild_id {
                            self.bridge_cache.lock().await.channel_guild.insert(channel_id.clone(), gid.clone());
                            guild_id = Some(gid.clone());
                        }
                    }
                }
                if let Some(ref gid) = guild_id {
                    // Try local cache first, fall back to REST
                    let author_info = {
                        let c = self.bridge_cache.lock().await;
                        resolve_role_color_local(gid, &info.author_id, &c)
                    };
                    match author_info {
                        Some(Some((color, name))) => {
                            info.author_role_color = Some(color);
                            info.author_role_name = Some(name);
                        }
                        Some(None) => {} // cached as no-role
                        None => {
                            if let Some((color, name)) =
                                get_author_role_info(&self.client, gid, &info.author_id, &self.bridge_cache).await
                            {
                                info.author_role_color = Some(color);
                                info.author_role_name = Some(name);
                            }
                        }
                    }
                    if let Some(ref rid) = info.reply_author_id {
                        let reply_info = {
                            let c = self.bridge_cache.lock().await;
                            resolve_role_color_local(gid, rid, &c)
                        };
                        match reply_info {
                            Some(Some((color, _))) => {
                                info.reply_author_role_color = Some(color);
                            }
                            Some(None) => {}
                            None => {
                                if let Some((color, _)) =
                                    get_author_role_info(&self.client, gid, rid, &self.bridge_cache).await
                                {
                                    info.reply_author_role_color = Some(color);
                                }
                            }
                        }
                    }
                }
                let _ = self.update_tx.send(UiUpdate::NewMessage(info));
            }
            GatewayEvent::MessageDelete(del) => {
                let update = if let Some(ref hooks) = self.gateway_hooks {
                    match hooks.on_message_delete(del) {
                        MessageDeleteResult::Remove => UiUpdate::MessageDeleted {
                            channel_id: del.channel_id.clone(),
                            message_id: del.id.clone(),
                        },
                        MessageDeleteResult::ShowAsDeleted {
                            channel_id,
                            message_id,
                            content,
                            author_name,
                            author_id,
                            timestamp,
                            author_avatar_url,
                        } => UiUpdate::MessageDeletedWithContent {
                            channel_id,
                            message_id,
                            content,
                            author_name,
                            author_id,
                            timestamp,
                            author_avatar_url,
                        },
                    }
                } else {
                    UiUpdate::MessageDeleted {
                        channel_id: del.channel_id.clone(),
                        message_id: del.id.clone(),
                    }
                };
                let _ = self.update_tx.send(update);
            }
            GatewayEvent::MessageUpdate(upd) => {
                if let Some(ref hooks) = self.gateway_hooks {
                    hooks.on_message_update(upd);
                }

                // Notify the UI even for embed/attachment-only edits.
                // If content wasn't updated, send the existing content as empty-string
                // so the UI knows the message was touched (edited_timestamp changed).
                let _ = self.update_tx.send(UiUpdate::MessageEdited {
                    channel_id: upd.channel_id.clone(),
                    message_id: upd.id.clone(),
                    new_content: upd.content.clone().unwrap_or_default(),
                });
            }
            GatewayEvent::MessageDeleteBulk(bulk) => {
                if let Some(ref hooks) = self.gateway_hooks {
                    hooks.on_message_delete_bulk(bulk);
                }
                for id in &bulk.ids {
                    let _ = self.update_tx.send(UiUpdate::MessageDeleted {
                        channel_id: bulk.channel_id.clone(),
                        message_id: id.clone(),
                    });
                }
            }
            GatewayEvent::Ready(ready) => {
                // Store current user ID in cache for mention detection
                self.bridge_cache.lock().await.my_user_id = ready.user.id.clone();

                let _ = self.update_tx.send(UiUpdate::LoginSuccess {
                    user_id: ready.user.id.clone(),
                    username: ready.user.display_name().to_string(),
                    avatar_url: Some(ready.user.avatar_url(128)),
                });
                let _ = self.update_tx.send(UiUpdate::Connected);

                // Extract guild list from READY
                let guilds: Vec<GuildInfo> = ready
                    .guilds
                    .iter()
                    .filter_map(|g| {
                        let id = g.get("id").and_then(|v| v.as_str())?;
                        // Guild name is in "properties.name" (new format) or "name" (old format)
                        let name = g
                            .get("properties")
                            .and_then(|p| p.get("name"))
                            .and_then(|v| v.as_str())
                            .or_else(|| g.get("name").and_then(|v| v.as_str()))
                            .unwrap_or("Unknown Guild");
                        let icon = g
                            .get("properties")
                            .and_then(|p| p.get("icon"))
                            .and_then(|v| v.as_str())
                            .or_else(|| g.get("icon").and_then(|v| v.as_str()));
                        let icon_url = icon.map(|hash| {
                            let ext = if hash.starts_with("a_") { "gif" } else { "png" };
                            format!(
                                "https://cdn.discordapp.com/icons/{}/{}.{}?size=128",
                                id, hash, ext
                            )
                        });
                        Some(GuildInfo {
                            id: id.to_string(),
                            name: name.to_string(),
                            icon_url,
                            has_unread: false,
                            mention_count: 0,
                        })
                    })
                    .collect();

                if !guilds.is_empty() {
                    tracing::info!("Loaded {} guilds from READY", guilds.len());
                    let _ = self.update_tx.send(UiUpdate::GuildsLoaded(guilds));
                }

                // Extract DM channels from READY private_channels
                // Build user lookup map from READY users array (Discord v10 sends
                // recipient_ids instead of full recipients in private_channels)
                if !ready.private_channels.is_empty() {
                    let user_map: HashMap<&str, &serde_json::Value> = ready
                        .users
                        .iter()
                        .filter_map(|u| {
                            let uid = u.get("id").and_then(|v| v.as_str())?;
                            Some((uid, u))
                        })
                        .collect();

                    let mut dm_infos: Vec<DmChannelInfo> = ready
                        .private_channels
                        .iter()
                        .filter_map(|c| extract_dm_from_json(c, &user_map))
                        .collect();

                    // Sort by recency
                    dm_infos.sort_by(|a, b| {
                        let a_id: u64 = a.last_message_id.as_deref().unwrap_or("0").parse().unwrap_or(0);
                        let b_id: u64 = b.last_message_id.as_deref().unwrap_or("0").parse().unwrap_or(0);
                        b_id.cmp(&a_id)
                    });

                    tracing::info!(
                        "Loaded {} DM channels from READY",
                        dm_infos.len()
                    );
                    // Populate bridge cache
                    self.bridge_cache.lock().await.dm_channels = Some(dm_infos.clone());
                    let _ = self.update_tx.send(UiUpdate::DmChannelsLoaded(dm_infos));
                }

                // Extract relationships (friends, pending, blocked) from READY, or fetch via API
                let relationships: Vec<RelationshipInfo> = {
                    let from_ready: Vec<RelationshipInfo> = ready
                        .relationships
                        .iter()
                        .filter_map(|v| serde_json::from_value::<Relationship>(v.clone()).ok())
                        .map(|r| relationship_info_from(&r))
                        .collect();
                    if !from_ready.is_empty() {
                        tracing::info!("Loaded {} relationships from READY", from_ready.len());
                        from_ready
                    } else if let Ok(rels) = self.client.get_relationships().await {
                        let infos: Vec<RelationshipInfo> =
                            rels.iter().map(relationship_info_from).collect();
                        tracing::info!("Loaded {} relationships via API", infos.len());
                        infos
                    } else {
                        vec![]
                    }
                };
                if !relationships.is_empty() {
                    let _ = self.update_tx.send(UiUpdate::RelationshipsLoaded(relationships));
                }

                // Extract channels from each guild in READY and populate cache (with permission-based is_hidden)
                let my_user_id = self.bridge_cache.lock().await.my_user_id.clone();
                for (guild_idx, guild) in ready.guilds.iter().enumerate() {
                    let guild_id = match guild.get("id").and_then(|v| v.as_str()) {
                        Some(id) => id,
                        None => continue,
                    };
                    let owner_id = guild
                        .get("owner_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    // Parse guild roles and cache
                    let roles: Vec<crate::client::Role> = guild
                        .get("roles")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    let everyone_permissions = roles
                        .iter()
                        .find(|r| r.id == guild_id)
                        .and_then(|r| r.permissions.parse::<u64>().ok())
                        .unwrap_or(0);
                    // Current user's role IDs in this guild (from merged_members; order matches guilds)
                    let my_role_ids: Vec<String> = ready
                        .merged_members
                        .get(guild_idx)
                        .and_then(|v| v.as_array())
                        .and_then(|arr| {
                            arr.iter().find(|m| {
                                m.get("user")
                                    .and_then(|u| u.get("id").and_then(|v| v.as_str()))
                                    == Some(my_user_id.as_str())
                            })
                        })
                        .and_then(|m| {
                            m.get("roles").and_then(|r| r.as_array()).map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                        })
                        .unwrap_or_default();
                    let member_role_permissions: Vec<u64> = my_role_ids
                        .iter()
                        .filter_map(|rid| {
                            roles
                                .iter()
                                .find(|r| r.id == *rid)
                                .and_then(|r| r.permissions.parse::<u64>().ok())
                        })
                        .collect();
                    {
                        let mut cache = self.bridge_cache.lock().await;
                        cache.guild_owners.insert(guild_id.to_string(), owner_id.to_string());
                        cache.guild_roles.insert(guild_id.to_string(), roles.clone());
                        cache
                            .my_guild_roles
                            .insert(guild_id.to_string(), my_role_ids.clone());
                    }
                    if let Some(channels_arr) = guild.get("channels").and_then(|v| v.as_array()) {
                        let channels: Vec<ChannelInfo> = channels_arr
                            .iter()
                            .filter_map(|c| {
                                let id = c.get("id").and_then(|v| v.as_str())?;
                                let name = c
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unnamed");
                                let channel_type =
                                    c.get("type").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                                let position =
                                    c.get("position").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                                let parent_id = c
                                    .get("parent_id")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                                let overwrites = overwrites_from_channel_json(c);
                                let is_hidden = channel_is_hidden(
                                    guild_id,
                                    &my_user_id,
                                    owner_id,
                                    everyone_permissions,
                                    &member_role_permissions,
                                    &my_role_ids,
                                    &overwrites,
                                );
                                Some(ChannelInfo {
                                    id: id.to_string(),
                                    guild_id: Some(guild_id.to_string()),
                                    name: name.to_string(),
                                    channel_type,
                                    position,
                                    parent_id,
                                    has_unread: false,
                                    mention_count: 0,
                                    is_hidden,
                                })
                            })
                            .collect();

                        if !channels.is_empty() {
                            tracing::info!(
                                "Loaded {} channels for guild {}",
                                channels.len(),
                                guild_id
                            );
                            let mut cache = self.bridge_cache.lock().await;
                            cache.channels.insert(guild_id.to_string(), channels.clone());
                            for ch in &channels {
                                cache.channel_guild.insert(ch.id.clone(), guild_id.to_string());
                            }
                        }
                    }
                }
            }
            GatewayEvent::RelationshipAdd(r) => {
                let info = relationship_info_from(r);
                let _ = self.update_tx.send(UiUpdate::RelationshipAdded(info));
            }
            GatewayEvent::RelationshipRemove(ev) => {
                let _ = self.update_tx.send(UiUpdate::RelationshipRemoved {
                    user_id: ev.id.clone(),
                });
            }
            GatewayEvent::MessageAck(ack) => {
                let guild_id = self.bridge_cache.lock().await.channel_guild.get(&ack.channel_id).cloned();
                let _ = self.update_tx.send(UiUpdate::UnreadUpdate {
                    channel_id: ack.channel_id.clone(),
                    guild_id,
                    has_unread: false,
                    mention_count: 0,
                });
            }
            GatewayEvent::ChannelUnreadUpdate(data) => {
                // Payload can be single object or array; Discord: { channel_id?, id?, mention_count?, ... }
                let items: Vec<_> = if let Some(arr) = data.as_array() {
                    arr.iter().collect()
                } else {
                    std::slice::from_ref(data).iter().collect()
                };
                for item in items {
                    let channel_id = item.get("channel_id").or_else(|| item.get("id")).and_then(|v| v.as_str()).map(String::from);
                    let mention_count = item.get("mention_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    if let Some(channel_id) = channel_id {
                        let guild_id = self.bridge_cache.lock().await.channel_guild.get(&channel_id).cloned();
                        let has_unread = mention_count > 0;
                        let _ = self.update_tx.send(UiUpdate::UnreadUpdate {
                            channel_id,
                            guild_id,
                            has_unread,
                            mention_count,
                        });
                    }
                }
            }
            GatewayEvent::TypingStart(ts) => {
                let member_nick = ts.member.as_ref().and_then(|m| m.nick.as_deref());
                let user_name = resolve_typing_user_name(
                    &self.bridge_cache,
                    &ts.channel_id,
                    ts.guild_id.as_deref(),
                    &ts.user_id,
                    member_nick,
                )
                .await;
                let role_color = if let Some(ref gid) = ts.guild_id {
                    get_author_role_info(&self.client, gid, &ts.user_id, &self.bridge_cache)
                        .await
                        .map(|(color, _)| color)
                } else {
                    None
                };
                let _ = self.update_tx.send(UiUpdate::TypingStarted {
                    channel_id: ts.channel_id.clone(),
                    user_name,
                    role_color,
                });
            }
            GatewayEvent::PresenceUpdate(ev) => {
                let user_id = ev.user.id.clone();
                let status = ev.status.clone();
                self.bridge_cache.lock().await.presence.insert(user_id.clone(), status.clone());
                let _ = self.update_tx.send(UiUpdate::PresenceUpdated { user_id, status });
            }
            GatewayEvent::Raw {
                event_type,
                data,
            } if event_type == "READY" => {
                // Fallback: if ReadyEvent typed parsing fails, the gateway sends Raw.
                // Extract minimal user info from the raw JSON so the UI still updates.
                let user_id = data
                    .get("user")
                    .and_then(|u| u.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let username = data
                    .get("user")
                    .and_then(|u| {
                        u.get("global_name")
                            .or_else(|| u.get("username"))
                    })
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let avatar_hash = data
                    .get("user")
                    .and_then(|u| u.get("avatar"))
                    .and_then(|v| v.as_str());
                let avatar_url = avatar_hash.map(|hash| {
                    format!(
                        "https://cdn.discordapp.com/avatars/{}/{}.webp?size=128",
                        user_id, hash
                    )
                });

                tracing::info!(
                    "Gateway READY (raw fallback): user={} ({})",
                    username,
                    user_id
                );

                let _ = self.update_tx.send(UiUpdate::LoginSuccess {
                    user_id: user_id.clone(),
                    username,
                    avatar_url,
                });
                let _ = self.update_tx.send(UiUpdate::Connected);
                self.bridge_cache.lock().await.my_user_id = user_id.clone();

                // Extract guilds from raw READY data
                if let Some(guilds_arr) = data.get("guilds").and_then(|v| v.as_array()) {
                    let guilds: Vec<GuildInfo> = guilds_arr
                        .iter()
                        .filter_map(|g| {
                            let id = g.get("id").and_then(|v| v.as_str())?;
                            let name = g
                                .get("properties")
                                .and_then(|p| p.get("name"))
                                .and_then(|v| v.as_str())
                                .or_else(|| g.get("name").and_then(|v| v.as_str()))
                                .unwrap_or("Unknown Guild");
                            let icon = g
                                .get("properties")
                                .and_then(|p| p.get("icon"))
                                .and_then(|v| v.as_str())
                                .or_else(|| g.get("icon").and_then(|v| v.as_str()));
                            let icon_url = icon.map(|hash| {
                                let ext = if hash.starts_with("a_") { "gif" } else { "png" };
                                format!(
                                    "https://cdn.discordapp.com/icons/{}/{}.{}?size=128",
                                    id, hash, ext
                                )
                            });
                            Some(GuildInfo {
                                id: id.to_string(),
                                name: name.to_string(),
                                icon_url,
                                has_unread: false,
                                mention_count: 0,
                            })
                        })
                        .collect();
                    if !guilds.is_empty() {
                        let _ = self.update_tx.send(UiUpdate::GuildsLoaded(guilds));
                    }

                    // Extract channels and populate cache (with permission-based is_hidden)
                    let merged_members_arr = data.get("merged_members").and_then(|v| v.as_array());
                    for (guild_idx, guild) in guilds_arr.iter().enumerate() {
                        let raw_gid = guild.get("id").and_then(|v| v.as_str());
                        let guild_id = raw_gid.unwrap_or("");
                        let owner_id = guild
                            .get("owner_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let roles: Vec<crate::client::Role> = guild
                            .get("roles")
                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                            .unwrap_or_default();
                        let everyone_permissions = roles
                            .iter()
                            .find(|r| r.id == guild_id)
                            .and_then(|r| r.permissions.parse::<u64>().ok())
                            .unwrap_or(0);
                        let my_role_ids: Vec<String> = merged_members_arr
                            .and_then(|arr| arr.get(guild_idx))
                            .and_then(|v| v.as_array())
                            .and_then(|arr| {
                                arr.iter().find(|m| {
                                    m.get("user")
                                        .and_then(|u| u.get("id").and_then(|v| v.as_str()))
                                        == Some(user_id.as_str())
                                })
                            })
                            .and_then(|m| {
                                m.get("roles").and_then(|r| r.as_array()).map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(String::from))
                                        .collect()
                                })
                            })
                            .unwrap_or_default();
                        let member_role_permissions: Vec<u64> = my_role_ids
                            .iter()
                            .filter_map(|rid| {
                                roles
                                    .iter()
                                    .find(|r| r.id == *rid)
                                    .and_then(|r| r.permissions.parse::<u64>().ok())
                            })
                            .collect();
                        if let Some(gid) = raw_gid {
                            let mut cache = self.bridge_cache.lock().await;
                            cache.guild_owners.insert(gid.to_string(), owner_id.to_string());
                            cache.guild_roles.insert(gid.to_string(), roles.clone());
                            cache
                                .my_guild_roles
                                .insert(gid.to_string(), my_role_ids.clone());
                        }
                        if let Some(channels_arr) =
                            guild.get("channels").and_then(|v| v.as_array())
                        {
                            let channels: Vec<ChannelInfo> = channels_arr
                                .iter()
                                .filter_map(|c| {
                                    let id = c.get("id").and_then(|v| v.as_str())?;
                                    let name = c
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unnamed");
                                    let channel_type = c
                                        .get("type")
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(0)
                                        as u8;
                                    let position = c
                                        .get("position")
                                        .and_then(|v| v.as_i64())
                                        .unwrap_or(0)
                                        as i32;
                                    let parent_id = c
                                        .get("parent_id")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string());
                                    let overwrites = overwrites_from_channel_json(c);
                                    let is_hidden = channel_is_hidden(
                                        guild_id,
                                        &user_id,
                                        owner_id,
                                        everyone_permissions,
                                        &member_role_permissions,
                                        &my_role_ids,
                                        &overwrites,
                                    );
                                    Some(ChannelInfo {
                                        id: id.to_string(),
                                        guild_id: raw_gid.map(|s| s.to_string()),
                                        name: name.to_string(),
                                        channel_type,
                                        position,
                                        parent_id,
                                        has_unread: false,
                                        mention_count: 0,
                                        is_hidden,
                                    })
                                })
                                .collect();
                            if !channels.is_empty() {
                                if let Some(gid) = raw_gid {
                                    let mut cache = self.bridge_cache.lock().await;
                                    cache.channels.insert(gid.to_string(), channels.clone());
                                    for ch in &channels {
                                        cache.channel_guild.insert(ch.id.clone(), gid.to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                // Extract DM channels from raw READY private_channels
                if let Some(pcs) = data.get("private_channels").and_then(|v| v.as_array()) {
                    let raw_user_map: HashMap<&str, &serde_json::Value> = data
                        .get("users")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|u| {
                                    let uid = u.get("id").and_then(|v| v.as_str())?;
                                    Some((uid, u))
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    let mut dm_infos: Vec<DmChannelInfo> = pcs
                        .iter()
                        .filter_map(|c| extract_dm_from_json(c, &raw_user_map))
                        .collect();

                    dm_infos.sort_by(|a, b| {
                        let a_id: u64 = a.last_message_id.as_deref().unwrap_or("0").parse().unwrap_or(0);
                        let b_id: u64 = b.last_message_id.as_deref().unwrap_or("0").parse().unwrap_or(0);
                        b_id.cmp(&a_id)
                    });

                    tracing::info!("Loaded {} DM channels from raw READY", dm_infos.len());
                    self.bridge_cache.lock().await.dm_channels = Some(dm_infos.clone());
                    let _ = self.update_tx.send(UiUpdate::DmChannelsLoaded(dm_infos));
                }
            }
            GatewayEvent::GuildCreate(data) => {
                // When a guild becomes available (e.g. after reconnect), populate cache so switching to it is instant.
                let guild_id = match data.get("id").and_then(|v| v.as_str()) {
                    Some(id) => id.to_string(),
                    None => return,
                };
                let owner_id = data
                    .get("owner_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let roles: Vec<crate::client::Role> = data
                    .get("roles")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();
                let everyone_permissions = roles
                    .iter()
                    .find(|r| r.id == guild_id)
                    .and_then(|r| r.permissions.parse::<u64>().ok())
                    .unwrap_or(0);
                let my_user_id = self.bridge_cache.lock().await.my_user_id.clone();
                let my_role_ids: Vec<String> = self
                    .bridge_cache
                    .lock()
                    .await
                    .my_guild_roles
                    .get(&guild_id)
                    .cloned()
                    .unwrap_or_default();
                let member_role_permissions: Vec<u64> = my_role_ids
                    .iter()
                    .filter_map(|rid| {
                        roles
                            .iter()
                            .find(|r| r.id == *rid)
                            .and_then(|r| r.permissions.parse::<u64>().ok())
                    })
                    .collect();
                {
                    let mut cache = self.bridge_cache.lock().await;
                    cache.guild_owners.insert(guild_id.clone(), owner_id.to_string());
                    cache.guild_roles.insert(guild_id.clone(), roles.clone());
                    if !my_role_ids.is_empty() {
                        cache.my_guild_roles.insert(guild_id.clone(), my_role_ids.clone());
                    }
                }
                if let Some(channels_arr) = data.get("channels").and_then(|v| v.as_array()) {
                    let channels: Vec<ChannelInfo> = channels_arr
                        .iter()
                        .filter_map(|c| {
                            let id = c.get("id").and_then(|v| v.as_str())?;
                            let name = c
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unnamed");
                            let channel_type =
                                c.get("type").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                            let position =
                                c.get("position").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                            let parent_id = c
                                .get("parent_id")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let overwrites = overwrites_from_channel_json(c);
                            let is_hidden = channel_is_hidden(
                                &guild_id,
                                &my_user_id,
                                owner_id,
                                everyone_permissions,
                                &member_role_permissions,
                                &my_role_ids,
                                &overwrites,
                            );
                            Some(ChannelInfo {
                                id: id.to_string(),
                                guild_id: Some(guild_id.clone()),
                                name: name.to_string(),
                                channel_type,
                                position,
                                parent_id,
                                has_unread: false,
                                mention_count: 0,
                                is_hidden,
                            })
                        })
                        .collect();
                    if !channels.is_empty() {
                        let mut cache = self.bridge_cache.lock().await;
                        cache.channels.insert(guild_id.clone(), channels.clone());
                        for ch in &channels {
                            cache.channel_guild.insert(ch.id.clone(), guild_id.clone());
                        }
                        tracing::info!(
                            "GuildCreate: cached {} channels for guild {}",
                            channels.len(),
                            guild_id
                        );
                    }
                }
            }
            GatewayEvent::GuildMemberListUpdate(data) => {
                let guild_id = match data.get("guild_id").and_then(|v| v.as_str()) {
                    Some(id) => id.to_string(),
                    None => return,
                };
                let ops = match data.get("ops").and_then(|v| v.as_array()) {
                    Some(o) => o,
                    None => return,
                };
                let guild_roles = self
                    .client
                    .get_guild_roles(&guild_id)
                    .await
                    .ok()
                    .map(|r| r.into_boxed_slice());
                let guild_roles_ref = guild_roles.as_deref();
                let mut members: Vec<MemberInfo> = Vec::new();
                for op in ops {
                    let op_type = op.get("op").and_then(|v| v.as_str()).unwrap_or("");
                    if op_type == "SYNC" {
                        let items: &[serde_json::Value] = op.get("items").and_then(|v| v.as_array()).map(|a| a.as_slice()).unwrap_or(&[]);
                        for item in items {
                            if let Some(member_obj) = item.get("member") {
                                if let Some(info) = member_info_from_member_json(member_obj, guild_roles_ref) {
                                    members.push(info);
                                }
                            }
                        }
                    }
                }
                {
                    let mut cache = self.bridge_cache.lock().await;
                    let existing = cache.members.entry(guild_id.clone()).or_default();
                    for info in &members {
                        if !existing.iter().any(|e| e.user_id == info.user_id) {
                            existing.push(info.clone());
                        }
                    }
                    let members_snapshot = cache.members.get(&guild_id).cloned().unwrap_or_default();
                    drop(cache);
                    let _ = self.update_tx.send(UiUpdate::MembersLoaded {
                        guild_id,
                        members: members_snapshot,
                    });
                }
            }
            GatewayEvent::GuildMembersChunk(c) => {
                let guild_roles = self
                    .client
                    .get_guild_roles(&c.guild_id)
                    .await
                    .ok()
                    .map(|r| r.into_boxed_slice());
                let guild_roles_ref = guild_roles.as_deref();
                let members: Vec<MemberInfo> = c
                    .members
                    .iter()
                    .filter_map(|m| member_info_from_member_json(m, guild_roles_ref))
                    .collect();
                if !members.is_empty() {
                    let guild_id = c.guild_id.clone();
                    let mut cache = self.bridge_cache.lock().await;
                    let existing = cache.members.entry(guild_id.clone()).or_default();
                    for info in &members {
                        if !existing.iter().any(|e| e.user_id == info.user_id) {
                            existing.push(info.clone());
                        }
                    }
                    let members_snapshot = cache.members.get(&guild_id).cloned().unwrap_or_default();
                    drop(cache);
                    let _ = self.update_tx.send(UiUpdate::MembersLoaded {
                        guild_id,
                        members: members_snapshot,
                    });
                }
            }
            GatewayEvent::MessageReactionAdd(ev) => {
                let my_user_id = self.bridge_cache.lock().await.my_user_id.clone();
                let emoji_display = gateway_emoji_display(&ev.emoji);
                let channel_id = ev.channel_id.clone();
                let message_id = ev.message_id.clone();
                let user_id = ev.user_id.clone();
                apply_reaction_update_and_send(
                    &self.bridge_cache,
                    &self.update_tx,
                    &channel_id,
                    &message_id,
                    &my_user_id,
                    |reactions| {
                        if let Some(r) = reactions.iter_mut().find(|r| r.emoji_display == emoji_display) {
                            r.count = r.count.saturating_add(1);
                            if user_id == my_user_id {
                                r.me = true;
                            }
                        } else {
                            reactions.push(ReactionInfo {
                                emoji_display: emoji_display.clone(),
                                count: 1,
                                me: user_id == my_user_id,
                            });
                        }
                    },
                )
                .await;
            }
            GatewayEvent::MessageReactionRemove(ev) => {
                let my_user_id = self.bridge_cache.lock().await.my_user_id.clone();
                let emoji_display = gateway_emoji_display(&ev.emoji);
                let channel_id = ev.channel_id.clone();
                let message_id = ev.message_id.clone();
                let user_id = ev.user_id.clone();
                apply_reaction_update_and_send(
                    &self.bridge_cache,
                    &self.update_tx,
                    &channel_id,
                    &message_id,
                    &my_user_id,
                    |reactions| {
                        if let Some(pos) = reactions.iter().position(|r| r.emoji_display == emoji_display) {
                            if reactions[pos].count <= 1 {
                                reactions.remove(pos);
                            } else {
                                reactions[pos].count -= 1;
                                if user_id == my_user_id {
                                    reactions[pos].me = false;
                                }
                            }
                        }
                    },
                )
                .await;
            }
            GatewayEvent::MessageReactionRemoveAll(ev) => {
                let channel_id = ev.channel_id.clone();
                let message_id = ev.message_id.clone();
                let my_user_id = self.bridge_cache.lock().await.my_user_id.clone();
                apply_reaction_update_and_send(
                    &self.bridge_cache,
                    &self.update_tx,
                    &channel_id,
                    &message_id,
                    &my_user_id,
                    |reactions| reactions.clear(),
                )
                .await;
            }
            GatewayEvent::MessageReactionRemoveEmoji(ev) => {
                let emoji_display = gateway_emoji_display(&ev.emoji);
                let channel_id = ev.channel_id.clone();
                let message_id = ev.message_id.clone();
                let my_user_id = self.bridge_cache.lock().await.my_user_id.clone();
                apply_reaction_update_and_send(
                    &self.bridge_cache,
                    &self.update_tx,
                    &channel_id,
                    &message_id,
                    &my_user_id,
                    |reactions| reactions.retain(|r| r.emoji_display != emoji_display),
                )
                .await;
            }
            _ => {
                // Other events logged at trace level for debugging
                tracing::trace!("Unhandled gateway event in bridge");
            }
        }
    }
}

/// Shared gateway command sender type — used to send op 4 (VoiceStateUpdate) etc.
pub type SharedGatewayCmd =
    Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Sender<crate::gateway::GatewayCommand>>>>;

/// In-memory cache for channels, messages, and DMs to avoid redundant REST calls.
pub struct BridgeCache {
    /// Cached channels per guild_id (populated from READY and REST)
    pub channels: HashMap<String, Vec<ChannelInfo>>,
    /// Cached DM channel list
    pub dm_channels: Option<Vec<DmChannelInfo>>,
    /// Cached messages per channel_id (kept current by gateway events)
    pub messages: HashMap<String, Vec<MessageInfo>>,
    /// Current user's ID (populated from READY)
    pub my_user_id: String,
    /// channel_id -> guild_id for resolving guild context (e.g. for role colors)
    pub channel_guild: HashMap<String, String>,
    /// (guild_id, user_id) -> (color hex, role name) for role-colored usernames and role pills
    pub role_info: HashMap<String, Option<(String, String)>>,
    /// Guild roles cache: guild_id -> Vec<Role>
    pub guild_roles: HashMap<String, Vec<crate::client::Role>>,
    /// Current user's role IDs per guild: guild_id -> Vec<role_id>
    pub my_guild_roles: HashMap<String, Vec<String>>,
    /// Guild owner IDs: guild_id -> owner_user_id
    pub guild_owners: HashMap<String, String>,
    /// Cached member list per guild_id (from Op 14 GUILD_MEMBER_LIST_UPDATE / Op 8 GUILD_MEMBERS_CHUNK)
    pub members: HashMap<String, Vec<MemberInfo>>,
    /// User presence: user_id -> status (online, idle, dnd, offline)
    pub presence: HashMap<String, String>,
    /// Guild IDs muted for notifications (no desktop/sound notifications)
    pub muted_guild_ids: HashSet<String>,
}

impl Default for BridgeCache {
    fn default() -> Self {
        Self {
            channels: HashMap::new(),
            dm_channels: None,
            messages: HashMap::new(),
            my_user_id: String::new(),
            channel_guild: HashMap::new(),
            role_info: HashMap::new(),
            guild_roles: HashMap::new(),
            my_guild_roles: HashMap::new(),
            guild_owners: HashMap::new(),
            members: HashMap::new(),
            presence: HashMap::new(),
            muted_guild_ids: HashSet::new(),
        }
    }
}

/// Shared bridge cache type
pub type SharedBridgeCache = Arc<tokio::sync::Mutex<BridgeCache>>;

/// Extract an invite code from a URL or raw code string.
///
/// Handles: `discord.gg/CODE`, `discord.com/invite/CODE`,
/// `https://discord.gg/CODE`, `https://discord.com/invite/CODE`,
/// or a bare code like `abc123`.
fn extract_invite_code(input: &str) -> String {
    let trimmed = input.trim();
    // Try to extract from discord.gg/<code> or discord.com/invite/<code>
    if let Some(pos) = trimmed.find("discord.gg/") {
        let after = &trimmed[pos + "discord.gg/".len()..];
        return after.split(&['?', '#', ' ', '/'][..]).next().unwrap_or("").to_string();
    }
    if let Some(pos) = trimmed.find("discord.com/invite/") {
        let after = &trimmed[pos + "discord.com/invite/".len()..];
        return after.split(&['?', '#', ' ', '/'][..]).next().unwrap_or("").to_string();
    }
    // Otherwise treat the whole string as a raw code
    trimmed.to_string()
}

/// Key for role info cache: "guild_id:user_id"
fn role_info_cache_key(guild_id: &str, user_id: &str) -> String {
    format!("{}:{}", guild_id, user_id)
}

/// Resolve typing user to a display name using bridge cache (members, DM list, message authors).
/// Prefer member_nick when present; otherwise guild members, then DM recipient, then message cache.
async fn resolve_typing_user_name(
    bridge_cache: &SharedBridgeCache,
    channel_id: &str,
    guild_id: Option<&str>,
    user_id: &str,
    member_nick: Option<&str>,
) -> String {
    if let Some(nick) = member_nick {
        if !nick.is_empty() {
            return nick.to_string();
        }
    }
    let cache = bridge_cache.lock().await;
    if let Some(gid) = guild_id {
        if let Some(members) = cache.members.get(gid) {
            if let Some(m) = members.iter().find(|m| m.user_id == user_id) {
                return m
                    .display_name
                    .as_deref()
                    .unwrap_or(m.username.as_str())
                    .to_string();
            }
        }
    }
    if let Some(ref dms) = cache.dm_channels {
        if let Some(dm) = dms.iter().find(|c| c.id == channel_id) {
            if dm.recipient_id == user_id {
                return dm.recipient_name.clone();
            }
        }
    }
    if let Some(messages) = cache.messages.get(channel_id) {
        if let Some(msg) = messages.iter().find(|m| m.author_id == user_id) {
            return msg.author_name.clone();
        }
    }
    "Someone".to_string()
}

/// Display string for gateway ReactionEmoji (unicode as-is, custom as :name:id)
fn gateway_emoji_display(emoji: &crate::gateway::ReactionEmoji) -> String {
    if let Some(ref id) = emoji.id {
        format!(
            "{}:{}",
            emoji.name.as_deref().unwrap_or("_"),
            id
        )
    } else {
        emoji.name.as_deref().unwrap_or("").to_string()
    }
}

/// Update message reactions in cache and send MessageReactionsUpdated if the message was found.
async fn apply_reaction_update_and_send(
    bridge_cache: &SharedBridgeCache,
    update_tx: &broadcast::Sender<UiUpdate>,
    channel_id: &str,
    message_id: &str,
    _my_user_id: &str,
    mut f: impl FnMut(&mut Vec<ReactionInfo>),
) {
    let reactions = {
        let mut cache = bridge_cache.lock().await;
        let Some(messages) = cache.messages.get_mut(channel_id) else {
            return;
        };
        let Some(msg) = messages.iter_mut().find(|m| m.id == message_id) else {
            return;
        };
        f(&mut msg.reactions);
        msg.reactions.clone()
    };
    let _ = update_tx.send(UiUpdate::MessageReactionsUpdated {
        channel_id: channel_id.to_string(),
        message_id: message_id.to_string(),
        reactions,
    });
}

/// Enrich profile JSON value with note and created_at (from user id snowflake).
fn enrich_profile_json_into(v: &mut serde_json::Value, note: &str) {
    const DISCORD_EPOCH_MS: u64 = 1420070400000;
    v["note"] = serde_json::Value::String(note.to_string());
    if let Some(id_str) = v.get("user").and_then(|u| u.get("id")).and_then(|id| id.as_str()) {
        if let Ok(id) = id_str.parse::<u64>() {
            let ms = (id >> 22) + DISCORD_EPOCH_MS;
            if let Some(dt) = chrono::Utc.timestamp_millis_opt(ms as i64).single() {
                v["created_at"] = serde_json::Value::String(dt.to_rfc3339());
            }
        }
    }
}

/// Enrich raw profile API JSON with note and created_at (from user id snowflake).
fn enrich_profile_json(raw_json: &str, note: &str) -> String {
    let mut v: serde_json::Value = match serde_json::from_str(raw_json) {
        Ok(x) => x,
        Err(_) => return raw_json.to_string(),
    };
    enrich_profile_json_into(&mut v, note);
    serde_json::to_string(&v).unwrap_or_else(|_| raw_json.to_string())
}

/// Resolve author role color/name from cached data without any REST calls.
/// Returns Some(info) if resolved from cache (info may be None meaning no colored role),
/// or None if the user is not in cache and a REST fallback is needed.
fn resolve_role_color_local(
    guild_id: &str,
    user_id: &str,
    cache: &BridgeCache,
) -> Option<Option<(String, String)>> {
    // Check role_info cache first (populated by prior REST lookups)
    let key = role_info_cache_key(guild_id, user_id);
    if let Some(cached) = cache.role_info.get(&key) {
        return Some(cached.clone());
    }

    // Try to resolve from member cache + guild roles cache
    let members = cache.members.get(guild_id)?;
    let member = members.iter().find(|m| m.user_id == user_id)?;

    // If member already has role_color populated (from Op 14 / gateway), use it directly
    if member.role_color.is_some() {
        return Some(
            member.role_color.as_ref().map(|color| {
                (color.clone(), member.role_name.clone().unwrap_or_default())
            }),
        );
    }

    // Member is in cache but has no pre-resolved role color — cache miss
    None
}

/// Resolve role colors for all unique authors in a message list.
/// Uses local cache first, falls back to parallel REST for misses.
async fn resolve_roles_for_messages(
    msg_infos: &mut [MessageInfo],
    guild_id: &str,
    client: &Arc<DiscordClient>,
    cache: &SharedBridgeCache,
) {
    // Collect unique user IDs from authors and reply authors
    let mut unique_user_ids: HashSet<String> =
        msg_infos.iter().map(|m| m.author_id.clone()).collect();
    for msg in msg_infos.iter() {
        if let Some(ref rid) = msg.reply_author_id {
            unique_user_ids.insert(rid.clone());
        }
    }

    // Phase 1: resolve locally from cache
    let mut author_role_infos: HashMap<String, Option<(String, String)>> = HashMap::new();
    let mut cache_misses: Vec<String> = Vec::new();
    {
        let c = cache.lock().await;
        for user_id in &unique_user_ids {
            match resolve_role_color_local(guild_id, user_id, &c) {
                Some(info) => { author_role_infos.insert(user_id.clone(), info); }
                None => { cache_misses.push(user_id.clone()); }
            }
        }
    }

    // Phase 2: parallel REST fallback for cache misses
    if !cache_misses.is_empty() {
        let futs: Vec<_> = cache_misses
            .iter()
            .map(|uid| get_author_role_info(client, guild_id, uid, cache))
            .collect();
        let results = futures_util::future::join_all(futs).await;
        for (uid, info) in cache_misses.into_iter().zip(results) {
            author_role_infos.insert(uid, info);
        }
    }

    // Apply resolved colors to messages
    for msg in msg_infos.iter_mut() {
        if let Some((color, name)) = author_role_infos.get(&msg.author_id).and_then(|o| o.as_ref()) {
            msg.author_role_color = Some(color.clone());
            msg.author_role_name = Some(name.clone());
        }
        if let Some(ref rid) = msg.reply_author_id {
            if let Some((color, _)) = author_role_infos.get(rid).and_then(|o| o.as_ref()) {
                msg.reply_author_role_color = Some(color.clone());
            }
        }
    }
}

/// Resolve the author's highest role (color + name) in the guild (first role with color != 0).
/// Uses and updates the bridge cache. Returns None for DMs, errors, or no colored role.
async fn get_author_role_info(
    client: &Arc<DiscordClient>,
    guild_id: &str,
    user_id: &str,
    cache: &SharedBridgeCache,
) -> Option<(String, String)> {
    let key = role_info_cache_key(guild_id, user_id);
    {
        let c = cache.lock().await;
        if let Some(cached) = c.role_info.get(&key) {
            return cached.clone();
        }
    }
    let roles = match client.get_member_roles(guild_id, user_id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::debug!("get_member_roles failed for {} in {}: {}", user_id, guild_id, e);
            return None;
        }
    };
    // Roles are sorted by position (highest first). First role with non-zero color.
    let info = roles
        .into_iter()
        .find(|r| r.color != 0)
        .map(|r| (r.color_hex(), r.name));
    {
        let mut c = cache.lock().await;
        c.role_info.insert(key, info.clone());
    }
    info
}

/// Fetches guild channels, roles, and profile from REST, updates the cache, and sends
/// ChannelsLoaded + MyGuildProfile. Used for SelectGuild: either awaited (cache miss) or
/// spawned (cache hit, background refresh).
async fn refresh_guild_channels_from_rest(
    client: Arc<DiscordClient>,
    cache: SharedBridgeCache,
    gateway_cmd: SharedGatewayCmd,
    update_tx: std::sync::mpsc::Sender<UiUpdate>,
    guild_id: String,
) {
    let my_user_id = cache.lock().await.my_user_id.clone();
    let channels_fut = client.get_guild_channels(&guild_id);
    let roles_fut = client.get_guild_roles(&guild_id);
    let member_fut = client.get_guild_member(&guild_id, &my_user_id);
    let guild_fut = client.get_guild(&guild_id);
    let (channels_res, roles_res, member_res, guild_res) = tokio::join!(
        channels_fut,
        roles_fut,
        member_fut,
        guild_fut,
    );
    let owner_id: String = guild_res
        .ok()
        .and_then(|g| g.owner_id.clone())
        .unwrap_or_default();
    if let Ok(roles) = &roles_res {
        let mut c = cache.lock().await;
        c.guild_roles.insert(guild_id.clone(), roles.clone());
        if let Ok(member) = &member_res {
            c.my_guild_roles
                .insert(guild_id.clone(), member.roles.clone());
        }
        if !owner_id.is_empty() {
            c.guild_owners.insert(guild_id.clone(), owner_id.clone());
        }
    }
    match channels_res {
        Ok(channels) => {
            let roles = roles_res.as_ref().ok().map(|r| r.as_slice()).unwrap_or(&[]);
            let member = member_res.as_ref().ok();
            let my_role_ids: Vec<String> = member
                .map(|m| m.roles.clone())
                .unwrap_or_default();
            let member_role_permissions: Vec<u64> = my_role_ids
                .iter()
                .filter_map(|rid| {
                    roles
                        .iter()
                        .find(|r| r.id == *rid)
                        .and_then(|r| r.permissions.parse::<u64>().ok())
                })
                .collect();
            let everyone_permissions = roles
                .iter()
                .find(|r| r.id == guild_id)
                .and_then(|r| r.permissions.parse::<u64>().ok())
                .unwrap_or(0);
            let channel_infos: Vec<ChannelInfo> = channels
                .iter()
                .map(|c| {
                    let overwrites: Vec<PermOverwrite> = c
                        .permission_overwrites
                        .as_deref()
                        .unwrap_or(&[])
                        .iter()
                        .map(|o| PermOverwrite {
                            id: o.id.clone(),
                            overwrite_type: o.overwrite_type,
                            allow: o.allow.parse().unwrap_or(0),
                            deny: o.deny.parse().unwrap_or(0),
                        })
                        .collect();
                    let is_hidden = channel_is_hidden(
                        &guild_id,
                        &my_user_id,
                        &owner_id,
                        everyone_permissions,
                        &member_role_permissions,
                        &my_role_ids,
                        &overwrites,
                    );
                    ChannelInfo {
                        id: c.id.clone(),
                        guild_id: c.guild_id.clone(),
                        name: c.name.clone().unwrap_or_else(|| "unnamed".to_string()),
                        channel_type: c.channel_type,
                        position: c.position.unwrap_or(0),
                        parent_id: c.parent_id.clone(),
                        has_unread: false,
                        mention_count: 0,
                        is_hidden,
                    }
                })
                .collect();
            tracing::info!(
                "Loaded {} channels for guild {} (REST refresh)",
                channel_infos.len(),
                guild_id
            );
            {
                let mut c = cache.lock().await;
                c.channels.insert(guild_id.clone(), channel_infos.clone());
                for ch in &channel_infos {
                    c.channel_guild.insert(ch.id.clone(), guild_id.clone());
                }
            }
            let _ = update_tx.send(UiUpdate::ChannelsLoaded(channel_infos.clone()));
            if let Ok(member) = member_res {
                let nick = member.nick.clone();
                let mut member_roles: Vec<&crate::client::Role> = member
                    .roles
                    .iter()
                    .filter_map(|rid| roles.iter().find(|r| r.id == *rid))
                    .collect();
                member_roles.sort_by(|a, b| b.position.cmp(&a.position));
                let roles_display: Vec<RoleDisplayInfo> = member_roles
                    .into_iter()
                    .map(|r| RoleDisplayInfo {
                        id: r.id.clone(),
                        name: r.name.clone(),
                        color: r.color_hex(),
                        position: r.position,
                    })
                    .collect();
                let _ = update_tx.send(UiUpdate::MyGuildProfile {
                    guild_id: guild_id.clone(),
                    nick,
                    roles: roles_display,
                });
            }
            if let Some(ch_id) = channel_infos.first().map(|c| c.id.clone()) {
                let mut channels_map = std::collections::HashMap::new();
                channels_map.insert(ch_id, vec![[0u32, 99]]);
                let req = LazyGuildRequest {
                    guild_id: guild_id.clone(),
                    channels: Some(channels_map),
                    typing: Some(true),
                    activities: Some(true),
                    threads: Some(false),
                    members: None,
                };
                if let Some(tx) = gateway_cmd.lock().await.as_ref() {
                    if tx.send(GatewayCommand::LazyGuild(req)).await.is_ok() {
                        tracing::debug!("Sent LazyGuild (op 14) for guild {} (after REST)", guild_id);
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to load channels for guild {}: {}", guild_id, e);
            let _ = update_tx.send(UiUpdate::Error(format!(
                "Failed to load channels: {}",
                e
            )));
        }
    }
}

/// Process a UI action by calling the appropriate REST API endpoints.
/// Runs on the async worker thread; results are sent back via `update_tx`.
/// `gateway_cmd` provides access to the gateway for voice state updates.
/// `cache` provides in-memory caching to avoid redundant REST calls.
pub async fn handle_ui_action(
    action: UiAction,
    client: &Arc<DiscordClient>,
    update_tx: &std::sync::mpsc::Sender<UiUpdate>,
    gateway_cmd: &SharedGatewayCmd,
    cache: &SharedBridgeCache,
    flags: &Arc<RwLock<FeatureFlags>>,
    plugin_enabled: &Arc<RwLock<std::collections::HashMap<String, bool>>>,
    plugin_manifests: &Arc<tokio::sync::RwLock<std::collections::HashMap<String, crate::plugins::manifest::PluginManifest>>>,
    message_logger_cache: Option<&Arc<std::sync::RwLock<crate::plugins::message_logger::MessageCache>>>,
    storage: &Storage,
    gateway_proxy: Option<&Arc<RwLock<Option<crate::proxy::ProxyConfig>>>>,
) {
    match action {
        UiAction::SelectGuild(guild_id) => {
            if guild_id.is_empty() {
                // Home / DMs selected — emit cached DMs immediately, then refresh from REST in background
                {
                    let c = cache.lock().await;
                    if let Some(ref cached_dms) = c.dm_channels {
                        tracing::debug!("Serving {} cached DM channels", cached_dms.len());
                        let _ = update_tx.send(UiUpdate::DmChannelsLoaded(cached_dms.clone()));
                    }
                }

                let client_bg = Arc::clone(client);
                let cache_bg = Arc::clone(cache);
                let update_tx_bg = update_tx.clone();
                tokio::spawn(async move {
                    match client_bg.get_dm_channels().await {
                        Ok(channels) => {
                            let mut dm_infos: Vec<DmChannelInfo> = channels
                                .iter()
                                .filter_map(|c| {
                                    if c.channel_type == 1 {
                                        let (name, rid, avatar) =
                                            if let Some(recipient) = c.recipients.as_ref().and_then(|r| r.first()) {
                                                (
                                                    recipient.display_name().to_string(),
                                                    recipient.id.clone(),
                                                    Some(recipient.avatar_url(64)),
                                                )
                                            } else {
                                                tracing::warn!("DM channel {} has no recipients, using fallback", c.id);
                                                ("Unknown User".to_string(), String::new(), None)
                                            };
                                        Some(DmChannelInfo {
                                            id: c.id.clone(),
                                            recipient_name: name,
                                            recipient_id: rid,
                                            recipient_avatar_url: avatar,
                                            channel_type: c.channel_type,
                                            last_message_id: c.last_message_id.clone(),
                                        })
                                    } else if c.channel_type == 3 {
                                        let recipients = c.recipients.as_deref().unwrap_or(&[]);
                                        let name = c.name.clone().unwrap_or_else(|| {
                                            if recipients.is_empty() {
                                                "Group DM".to_string()
                                            } else {
                                                recipients
                                                    .iter()
                                                    .map(|u| u.display_name().to_string())
                                                    .collect::<Vec<_>>()
                                                    .join(", ")
                                            }
                                        });
                                        let first_recipient_id = recipients
                                            .first()
                                            .map(|u| u.id.clone())
                                            .unwrap_or_default();
                                        Some(DmChannelInfo {
                                            id: c.id.clone(),
                                            recipient_name: name,
                                            recipient_id: first_recipient_id,
                                            recipient_avatar_url: None,
                                            channel_type: c.channel_type,
                                            last_message_id: c.last_message_id.clone(),
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            dm_infos.sort_by(|a, b| {
                                let a_id: u64 = a.last_message_id.as_deref().unwrap_or("0").parse().unwrap_or(0);
                                let b_id: u64 = b.last_message_id.as_deref().unwrap_or("0").parse().unwrap_or(0);
                                b_id.cmp(&a_id)
                            });

                            tracing::info!("Loaded {} DM channels (REST refresh)", dm_infos.len());
                            cache_bg.lock().await.dm_channels = Some(dm_infos.clone());
                            let _ = update_tx_bg.send(UiUpdate::DmChannelsLoaded(dm_infos));
                        }
                        Err(e) => {
                            tracing::error!("Failed to load DM channels: {}", e);
                            let _ = update_tx_bg.send(UiUpdate::Error(format!("Failed to load DMs: {}", e)));
                        }
                    }
                });
            } else {
                // Guild selected — emit cached channels and members immediately, then refresh from REST
                {
                    let c = cache.lock().await;
                    if let Some(cached_channels) = c.channels.get(&guild_id) {
                        tracing::debug!("Serving {} cached channels for guild {}", cached_channels.len(), guild_id);
                        let _ = update_tx.send(UiUpdate::ChannelsLoaded(cached_channels.clone()));
                    }
                    if let Some(cached_members) = c.members.get(&guild_id) {
                        tracing::debug!("Serving {} cached members for guild {}", cached_members.len(), guild_id);
                        let _ = update_tx.send(UiUpdate::MembersLoaded {
                            guild_id: guild_id.clone(),
                            members: cached_members.clone(),
                        });
                    }
                }

                // Request member list (Op 14 Lazy Guild) when we have a channel for this guild
                let first_channel_id = cache.lock().await.channels.get(&guild_id).and_then(|chans| chans.first()).map(|c| c.id.clone());
                if let Some(ref ch_id) = first_channel_id {
                    let mut channels_map = std::collections::HashMap::new();
                    channels_map.insert(ch_id.clone(), vec![[0u32, 99]]);
                    let req = LazyGuildRequest {
                        guild_id: guild_id.clone(),
                        channels: Some(channels_map),
                        typing: Some(true),
                        activities: Some(true),
                        threads: Some(false),
                        members: None,
                    };
                    if let Some(tx) = gateway_cmd.lock().await.as_ref() {
                        if tx.send(GatewayCommand::LazyGuild(req)).await.is_ok() {
                            tracing::debug!("Sent LazyGuild (op 14) for guild {}", guild_id);
                        }
                    }
                }

                // Always refresh from REST in background (never block the action handler)
                let client_bg = Arc::clone(client);
                let cache_bg = Arc::clone(cache);
                let gateway_cmd_bg = Arc::clone(gateway_cmd);
                let update_tx_bg = update_tx.clone();
                let guild_id_bg = guild_id.clone();
                tokio::spawn(async move {
                    refresh_guild_channels_from_rest(
                        client_bg,
                        cache_bg,
                        gateway_cmd_bg,
                        update_tx_bg,
                        guild_id_bg,
                    )
                    .await;
                });
            }
        }

        UiAction::SelectChannel(channel_id, channel_type) => {
            if crate::client::channel_type_is_voice(channel_type) {
                // Voice/stage channels — don't fetch messages
                tracing::info!("Selected voice channel: {} (type {})", channel_id, channel_type);
            } else if crate::client::channel_type_supports_messages(channel_type) {
                // Emit cached messages immediately if available (gateway events keep them current)
                let needs_fetch = {
                    let c = cache.lock().await;
                    if let Some(cached_msgs) = c.messages.get(&channel_id) {
                        tracing::debug!(
                            "Serving {} cached messages for channel {}",
                            cached_msgs.len(), channel_id
                        );
                        let _ = update_tx.send(UiUpdate::MessagesLoaded(cached_msgs.clone()));
                        false
                    } else {
                        true // no cache, must fetch
                    }
                };

                if needs_fetch {
                    // Text-based channels — fetch recent messages
                    let my_uid = cache.lock().await.my_user_id.clone();
                    // Resolve guild_id for role colors (use cache when populated from READY/GuildCreate)
                    let guild_id = {
                        let gid = cache.lock().await.channel_guild.get(&channel_id).cloned();
                        if gid.is_some() {
                            gid
                        } else {
                            match client.get_channel(&channel_id).await {
                                Ok(ch) => {
                                    let gid = ch.guild_id.clone();
                                    if let Some(ref g) = gid {
                                        cache.lock().await.channel_guild.insert(channel_id.clone(), g.clone());
                                    }
                                    gid
                                }
                                Err(e) => {
                                    tracing::debug!("get_channel for {} failed (may be DM): {}", channel_id, e);
                                    None
                                }
                            }
                        }
                    };
                    match client.get_messages(&channel_id, 50).await {
                        Ok(messages) => {
                            let mut msg_infos: Vec<MessageInfo> =
                                messages.iter().map(|m| MessageInfo::from_message_with_context(m, &my_uid, &[])).collect();
                            // Fill author and reply-author role colors for guild channels
                            if let Some(ref gid) = guild_id {
                                resolve_roles_for_messages(&mut msg_infos, gid, client, cache).await;
                            }
                            tracing::info!(
                                "Loaded {} messages for channel {} (REST refresh)",
                                msg_infos.len(),
                                channel_id
                            );
                            // Update cache
                            cache.lock().await.messages.insert(
                                channel_id.clone(),
                                msg_infos.clone(),
                            );
                            let _ = update_tx.send(UiUpdate::MessagesLoaded(msg_infos));
                        }
                        Err(e) => {
                            tracing::error!("Failed to load messages for channel {}: {}", channel_id, e);
                            let _ = update_tx.send(UiUpdate::Error(format!(
                                "Failed to load messages: {}",
                                e
                            )));
                        }
                    }
                }
            } else {
                // Category, forum, or other unsupported channel types
                tracing::info!("Selected non-message channel: {} (type {})", channel_id, channel_type);
            }
        }

        UiAction::SendMessage {
            channel_id,
            content,
            silent,
        } => {
            let msg = crate::client::CreateMessage {
                content: Some(content),
                tts: None,
                embeds: None,
                message_reference: None,
                nonce: None,
                flags: if silent { Some(4096) } else { None }, // 4096 = SUPPRESS_NOTIFICATIONS
                sticker_ids: None,
            };
            if let Err(e) = client.send_message(&channel_id, msg).await {
                tracing::error!("Failed to send message: {}", e);
                let _ = update_tx.send(UiUpdate::Error(format!("Failed to send: {}", e)));
            }
        }

        UiAction::SendMessageEx {
            channel_id,
            content,
            silent,
            reply_to_message_id,
            sticker_ids,
            attachment_paths,
        } => {
            let message_reference = reply_to_message_id.map(|mid| {
                crate::client::MessageReference {
                    message_id: Some(mid),
                    channel_id: Some(channel_id.clone()),
                    guild_id: None,
                    fail_if_not_exists: Some(false),
                }
            });
            let msg = crate::client::CreateMessage {
                content: Some(content),
                tts: None,
                embeds: None,
                message_reference,
                nonce: Some(uuid::Uuid::new_v4().to_string()),
                flags: if silent { Some(4096) } else { None },
                sticker_ids: sticker_ids.clone(),
            };

            let has_attachments = attachment_paths.as_ref().map_or(false, |v| !v.is_empty());
            if has_attachments {
                let paths = attachment_paths.as_ref().unwrap();
                let mut files: Vec<(String, Vec<u8>, String)> = Vec::with_capacity(paths.len());
                for path_str in paths {
                    let path = std::path::Path::new(path_str);
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file")
                        .to_string();
                    match tokio::fs::read(path_str).await {
                        Ok(bytes) => {
                            let content_type =
                                mime_from_extension(&filename).to_string();
                            if let Err(e) =
                                attachments::validate_file_size(bytes.len() as u64, attachments::MAX_FILE_SIZE_FREE)
                            {
                                let _ = update_tx.send(UiUpdate::Error(e.to_string()));
                                break;
                            }
                            files.push((filename, bytes, content_type));
                        }
                        Err(e) => {
                            let _ = update_tx.send(UiUpdate::Error(format!(
                                "Failed to read file {}: {}",
                                path_str, e
                            )));
                            break;
                        }
                    }
                }
                if files.len() == paths.len() {
                    if let Err(e) = client.send_message_multipart(&channel_id, msg, &files).await {
                        tracing::error!("Failed to send message with attachments: {}", e);
                        let _ = update_tx.send(UiUpdate::Error(format!("Failed to send: {}", e)));
                    }
                }
            } else if let Err(e) = client.send_message(&channel_id, msg).await {
                tracing::error!("Failed to send message: {}", e);
                let _ = update_tx.send(UiUpdate::Error(format!("Failed to send: {}", e)));
            }
        }

        UiAction::StartTyping(channel_id) => {
            let _ = client.trigger_typing(&channel_id).await;
        }

        UiAction::DeleteMessage {
            channel_id,
            message_id,
        } => {
            if let Err(e) = client.delete_message(&channel_id, &message_id).await {
                let _ = update_tx.send(UiUpdate::Error(format!("Failed to delete: {}", e)));
            }
        }

        UiAction::PinMessage {
            channel_id,
            message_id,
        } => {
            if let Err(e) = client.pin_message(&channel_id, &message_id).await {
                let _ = update_tx.send(UiUpdate::Error(format!("Failed to pin: {}", e)));
            }
        }

        UiAction::UnpinMessage {
            channel_id,
            message_id,
        } => {
            if let Err(e) = client.unpin_message(&channel_id, &message_id).await {
                let _ = update_tx.send(UiUpdate::Error(format!("Failed to unpin: {}", e)));
            }
        }

        UiAction::OpenPins(channel_id) => {
            let my_uid = cache.lock().await.my_user_id.clone();
            match client.get_pinned_messages(&channel_id).await {
                Ok(messages) => {
                    let msg_infos: Vec<MessageInfo> = messages
                        .iter()
                        .map(|m| MessageInfo::from_message_with_context(m, &my_uid, &[]))
                        .collect();
                    let _ = update_tx.send(UiUpdate::PinsLoaded {
                        channel_id,
                        messages: msg_infos,
                    });
                }
                Err(e) => {
                    let _ = update_tx.send(UiUpdate::Error(format!("Failed to load pins: {}", e)));
                }
            }
        }

        UiAction::EditMessage {
            channel_id,
            message_id,
            content,
        } => {
            let edit = crate::client::EditMessage {
                content: Some(content),
                embeds: None,
                flags: None,
                allowed_mentions: None,
            };
            if let Err(e) = client.edit_message(&channel_id, &message_id, &edit).await {
                let _ = update_tx.send(UiUpdate::Error(format!("Failed to edit: {}", e)));
            }
        }

        UiAction::LoadMoreMessages {
            channel_id,
            before_message_id,
        } => {
            let my_uid = cache.lock().await.my_user_id.clone();
            let mut guild_id = cache.lock().await.channel_guild.get(&channel_id).cloned();
            if guild_id.is_none() {
                if let Ok(ch) = client.get_channel(&channel_id).await {
                    if let Some(ref gid) = ch.guild_id {
                        cache.lock().await.channel_guild.insert(channel_id.clone(), gid.clone());
                        guild_id = Some(gid.clone());
                    }
                }
            }
            match client.get_messages_before(&channel_id, &before_message_id, 50).await {
                Ok(messages) => {
                    let has_more = messages.len() == 50;
                    let mut msg_infos: Vec<MessageInfo> =
                        messages.iter().map(|m| MessageInfo::from_message_with_context(m, &my_uid, &[])).collect();
                    if let Some(ref gid) = guild_id {
                        resolve_roles_for_messages(&mut msg_infos, gid, client, cache).await;
                    }
                    tracing::info!(
                        "Loaded {} more messages for channel {} (has_more={})",
                        msg_infos.len(),
                        channel_id,
                        has_more
                    );
                    let _ = update_tx.send(UiUpdate::MoreMessagesLoaded {
                        channel_id,
                        messages: msg_infos,
                        has_more,
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to load more messages for channel {}: {}", channel_id, e);
                    let _ = update_tx.send(UiUpdate::Error(format!(
                        "Failed to load more messages: {}",
                        e
                    )));
                }
            }
        }

        UiAction::OpenDm(recipient_id) => {
            let my_uid = cache.lock().await.my_user_id.clone();
            match client.create_dm(&recipient_id).await {
                Ok(channel) => {
                    // Load messages for the newly opened DM
                    match client.get_messages(&channel.id, 50).await {
                        Ok(messages) => {
                            let msg_infos: Vec<MessageInfo> =
                                messages.iter().map(|m| MessageInfo::from_message_with_context(m, &my_uid, &[])).collect();
                            let _ = update_tx.send(UiUpdate::MessagesLoaded(msg_infos));
                        }
                        Err(e) => {
                            tracing::error!("Failed to load DM messages: {}", e);
                            let _ = update_tx.send(UiUpdate::Error(format!(
                                "Failed to load DM messages: {}",
                                e
                            )));
                        }
                    }
                }
                Err(e) => {
                    let _ = update_tx.send(UiUpdate::Error(format!("Failed to open DM: {}", e)));
                }
            }
        }

        UiAction::JoinGuildByInvite { invite_code_or_url } => {
            // Extract invite code from URL or use raw code
            let code = extract_invite_code(&invite_code_or_url);
            if code.is_empty() {
                let _ = update_tx.send(UiUpdate::JoinGuildFailed("Invalid invite code or URL".to_string()));
            } else {
                match client.accept_invite(&code).await {
                    Ok(invite) => {
                        if let Some(guild) = invite.guild {
                            let icon_url = guild.icon.as_deref().map(|hash| {
                                let ext = if hash.starts_with("a_") { "gif" } else { "png" };
                                format!(
                                    "https://cdn.discordapp.com/icons/{}/{}.{}?size=128",
                                    guild.id, hash, ext
                                )
                            });
                            let guild_info = GuildInfo {
                                id: guild.id,
                                name: guild.name,
                                icon_url,
                                has_unread: false,
                                mention_count: 0,
                            };
                            tracing::info!("Joined guild: {} ({})", guild_info.name, guild_info.id);
                            let _ = update_tx.send(UiUpdate::JoinGuildSuccess(guild_info));
                        } else {
                            // Invite accepted but no guild info returned (shouldn't happen for guild invites)
                            let _ = update_tx.send(UiUpdate::JoinGuildFailed(
                                "Joined but no guild info returned".to_string(),
                            ));
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to accept invite '{}': {}", code, e);
                        let _ = update_tx.send(UiUpdate::JoinGuildFailed(format!(
                            "Failed to join: {}",
                            e
                        )));
                    }
                }
            }
        }

        UiAction::LeaveGuild(guild_id) => {
            match client.leave_guild(&guild_id).await {
                Ok(()) => {
                    match client.get_guilds().await {
                        Ok(guilds) => {
                            let guild_infos: Vec<GuildInfo> = guilds
                                .into_iter()
                                .map(|g| GuildInfo {
                                    id: g.id.clone(),
                                    name: g.name.clone(),
                                    icon_url: g.icon_url(128),
                                    has_unread: false,
                                    mention_count: 0,
                                })
                                .collect();
                            let _ = update_tx.send(UiUpdate::GuildsLoaded(guild_infos));
                        }
                        Err(e) => {
                            let _ = update_tx.send(UiUpdate::Error(format!(
                                "Left guild but failed to refresh list: {}",
                                e
                            )));
                        }
                    }
                    // Clear bridge cache for the left guild
                    let mut c = cache.lock().await;
                    c.channels.remove(&guild_id);
                    c.guild_roles.remove(&guild_id);
                    c.my_guild_roles.remove(&guild_id);
                    c.guild_owners.remove(&guild_id);
                    c.members.remove(&guild_id);
                    let prefix = format!("{}:", guild_id);
                    c.channel_guild.retain(|_, gid| gid != &guild_id);
                    c.role_info.retain(|k, _| !k.starts_with(&prefix));
                }
                Err(e) => {
                    let _ = update_tx.send(UiUpdate::Error(format!(
                        "Failed to leave server: {}",
                        e
                    )));
                }
            }
        }

        UiAction::MuteGuild(guild_id) => {
            {
                let mut c = cache.lock().await;
                c.muted_guild_ids.insert(guild_id.clone());
            }
            let _ = update_tx.send(UiUpdate::GuildMuteStateChanged {
                guild_id,
                muted: true,
            });
        }

        UiAction::UnmuteGuild(guild_id) => {
            {
                let mut c = cache.lock().await;
                c.muted_guild_ids.remove(&guild_id);
            }
            let _ = update_tx.send(UiUpdate::GuildMuteStateChanged {
                guild_id: guild_id.clone(),
                muted: false,
            });
        }

        UiAction::SendFriendRequest { username } => {
            match client.send_friend_request(&username).await {
                Ok(()) => {
                    tracing::info!("Friend request sent to {}", username);
                }
                Err(e) => {
                    let _ = update_tx.send(UiUpdate::Error(format!(
                        "Failed to send friend request: {}",
                        e
                    )));
                }
            }
        }

        UiAction::AcceptFriendRequest { user_id } => {
            if let Err(e) = client.accept_friend_request(&user_id).await {
                let _ = update_tx.send(UiUpdate::Error(format!(
                    "Failed to accept friend request: {}",
                    e
                )));
            }
        }

        UiAction::RemoveRelationship { user_id } => {
            if let Err(e) = client.remove_relationship(&user_id).await {
                let _ = update_tx.send(UiUpdate::Error(format!(
                    "Failed to remove relationship: {}",
                    e
                )));
            }
        }

        UiAction::BlockUser { user_id } => {
            if let Err(e) = client.block_user(&user_id).await {
                let _ = update_tx.send(UiUpdate::Error(format!(
                    "Failed to block user: {}",
                    e
                )));
            }
        }

        UiAction::FetchUserProfile { user_id, guild_id } => {
            // Parallel fetch: profile + note
            let profile_fut = async {
                match guild_id.as_ref() {
                    Some(gid) => client
                        .get_user_profile_in_guild(&user_id, gid)
                        .await
                        .map(|(_, body)| body),
                    None => client.get_user_profile(&user_id).await.map(|(_, body)| body),
                }
            };
            let note_fut = async {
                client.get_note(&user_id).await.ok().flatten().unwrap_or_default()
            };
            let (raw_json, note) = tokio::join!(profile_fut, note_fut);
            match raw_json {
                Ok(raw_json) => {
                    let mut v: serde_json::Value = match serde_json::from_str(&raw_json) {
                        Ok(x) => x,
                        Err(_) => {
                            let profile_json = enrich_profile_json(&raw_json, &note);
                            let _ = update_tx.send(UiUpdate::UserProfileLoaded {
                                user_id: user_id.clone(),
                                profile_json,
                                raw_json,
                            });
                            return;
                        }
                    };
                    enrich_profile_json_into(&mut v, &note);
                    if let Some(gid) = &guild_id {
                        let is_owner = {
                            let c = cache.lock().await;
                            c.guild_owners.get(gid).map(|s| s.as_str()) == Some(user_id.as_str())
                        };
                        let (member, roles) = tokio::join!(
                            client.get_guild_member(gid, &user_id),
                            client.get_member_roles(gid, &user_id)
                        );
                        let member = member.ok();
                        let roles = roles.unwrap_or_default();
                        let perms_u64 = member
                            .as_ref()
                            .and_then(|m| m.permissions.as_ref())
                            .and_then(|p| p.parse::<u64>().ok());
                        let perm_names: Vec<String> = if is_owner {
                            vec!["Server Owner".to_string()]
                        } else {
                            perms_u64
                                .map(permission_names)
                                .unwrap_or_default()
                                .into_iter()
                                .map(String::from)
                                .collect()
                        };
                        let roles_json: Vec<serde_json::Value> = roles
                            .iter()
                            .map(|r| {
                                serde_json::json!({
                                    "id": r.id,
                                    "name": r.name,
                                    "color": r.color_hex()
                                })
                            })
                            .collect();
                        let key = if v.get("guild_member_profile").is_some() {
                            "guild_member_profile"
                        } else {
                            "guild_member"
                        };
                        if let Some(obj) = v.get_mut(key) {
                            obj["is_owner"] = serde_json::json!(is_owner);
                            if let Some(p) = perms_u64 {
                                obj["permissions"] =
                                    serde_json::Value::String(p.to_string());
                            }
                            obj["permission_names"] = serde_json::Value::Array(
                                perm_names
                                    .into_iter()
                                    .map(serde_json::Value::String)
                                    .collect(),
                            );
                            obj["roles"] = serde_json::Value::Array(roles_json);
                        }
                    }
                    let profile_json =
                        serde_json::to_string(&v).unwrap_or_else(|_| raw_json.clone());
                    let _ = update_tx.send(UiUpdate::UserProfileLoaded {
                        user_id: user_id.clone(),
                        profile_json,
                        raw_json,
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch user profile for {}: {}", user_id, e);
                    let _ = update_tx.send(UiUpdate::Error(format!(
                        "Failed to load profile: {}",
                        e
                    )));
                }
            }
        }

        UiAction::AddReaction {
            channel_id,
            message_id,
            emoji,
        } => {
            if let Err(e) = client.add_reaction(&channel_id, &message_id, &emoji).await {
                let _ = update_tx.send(UiUpdate::Error(format!("Failed to react: {}", e)));
            }
        }

        UiAction::RemoveReaction {
            channel_id,
            message_id,
            emoji,
        } => {
            if let Err(e) = client.remove_own_reaction(&channel_id, &message_id, &emoji).await {
                let _ = update_tx.send(UiUpdate::Error(format!("Failed to remove reaction: {}", e)));
            }
        }

        UiAction::MarkAllRead => {
            // Use ack_bulk to mark all channels as read
            // For now, we just log — full implementation requires read state tracking
            tracing::info!("Mark all as read requested");
            // The read state manager would produce the ack list, but since we don't
            // maintain full read state in the bridge yet, we'll fire the API call
            // with an empty list (no-op) as a placeholder
            if let Err(e) = client.ack_bulk(vec![]).await {
                let _ = update_tx.send(UiUpdate::Error(format!("Failed to mark as read: {}", e)));
            }
        }

        UiAction::Logout => {
            client.logout().await;
            let _ = update_tx.send(UiUpdate::Disconnected);
        }

        UiAction::SwitchAccount(_account_id) => {
            if let Some(tx) = gateway_cmd.lock().await.as_ref() {
                let _ = tx.send(GatewayCommand::Close).await;
            }
        }

        UiAction::SetStatus(status) => {
            if let Err(e) = client.set_status(&status).await {
                let _ = update_tx.send(UiUpdate::Error(format!("Failed to set status: {}", e)));
            }
        }

        UiAction::SetCustomStatus(text) => {
            if let Err(e) = client.set_custom_status(text.as_deref(), None).await {
                let _ = update_tx.send(UiUpdate::Error(format!("Failed to set custom status: {}", e)));
            }
        }

        UiAction::SearchGifs(query) => {
            let result = if query.is_empty() {
                crate::features::gif_picker::trending_gifs(20).await
            } else {
                crate::features::gif_picker::search_gifs(&query, 20).await
            };
            match result {
                Ok(gifs) => {
                    let _ = update_tx.send(UiUpdate::GifsLoaded(gifs));
                }
                Err(e) => {
                    tracing::error!("GIF search failed: {}", e);
                    let _ = update_tx.send(UiUpdate::GifsLoaded(vec![]));
                }
            }
        }

        UiAction::LoadStickerPacks => {
            match client.list_sticker_packs().await {
                Ok(packs) => {
                    let _ = update_tx.send(UiUpdate::StickerPacksLoaded(packs));
                }
                Err(e) => {
                    tracing::error!("Failed to load sticker packs: {}", e);
                    let _ = update_tx.send(UiUpdate::StickerPacksLoaded(vec![]));
                }
            }
        }

        UiAction::LoadGuildEmojis(guild_id) => {
            if guild_id.is_empty() {
                let _ = update_tx.send(UiUpdate::GuildEmojisLoaded(vec![]));
            } else {
                match client.list_guild_emojis(&guild_id).await {
                    Ok(emojis) => {
                        let _ = update_tx.send(UiUpdate::GuildEmojisLoaded(emojis));
                    }
                    Err(e) => {
                        tracing::error!("Failed to load guild emojis: {}", e);
                        let _ = update_tx.send(UiUpdate::GuildEmojisLoaded(vec![]));
                    }
                }
            }
        }


        UiAction::SetPluginEnabled { plugin_id, enabled } => {
            let mut plugins = plugin_enabled.write().await;
            plugins.insert(plugin_id.clone(), enabled);
            drop(plugins);
            if let Ok(mut settings) = storage.load_settings() {
                settings.plugin_enabled.insert(plugin_id.clone(), enabled);
                let _ = storage.save_settings(&settings);
            }
            // Sync FeatureFlags so behavior updates immediately
            {
                let mut f = flags.write().await;
                match plugin_id.as_str() {
                    "clear-urls" => f.clear_urls = enabled,
                    "silent-messages" => f.silent_message_toggle = enabled,
                    "pin-dms" => f.pin_dms = enabled,
                    "show-hidden-channels" => f.show_hidden_channels = enabled,
                    "fake-mute" => f.fake_mute = enabled,
                    "fake-deafen" => f.fake_deafen = enabled,
                    _ => {}
                }
            }
            // Sync plugin UI: add when enabled, remove when disabled
            if enabled {
                let manifests = plugin_manifests.read().await;
                if let Some(manifest) = manifests.get(&plugin_id) {
                    if !manifest.ui.buttons.is_empty() || !manifest.ui.modals.is_empty() {
                        let _ = update_tx.send(UiUpdate::PluginUiUpdated {
                            plugin_id: plugin_id.clone(),
                            buttons: manifest.ui.buttons.clone(),
                            modals: manifest.ui.modals.clone(),
                        });
                    }
                }
            } else {
                let _ = update_tx.send(UiUpdate::PluginUiRemoved {
                    plugin_id: plugin_id.clone(),
                });
            }
        }

        UiAction::PluginButtonClicked { plugin_id, button_id } => {
            tracing::debug!("Plugin button clicked: {} / {}", plugin_id, button_id);
            // Message logger Export button opens modal via QML; no direct action here
        }

        UiAction::PluginModalSubmitted {
            plugin_id,
            modal_id,
            fields,
        } => {
            if plugin_id == "message-logger" && modal_id == "export_modal" {
                if let Some(cache) = message_logger_cache {
                    let format = fields
                        .get("format")
                        .map(|s| s.to_lowercase())
                        .unwrap_or_else(|| "json".to_string());
                    let path = fields.get("path").cloned().unwrap_or_default();
                    if let Ok(guard) = cache.read() {
                        let messages = guard.all();
                        let content = if format == "csv" {
                            crate::plugins::message_logger::export::export_csv(&messages)
                        } else {
                            crate::plugins::message_logger::export::export_json(&messages)
                        };
                        if path.is_empty() {
                            let _ = update_tx.send(UiUpdate::Error(
                                "Export path required. Enter a file path in the modal.".to_string(),
                            ));
                        } else if let Err(e) = std::fs::write(&path, &content) {
                            let _ = update_tx.send(UiUpdate::Error(format!(
                                "Export failed: {}",
                                e
                            )));
                        } else {
                            let _ = update_tx.send(UiUpdate::Error(format!(
                                "Exported {} messages to {}",
                                messages.len(),
                                path
                            ))); // Shown in UI (reusing Error for status)
                        }
                    }
                }
            }
        }

        UiAction::InstallPlugin(repo_url) => {
            if let Some(loader) = crate::plugins::PluginLoader::new() {
                match loader.install_from_git(&repo_url) {
                    Ok(path) => {
                        tracing::info!("Plugin installed to {}", path.display());
                        // Refresh manifests and notify UI
                        let mut manifests = plugin_manifests.write().await;
                        *manifests = crate::plugins::all_manifests();
                        let _ = update_tx.send(UiUpdate::PluginsRefreshed);
                    }
                    Err(e) => {
                        let _ = update_tx.send(UiUpdate::Error(format!("Plugin install failed: {}", e)));
                    }
                }
            } else {
                let _ = update_tx.send(UiUpdate::Error("Plugin loader not available".to_string()));
            }
        }

        UiAction::RefreshPlugins => {
            let mut manifests = plugin_manifests.write().await;
            *manifests = crate::plugins::all_manifests();
            let _ = update_tx.send(UiUpdate::PluginsRefreshed);
        }

        UiAction::CheckPluginUpdates => {
            let tx = update_tx.clone();
            tokio::task::spawn_blocking(move || {
                let results = crate::plugins::check_plugin_updates();
                let json = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
                let _ = tx.send(UiUpdate::PluginUpdatesAvailable(json));
            });
        }

        UiAction::SetProxySettings {
            enabled,
            host,
            port,
            username,
            password,
        } => {
            if let Ok(storage) = Storage::new() {
                let mut settings = storage.load_settings().unwrap_or_default();
                settings.proxy_settings.enabled = enabled;
                settings.proxy_settings.host = host;
                settings.proxy_settings.port = port;
                settings.proxy_settings.username = username;
                settings.proxy_settings.password = password;
                let _ = storage.save_settings(&settings);
                let new_proxy = settings.proxy_settings.to_proxy_config();
                if let Some(ref proxy_config) = new_proxy {
                    if let Err(e) = client.set_proxy(Some(proxy_config.clone())).await {
                        tracing::error!("Failed to set proxy: {}", e);
                    } else {
                        tracing::info!("Proxy configured successfully");
                    }
                } else {
                    if let Err(e) = client.set_proxy(None).await {
                        tracing::error!("Failed to clear proxy: {}", e);
                    } else {
                        tracing::info!("Proxy disabled");
                    }
                }
                // Update gateway proxy and trigger reconnect
                if let Some(gw_proxy) = gateway_proxy {
                    *gw_proxy.write().await = new_proxy;
                    let guard = gateway_cmd.lock().await;
                    if let Some(ref tx) = *guard {
                        let _ = tx.send(crate::gateway::GatewayCommand::Reconnect).await;
                    }
                }
            } else {
                tracing::error!("Failed to open storage for proxy settings");
            }
        }

        _ => {
            tracing::debug!("Unhandled UI action: {:?}", action);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bridge_creation() {
        let client = DiscordClient::new().await.unwrap();
        let (bridge, action_tx, mut update_rx) = BackendBridge::new(
            Arc::new(client),
            FeatureFlags::standard(),
            std::collections::HashMap::new(),
            None,
        );

        // Verify channels work
        action_tx
            .send(UiAction::SetStatus("online".to_string()))
            .await
            .unwrap();
        let _ = bridge.update_tx.send(UiUpdate::Connected);
        let update = update_rx.recv().await.unwrap();
        assert!(matches!(update, UiUpdate::Connected));
    }

    #[tokio::test]
    async fn test_bridge_cache_channels() {
        let mut bc = BridgeCache::default();
        let channels = vec![ChannelInfo {
            id: "ch1".into(),
            guild_id: Some("g1".into()),
            name: "general".into(),
            channel_type: 0,
            position: 0,
            parent_id: None,
            has_unread: false,
            mention_count: 0,
            is_hidden: false,
        }];
        bc.channels.insert("g1".into(), channels.clone());
        assert_eq!(bc.channels.get("g1").unwrap().len(), 1);
        assert!(bc.channels.get("g2").is_none());
    }

    #[tokio::test]
    async fn test_bridge_cache_messages_no_ttl() {
        let mut bc = BridgeCache::default();
        let msgs = vec![MessageInfo {
            id: "m1".into(),
            channel_id: "ch1".into(),
            author_name: "User".into(),
            author_id: "u1".into(),
            author_avatar_url: None,
            content: "hello".into(),
            timestamp: "2024-01-01".into(),
            is_deleted: false,
            edit_count: 0,
            message_type: 0, reply_author_name: None, reply_content: None,
            reply_author_id: None, reply_author_role_color: None, mentions_me: false, mention_everyone: false,
            author_role_color: None,
            author_role_name: None,
            author_public_flags: 0,
            author_bot: false,
            author_premium_type: 0,
            attachments_json: "[]".to_string(),
            stickers_json: "[]".to_string(),
            embeds_json: "[]".to_string(),
            content_html: "hello".to_string(),
            reactions: vec![],
        }];
        bc.messages.insert("ch1".into(), msgs);
        // Cache is available regardless of age
        let cached = bc.messages.get("ch1").unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].id, "m1");
    }

    #[tokio::test]
    async fn test_message_delete_with_logger() {
        use crate::client::Message;
        use crate::gateway::MessageCreateEvent;
        use crate::plugins::message_logger::MessageLoggerHandler;

        let client = DiscordClient::new().await.unwrap();
        let mut plugin_enabled = std::collections::HashMap::new();
        plugin_enabled.insert("message-logger".to_string(), true);
        let handler: Arc<dyn crate::plugins::GatewayEventHooks> =
            Arc::new(MessageLoggerHandler::new(10000));
        let hooks = Some(handler);
        let (bridge, _, mut update_rx) =
            BackendBridge::new(Arc::new(client), FeatureFlags::full(), plugin_enabled, hooks);

        // Simulate MESSAGE_CREATE to populate the plugin's cache
        let create_event = GatewayEvent::MessageCreate(MessageCreateEvent {
            message: Message {
                id: "m1".to_string(),
                channel_id: "ch1".to_string(),
                author: Some(crate::client::User {
                    id: "u1".to_string(),
                    username: "Test".to_string(),
                    discriminator: "0".to_string(),
                    global_name: Some("Test".to_string()),
                    avatar: None,
                    bot: None,
                    system: None,
                    mfa_enabled: None,
                    banner: None,
                    accent_color: None,
                    locale: None,
                    verified: None,
                    email: None,
                    flags: None,
                    premium_type: None,
                    public_flags: None,
                    avatar_decoration_data: None,
                    bio: None,
                }),
                content: "hello".to_string(),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
                edited_timestamp: None,
                tts: false,
                mention_everyone: false,
                mentions: vec![],
                mention_roles: vec![],
                attachments: vec![],
                embeds: vec![],
                sticker_items: vec![],
                message_type: 0,
                referenced_message: None,
                flags: None,
                reactions: vec![],
            },
            guild_id: None,
            member: None,
        });
        bridge.handle_gateway_event(&create_event).await;
        // Consume the NewMessage update
        let _ = update_rx.recv().await;

        // Handle MESSAGE_DELETE — plugin should return ShowAsDeleted
        let delete_event = GatewayEvent::MessageDelete(crate::gateway::MessageDeleteEvent {
            id: "m1".to_string(),
            channel_id: "ch1".to_string(),
            guild_id: None,
        });
        bridge.handle_gateway_event(&delete_event).await;

        let update = update_rx.recv().await.unwrap();
        assert!(matches!(update, UiUpdate::MessageDeletedWithContent { .. }));
    }
}
