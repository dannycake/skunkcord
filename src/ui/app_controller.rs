// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! App controller for GUI-first login flow
//!
//! Bridges QML login UI to the backend. Exposes methods that QML calls
//! directly (select_guild, select_channel, send_message, etc.), which
//! are forwarded to the async backend via a tokio channel.

use crate::bridge::{ChannelInfo, DmChannelInfo, GuildInfo, LoginRequest, MemberInfo, MessageInfo, ReactionInfo, RelationshipInfo, RoleDisplayInfo, UiAction, UiUpdate, VoiceParticipant};
use crate::captcha::widget;
use crate::captcha::CaptchaChallenge;
use crate::plugins;
use crate::plugins::manifest::{PluginUiButton, PluginUiModal};
use crate::storage::{ProxyMode, Storage};
use qmetaobject::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;

/// Controller exposed to QML for login, navigation, and messaging
#[derive(QObject, Default)]
pub struct AppController {
    base: qt_base_class!(trait QObject),

    // ── Properties (readable from QML) ──
    /// Whether user is logged in
    is_logged_in: qt_property!(bool; NOTIFY state_changed),
    /// Error message to display (e.g. invalid token)
    error_message: qt_property!(QString; NOTIFY state_changed),
    /// Whether login is in progress (validating token)
    is_loading: qt_property!(bool; NOTIFY state_changed),
    /// Current user display name (after login)
    user_name: qt_property!(QString; NOTIFY state_changed),
    /// Current user ID
    user_id: qt_property!(QString; NOTIFY state_changed),
    /// Current user avatar URL
    user_avatar: qt_property!(QString; NOTIFY state_changed),
    /// Gateway connection state: "connected", "disconnected", "reconnecting"
    connection_state: qt_property!(QString; NOTIFY state_changed),
    /// Typing indicator text (e.g. "User is typing...")
    typing_display: qt_property!(QString; NOTIFY state_changed),
    /// Typing users as JSON array of { name, roleColor } for role-colored display
    typing_display_json: qt_property!(QString; NOTIFY state_changed),
    /// Voice connection state: "disconnected", "connecting_gateway", "discovering", "connected", "failed:..."
    voice_connection_state: qt_property!(QString; NOTIFY state_changed),
    /// Whether MFA code input should be shown (credential login)
    mfa_required: qt_property!(bool; NOTIFY state_changed),
    /// Generated captcha widget HTML for WebView (when captcha_visible)
    captcha_html: qt_property!(QString; NOTIFY state_changed),
    /// Whether captcha popup should be shown
    captcha_visible: qt_property!(bool; NOTIFY state_changed),
    /// Login mode: "credentials" or "token" (for QML toggle)
    login_mode: qt_property!(QString; NOTIFY state_changed),
    /// JSON array of guild IDs muted for notifications (e.g. "[\"id1\",\"id2\"]") — use JSON.parse in QML
    muted_guild_ids_json: qt_property!(QString; NOTIFY state_changed),
    /// JSON array of saved accounts for switcher: [{ "id": "user_id", "name": "display_name" }]
    accounts_json: qt_property!(QString; NOTIFY state_changed),

    state_changed: qt_signal!(),

    // ── QML-callable methods ──
    /// Login with token (called from QML when user clicks Login)
    login: qt_method!(fn(&mut self, token: QString)),
    /// Login with email/password (credential flow)
    login_credentials: qt_method!(fn(&mut self, email: QString, password: QString)),
    /// Submit MFA code (after MfaRequired)
    submit_mfa_code: qt_method!(fn(&mut self, code: QString)),
    /// Cancel MFA (close overlay, resume at login form)
    cancel_mfa: qt_method!(fn(&mut self)),
    /// Submit solved captcha token (after CaptchaRequired)
    submit_captcha: qt_method!(fn(&mut self, captcha_token: QString)),
    /// Set login form mode: "credentials" or "token"
    set_login_mode: qt_method!(fn(&mut self, mode: QString)),
    /// Check for backend updates (called from QML Timer)
    check_for_updates: qt_method!(fn(&mut self)),
    /// Consume pending guild data (returns JSON array, clears buffer)
    consume_guilds: qt_method!(fn(&mut self) -> QString),
    /// Consume pending channel data (returns JSON array, clears buffer)
    consume_channels: qt_method!(fn(&mut self) -> QString),
    /// Consume pending new messages from gateway (returns JSON array, clears buffer)
    consume_messages: qt_method!(fn(&mut self) -> QString),
    /// Consume loaded messages from REST API (returns JSON array, clears buffer)
    consume_loaded_messages: qt_method!(fn(&mut self) -> QString),
    /// Consume DM channels (returns JSON array, clears buffer)
    consume_dm_channels: qt_method!(fn(&mut self) -> QString),
    /// Consume pending message edits (returns JSON array, clears buffer)
    consume_message_edits: qt_method!(fn(&mut self) -> QString),
    /// Consume pending message deletions (returns JSON array, clears buffer)
    consume_message_deletions: qt_method!(fn(&mut self) -> QString),
    /// Consume more (older) messages for pagination (returns JSON object with messages + hasMore)
    consume_more_messages: qt_method!(fn(&mut self) -> QString),
    /// Consume pending voice state change (returns state string or empty)
    consume_voice_state: qt_method!(fn(&mut self) -> QString),
    /// Consume voice channel participants (returns JSON object { channelId, participants } or empty)
    consume_voice_participants: qt_method!(fn(&mut self) -> QString),
    /// Consume voice connection stats (returns JSON object with ping, encryption, endpoint, etc. or empty)
    consume_voice_stats: qt_method!(fn(&mut self) -> QString),
    /// Consume speaking state changes (returns JSON array of { userId, speaking } or empty)
    consume_speaking_users: qt_method!(fn(&mut self) -> QString),
    /// Search GIFs (empty string = trending)
    search_gifs: qt_method!(fn(&mut self, query: QString)),
    /// Consume pending GIF results (returns JSON array)
    consume_gifs: qt_method!(fn(&mut self) -> QString),
    /// Load sticker packs for the sticker picker
    load_sticker_packs: qt_method!(fn(&mut self)),
    /// Consume pending sticker packs (returns JSON array)
    consume_sticker_packs: qt_method!(fn(&mut self) -> QString),
    /// Load guild emojis for the emoji picker (pass current guild id; empty for DMs)
    load_guild_emojis: qt_method!(fn(&mut self, guild_id: QString)),
    /// Consume pending guild emojis (returns JSON array)
    consume_guild_emojis: qt_method!(fn(&mut self) -> QString),
    /// Get typing users in a specific channel (returns comma-separated list of names, or empty)
    get_typing_in_channel: qt_method!(fn(&self, channel_id: QString) -> QString),
    /// Update typing display for a specific channel (called when channel changes)
    update_typing_for_channel: qt_method!(fn(&mut self, channel_id: QString)),

    /// Select a guild — triggers channel loading; empty string = DMs
    select_guild: qt_method!(fn(&mut self, guild_id: QString)),
    /// Select a channel — triggers message loading (channel_type passed for UI routing)
    select_channel: qt_method!(fn(&mut self, channel_id: QString, channel_type: i32)),
    /// Send a message to a channel
    send_message: qt_method!(fn(&mut self, channel_id: QString, content: QString)),
    /// Open a DM with a specific user
    open_dm: qt_method!(fn(&mut self, recipient_id: QString)),
    /// Send a friend request by username
    send_friend_request: qt_method!(fn(&mut self, username: QString)),
    /// Accept an incoming friend request
    accept_friend_request: qt_method!(fn(&mut self, user_id: QString)),
    /// Remove a relationship (unfriend, reject request, or unblock)
    remove_relationship: qt_method!(fn(&mut self, user_id: QString)),
    /// Block a user
    block_user: qt_method!(fn(&mut self, user_id: QString)),
    /// Load more (older) messages for pagination
    load_more_messages: qt_method!(fn(&mut self, channel_id: QString, before_message_id: QString)),
    /// Delete a message
    delete_message: qt_method!(fn(&mut self, channel_id: QString, message_id: QString)),
    /// Edit a message
    edit_message: qt_method!(fn(&mut self, channel_id: QString, message_id: QString, content: QString)),
    /// Pin a message
    pin_message: qt_method!(fn(&mut self, channel_id: QString, message_id: QString)),
    /// Unpin a message
    unpin_message: qt_method!(fn(&mut self, channel_id: QString, message_id: QString)),
    /// Open pins viewer for a channel (loads pins and shows in UI)
    open_pins: qt_method!(fn(&mut self, channel_id: QString)),
    /// Consume pending pinned messages (returns JSON array, clears buffer)
    consume_pins: qt_method!(fn(&mut self) -> QString),
    /// Consume pending member list for a guild (returns JSON object with guildId + members array, clears buffer)
    consume_members: qt_method!(fn(&mut self) -> QString),
    /// Consume pending my guild profile (returns JSON object with guildId, nick, roles; clears buffer)
    consume_my_profile: qt_method!(fn(&mut self) -> QString),
    /// Consume pending reaction updates (returns JSON array of { channelId, messageId, reactions }, clears buffer)
    consume_reaction_updates: qt_method!(fn(&mut self) -> QString),
    /// Consume pending unread updates (returns JSON array of { channelId, guildId, hasUnread, mentionCount }, clears buffer)
    consume_unread_updates: qt_method!(fn(&mut self) -> QString),
    /// Consume pending relationships (returns JSON array of { userId, username, avatarUrl, relationshipType }, clears dirty flag)
    consume_relationships: qt_method!(fn(&mut self) -> QString),
    /// Copy a message link to clipboard (channel_id, guild_id empty for DMs, message_id)
    copy_message_link: qt_method!(fn(&self, channel_id: QString, guild_id: QString, message_id: QString)),
    /// Copy arbitrary text to clipboard (e.g. user ID, raw JSON)
    copy_to_clipboard: qt_method!(fn(&self, text: QString)),
    /// Clear the error message (e.g. after user dismisses error toast)
    clear_error: qt_method!(fn(&mut self)),
    /// Set up a reply to a message
    reply_to_message: qt_method!(fn(&mut self, message_id: QString)),
    /// Add a reaction to a message
    add_reaction: qt_method!(fn(&mut self, channel_id: QString, message_id: QString, emoji: QString)),
    /// Remove own reaction from a message
    remove_reaction: qt_method!(fn(&mut self, channel_id: QString, message_id: QString, emoji: QString)),
    /// Send a message with optional silent flag and reply reference
    send_message_ex: qt_method!(fn(&mut self, channel_id: QString, content: QString, silent: bool, reply_to: QString)),
    /// Send a message with optional stickers and/or file attachment paths (JSON arrays: sticker_ids_json, attachment_paths_json)
    send_message_with_options: qt_method!(fn(&mut self, channel_id: QString, content: QString, silent: bool, reply_to: QString, sticker_ids_json: QString, attachment_paths_json: QString)),
    /// Trigger typing indicator
    start_typing: qt_method!(fn(&mut self, channel_id: QString)),
    /// Logout
    logout: qt_method!(fn(&mut self)),
    /// Switch to another account by user ID (must have a saved session)
    switch_account: qt_method!(fn(&mut self, account_id: QString)),
    /// Mark all channels as read
    mark_all_read: qt_method!(fn(&mut self)),
    /// Set plugin enabled state (plugin_id, enabled)
    set_plugin_enabled: qt_method!(fn(&mut self, plugin_id: QString, enabled: bool)),
    /// Install plugin from git URL
    install_plugin: qt_method!(fn(&mut self, repo_url: QString)),
    /// Refresh plugin list from disk
    refresh_plugins: qt_method!(fn(&mut self)),
    /// Check for plugin updates (git repos)
    check_plugin_updates: qt_method!(fn(&mut self)),
    /// Consume plugin UI state (returns JSON: { pluginId: { buttons, modals } })
    consume_plugin_ui: qt_method!(fn(&mut self) -> QString),
    /// Notify backend that a plugin button was clicked
    plugin_button_clicked: qt_method!(fn(&mut self, plugin_id: QString, button_id: QString)),
    /// Notify backend that a plugin modal was submitted (fields_json: object string)
    plugin_modal_submitted: qt_method!(fn(&mut self, plugin_id: QString, modal_id: QString, fields_json: QString)),
    /// Set user status (online, idle, dnd, invisible)
    set_status: qt_method!(fn(&mut self, status: QString)),
    /// Set custom status text (empty to clear)
    set_custom_status: qt_method!(fn(&mut self, text: QString)),
    /// Join a voice channel
    join_voice: qt_method!(fn(&mut self, guild_id: QString, channel_id: QString)),
    /// Leave voice channel
    leave_voice: qt_method!(fn(&mut self)),
    /// Toggle mute
    toggle_mute: qt_method!(fn(&mut self)),
    /// Toggle deafen
    toggle_deafen: qt_method!(fn(&mut self)),
    /// Toggle fake mute (appear muted but still receive)
    toggle_fake_mute: qt_method!(fn(&mut self)),
    /// Toggle fake deafen (appear deafened but still hear)
    toggle_fake_deafen: qt_method!(fn(&mut self)),
    /// Update voice state with mute/deaf flags (sends op 4 with current flags)
    update_voice_state: qt_method!(fn(&mut self, guild_id: QString, channel_id: QString, self_mute: bool, self_deaf: bool)),
    /// Request Mullvad server list
    load_mullvad_servers: qt_method!(fn(&mut self)),
    /// Consume Mullvad servers (returns JSON array)
    consume_mullvad_servers: qt_method!(fn(&mut self) -> QString),
    /// Set proxy configuration
    set_proxy_settings: qt_method!(fn(&mut self, enabled: bool, mode: QString, mullvad_country: QString, mullvad_city: QString, mullvad_server: QString, custom_host: QString, custom_port: i32)),
    /// Get current proxy settings (returns JSON)
    get_proxy_settings: qt_method!(fn(&self) -> QString),
    /// Fetch full user profile for the profile popup (guild_id empty for DMs)
    fetch_user_profile: qt_method!(fn(&mut self, user_id: QString, guild_id: QString)),
    /// Consume pending user profile (curated JSON for UI; empty if none)
    consume_user_profile: qt_method!(fn(&mut self) -> QString),
    /// Consume pending raw API JSON for the profile (for developer copy)
    consume_user_profile_raw: qt_method!(fn(&mut self) -> QString),
    /// Get user presence status (online, idle, dnd, offline); empty if unknown
    get_user_status: qt_method!(fn(&self, user_id: QString) -> QString),
    /// Increments when presence map changes; bind in QML to refresh status dots
    presence_version: qt_property!(u32; NOTIFY state_changed),
    /// Get plugin enabled states (returns JSON: { pluginId: bool })
    get_plugin_enabled_states: qt_method!(fn(&self) -> QString),
    /// Get plugin list from manifests (returns JSON array: [{id, name, description}, ...])
    get_plugin_list: qt_method!(fn(&self) -> QString),
    /// Check if a plugin is enabled (reads from storage)
    is_plugin_enabled: qt_method!(fn(&self, plugin_id: QString) -> bool),
    /// Consume plugins-refreshed flag (true if refresh happened since last call)
    consume_plugins_refreshed: qt_method!(fn(&mut self) -> bool),
    /// Consume plugin updates check result (JSON array, empty if none)
    consume_plugin_updates: qt_method!(fn(&mut self) -> QString),
    /// Get deleted message display style: "strikethrough", "faded", or "deleted"
    get_deleted_message_style: qt_method!(fn(&self) -> QString),
    /// Set deleted message display style (persists to storage)
    set_deleted_message_style: qt_method!(fn(&mut self, style: QString)),
    /// Join a guild by invite code or URL (discord.gg/... or raw code)
    join_guild_by_invite: qt_method!(fn(&mut self, invite_code_or_url: QString)),
    /// Leave the current guild (server)
    leave_guild: qt_method!(fn(&mut self, guild_id: QString)),
    /// Mute notifications for a guild
    mute_guild: qt_method!(fn(&mut self, guild_id: QString)),
    /// Unmute notifications for a guild
    unmute_guild: qt_method!(fn(&mut self, guild_id: QString)),
    /// Consume pending join-guild result (returns JSON: {success:true,guild:{...}} or {success:false,error:"..."}, or empty)
    consume_join_guild_result: qt_method!(fn(&mut self) -> QString),
    /// Consume pending RPC invite from browser handoff (returns invite code/URL string, or empty)
    consume_rpc_invite: qt_method!(fn(&mut self) -> QString),

    // ── Internal channels ──
    /// Channel to send login requests to worker thread
    login_tx: Option<Mutex<Option<Sender<LoginRequest>>>>,
    /// Stored MFA ticket when mfa_required is true (for submit_mfa_code)
    mfa_ticket: String,
    /// Login instance ID from MfaRequired (sent with MFA code for Discord to accept)
    mfa_login_instance_id: String,
    /// Last captcha rqtoken (for submit_captcha retry header)
    captcha_rqtoken: String,
    /// Channel to receive UI updates from worker thread
    update_rx: Option<Mutex<Option<Receiver<UiUpdate>>>>,
    /// Channel to send UI actions to worker thread (tokio unbounded)
    #[allow(clippy::type_complexity)]
    action_tx: Option<tokio::sync::mpsc::UnboundedSender<UiAction>>,

    // ── Buffered data waiting to be consumed by QML ──
    pending_guilds: Vec<GuildInfo>,
    pending_channels: Vec<ChannelInfo>,
    pending_messages: Vec<MessageInfo>,
    pending_loaded_messages: Vec<MessageInfo>,
    pending_dm_channels: Vec<DmChannelInfo>,
    pending_message_edits: Vec<(String, String, String)>,  // (channel_id, message_id, new_content)
    /// (channel_id, message_id, optional content for "show as deleted")
    pending_message_deletions: Vec<(
        String,
        String,
        Option<(String, String, String, String, Option<String>)>,
    )>,
    pending_more_messages: Option<(String, Vec<MessageInfo>, bool)>, // (channel_id, messages, has_more)
    pending_typing_events: Vec<TypingEvent>,
    pending_voice_state: Option<String>,
    /// (channel_id, participants) from VoiceParticipantsChanged
    pending_voice_participants: Option<(String, Vec<VoiceParticipant>)>,
    /// Voice stats from VoiceStats update
    pending_voice_stats: Option<(u32, String, String, u32, u64, u64, u64)>,
    /// (user_id, speaking) deltas from VoiceSpeakingChanged
    pending_speaking_changes: Vec<(String, bool)>,
    pending_gifs: Vec<crate::features::gif_picker::GifResult>,
    pending_pins: Option<Vec<MessageInfo>>,
    /// (guild_id, members) from MembersLoaded
    pending_members: Option<(String, Vec<MemberInfo>)>,
    /// (guild_id, nick, roles) from MyGuildProfile
    pending_my_profile: Option<(String, Option<String>, Vec<RoleDisplayInfo>)>,
    /// (channel_id, message_id, reactions) from MessageReactionsUpdated
    pending_reaction_updates: Vec<(String, String, Vec<ReactionInfo>)>,
    pending_sticker_packs: Option<Vec<crate::client::StickerPack>>,
    pending_guild_emojis: Option<Vec<crate::client::GuildEmoji>>,
    /// (channel_id, guild_id_opt, has_unread, mention_count) from UnreadUpdate
    pending_unread_updates: Vec<(String, Option<String>, bool, u32)>,
    /// Pending Mullvad server list (JSON string)
    mullvad_servers_buf: Vec<String>,
    /// Current channel ID for filtering typing display
    current_channel_for_typing: String,
    /// Pending user profile (curated JSON) from UserProfileLoaded
    pending_user_profile: Option<String>,
    /// Pending raw API JSON from UserProfileLoaded
    pending_user_profile_raw: Option<String>,
    /// user_id -> status (online, idle, dnd, offline) from PresenceUpdated
    user_presence: HashMap<String, String>,
    /// Plugin UI state: plugin_id -> (buttons, modals)
    plugin_ui: std::collections::HashMap<String, (Vec<PluginUiButton>, Vec<PluginUiModal>)>,
    /// Plugins were refreshed from disk — UI should reload plugin list
    plugins_refreshed: bool,
    /// Pending plugin update check result (JSON array)
    plugin_updates_buf: Option<String>,
    /// Pending join-guild result: Ok(GuildInfo) on success, Err(error_string) on failure
    pending_join_guild_result: Option<Result<GuildInfo, String>>,
    /// Pending RPC invites received from browser handoff
    pending_rpc_invites: Vec<String>,
    /// Guild IDs muted for notifications (updated by GuildMuteStateChanged)
    muted_guild_ids: Vec<String>,
    /// Current relationships list (friends, pending, blocked); updated by RelationshipsLoaded / RelationshipAdded / RelationshipRemoved
    pending_relationships: Vec<RelationshipInfo>,
    /// True when pending_relationships was updated and not yet consumed
    relationships_dirty: bool,
}

/// Tracks a typing start event for display
struct TypingEvent {
    channel_id: String,
    user_name: String,
    role_color: Option<String>,
    timestamp: std::time::Instant,
}

impl AppController {
    pub fn new(
        login_tx: Sender<LoginRequest>,
        action_tx: tokio::sync::mpsc::UnboundedSender<UiAction>,
        update_rx: Receiver<UiUpdate>,
    ) -> Self {
        let mut ctrl: AppController = Default::default();
        ctrl.login_tx = Some(Mutex::new(Some(login_tx)));
        ctrl.update_rx = Some(Mutex::new(Some(update_rx)));
        ctrl.action_tx = Some(action_tx);
        ctrl.login_mode = QString::from("credentials");
        ctrl
    }

    // ── Helper: serialize a MessageInfo to JSON for QML ──
    fn message_to_json(m: crate::bridge::MessageInfo) -> serde_json::Value {
        serde_json::json!({
            "messageId": m.id,
            "channelId": m.channel_id,
            "authorName": m.author_name,
            "authorId": m.author_id,
            "authorAvatarUrl": m.author_avatar_url,
            "content": m.content,
            "timestamp": m.timestamp,
            "isDeleted": m.is_deleted,
            "messageType": m.message_type,
            "replyAuthorName": m.reply_author_name.unwrap_or_default(),
            "replyContent": m.reply_content.unwrap_or_default(),
            "replyAuthorId": m.reply_author_id.unwrap_or_default(),
            "replyAuthorRoleColor": m.reply_author_role_color.as_deref().unwrap_or(""),
            "mentionsMe": m.mentions_me,
            "mentionEveryone": m.mention_everyone,
            "authorRoleColor": m.author_role_color.as_deref().unwrap_or(""),
            "authorRoleName": m.author_role_name.as_deref().unwrap_or(""),
            "authorPublicFlags": m.author_public_flags,
            "authorBot": m.author_bot,
            "authorPremiumType": m.author_premium_type,
            "attachmentsJson": m.attachments_json,
            "stickersJson": m.stickers_json,
            "embedsJson": m.embeds_json,
            "contentHtml": m.content_html,
            "reactions": m.reactions.iter().map(|r| serde_json::json!({
                "emoji": r.emoji_display,
                "count": r.count,
                "me": r.me,
            })).collect::<Vec<_>>(),
        })
    }

    // ── Helper: send an action to the backend ──
    fn send_action(&self, action: UiAction) {
        if let Some(ref tx) = self.action_tx {
            if let Err(e) = tx.send(action) {
                tracing::error!("Failed to send UI action: {}", e);
            }
        }
    }

    // ── Login ──
    fn login(&mut self, token: QString) {
        let token = token.trimmed().to_string();
        let token = token.trim_matches('"').trim_matches('\'').to_string();

        if token.is_empty() {
            self.error_message = QString::from("Please enter your token");
            self.state_changed();
            return;
        }

        self.error_message = QString::default();
        self.is_loading = true;
        self.state_changed();

        if let Some(ref mtx) = self.login_tx {
            if let Ok(guard) = mtx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(LoginRequest::Token(token));
                }
            }
        }
    }

    fn login_credentials(&mut self, email: QString, password: QString) {
        let email = email.trimmed().to_string();
        let password = password.trimmed().to_string();
        if email.is_empty() || password.is_empty() {
            self.error_message = QString::from("Please enter email and password");
            self.state_changed();
            return;
        }
        self.error_message = QString::default();
        self.is_loading = true;
        self.mfa_required = false;
        self.captcha_visible = false;
        self.state_changed();
        if let Some(ref mtx) = self.login_tx {
            if let Ok(guard) = mtx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(LoginRequest::Credentials {
                        email,
                        password,
                    });
                }
            }
        }
    }

    fn submit_mfa_code(&mut self, code: QString) {
        let code = code.trimmed().to_string();
        if code.is_empty() || self.mfa_ticket.is_empty() {
            return;
        }
        let ticket = std::mem::take(&mut self.mfa_ticket);
        let login_instance_id = if self.mfa_login_instance_id.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.mfa_login_instance_id))
        };
        if let Some(ref mtx) = self.login_tx {
            if let Ok(guard) = mtx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(LoginRequest::MfaCode {
                        ticket,
                        code,
                        login_instance_id,
                    });
                }
            }
        }
        self.state_changed();
    }

    fn cancel_mfa(&mut self) {
        if self.mfa_required {
            self.mfa_required = false;
            self.mfa_ticket.clear();
            self.mfa_login_instance_id.clear();
        }
        if let Some(ref mtx) = self.login_tx {
            if let Ok(guard) = mtx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(LoginRequest::CancelMfa);
                }
            }
        }
        self.state_changed();
    }

    fn submit_captcha(&mut self, captcha_token: QString) {
        let captcha_key = captcha_token.trimmed().to_string();
        if captcha_key.is_empty() {
            return;
        }
        let rqtoken = if self.captcha_rqtoken.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.captcha_rqtoken))
        };
        if let Some(ref mtx) = self.login_tx {
            if let Ok(guard) = mtx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(LoginRequest::CaptchaSolution {
                        captcha_key,
                        rqtoken,
                    });
                }
            }
        }
        self.captcha_visible = false;
        self.captcha_html = QString::default();
        self.state_changed();
    }

    fn set_login_mode(&mut self, mode: QString) {
        let m = mode.trimmed().to_string();
        if m == "token" || m == "credentials" {
            self.login_mode = QString::from(m.as_str());
            self.state_changed();
        }
    }

    // ── Navigation actions ──
    fn select_guild(&mut self, guild_id: QString) {
        self.send_action(UiAction::SelectGuild(guild_id.to_string()));
    }

    fn select_channel(&mut self, channel_id: QString, channel_type: i32) {
        self.send_action(UiAction::SelectChannel(
            channel_id.to_string(),
            channel_type as u8,
        ));
    }

    fn send_message(&mut self, channel_id: QString, content: QString) {
        let content = content.to_string().trim().to_string();
        if content.is_empty() {
            return;
        }
        self.send_action(UiAction::SendMessage {
            channel_id: channel_id.to_string(),
            content,
            silent: false,
        });
    }

    fn open_dm(&mut self, recipient_id: QString) {
        self.send_action(UiAction::OpenDm(recipient_id.to_string()));
    }

    fn join_guild_by_invite(&mut self, invite_code_or_url: QString) {
        let input = invite_code_or_url.trimmed().to_string();
        if !input.is_empty() {
            self.send_action(UiAction::JoinGuildByInvite {
                invite_code_or_url: input,
            });
        }
    }

    fn leave_guild(&mut self, guild_id: QString) {
        let id = guild_id.trimmed().to_string();
        if !id.is_empty() {
            self.send_action(UiAction::LeaveGuild(id));
        }
    }

    fn mute_guild(&mut self, guild_id: QString) {
        let id = guild_id.trimmed().to_string();
        if !id.is_empty() {
            self.send_action(UiAction::MuteGuild(id));
        }
    }

    fn unmute_guild(&mut self, guild_id: QString) {
        let id = guild_id.trimmed().to_string();
        if !id.is_empty() {
            self.send_action(UiAction::UnmuteGuild(id));
        }
    }

    fn send_friend_request(&mut self, username: QString) {
        self.send_action(UiAction::SendFriendRequest {
            username: username.to_string(),
        });
    }

    fn accept_friend_request(&mut self, user_id: QString) {
        self.send_action(UiAction::AcceptFriendRequest {
            user_id: user_id.to_string(),
        });
    }

    fn remove_relationship(&mut self, user_id: QString) {
        self.send_action(UiAction::RemoveRelationship {
            user_id: user_id.to_string(),
        });
    }

    fn block_user(&mut self, user_id: QString) {
        self.send_action(UiAction::BlockUser {
            user_id: user_id.to_string(),
        });
    }

    fn load_more_messages(&mut self, channel_id: QString, before_message_id: QString) {
        self.send_action(UiAction::LoadMoreMessages {
            channel_id: channel_id.to_string(),
            before_message_id: before_message_id.to_string(),
        });
    }

    fn delete_message(&mut self, channel_id: QString, message_id: QString) {
        self.send_action(UiAction::DeleteMessage {
            channel_id: channel_id.to_string(),
            message_id: message_id.to_string(),
        });
    }

    fn pin_message(&mut self, channel_id: QString, message_id: QString) {
        self.send_action(UiAction::PinMessage {
            channel_id: channel_id.to_string(),
            message_id: message_id.to_string(),
        });
    }

    fn unpin_message(&mut self, channel_id: QString, message_id: QString) {
        self.send_action(UiAction::UnpinMessage {
            channel_id: channel_id.to_string(),
            message_id: message_id.to_string(),
        });
    }

    fn open_pins(&mut self, channel_id: QString) {
        self.send_action(UiAction::OpenPins(channel_id.to_string()));
    }

    fn copy_message_link(&self, channel_id: QString, guild_id: QString, message_id: QString) {
        let channel_id = channel_id.trimmed().to_string();
        let guild_id = guild_id.trimmed().to_string();
        let message_id = message_id.trimmed().to_string();
        if channel_id.is_empty() || message_id.is_empty() {
            return;
        }
        let url = if guild_id.is_empty() {
            format!("https://discord.com/channels/@me/{}/{}", channel_id, message_id)
        } else {
            format!("https://discord.com/channels/{}/{}/{}", guild_id, channel_id, message_id)
        };
        #[cfg(feature = "desktop")]
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if clipboard.set_text(&url).is_err() {
                tracing::warn!("Failed to set clipboard");
            }
        }
    }

    fn copy_to_clipboard(&self, text: QString) {
        let s = text.trimmed().to_string();
        if s.is_empty() {
            return;
        }
        #[cfg(feature = "desktop")]
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if clipboard.set_text(&s).is_err() {
                tracing::warn!("Failed to set clipboard");
            }
        }
    }

    fn clear_error(&mut self) {
        self.error_message = QString::default();
        self.state_changed();
    }

    fn reply_to_message(&mut self, _message_id: QString) {
        // Reply state is managed in QML (replyToMessageId / replyToAuthor)
        tracing::debug!("Reply requested for message: {}", _message_id.to_string());
    }

    fn edit_message(&mut self, channel_id: QString, message_id: QString, content: QString) {
        let content = content.to_string().trim().to_string();
        if content.is_empty() {
            return;
        }
        self.send_action(UiAction::EditMessage {
            channel_id: channel_id.to_string(),
            message_id: message_id.to_string(),
            content,
        });
    }

    fn add_reaction(&mut self, channel_id: QString, message_id: QString, emoji: QString) {
        self.send_action(UiAction::AddReaction {
            channel_id: channel_id.to_string(),
            message_id: message_id.to_string(),
            emoji: emoji.to_string(),
        });
    }

    fn remove_reaction(&mut self, channel_id: QString, message_id: QString, emoji: QString) {
        self.send_action(UiAction::RemoveReaction {
            channel_id: channel_id.to_string(),
            message_id: message_id.to_string(),
            emoji: emoji.to_string(),
        });
    }

    fn send_message_ex(&mut self, channel_id: QString, content: QString, silent: bool, reply_to: QString) {
        self.send_message_with_options_impl(
            channel_id,
            content,
            silent,
            reply_to,
            None,
            None,
        );
    }

    fn send_message_with_options(
        &mut self,
        channel_id: QString,
        content: QString,
        silent: bool,
        reply_to: QString,
        sticker_ids_json: QString,
        attachment_paths_json: QString,
    ) {
        let sticker_ids: Option<Vec<String>> = serde_json::from_str(sticker_ids_json.trimmed().to_string().as_str()).ok();
        let attachment_paths: Option<Vec<String>> =
            serde_json::from_str(attachment_paths_json.trimmed().to_string().as_str()).ok();
        self.send_message_with_options_impl(
            channel_id,
            content,
            silent,
            reply_to,
            sticker_ids,
            attachment_paths,
        );
    }

    fn send_message_with_options_impl(
        &mut self,
        channel_id: QString,
        content: QString,
        silent: bool,
        reply_to: QString,
        sticker_ids: Option<Vec<String>>,
        attachment_paths: Option<Vec<String>>,
    ) {
        let content = content.to_string().trim().to_string();
        let has_attachments = attachment_paths.as_ref().map_or(false, |v| !v.is_empty());
        let has_stickers = sticker_ids.as_ref().map_or(false, |v| !v.is_empty());
        if content.is_empty() && !has_attachments && !has_stickers {
            return;
        }
        let reply_to_str = reply_to.to_string();
        let message_reference = if reply_to_str.is_empty() {
            None
        } else {
            Some(reply_to_str)
        };
        self.send_action(UiAction::SendMessageEx {
            channel_id: channel_id.to_string(),
            content,
            silent,
            reply_to_message_id: message_reference,
            sticker_ids,
            attachment_paths,
        });
    }

    fn start_typing(&mut self, channel_id: QString) {
        self.send_action(UiAction::StartTyping(channel_id.to_string()));
    }

    fn logout(&mut self) {
        self.send_action(UiAction::Logout);
        self.is_logged_in = false;
        self.user_name = QString::default();
        self.user_id = QString::default();
        self.user_avatar = QString::default();
        self.connection_state = QString::from("disconnected");
        self.error_message = QString::default();
        self.pending_guilds.clear();
        self.pending_channels.clear();
        self.pending_messages.clear();
        self.pending_loaded_messages.clear();
        self.pending_dm_channels.clear();
        self.pending_relationships.clear();
        self.relationships_dirty = false;
        self.pending_message_edits.clear();
        self.pending_message_deletions.clear();
        self.pending_more_messages = None;
        self.pending_typing_events.clear();
        self.pending_members = None;
        self.state_changed();
    }

    fn switch_account(&mut self, account_id: QString) {
        let id = account_id.to_string();
        if let Some(ref mtx) = self.login_tx {
            if let Ok(guard) = mtx.lock() {
                if let Some(ref tx) = *guard {
                    let _ = tx.send(LoginRequest::SwitchAccount(id.clone()));
                }
            }
        }
        self.send_action(UiAction::SwitchAccount(id));
    }

    fn mark_all_read(&mut self) {
        self.send_action(UiAction::MarkAllRead);
    }

    fn set_plugin_enabled(&mut self, plugin_id: QString, enabled: bool) {
        self.send_action(UiAction::SetPluginEnabled {
            plugin_id: plugin_id.to_string(),
            enabled,
        });
    }

    fn install_plugin(&mut self, repo_url: QString) {
        self.send_action(UiAction::InstallPlugin(repo_url.to_string()));
    }

    fn refresh_plugins(&mut self) {
        self.send_action(UiAction::RefreshPlugins);
    }

    fn check_plugin_updates(&mut self) {
        self.send_action(UiAction::CheckPluginUpdates);
    }

    fn consume_plugin_ui(&mut self) -> QString {
        let mut obj = serde_json::Map::new();
        for (plugin_id, (buttons, modals)) in &self.plugin_ui {
            let buttons_json = serde_json::to_value(buttons).unwrap_or(serde_json::Value::Array(vec![]));
            let modals_json = serde_json::to_value(modals).unwrap_or(serde_json::Value::Array(vec![]));
            obj.insert(
                plugin_id.clone(),
                serde_json::json!({ "buttons": buttons_json, "modals": modals_json }),
            );
        }
        QString::from(
            serde_json::to_string(&serde_json::Value::Object(obj)).unwrap_or_default().as_str(),
        )
    }

    fn plugin_button_clicked(&mut self, plugin_id: QString, button_id: QString) {
        self.send_action(UiAction::PluginButtonClicked {
            plugin_id: plugin_id.to_string(),
            button_id: button_id.to_string(),
        });
    }

    fn plugin_modal_submitted(
        &mut self,
        plugin_id: QString,
        modal_id: QString,
        fields_json: QString,
    ) {
        let fields: std::collections::HashMap<String, String> =
            serde_json::from_str(fields_json.trimmed().to_string().as_str()).unwrap_or_default();
        self.send_action(UiAction::PluginModalSubmitted {
            plugin_id: plugin_id.to_string(),
            modal_id: modal_id.to_string(),
            fields,
        });
    }

    fn set_status(&mut self, status: QString) {
        self.send_action(UiAction::SetStatus(status.to_string()));
    }

    fn set_custom_status(&mut self, text: QString) {
        let text = text.to_string();
        let opt = if text.is_empty() { None } else { Some(text) };
        self.send_action(UiAction::SetCustomStatus(opt));
    }

    fn join_voice(&mut self, guild_id: QString, channel_id: QString) {
        let gid = guild_id.to_string();
        let guild = if gid.is_empty() { None } else { Some(gid) };
        self.send_action(UiAction::JoinVoice {
            guild_id: guild,
            channel_id: channel_id.to_string(),
        });
    }

    fn leave_voice(&mut self) {
        self.send_action(UiAction::LeaveVoice);
    }

    fn toggle_mute(&mut self) {
        self.send_action(UiAction::ToggleMute);
    }

    fn toggle_deafen(&mut self) {
        self.send_action(UiAction::ToggleDeafen);
    }

    fn toggle_fake_mute(&mut self) {
        self.send_action(UiAction::ToggleFakeMute);
    }

    fn toggle_fake_deafen(&mut self) {
        self.send_action(UiAction::ToggleFakeDeafen);
    }

    fn update_voice_state(
        &mut self,
        guild_id: QString,
        channel_id: QString,
        self_mute: bool,
        self_deaf: bool,
    ) {
        // Send a direct voice state update with explicit flags
        let gid = guild_id.to_string();
        let guild = if gid.is_empty() { None } else { Some(gid) };
        self.send_action(UiAction::JoinVoice {
            guild_id: guild,
            channel_id: channel_id.to_string(),
        });
        // The mute/deaf flags are managed via a separate gateway command;
        // for now the flags are sent as part of join and the QML manages state
        let _ = (self_mute, self_deaf); // silence unused warnings — flags handled by QML
    }

    /// Update the typing display text and JSON from pending events for current channel
    fn update_typing_display(&mut self) {
        let events: Vec<&TypingEvent> = self
            .pending_typing_events
            .iter()
            .filter(|e| e.channel_id == self.current_channel_for_typing)
            .collect();
        let names: Vec<&str> = events.iter().map(|e| e.user_name.as_str()).collect();
        let text = match names.len() {
            0 => String::new(),
            1 => format!("{} is typing...", names[0]),
            2 => format!("{} and {} are typing...", names[0], names[1]),
            n => format!("{}, {}, and {} others are typing...", names[0], names[1], n - 2),
        };
        self.typing_display = QString::from(text.as_str());
        let json_arr: Vec<serde_json::Value> = events
            .iter()
            .map(|e| {
                serde_json::json!({
                    "name": e.user_name,
                    "roleColor": e.role_color.as_deref().unwrap_or("")
                })
            })
            .collect();
        self.typing_display_json = QString::from(
            serde_json::to_string(&json_arr).unwrap_or_else(|_| "[]".to_string()).as_str(),
        );
        self.state_changed();
    }

    /// Get comma-separated list of typing users in a specific channel
    fn get_typing_in_channel(&self, channel_id: QString) -> QString {
        let channel_id_str = channel_id.to_string();
        let names: Vec<&str> = self
            .pending_typing_events
            .iter()
            .filter(|e| e.channel_id == channel_id_str)
            .map(|e| e.user_name.as_str())
            .collect();
        QString::from(names.join(", "))
    }

    /// Update the typing display for a new channel
    fn update_typing_for_channel(&mut self, channel_id: QString) {
        self.current_channel_for_typing = channel_id.to_string();
        self.update_typing_display();
    }

    // ── Poll for backend updates ──
    fn check_for_updates(&mut self) {
        // Collect all pending updates first, then drop the rx borrow
        let updates: Vec<UiUpdate> = {
            let mtx = match self.update_rx.as_ref() {
                Some(m) => m,
                None => return,
            };
            let rx_guard = match mtx.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            let rx = match rx_guard.as_ref() {
                Some(rx) => rx,
                None => return,
            };
            let mut buf = Vec::new();
            while let Ok(update) = rx.try_recv() {
                buf.push(update);
            }
            buf
        };

        let mut typing_changed = false;

        for update in updates {
            match update {
                UiUpdate::LoginSuccess {
                    user_id,
                    username,
                    avatar_url,
                } => {
                    self.is_logged_in = true;
                    self.is_loading = false;
                    self.error_message = QString::default();
                    self.mfa_required = false;
                    self.mfa_ticket.clear();
                    self.mfa_login_instance_id.clear();
                    self.captcha_visible = false;
                    self.captcha_html = QString::default();
                    self.user_name = QString::from(username.as_str());
                    self.user_id = QString::from(user_id.as_str());
                    self.user_avatar = QString::from(
                        avatar_url.as_deref().unwrap_or(""),
                    );
                    self.connection_state = QString::from("connected");
                    self.state_changed();
                }
                UiUpdate::LoginFailed(msg) => {
                    self.is_loading = false;
                    self.error_message = QString::from(msg.as_str());
                    self.mfa_required = false;
                    self.captcha_visible = false;
                    self.state_changed();
                }
                UiUpdate::MfaRequired {
                    ticket,
                    login_instance_id,
                    sms: _,
                    totp: _,
                    backup: _,
                } => {
                    self.mfa_required = true;
                    self.mfa_ticket = ticket;
                    self.mfa_login_instance_id = login_instance_id.unwrap_or_default();
                    self.state_changed();
                }
                UiUpdate::CaptchaRequired {
                    sitekey,
                    rqdata,
                    rqtoken,
                    captcha_session_id,
                } => {
                    let challenge = CaptchaChallenge {
                        captcha_service: "hcaptcha".to_string(),
                        captcha_sitekey: sitekey,
                        captcha_rqdata: rqdata,
                        captcha_rqtoken: rqtoken.clone(),
                        captcha_session_id,
                        captcha_key: None,
                    };
                    self.captcha_html = QString::from(widget::generate_captcha_html(&challenge).as_str());
                    self.captcha_rqtoken = rqtoken.unwrap_or_default();
                    self.captcha_visible = true;
                    self.state_changed();
                }
                UiUpdate::Error(msg) => {
                    self.is_loading = false;
                    self.error_message = QString::from(msg.as_str());
                    self.state_changed();
                }
                UiUpdate::GuildsLoaded(guilds) => {
                    self.pending_guilds = guilds;
                }
                UiUpdate::ChannelsLoaded(channels) => {
                    self.pending_channels = channels;
                }
                UiUpdate::MyGuildProfile { guild_id, nick, roles } => {
                    self.pending_my_profile = Some((guild_id, nick, roles));
                }
                UiUpdate::DmChannelsLoaded(dm_channels) => {
                    self.pending_dm_channels = dm_channels;
                }
                UiUpdate::RelationshipsLoaded(relationships) => {
                    self.pending_relationships = relationships;
                    self.relationships_dirty = true;
                }
                UiUpdate::RelationshipAdded(info) => {
                    // Remove if already present (e.g. type change), then push
                    self.pending_relationships.retain(|r| r.user_id != info.user_id);
                    self.pending_relationships.push(info);
                    self.relationships_dirty = true;
                }
                UiUpdate::RelationshipRemoved { user_id } => {
                    self.pending_relationships.retain(|r| r.user_id != user_id);
                    self.relationships_dirty = true;
                }
                UiUpdate::MessagesLoaded(messages) => {
                    self.pending_loaded_messages = messages;
                }
                UiUpdate::NewMessage(msg) => {
                    self.pending_messages.push(msg);
                }
                UiUpdate::MessageEdited {
                    channel_id,
                    message_id,
                    new_content,
                } => {
                    self.pending_message_edits
                        .push((channel_id, message_id, new_content));
                }
                UiUpdate::MessageDeleted {
                    channel_id,
                    message_id,
                } => {
                    self.pending_message_deletions
                        .push((channel_id, message_id, None));
                }
                UiUpdate::MessageDeletedWithContent {
                    channel_id,
                    message_id,
                    content,
                    author_name,
                    author_id,
                    timestamp,
                    author_avatar_url,
                } => {
                    self.pending_message_deletions.push((
                        channel_id,
                        message_id,
                        Some((content, author_name, author_id, timestamp, author_avatar_url)),
                    ));
                }
                UiUpdate::MoreMessagesLoaded {
                    channel_id,
                    messages,
                    has_more,
                } => {
                    self.pending_more_messages = Some((channel_id, messages, has_more));
                }
                UiUpdate::PinsLoaded { messages, .. } => {
                    self.pending_pins = Some(messages);
                }
                UiUpdate::MembersLoaded { guild_id, members } => {
                    self.pending_members = Some((guild_id, members));
                }
                UiUpdate::Connected => {
                    self.connection_state = QString::from("connected");
                    self.state_changed();
                }
                UiUpdate::AccountsList(accounts) => {
                    let arr: Vec<serde_json::Value> = accounts
                        .into_iter()
                        .map(|(id, name)| serde_json::json!({ "id": id, "name": name }))
                        .collect();
                    self.accounts_json = QString::from(
                        serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string()).as_str(),
                    );
                    self.state_changed();
                }
                UiUpdate::Disconnected => {
                    self.connection_state = QString::from("disconnected");
                    self.state_changed();
                }
                UiUpdate::Reconnecting => {
                    self.connection_state = QString::from("reconnecting");
                    self.state_changed();
                }
                UiUpdate::TypingStarted {
                    channel_id,
                    user_name,
                    role_color,
                } => {
                    self.pending_typing_events.push(TypingEvent {
                        channel_id,
                        user_name,
                        role_color,
                        timestamp: std::time::Instant::now(),
                    });
                    typing_changed = true;
                }
                UiUpdate::VoiceStateChanged(state_str) => {
                    self.pending_voice_state = Some(state_str);
                }
                UiUpdate::VoiceParticipantsChanged { channel_id, participants } => {
                    self.pending_voice_participants = Some((channel_id, participants));
                }
                UiUpdate::VoiceConnectionProgress(s) => {
                    self.voice_connection_state = QString::from(s.as_str());
                }
                UiUpdate::VoiceSpeakingChanged { user_id, speaking } => {
                    self.pending_speaking_changes.push((user_id, speaking));
                }
                UiUpdate::VoiceStats {
                    ping_ms,
                    encryption_mode,
                    endpoint,
                    ssrc,
                    packets_sent,
                    packets_received,
                    connection_duration_secs,
                } => {
                    self.pending_voice_stats = Some((
                        ping_ms,
                        encryption_mode,
                        endpoint,
                        ssrc,
                        packets_sent,
                        packets_received,
                        connection_duration_secs,
                    ));
                }
                UiUpdate::GifsLoaded(gifs) => {
                    self.pending_gifs = gifs;
                }
                UiUpdate::StickerPacksLoaded(packs) => {
                    self.pending_sticker_packs = Some(packs);
                }
                UiUpdate::GuildEmojisLoaded(emojis) => {
                    self.pending_guild_emojis = Some(emojis);
                }
                UiUpdate::MessageReactionsUpdated {
                    channel_id,
                    message_id,
                    reactions,
                } => {
                    self.pending_reaction_updates
                        .push((channel_id, message_id, reactions));
                }
                UiUpdate::UnreadUpdate {
                    channel_id,
                    guild_id,
                    has_unread,
                    mention_count,
                } => {
                    self.pending_unread_updates
                        .push((channel_id, guild_id.clone(), has_unread, mention_count));
                }
                UiUpdate::MullvadServersLoaded(json) => {
                    self.mullvad_servers_buf.push(json);
                }
                UiUpdate::UserProfileLoaded {
                    profile_json,
                    raw_json,
                    ..
                } => {
                    self.pending_user_profile = Some(profile_json);
                    self.pending_user_profile_raw = Some(raw_json);
                    self.state_changed();
                }
                UiUpdate::PresenceUpdated { user_id, status } => {
                    self.user_presence.insert(user_id.clone(), status.clone());
                    self.presence_version = self.presence_version.wrapping_add(1);
                    self.state_changed();
                }
                UiUpdate::PluginUiUpdated {
                    plugin_id,
                    buttons,
                    modals,
                } => {
                    self.plugin_ui
                        .insert(plugin_id, (buttons, modals));
                }
                UiUpdate::PluginUiRemoved { plugin_id } => {
                    self.plugin_ui.remove(&plugin_id);
                }
                UiUpdate::PluginsRefreshed => {
                    self.plugins_refreshed = true;
                }
                UiUpdate::PluginUpdatesAvailable(json) => {
                    self.plugin_updates_buf = Some(json);
                }
                UiUpdate::JoinGuildSuccess(guild_info) => {
                    self.pending_join_guild_result = Some(Ok(guild_info));
                }
                UiUpdate::JoinGuildFailed(err) => {
                    self.pending_join_guild_result = Some(Err(err));
                }
                UiUpdate::GuildMuteStateChanged { guild_id, muted } => {
                    if muted {
                        if !self.muted_guild_ids.contains(&guild_id) {
                            self.muted_guild_ids.push(guild_id);
                        }
                    } else {
                        self.muted_guild_ids.retain(|id| id != &guild_id);
                    }
                    self.muted_guild_ids_json = QString::from(
                        serde_json::to_string(&self.muted_guild_ids).unwrap_or_else(|_| "[]".to_string()).as_str(),
                    );
                    self.state_changed();
                }
                UiUpdate::RpcInviteReceived(invite) => {
                    self.pending_rpc_invites.push(invite);
                }
                _ => {}
            }
        }

        // Expire old typing events (> 8 seconds)
        let now = std::time::Instant::now();
        let before = self.pending_typing_events.len();
        self.pending_typing_events
            .retain(|e| now.duration_since(e.timestamp).as_secs() < 8);
        if self.pending_typing_events.len() != before {
            typing_changed = true;
        }

        if typing_changed {
            self.update_typing_display();
        }
    }

    // ── Data consumption methods (called by QML Timer) ──
    fn consume_guilds(&mut self) -> QString {
        if self.pending_guilds.is_empty() {
            return QString::default();
        }
        let guilds: Vec<serde_json::Value> = self
            .pending_guilds
            .drain(..)
            .map(|g| {
                serde_json::json!({
                    "guildId": g.id,
                    "name": g.name,
                    "iconUrl": g.icon_url.unwrap_or_default(),
                    "hasUnread": g.has_unread,
                    "mentionCount": g.mention_count,
                })
            })
            .collect();
        QString::from(serde_json::to_string(&guilds).unwrap_or_default().as_str())
    }

    fn consume_channels(&mut self) -> QString {
        if self.pending_channels.is_empty() {
            return QString::default();
        }
        let channels: Vec<serde_json::Value> = self
            .pending_channels
            .drain(..)
            .map(|c| {
                serde_json::json!({
                    "channelId": c.id,
                    "guildId": c.guild_id.unwrap_or_default(),
                    "name": c.name,
                    "channelType": c.channel_type,
                    "position": c.position,
                    "parentId": c.parent_id.unwrap_or_default(),
                    "hasUnread": c.has_unread,
                    "mentionCount": c.mention_count,
                    "isHidden": c.is_hidden,
                })
            })
            .collect();
        QString::from(serde_json::to_string(&channels).unwrap_or_default().as_str())
    }

    fn consume_messages(&mut self) -> QString {
        if self.pending_messages.is_empty() {
            return QString::default();
        }
        let messages: Vec<serde_json::Value> = self
            .pending_messages
            .drain(..)
            .map(Self::message_to_json)
            .collect();
        QString::from(serde_json::to_string(&messages).unwrap_or_default().as_str())
    }

    fn consume_loaded_messages(&mut self) -> QString {
        if self.pending_loaded_messages.is_empty() {
            return QString::default();
        }
        let messages: Vec<serde_json::Value> = self
            .pending_loaded_messages
            .drain(..)
            .map(Self::message_to_json)
            .collect();
        QString::from(serde_json::to_string(&messages).unwrap_or_default().as_str())
    }

    fn consume_dm_channels(&mut self) -> QString {
        if self.pending_dm_channels.is_empty() {
            return QString::default();
        }
        let dms: Vec<serde_json::Value> = self
            .pending_dm_channels
            .drain(..)
            .map(|d| {
                serde_json::json!({
                    "channelId": d.id,
                    "recipientName": d.recipient_name,
                    "recipientId": d.recipient_id,
                    "recipientAvatarUrl": d.recipient_avatar_url.unwrap_or_default(),
                    "channelType": d.channel_type,
                    "lastMessageId": d.last_message_id.unwrap_or_default(),
                })
            })
            .collect();
        QString::from(serde_json::to_string(&dms).unwrap_or_default().as_str())
    }

    fn consume_relationships(&mut self) -> QString {
        if !self.relationships_dirty {
            return QString::default();
        }
        self.relationships_dirty = false;
        match serde_json::to_string(&self.pending_relationships) {
            Ok(json) => QString::from(json.as_str()),
            Err(_) => QString::default(),
        }
    }

    fn consume_message_edits(&mut self) -> QString {
        if self.pending_message_edits.is_empty() {
            return QString::default();
        }
        let edits: Vec<serde_json::Value> = self
            .pending_message_edits
            .drain(..)
            .map(|(channel_id, message_id, new_content)| {
                serde_json::json!({
                    "channelId": channel_id,
                    "messageId": message_id,
                    "newContent": new_content,
                })
            })
            .collect();
        QString::from(serde_json::to_string(&edits).unwrap_or_default().as_str())
    }

    fn consume_message_deletions(&mut self) -> QString {
        if self.pending_message_deletions.is_empty() {
            return QString::default();
        }
        let deletions: Vec<serde_json::Value> = self
            .pending_message_deletions
            .drain(..)
            .map(|(channel_id, message_id, content)| {
                let mut obj = serde_json::json!({
                    "channelId": channel_id,
                    "messageId": message_id,
                });
                if let Some((content, author_name, author_id, timestamp, author_avatar_url)) = content {
                    obj["content"] = serde_json::json!(content);
                    obj["authorName"] = serde_json::json!(author_name);
                    obj["authorId"] = serde_json::json!(author_id);
                    obj["timestamp"] = serde_json::json!(timestamp);
                    obj["authorAvatarUrl"] = serde_json::json!(author_avatar_url);
                    obj["isDeleted"] = serde_json::json!(true);
                }
                obj
            })
            .collect();
        QString::from(serde_json::to_string(&deletions).unwrap_or_default().as_str())
    }

    fn consume_more_messages(&mut self) -> QString {
        let data = match self.pending_more_messages.take() {
            Some(d) => d,
            None => return QString::default(),
        };
        let (channel_id, messages, has_more) = data;
        let msg_values: Vec<serde_json::Value> = messages
            .into_iter()
            .map(Self::message_to_json)
            .collect();
        let result = serde_json::json!({
            "channelId": channel_id,
            "messages": msg_values,
            "hasMore": has_more,
        });
        QString::from(serde_json::to_string(&result).unwrap_or_default().as_str())
    }

    fn consume_voice_state(&mut self) -> QString {
        match self.pending_voice_state.take() {
            Some(state) => QString::from(state.as_str()),
            None => QString::default(),
        }
    }

    fn consume_voice_participants(&mut self) -> QString {
        let Some((channel_id, participants)) = self.pending_voice_participants.take() else {
            return QString::default();
        };
        let arr: Vec<serde_json::Value> = participants
            .into_iter()
            .map(|p| {
                serde_json::json!({
                    "userId": p.user_id,
                    "username": p.username,
                    "avatarUrl": p.avatar_url,
                    "selfMute": p.self_mute,
                    "selfDeaf": p.self_deaf,
                    "serverMute": p.server_mute,
                    "serverDeaf": p.server_deaf,
                    "speaking": p.speaking,
                    "selfVideo": p.self_video,
                    "selfStream": p.self_stream,
                    "suppress": p.suppress,
                })
            })
            .collect();
        let obj = serde_json::json!({ "channelId": channel_id, "participants": arr });
        QString::from(serde_json::to_string(&obj).unwrap_or_default().as_str())
    }

    fn consume_voice_stats(&mut self) -> QString {
        let Some((ping_ms, encryption_mode, endpoint, ssrc, packets_sent, packets_received, connection_duration_secs)) =
            self.pending_voice_stats.take()
        else {
            return QString::default();
        };
        let obj = serde_json::json!({
            "pingMs": ping_ms,
            "encryptionMode": encryption_mode,
            "endpoint": endpoint,
            "ssrc": ssrc,
            "packetsSent": packets_sent,
            "packetsReceived": packets_received,
            "connectionDurationSecs": connection_duration_secs,
        });
        QString::from(serde_json::to_string(&obj).unwrap_or_default().as_str())
    }

    fn consume_speaking_users(&mut self) -> QString {
        if self.pending_speaking_changes.is_empty() {
            return QString::default();
        }
        let arr: Vec<serde_json::Value> = self
            .pending_speaking_changes
            .drain(..)
            .map(|(user_id, speaking)| serde_json::json!({ "userId": user_id, "speaking": speaking }))
            .collect();
        QString::from(serde_json::to_string(&arr).unwrap_or_default().as_str())
    }

    fn search_gifs(&mut self, query: QString) {
        self.send_action(UiAction::SearchGifs(query.to_string()));
    }

    fn consume_gifs(&mut self) -> QString {
        if self.pending_gifs.is_empty() {
            return QString::default();
        }
        let gifs: Vec<serde_json::Value> = self
            .pending_gifs
            .drain(..)
            .map(|g| {
                serde_json::json!({
                    "id": g.id,
                    "title": g.title,
                    "gifUrl": g.gif_url,
                    "previewUrl": g.preview_url,
                    "width": g.width,
                    "height": g.height,
                })
            })
            .collect();
        QString::from(serde_json::to_string(&gifs).unwrap_or_default().as_str())
    }

    fn load_sticker_packs(&mut self) {
        self.send_action(UiAction::LoadStickerPacks);
    }

    fn consume_sticker_packs(&mut self) -> QString {
        let packs = match self.pending_sticker_packs.take() {
            Some(p) => p,
            None => return QString::default(),
        };
        let packs_with_urls: Vec<serde_json::Value> = packs
            .into_iter()
            .map(|pack| {
                let stickers: Vec<serde_json::Value> = pack
                    .stickers
                    .into_iter()
                    .map(|s| {
                        let url = crate::client::sticker_cdn_url(&s.id, s.format_type);
                        serde_json::json!({
                            "id": s.id,
                            "name": s.name,
                            "format_type": s.format_type,
                            "url": url,
                        })
                    })
                    .collect();
                serde_json::json!({
                    "id": pack.id,
                    "name": pack.name,
                    "stickers": stickers,
                })
            })
            .collect();
        QString::from(
            serde_json::to_string(&packs_with_urls)
                .unwrap_or_else(|_| "[]".to_string())
                .as_str(),
        )
    }

    fn load_guild_emojis(&mut self, guild_id: QString) {
        self.send_action(UiAction::LoadGuildEmojis(guild_id.trimmed().to_string()));
    }

    fn consume_guild_emojis(&mut self) -> QString {
        let emojis = match self.pending_guild_emojis.take() {
            Some(e) => e,
            None => return QString::default(),
        };
        let arr: Vec<serde_json::Value> = emojis
            .into_iter()
            .map(|e| {
                let name = e.name.as_deref().unwrap_or("");
                let animated = e.animated.unwrap_or(false);
                let ext = if animated { "gif" } else { "png" };
                let url = format!("https://cdn.discordapp.com/emojis/{}.{}", e.id, ext);
                serde_json::json!({
                    "id": e.id,
                    "name": name,
                    "animated": animated,
                    "url": url,
                })
            })
            .collect();
        QString::from(serde_json::to_string(&arr).unwrap_or_default().as_str())
    }

    fn load_mullvad_servers(&mut self) {
        self.send_action(UiAction::GetMullvadServers);
    }

    fn consume_mullvad_servers(&mut self) -> QString {
        let json = if self.mullvad_servers_buf.is_empty() {
            String::new()
        } else {
            self.mullvad_servers_buf.remove(0)
        };
        QString::from(json.as_str())
    }

    fn set_proxy_settings(
        &mut self,
        enabled: bool,
        mode: QString,
        mullvad_country: QString,
        mullvad_city: QString,
        mullvad_server: QString,
        custom_host: QString,
        custom_port: i32,
    ) {
        let mode_s = mode.trimmed().to_string();
        let mullvad_country = if mullvad_country.trimmed().is_empty() {
            None
        } else {
            Some(mullvad_country.trimmed().to_string())
        };
        let mullvad_city = if mullvad_city.trimmed().is_empty() {
            None
        } else {
            Some(mullvad_city.trimmed().to_string())
        };
        let mullvad_server = if mullvad_server.trimmed().is_empty() {
            None
        } else {
            Some(mullvad_server.trimmed().to_string())
        };
        let custom_host = if custom_host.trimmed().is_empty() {
            None
        } else {
            Some(custom_host.trimmed().to_string())
        };
        let custom_port = if custom_port > 0 {
            Some(custom_port as u16)
        } else {
            None
        };

        // Persist to storage immediately so settings are saved even if user never logs in
        if let Ok(storage) = Storage::new() {
            let mut settings = storage.load_settings().unwrap_or_default();
            settings.proxy_settings.enabled = enabled;
            settings.proxy_settings.mode = if mode_s == "mullvad" {
                ProxyMode::Mullvad
            } else {
                ProxyMode::Custom
            };
            settings.proxy_settings.mullvad_country = mullvad_country.clone();
            settings.proxy_settings.mullvad_city = mullvad_city.clone();
            settings.proxy_settings.mullvad_server = mullvad_server.clone();
            if let (Some(host), Some(port)) = (custom_host.clone(), custom_port) {
                settings.proxy_settings.custom_host = host;
                settings.proxy_settings.custom_port = port;
            }
            let _ = storage.save_settings(&settings);
        }

        self.send_action(UiAction::SetProxySettings {
            enabled,
            mode: mode_s,
            mullvad_country,
            mullvad_city,
            mullvad_server,
            custom_host,
            custom_port,
        });
    }

    fn get_proxy_settings(&self) -> QString {
        let settings = Storage::new()
            .ok()
            .and_then(|s| s.load_settings().ok())
            .unwrap_or_default();
        let mode_str = match settings.proxy_settings.mode {
            ProxyMode::Mullvad => "mullvad",
            ProxyMode::Custom => "custom",
        };
        let json = serde_json::json!({
            "enabled": settings.proxy_settings.enabled,
            "mode": mode_str,
            "mullvad_country": settings.proxy_settings.mullvad_country,
            "mullvad_city": settings.proxy_settings.mullvad_city,
            "mullvad_server": settings.proxy_settings.mullvad_server,
            "custom_host": settings.proxy_settings.custom_host,
            "custom_port": settings.proxy_settings.custom_port,
        });
        QString::from(serde_json::to_string(&json).unwrap_or_default().as_str())
    }

    fn fetch_user_profile(&mut self, user_id: QString, guild_id: QString) {
        let uid = user_id.trimmed().to_string();
        if uid.is_empty() {
            return;
        }
        let gid = guild_id.trimmed().to_string();
        let guild_id_opt = if gid.is_empty() { None } else { Some(gid) };
        self.send_action(UiAction::FetchUserProfile {
            user_id: uid,
            guild_id: guild_id_opt,
        });
    }

    fn consume_user_profile(&mut self) -> QString {
        QString::from(
            self.pending_user_profile
                .take()
                .as_deref()
                .unwrap_or("")
                .to_string()
                .as_str(),
        )
    }

    fn consume_user_profile_raw(&mut self) -> QString {
        QString::from(
            self.pending_user_profile_raw
                .take()
                .as_deref()
                .unwrap_or("")
                .to_string()
                .as_str(),
        )
    }

    fn get_user_status(&self, user_id: QString) -> QString {
        let uid = user_id.trimmed().to_string();
        self.user_presence
            .get(&uid)
            .map(|s| QString::from(s.as_str()))
            .unwrap_or_default()
    }

    fn get_plugin_enabled_states(&self) -> QString {
        let settings = Storage::new()
            .ok()
            .and_then(|s| s.load_settings().ok())
            .unwrap_or_default();
        let map: serde_json::Map<String, serde_json::Value> = settings
            .plugin_enabled
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::json!(*v)))
            .collect();
        QString::from(
            serde_json::to_string(&serde_json::Value::Object(map)).unwrap_or_default().as_str(),
        )
    }

    fn is_plugin_enabled(&self, plugin_id: QString) -> bool {
        let settings = Storage::new()
            .ok()
            .and_then(|s| s.load_settings().ok())
            .unwrap_or_default();
        settings
            .plugin_enabled
            .get(plugin_id.trimmed().to_string().as_str())
            .copied()
            .unwrap_or(false)
    }

    fn get_plugin_list(&self) -> QString {
        let plugins = plugins::plugin_list_for_ui();
        let arr: Vec<serde_json::Value> = plugins
            .into_iter()
            .map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "name": m.name,
                    "description": m.description,
                })
            })
            .collect();
        QString::from(
            serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string()).as_str(),
        )
    }

    fn consume_plugins_refreshed(&mut self) -> bool {
        std::mem::take(&mut self.plugins_refreshed)
    }

    fn consume_plugin_updates(&mut self) -> QString {
        self.plugin_updates_buf
            .take()
            .map(QString::from)
            .unwrap_or_default()
    }

    fn consume_join_guild_result(&mut self) -> QString {
        match self.pending_join_guild_result.take() {
            None => QString::default(),
            Some(Ok(g)) => {
                let json = serde_json::json!({
                    "success": true,
                    "guild": {
                        "guildId": g.id,
                        "name": g.name,
                        "iconUrl": g.icon_url.unwrap_or_default(),
                        "hasUnread": g.has_unread,
                        "mentionCount": g.mention_count,
                    }
                });
                QString::from(serde_json::to_string(&json).unwrap_or_default().as_str())
            }
            Some(Err(e)) => {
                let json = serde_json::json!({
                    "success": false,
                    "error": e,
                });
                QString::from(serde_json::to_string(&json).unwrap_or_default().as_str())
            }
        }
    }

    fn consume_rpc_invite(&mut self) -> QString {
        if self.pending_rpc_invites.is_empty() {
            return QString::default();
        }
        let invite = self.pending_rpc_invites.remove(0);
        QString::from(invite.as_str())
    }

    fn get_deleted_message_style(&self) -> QString {
        let settings = Storage::new()
            .ok()
            .and_then(|s| s.load_settings().ok())
            .unwrap_or_default();
        let style = settings.client_settings.deleted_message_style.as_str();
        let valid = ["strikethrough", "faded", "deleted"].contains(&style);
        QString::from(if valid { style } else { "strikethrough" })
    }

    fn set_deleted_message_style(&mut self, style: QString) {
        let style_s = style.trimmed().to_string();
        let valid = ["strikethrough", "faded", "deleted"].contains(&style_s.as_str());
        let style_s = if valid { style_s } else { "strikethrough".to_string() };
        if let (Ok(storage), Ok(mut settings)) = (Storage::new(), Storage::new().and_then(|s| s.load_settings())) {
            settings.client_settings.deleted_message_style = style_s;
            let _ = storage.save_settings(&settings);
        }
    }

    fn consume_pins(&mut self) -> QString {
        let messages = match self.pending_pins.take() {
            Some(m) => m,
            None => return QString::default(),
        };
        let arr: Vec<serde_json::Value> = messages.into_iter().map(Self::message_to_json).collect();
        QString::from(serde_json::to_string(&arr).unwrap_or_default().as_str())
    }

    fn consume_reaction_updates(&mut self) -> QString {
        if self.pending_reaction_updates.is_empty() {
            return QString::default();
        }
        let updates: Vec<serde_json::Value> = self
            .pending_reaction_updates
            .drain(..)
            .map(|(channel_id, message_id, reactions)| {
                let reactions_json: Vec<serde_json::Value> = reactions
                    .into_iter()
                    .map(|r| {
                        serde_json::json!({
                            "emoji": r.emoji_display,
                            "count": r.count,
                            "me": r.me,
                        })
                    })
                    .collect();
                serde_json::json!({
                    "channelId": channel_id,
                    "messageId": message_id,
                    "reactions": reactions_json,
                })
            })
            .collect();
        QString::from(serde_json::to_string(&updates).unwrap_or_default().as_str())
    }

    fn consume_unread_updates(&mut self) -> QString {
        if self.pending_unread_updates.is_empty() {
            return QString::default();
        }
        let arr: Vec<serde_json::Value> = self
            .pending_unread_updates
            .drain(..)
            .map(|(channel_id, guild_id, has_unread, mention_count)| {
                serde_json::json!({
                    "channelId": channel_id,
                    "guildId": guild_id.unwrap_or_default(),
                    "hasUnread": has_unread,
                    "mentionCount": mention_count,
                })
            })
            .collect();
        QString::from(serde_json::to_string(&arr).unwrap_or_default().as_str())
    }

    fn consume_members(&mut self) -> QString {
        let (guild_id, members) = match self.pending_members.take() {
            Some(m) => m,
            None => return QString::default(),
        };
        let arr: Vec<serde_json::Value> = members
            .into_iter()
            .map(|m| {
                serde_json::json!({
                    "memberId": m.user_id,
                    "username": m.username,
                    "displayName": m.display_name.unwrap_or_default(),
                    "avatarUrl": m.avatar_url.unwrap_or_default(),
                    "roleName": m.role_name.unwrap_or_default(),
                    "roleColor": m.role_color.unwrap_or_default(),
                    "publicFlags": m.public_flags.unwrap_or(0),
                    "bot": m.bot.unwrap_or(false),
                    "premiumType": m.premium_type.unwrap_or(0),
                })
            })
            .collect();
        let out = serde_json::json!({ "guildId": guild_id, "members": arr });
        QString::from(serde_json::to_string(&out).unwrap_or_default().as_str())
    }

    fn consume_my_profile(&mut self) -> QString {
        let (guild_id, nick, roles) = match self.pending_my_profile.take() {
            Some(p) => p,
            None => return QString::default(),
        };
        let roles_arr: Vec<serde_json::Value> = roles
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "name": r.name,
                    "color": r.color,
                    "position": r.position,
                })
            })
            .collect();
        let out = serde_json::json!({
            "guildId": guild_id,
            "nick": nick.unwrap_or_default(),
            "roles": roles_arr,
        });
        QString::from(serde_json::to_string(&out).unwrap_or_default().as_str())
    }
}
