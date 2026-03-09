// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Mobile FFI layer
//!
//! Exposes the Discord client as a C-compatible API for use from
//! Kotlin/Java (Android) and Swift (iOS). This is the bridge between
//! the shared Rust core and native mobile UIs.
//!
//! The desktop build uses Qt/QML directly. Mobile builds use this FFI
//! layer instead, with native UI on each platform.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

use crate::app_runner::run_app_with_updates;
use crate::bridge::{LoginRequest, UiAction, UiUpdate};
use crate::features::FeatureFlags;
use crate::storage::Storage;

/// Event type constants for the UI update callback (must match include/skunkcord.h).
pub mod event_type {
    pub const LOGIN_SUCCESS: i32 = 1;
    pub const LOGIN_FAILED: i32 = 2;
    pub const GUILDS_LOADED: i32 = 3;
    pub const CHANNELS_LOADED: i32 = 4;
    pub const DM_CHANNELS_LOADED: i32 = 5;
    pub const MESSAGES_LOADED: i32 = 6;
    pub const NEW_MESSAGE: i32 = 7;
    pub const MESSAGE_DELETED: i32 = 8;
    pub const MESSAGE_EDITED: i32 = 9;
    pub const VOICE_STATE_CHANGED: i32 = 10;
    pub const USER_SPEAKING: i32 = 11;
    pub const UNREAD_UPDATE: i32 = 12;
    pub const CAPTCHA_REQUIRED: i32 = 13;
    pub const CONNECTED: i32 = 14;
    pub const DISCONNECTED: i32 = 15;
    pub const RECONNECTING: i32 = 16;
    pub const MORE_MESSAGES_LOADED: i32 = 17;
    pub const PINS_LOADED: i32 = 18;
    pub const TYPING_STARTED: i32 = 19;
    pub const MESSAGE_REACTIONS_UPDATED: i32 = 20;
    pub const GIFS_LOADED: i32 = 21;
    pub const STICKER_PACKS_LOADED: i32 = 22;
    pub const GUILD_EMOJIS_LOADED: i32 = 23;
    pub const MEMBERS_LOADED: i32 = 24;
    pub const VOICE_PARTICIPANTS_CHANGED: i32 = 25;
    pub const VOICE_CONNECTION_PROGRESS: i32 = 26;
    pub const VOICE_SPEAKING_CHANGED: i32 = 27;
    pub const VOICE_STATS: i32 = 28;
    pub const ERROR: i32 = 29;
    pub const MY_GUILD_PROFILE: i32 = 30;
    pub const MULLVAD_SERVERS_LOADED: i32 = 31;
    pub const USER_PROFILE_LOADED: i32 = 32;
    pub const PRESENCE_UPDATED: i32 = 33;
    pub const MFA_REQUIRED: i32 = 34;
    pub const PLUGIN_UI_UPDATED: i32 = 35;
    pub const PLUGIN_UI_REMOVED: i32 = 36;
    pub const PLUGINS_REFRESHED: i32 = 37;
    pub const PLUGIN_UPDATES_AVAILABLE: i32 = 38;
    pub const JOIN_GUILD_SUCCESS: i32 = 39;
    pub const JOIN_GUILD_FAILED: i32 = 40;
    pub const RPC_INVITE_RECEIVED: i32 = 41;
    pub const RELATIONSHIPS_UPDATE: i32 = 42; // RelationshipsLoaded, RelationshipAdded, or RelationshipRemoved
    pub const GUILD_MUTE_STATE_CHANGED: i32 = 43;
    pub const ACCOUNTS_LIST: i32 = 44;
}

/// Global tokio runtime for FFI
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// Sender for login request (set once in skunkcord_init)
static LOGIN_TX: OnceLock<Mutex<Option<mpsc::Sender<LoginRequest>>>> = OnceLock::new();

/// Sender for UI actions (set once in skunkcord_init)
static ACTION_TX: OnceLock<Mutex<Option<tokio::sync::mpsc::UnboundedSender<UiAction>>>> = OnceLock::new();

/// Callback for UI updates (set by native code via skunkcord_set_update_callback)
static UPDATE_CALLBACK: OnceLock<Mutex<Option<UiUpdateCallback>>> = OnceLock::new();

type UiUpdateCallback = extern "C" fn(event_type: i32, json: *const c_char);

fn get_runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"))
}

/// Helper: convert C string to Rust string
unsafe fn c_str_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

/// Helper: convert Rust string to C string (caller must free with skunkcord_free_string)
fn string_to_c(s: String) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

fn event_type_and_json(update: &UiUpdate) -> Option<(i32, String)> {
    let json = serde_json::to_string(update).ok()?;
    let ty = match update {
        UiUpdate::LoginSuccess { .. } => event_type::LOGIN_SUCCESS,
        UiUpdate::LoginFailed(_) => event_type::LOGIN_FAILED,
        UiUpdate::GuildsLoaded(_) => event_type::GUILDS_LOADED,
        UiUpdate::ChannelsLoaded(_) => event_type::CHANNELS_LOADED,
        UiUpdate::DmChannelsLoaded(_) => event_type::DM_CHANNELS_LOADED,
        UiUpdate::MessagesLoaded(_) => event_type::MESSAGES_LOADED,
        UiUpdate::NewMessage(_) => event_type::NEW_MESSAGE,
        UiUpdate::MessageDeleted { .. } | UiUpdate::MessageDeletedWithContent { .. } => {
            event_type::MESSAGE_DELETED
        }
        UiUpdate::MessageEdited { .. } => event_type::MESSAGE_EDITED,
        UiUpdate::VoiceStateChanged(_) => event_type::VOICE_STATE_CHANGED,
        UiUpdate::UserSpeaking { .. } => event_type::USER_SPEAKING,
        UiUpdate::UnreadUpdate { .. } => event_type::UNREAD_UPDATE,
        UiUpdate::CaptchaRequired { .. } => event_type::CAPTCHA_REQUIRED,
        UiUpdate::MfaRequired { .. } => event_type::MFA_REQUIRED,
        UiUpdate::Connected => event_type::CONNECTED,
        UiUpdate::Disconnected => event_type::DISCONNECTED,
        UiUpdate::Reconnecting => event_type::RECONNECTING,
        UiUpdate::MoreMessagesLoaded { .. } => event_type::MORE_MESSAGES_LOADED,
        UiUpdate::PinsLoaded { .. } => event_type::PINS_LOADED,
        UiUpdate::TypingStarted { .. } => event_type::TYPING_STARTED,
        UiUpdate::MessageReactionsUpdated { .. } => event_type::MESSAGE_REACTIONS_UPDATED,
        UiUpdate::GifsLoaded(_) => event_type::GIFS_LOADED,
        UiUpdate::StickerPacksLoaded(_) => event_type::STICKER_PACKS_LOADED,
        UiUpdate::GuildEmojisLoaded(_) => event_type::GUILD_EMOJIS_LOADED,
        UiUpdate::MembersLoaded { .. } => event_type::MEMBERS_LOADED,
        UiUpdate::VoiceParticipantsChanged { .. } => event_type::VOICE_PARTICIPANTS_CHANGED,
        UiUpdate::VoiceConnectionProgress(_) => event_type::VOICE_CONNECTION_PROGRESS,
        UiUpdate::VoiceSpeakingChanged { .. } => event_type::VOICE_SPEAKING_CHANGED,
        UiUpdate::VoiceStats { .. } => event_type::VOICE_STATS,
        UiUpdate::Error(_) => event_type::ERROR,
        UiUpdate::MyGuildProfile { .. } => event_type::MY_GUILD_PROFILE,
        UiUpdate::MullvadServersLoaded(_) => event_type::MULLVAD_SERVERS_LOADED,
        UiUpdate::UserProfileLoaded { .. } => event_type::USER_PROFILE_LOADED,
        UiUpdate::PresenceUpdated { .. } => event_type::PRESENCE_UPDATED,
        UiUpdate::PluginUiUpdated { .. } => event_type::PLUGIN_UI_UPDATED,
        UiUpdate::PluginUiRemoved { .. } => event_type::PLUGIN_UI_REMOVED,
        UiUpdate::PluginsRefreshed => event_type::PLUGINS_REFRESHED,
        UiUpdate::PluginUpdatesAvailable(_) => event_type::PLUGIN_UPDATES_AVAILABLE,
        UiUpdate::JoinGuildSuccess(_) => event_type::JOIN_GUILD_SUCCESS,
        UiUpdate::JoinGuildFailed(_) => event_type::JOIN_GUILD_FAILED,
        UiUpdate::RpcInviteReceived(_) => event_type::RPC_INVITE_RECEIVED,
        UiUpdate::RelationshipsLoaded(_) | UiUpdate::RelationshipAdded(_) | UiUpdate::RelationshipRemoved { .. } => event_type::RELATIONSHIPS_UPDATE,
        UiUpdate::GuildMuteStateChanged { .. } => event_type::GUILD_MUTE_STATE_CHANGED,
        UiUpdate::AccountsList(_) => event_type::ACCOUNTS_LIST,
    };
    Some((ty, json))
}

// ==================== Initialization ====================

/// Initialize the Discord client library. Call once at app startup.
/// Starts the backend worker and update forwarder threads.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn skunkcord_init() -> i32 {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("skunkcord=info")
        .try_init();

    let _ = get_runtime();

    let (login_tx, login_rx) = mpsc::channel::<LoginRequest>();
    let (update_tx, update_rx) = mpsc::channel::<UiUpdate>();
    let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel::<UiAction>();

    let _ = LOGIN_TX.set(Mutex::new(Some(login_tx)));
    let _ = ACTION_TX.set(Mutex::new(Some(action_tx)));
    let _ = UPDATE_CALLBACK.set(Mutex::new(None));

    // Thread: receive UI updates and call C callback
    thread::spawn(move || {
        while let Ok(update) = update_rx.recv() {
            if let Some((ty, json)) = event_type_and_json(&update) {
                if let Ok(cstr) = CString::new(json) {
                    if let Some(cb) = UPDATE_CALLBACK.get().and_then(|m| m.lock().ok()).and_then(|g| *g) {
                        cb(ty, cstr.as_ptr());
                    }
                }
            }
        }
    });

    // Thread: run tokio backend loop (login -> run_app_with_updates)
    thread::spawn(move || {
        let storage = match Storage::new() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Storage::new failed: {}", e);
                return;
            }
        };
        let settings = storage.load_settings().unwrap_or_default();
        let feature_flags = settings.feature_flags.clone();

        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => {
                let _ = update_tx.send(UiUpdate::LoginFailed(e.to_string()));
                return;
            }
        };

        rt.block_on(async {
            let action_rx = Arc::new(tokio::sync::Mutex::new(action_rx));
            loop {
                let token = match login_rx.recv() {
                    Ok(LoginRequest::Token(t)) => t,
                    Ok(_) => continue,
                    Err(_) => break,
                };
                if let Err(e) = run_app_with_updates(
                    token,
                    storage.clone(),
                    settings.clone(),
                    feature_flags.clone(),
                    update_tx.clone(),
                    Arc::clone(&action_rx),
                )
                .await
                {
                    let _ = update_tx.send(UiUpdate::LoginFailed(e.to_string()));
                }
            }
        });
    });

    0
}

/// Register the callback for UI updates. The callback is invoked with (event_type, json).
/// The json pointer is valid only for the duration of the callback.
#[no_mangle]
pub extern "C" fn skunkcord_set_update_callback(cb: UiUpdateCallback) {
    if let Some(m) = UPDATE_CALLBACK.get() {
        if let Ok(mut g) = m.lock() {
            *g = Some(cb);
        }
    }
}

/// Free a string returned by the library
#[no_mangle]
pub unsafe extern "C" fn skunkcord_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

/// Get the library version
#[no_mangle]
pub extern "C" fn skunkcord_version() -> *mut c_char {
    string_to_c(env!("CARGO_PKG_VERSION").to_string())
}

// ==================== Login / session ====================

/// Send a login token to the backend. The backend will validate and connect.
/// Returns 0 on success, -1 if init not done or send failed.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_login(token: *const c_char) -> i32 {
    let t = c_str_to_string(token);
    if let Some(m) = LOGIN_TX.get() {
        if let Ok(g) = m.lock() {
            if let Some(ref tx) = *g {
                return if tx.send(LoginRequest::Token(t)).is_ok() { 0 } else { -1 };
            }
        }
    }
    -1
}

/// Send logout action (disconnects gateway and clears session from backend).
#[no_mangle]
pub extern "C" fn skunkcord_logout() -> i32 {
    send_action(UiAction::Logout)
}

// ==================== Navigation ====================

/// Select a guild (empty string = DMs / Home).
#[no_mangle]
pub unsafe extern "C" fn skunkcord_select_guild(guild_id: *const c_char) -> i32 {
    send_action(UiAction::SelectGuild(c_str_to_string(guild_id)))
}

/// Select a channel (triggers message load). channel_type: 0 = guild text, 1 = DM, etc.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_select_channel(channel_id: *const c_char, channel_type: u8) -> i32 {
    send_action(UiAction::SelectChannel(c_str_to_string(channel_id), channel_type))
}

// ==================== Messaging ====================

/// Send a message. silent: 1 = true, 0 = false.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_send_message(
    channel_id: *const c_char,
    content: *const c_char,
    silent: i32,
) -> i32 {
    send_action(UiAction::SendMessage {
        channel_id: c_str_to_string(channel_id),
        content: c_str_to_string(content),
        silent: silent != 0,
    })
}

/// Start typing in a channel.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_start_typing(channel_id: *const c_char) -> i32 {
    send_action(UiAction::StartTyping(c_str_to_string(channel_id)))
}

/// Edit a message.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_edit_message(
    channel_id: *const c_char,
    message_id: *const c_char,
    content: *const c_char,
) -> i32 {
    send_action(UiAction::EditMessage {
        channel_id: c_str_to_string(channel_id),
        message_id: c_str_to_string(message_id),
        content: c_str_to_string(content),
    })
}

/// Delete a message.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_delete_message(
    channel_id: *const c_char,
    message_id: *const c_char,
) -> i32 {
    send_action(UiAction::DeleteMessage {
        channel_id: c_str_to_string(channel_id),
        message_id: c_str_to_string(message_id),
    })
}

/// Pin a message.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_pin_message(
    channel_id: *const c_char,
    message_id: *const c_char,
) -> i32 {
    send_action(UiAction::PinMessage {
        channel_id: c_str_to_string(channel_id),
        message_id: c_str_to_string(message_id),
    })
}

/// Unpin a message.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_unpin_message(
    channel_id: *const c_char,
    message_id: *const c_char,
) -> i32 {
    send_action(UiAction::UnpinMessage {
        channel_id: c_str_to_string(channel_id),
        message_id: c_str_to_string(message_id),
    })
}

/// Open pinned messages for a channel.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_open_pins(channel_id: *const c_char) -> i32 {
    send_action(UiAction::OpenPins(c_str_to_string(channel_id)))
}

/// Add a reaction. emoji: unicode or ":name:id".
#[no_mangle]
pub unsafe extern "C" fn skunkcord_add_reaction(
    channel_id: *const c_char,
    message_id: *const c_char,
    emoji: *const c_char,
) -> i32 {
    send_action(UiAction::AddReaction {
        channel_id: c_str_to_string(channel_id),
        message_id: c_str_to_string(message_id),
        emoji: c_str_to_string(emoji),
    })
}

/// Remove own reaction.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_remove_reaction(
    channel_id: *const c_char,
    message_id: *const c_char,
    emoji: *const c_char,
) -> i32 {
    send_action(UiAction::RemoveReaction {
        channel_id: c_str_to_string(channel_id),
        message_id: c_str_to_string(message_id),
        emoji: c_str_to_string(emoji),
    })
}

// ==================== Voice ====================

/// Join a voice channel. guild_id can be null/empty for group DMs.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_join_voice(guild_id: *const c_char, channel_id: *const c_char) -> i32 {
    let g = c_str_to_string(guild_id);
    send_action(UiAction::JoinVoice {
        guild_id: if g.is_empty() { None } else { Some(g) },
        channel_id: c_str_to_string(channel_id),
    })
}

#[no_mangle]
pub extern "C" fn skunkcord_leave_voice() -> i32 {
    send_action(UiAction::LeaveVoice)
}

#[no_mangle]
pub extern "C" fn skunkcord_toggle_mute() -> i32 {
    send_action(UiAction::ToggleMute)
}

#[no_mangle]
pub extern "C" fn skunkcord_toggle_deafen() -> i32 {
    send_action(UiAction::ToggleDeafen)
}

#[no_mangle]
pub extern "C" fn skunkcord_toggle_fake_mute() -> i32 {
    send_action(UiAction::ToggleFakeMute)
}

#[no_mangle]
pub extern "C" fn skunkcord_toggle_fake_deafen() -> i32 {
    send_action(UiAction::ToggleFakeDeafen)
}

// ==================== Profile / status ====================

#[no_mangle]
pub unsafe extern "C" fn skunkcord_set_feature_profile(_profile: *const c_char) -> i32 {
    // No-op: feature profiles removed
    0
}

#[no_mangle]
pub unsafe extern "C" fn skunkcord_set_status(status: *const c_char) -> i32 {
    send_action(UiAction::SetStatus(c_str_to_string(status)))
}

#[no_mangle]
pub unsafe extern "C" fn skunkcord_set_custom_status(text: *const c_char) -> i32 {
    let s = c_str_to_string(text);
    send_action(UiAction::SetCustomStatus(if s.is_empty() { None } else { Some(s) }))
}

// ==================== Other actions ====================

#[no_mangle]
pub unsafe extern "C" fn skunkcord_switch_account(account_id: *const c_char) -> i32 {
    send_action(UiAction::SwitchAccount(c_str_to_string(account_id)))
}

#[no_mangle]
pub extern "C" fn skunkcord_mark_all_read() -> i32 {
    send_action(UiAction::MarkAllRead)
}

#[no_mangle]
pub unsafe extern "C" fn skunkcord_captcha_solved(token: *const c_char) -> i32 {
    send_action(UiAction::CaptchaSolved(c_str_to_string(token)))
}

/// Submit MFA TOTP code. ticket comes from MFA_REQUIRED event JSON.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_submit_mfa_code(ticket: *const c_char, code: *const c_char) -> i32 {
    let t = c_str_to_string(ticket);
    let c = c_str_to_string(code);
    if let Some(m) = LOGIN_TX.get() {
        if let Ok(g) = m.lock() {
            if let Some(ref tx) = *g {
                return if tx
                    .send(LoginRequest::MfaCode {
                        ticket: t,
                        code: c,
                        login_instance_id: None,
                    })
                    .is_ok()
                {
                    0
                } else {
                    -1
                };
            }
        }
    }
    -1
}

/// Fetch user profile (for profile popup). guild_id can be null/empty.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_fetch_user_profile(
    user_id: *const c_char,
    guild_id: *const c_char,
) -> i32 {
    let uid = c_str_to_string(user_id);
    let gid = c_str_to_string(guild_id);
    send_action(UiAction::FetchUserProfile {
        user_id: uid,
        guild_id: if gid.is_empty() { None } else { Some(gid) },
    })
}

// ==================== Plugins ====================

/// Set plugin enabled/disabled.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_set_plugin_enabled(
    plugin_id: *const c_char,
    enabled: i32,
) -> i32 {
    send_action(UiAction::SetPluginEnabled {
        plugin_id: c_str_to_string(plugin_id),
        enabled: enabled != 0,
    })
}

/// Notify backend that a plugin toolbar button was clicked.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_plugin_button_clicked(
    plugin_id: *const c_char,
    button_id: *const c_char,
) -> i32 {
    send_action(UiAction::PluginButtonClicked {
        plugin_id: c_str_to_string(plugin_id),
        button_id: c_str_to_string(button_id),
    })
}

/// Submit a plugin modal form.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_plugin_modal_submitted(
    plugin_id: *const c_char,
    modal_id: *const c_char,
    fields_json: *const c_char,
) -> i32 {
    let fields: std::collections::HashMap<String, String> =
        serde_json::from_str(&c_str_to_string(fields_json)).unwrap_or_default();
    send_action(UiAction::PluginModalSubmitted {
        plugin_id: c_str_to_string(plugin_id),
        modal_id: c_str_to_string(modal_id),
        fields,
    })
}

/// Set deleted message display style. Persists to storage directly.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_set_deleted_message_style(style: *const c_char) -> i32 {
    let style_s = c_str_to_string(style);
    let valid = ["strikethrough", "faded", "deleted"].contains(&style_s.as_str());
    let style_s = if valid { style_s } else { "strikethrough".to_string() };
    if let (Ok(storage), Ok(mut settings)) =
        (Storage::new(), Storage::new().and_then(|s| s.load_settings()))
    {
        settings.client_settings.deleted_message_style = style_s;
        let _ = storage.save_settings(&settings);
        0
    } else {
        -1
    }
}

// ==================== Synchronous getters ====================

/// Get deleted message display style (caller must free with skunkcord_free_string).
#[no_mangle]
pub extern "C" fn skunkcord_get_deleted_message_style() -> *mut c_char {
    let style = Storage::new()
        .ok()
        .and_then(|s| s.load_settings().ok())
        .map(|s| {
            let st = s.client_settings.deleted_message_style;
            if ["strikethrough", "faded", "deleted"].contains(&st.as_str()) {
                st
            } else {
                "strikethrough".to_string()
            }
        })
        .unwrap_or_else(|| "strikethrough".to_string());
    string_to_c(style)
}

/// Check if a plugin is enabled. Returns 1 (true) or 0 (false).
#[no_mangle]
pub unsafe extern "C" fn skunkcord_is_plugin_enabled(plugin_id: *const c_char) -> i32 {
    let pid = c_str_to_string(plugin_id);
    let enabled = Storage::new()
        .ok()
        .and_then(|s| s.load_settings().ok())
        .and_then(|s| s.plugin_enabled.get(&pid).copied())
        .unwrap_or(false);
    if enabled { 1 } else { 0 }
}

/// Get plugin list as JSON array (caller must free with skunkcord_free_string).
#[no_mangle]
pub extern "C" fn skunkcord_get_plugin_list() -> *mut c_char {
    let plugins = crate::plugins::plugin_list_for_ui();
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
    string_to_c(serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string()))
}

/// Get plugin enabled states as JSON object (caller must free with skunkcord_free_string).
#[no_mangle]
pub extern "C" fn skunkcord_get_plugin_enabled_states() -> *mut c_char {
    let settings = Storage::new()
        .ok()
        .and_then(|s| s.load_settings().ok())
        .unwrap_or_default();
    let map: serde_json::Map<String, serde_json::Value> = settings
        .plugin_enabled
        .iter()
        .map(|(k, v)| (k.clone(), serde_json::json!(*v)))
        .collect();
    string_to_c(
        serde_json::to_string(&serde_json::Value::Object(map)).unwrap_or_else(|_| "{}".to_string()),
    )
}

/// Open a DM with a user.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_open_dm(recipient_id: *const c_char) -> i32 {
    send_action(UiAction::OpenDm(c_str_to_string(recipient_id)))
}

/// Load more (older) messages for pagination.
#[no_mangle]
pub unsafe extern "C" fn skunkcord_load_more_messages(
    channel_id: *const c_char,
    before_message_id: *const c_char,
) -> i32 {
    send_action(UiAction::LoadMoreMessages {
        channel_id: c_str_to_string(channel_id),
        before_message_id: c_str_to_string(before_message_id),
    })
}

#[no_mangle]
pub unsafe extern "C" fn skunkcord_search_gifs(query: *const c_char) -> i32 {
    send_action(UiAction::SearchGifs(c_str_to_string(query)))
}

#[no_mangle]
pub extern "C" fn skunkcord_load_sticker_packs() -> i32 {
    send_action(UiAction::LoadStickerPacks)
}

#[no_mangle]
pub unsafe extern "C" fn skunkcord_load_guild_emojis(guild_id: *const c_char) -> i32 {
    send_action(UiAction::LoadGuildEmojis(c_str_to_string(guild_id)))
}

#[no_mangle]
pub extern "C" fn skunkcord_get_mullvad_servers() -> i32 {
    send_action(UiAction::GetMullvadServers)
}

fn send_action(action: UiAction) -> i32 {
    if let Some(m) = ACTION_TX.get() {
        if let Ok(g) = m.lock() {
            if let Some(ref tx) = *g {
                return if tx.send(action).is_ok() { 0 } else { -1 };
            }
        }
    }
    -1
}

// ==================== Platform Detection ====================

/// Get mobile super properties JSON for Android
#[no_mangle]
pub extern "C" fn skunkcord_android_super_properties() -> *mut c_char {
    let props = crate::fingerprint::super_properties::MobileSuperProperties::android();
    let json = serde_json::to_string(&props).unwrap_or_default();
    string_to_c(json)
}

/// Get mobile super properties JSON for iOS
#[no_mangle]
pub extern "C" fn skunkcord_ios_super_properties() -> *mut c_char {
    let props = crate::fingerprint::super_properties::MobileSuperProperties::ios();
    let json = serde_json::to_string(&props).unwrap_or_default();
    string_to_c(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert_eq!(skunkcord_init(), 0);
    }

    #[test]
    fn test_version() {
        let ver = skunkcord_version();
        unsafe {
            let s = CStr::from_ptr(ver).to_string_lossy();
            assert!(!s.is_empty());
            skunkcord_free_string(ver);
        }
    }

    #[test]
    fn test_android_props() {
        let props = skunkcord_android_super_properties();
        unsafe {
            let s = CStr::from_ptr(props).to_string_lossy();
            assert!(s.contains("Android"));
            skunkcord_free_string(props);
        }
    }

    #[test]
    fn test_ios_props() {
        let props = skunkcord_ios_super_properties();
        unsafe {
            let s = CStr::from_ptr(props).to_string_lossy();
            assert!(s.contains("iOS"));
            skunkcord_free_string(props);
        }
    }
}
