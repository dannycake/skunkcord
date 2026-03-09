// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! UI Test Runner
//!
//! Run only the Qt UI with mock data for testing the interface
//! without connecting to Discord's servers. Uses main.qml (or mobile.qml)
//! and AppController driven by a mock worker that sends UiUpdates
//! (including profile and presence).
//!
//! Usage:
//!   cargo run --bin ui_test           # desktop UI (1100x700)
//!   cargo run --bin ui_test -- --mobile  # mobile UI (390x844)

use skunkcord::bridge::{ChannelInfo, GuildInfo, LoginRequest, MemberInfo, MessageInfo, ReactionInfo, UiAction, UiUpdate};
use skunkcord::ui::AppController;

use qmetaobject::prelude::*;
use qmetaobject::QObjectPinned;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

/// Mock backend that provides test data
struct MockBackend {
    guilds: Vec<GuildInfo>,
    channels: Vec<ChannelInfo>,
    messages: Vec<MessageInfo>,
    members: Vec<MemberInfo>,
}

impl MockBackend {
    fn new() -> Self {
        Self {
            guilds: vec![
                GuildInfo {
                    id: "guild_001".to_string(),
                    name: "Test Server".to_string(),
                    icon_url: None,
                    has_unread: true,
                    mention_count: 3,
                },
                GuildInfo {
                    id: "guild_002".to_string(),
                    name: "Development".to_string(),
                    icon_url: None,
                    has_unread: false,
                    mention_count: 0,
                },
                GuildInfo {
                    id: "guild_003".to_string(),
                    name: "Gaming".to_string(),
                    icon_url: None,
                    has_unread: true,
                    mention_count: 12,
                },
            ],
            channels: vec![
                // Uncategorized
                ChannelInfo {
                    id: "ch_002".to_string(),
                    guild_id: Some("guild_001".to_string()),
                    name: "announcements".to_string(),
                    channel_type: 0,
                    position: 0,
                    parent_id: None,
                    has_unread: false,
                    mention_count: 0,
                    is_hidden: false,
                },
                // Category: TEXT CHANNELS
                ChannelInfo {
                    id: "cat_text".to_string(),
                    guild_id: Some("guild_001".to_string()),
                    name: "TEXT CHANNELS".to_string(),
                    channel_type: 4,
                    position: 0,
                    parent_id: None,
                    has_unread: false,
                    mention_count: 0,
                    is_hidden: false,
                },
                ChannelInfo {
                    id: "ch_001".to_string(),
                    guild_id: Some("guild_001".to_string()),
                    name: "general".to_string(),
                    channel_type: 0,
                    position: 0,
                    parent_id: Some("cat_text".to_string()),
                    has_unread: true,
                    mention_count: 3,
                    is_hidden: false,
                },
                ChannelInfo {
                    id: "ch_004".to_string(),
                    guild_id: Some("guild_001".to_string()),
                    name: "dev-talk".to_string(),
                    channel_type: 0,
                    position: 1,
                    parent_id: Some("cat_text".to_string()),
                    has_unread: true,
                    mention_count: 0,
                    is_hidden: false,
                },
                ChannelInfo {
                    id: "ch_005".to_string(),
                    guild_id: Some("guild_001".to_string()),
                    name: "off-topic".to_string(),
                    channel_type: 0,
                    position: 2,
                    parent_id: Some("cat_text".to_string()),
                    has_unread: false,
                    mention_count: 0,
                    is_hidden: false,
                },
                ChannelInfo {
                    id: "ch_hidden".to_string(),
                    guild_id: Some("guild_001".to_string()),
                    name: "secret".to_string(),
                    channel_type: 0,
                    position: 3,
                    parent_id: Some("cat_text".to_string()),
                    has_unread: false,
                    mention_count: 0,
                    is_hidden: true,
                },
                // Category: VOICE CHANNELS
                ChannelInfo {
                    id: "cat_voice".to_string(),
                    guild_id: Some("guild_001".to_string()),
                    name: "VOICE CHANNELS".to_string(),
                    channel_type: 4,
                    position: 1,
                    parent_id: None,
                    has_unread: false,
                    mention_count: 0,
                    is_hidden: false,
                },
                ChannelInfo {
                    id: "ch_003".to_string(),
                    guild_id: Some("guild_001".to_string()),
                    name: "voice-chat".to_string(),
                    channel_type: 2,
                    position: 0,
                    parent_id: Some("cat_voice".to_string()),
                    has_unread: false,
                    mention_count: 0,
                    is_hidden: false,
                },
            ],
            messages: vec![
                MessageInfo {
                    id: "msg_001".to_string(),
                    channel_id: "ch_001".to_string(),
                    author_name: "Alice".to_string(),
                    author_id: "user_001".to_string(),
                    author_avatar_url: None,
                    content: "Hey everyone! Welcome to the test server 👋".to_string(),
                    timestamp: "Today at 10:30 AM".to_string(),
                    is_deleted: false,
                    edit_count: 0,
                    message_type: 0, reply_author_name: None, reply_content: None,
                    reply_author_id: None, reply_author_role_color: None, mentions_me: false, mention_everyone: false,
                    author_role_color: Some("#5865f2".to_string()),
                    author_role_name: Some("Moderator".to_string()),
                    author_public_flags: 0,
                    author_bot: false,
                    author_premium_type: 1, // Nitro
                    attachments_json: "[]".to_string(),
                    stickers_json: "[]".to_string(),
                    embeds_json: "[]".to_string(),
                    content_html: "".to_string(),
                    reactions: vec![
                        ReactionInfo { emoji_display: "👍".to_string(), count: 2, me: true },
                        ReactionInfo { emoji_display: "❤".to_string(), count: 1, me: false },
                    ],
                },
                MessageInfo {
                    id: "msg_002".to_string(),
                    channel_id: "ch_001".to_string(),
                    author_name: "Bob".to_string(),
                    author_id: "user_002".to_string(),
                    author_avatar_url: None,
                    content: "Thanks! This UI looks great.".to_string(),
                    timestamp: "Today at 10:32 AM".to_string(),
                    is_deleted: false,
                    edit_count: 0,
                    // Reply to Alice's message
                    message_type: 19, reply_author_name: Some("Alice".to_string()),
                    reply_content: Some("Hey everyone! Welcome to the test server 👋".to_string()),
                    reply_author_id: Some("user_001".to_string()),
                    reply_author_role_color: Some("#5865f2".to_string()), // Alice's role color
                    mentions_me: false, mention_everyone: false,
                    author_role_color: None,
                    author_role_name: None,
                    author_public_flags: 0,
                    author_bot: false,
                    author_premium_type: 0,
                    attachments_json: "[]".to_string(),
                    stickers_json: "[]".to_string(),
                    embeds_json: "[]".to_string(),
                    content_html: "".to_string(),
                    reactions: vec![],
                },
                MessageInfo {
                    id: "msg_003".to_string(),
                    channel_id: "ch_001".to_string(),
                    author_name: "Charlie".to_string(),
                    author_id: "user_003".to_string(),
                    author_avatar_url: None,
                    content: "Testing **bold** and *italic* markdown rendering here!".to_string(),
                    timestamp: "Today at 10:35 AM".to_string(),
                    is_deleted: false,
                    edit_count: 1,
                    message_type: 0, reply_author_name: None, reply_content: None,
                    reply_author_id: None, reply_author_role_color: None, mentions_me: true, mention_everyone: false,
                    author_role_color: None,
                    author_role_name: None,
                    author_public_flags: 4, // HypeSquad Events
                    author_bot: false,
                    author_premium_type: 0,
                    attachments_json: "[]".to_string(),
                    stickers_json: "[]".to_string(),
                    embeds_json: "[]".to_string(),
                    content_html: "".to_string(),
                    reactions: vec![
                        ReactionInfo { emoji_display: "😂".to_string(), count: 1, me: false },
                    ],
                },
                MessageInfo {
                    id: "msg_004".to_string(),
                    channel_id: "ch_001".to_string(),
                    author_name: "DeletedUser".to_string(),
                    author_id: "user_004".to_string(),
                    author_avatar_url: None,
                    content: "[Message Deleted] This was logged by message logger".to_string(),
                    timestamp: "Today at 10:40 AM".to_string(),
                    is_deleted: true,
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
                    content_html: "".to_string(),
                    reactions: vec![],
                },
                MessageInfo {
                    id: "msg_005".to_string(),
                    channel_id: "ch_001".to_string(),
                    author_name: "Alice".to_string(),
                    author_id: "user_001".to_string(),
                    author_avatar_url: None,
                    content: "Here's a code block:\n```rust\nfn main() {\n    println!(\"Hello, Discord!\");\n}\n```".to_string(),
                    timestamp: "Today at 10:45 AM".to_string(),
                    is_deleted: false,
                    edit_count: 0,
                    // System message type (member join)
                    message_type: 7, reply_author_name: None, reply_content: None,
                    reply_author_id: None, reply_author_role_color: None, mentions_me: false, mention_everyone: false,
                    author_role_color: None,
                    author_role_name: None,
                    author_public_flags: 0,
                    author_bot: false,
                    author_premium_type: 0,
                    attachments_json: "[]".to_string(),
                    stickers_json: "[]".to_string(),
                    embeds_json: "[]".to_string(),
                    content_html: "".to_string(),
                    reactions: vec![],
                },
                MessageInfo {
                    id: "msg_006".to_string(),
                    channel_id: "ch_001".to_string(),
                    author_name: "TestBot".to_string(),
                    author_id: "user_005".to_string(),
                    author_avatar_url: None,
                    content: "🎉 Event starting soon! Check out ||spoiler text|| for details.".to_string(),
                    timestamp: "Today at 11:00 AM".to_string(),
                    is_deleted: false,
                    edit_count: 0,
                    message_type: 0, reply_author_name: None, reply_content: None,
                    reply_author_id: None, reply_author_role_color: None, mentions_me: false, mention_everyone: true,
                    author_role_color: Some("#57f287".to_string()),
                    author_role_name: Some("Bot".to_string()),
                    author_public_flags: 0,
                    author_bot: true,
                    author_premium_type: 0,
                    attachments_json: "[]".to_string(),
                    stickers_json: "[]".to_string(),
                    embeds_json: "[]".to_string(),
                    content_html: "".to_string(),
                    reactions: vec![],
                },
                MessageInfo {
                    id: "msg_007".to_string(),
                    channel_id: "ch_001".to_string(),
                    author_name: "EmbedBot".to_string(),
                    author_id: "user_006".to_string(),
                    author_avatar_url: None,
                    content: "Link preview and rich embed test:".to_string(),
                    timestamp: "Today at 11:05 AM".to_string(),
                    is_deleted: false,
                    edit_count: 0,
                    message_type: 0,
                    reply_author_name: None,
                    reply_content: None,
                    reply_author_id: None,
                    reply_author_role_color: None,
                    mentions_me: false,
                    mention_everyone: false,
                    author_role_color: Some("#eb459e".to_string()),
                    author_role_name: Some("Bot".to_string()),
                    author_public_flags: 0,
                    author_bot: true,
                    author_premium_type: 0,
                    attachments_json: "[]".to_string(),
                    stickers_json: "[]".to_string(),
                    embeds_json: r#"[{"title":"Example Embed","description":"This is a rich embed with title, description, thumbnail, image, and fields.","url":"https://discord.com","color":5814783,"author":{"name":"Embed Author","icon_url":"https://cdn.discordapp.com/embed/avatars/0.png"},"thumbnail":{"url":"https://cdn.discordapp.com/embed/avatars/1.png","width":80,"height":80},"image":{"url":"https://cdn.discordapp.com/embed/avatars/2.png","width":400,"height":200},"fields":[{"name":"Field 1","value":"Inline value","inline":true},{"name":"Field 2","value":"Another inline","inline":true},{"name":"Full-width field","value":"This field spans the full width.","inline":false}],"footer":{"text":"Embed footer • example.com"}}]"#.to_string(),
                    content_html: "".to_string(),
                    reactions: vec![],
                },
            ],
            members: vec![
                MemberInfo {
                    user_id: "user_001".to_string(),
                    username: "alice".to_string(),
                    display_name: Some("Alice".to_string()),
                    avatar_url: None,
                    role_name: Some("Moderator".to_string()),
                    role_color: Some("#5865f2".to_string()),
                    public_flags: None,
                    bot: None,
                    premium_type: None,
                },
                MemberInfo {
                    user_id: "user_002".to_string(),
                    username: "bob".to_string(),
                    display_name: Some("Bob".to_string()),
                    avatar_url: None,
                    role_name: None,
                    role_color: None,
                    public_flags: None,
                    bot: None,
                    premium_type: None,
                },
                MemberInfo {
                    user_id: "user_003".to_string(),
                    username: "charlie".to_string(),
                    display_name: None,
                    avatar_url: None,
                    role_name: None,
                    role_color: None,
                    public_flags: None,
                    bot: None,
                    premium_type: None,
                },
                MemberInfo {
                    user_id: "user_005".to_string(),
                    username: "testbot".to_string(),
                    display_name: Some("TestBot".to_string()),
                    avatar_url: None,
                    role_name: None,
                    role_color: None,
                    public_flags: None,
                    bot: Some(true),
                    premium_type: None,
                },
            ],
        }
    }
}

/// Mock user profile JSON (curated for UI) — used when FetchUserProfile is received.
/// Includes is_owner, permission_names, mutual_friends, and roles as objects for exhaustive profile popup.
const MOCK_PROFILE_JSON: &str = r##"{"user":{"id":"user_001","username":"alice","global_name":"Alice","avatar":null,"accent_color":5814783,"banner":null},"bio":"Test bio for UI.","created_at":"2020-01-15T12:00:00Z","note":"","guild_member_profile":{"nick":"Alice","roles":[{"id":"role_1","name":"Moderator","color":"#58a0a3"},{"id":"role_2","name":"Helper","color":"#99aab5"}],"joined_at":"2021-06-01T12:00:00Z","is_owner":false,"permission_names":["Manage Server","Kick Members","View Channel","Send Messages"]},"mutual_guilds":[{"id":"g2","nick":null}],"mutual_friends":[{"id":"user_002","username":"bob","global_name":"Bob","avatar":null}]}"##;
/// Raw API response for "Copy Raw JSON"
const MOCK_PROFILE_RAW_JSON: &str = r#"{"user":{"id":"user_001","username":"alice","global_name":"Alice","avatar":null,"accent_color":5814783},"connected_accounts":[],"premium_since":null}"#;

/// Spawns the mock worker: sends initial UiUpdates and responds to UiAction (SelectGuild, SelectChannel, FetchUserProfile).
fn spawn_mock_worker(
    backend: MockBackend,
    update_tx: mpsc::Sender<UiUpdate>,
    mut action_rx: tokio::sync::mpsc::UnboundedReceiver<UiAction>,
) {
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            // Logged-in state
            let _ = update_tx.send(UiUpdate::LoginSuccess {
                user_id: "test_user".to_string(),
                username: "TestUser".to_string(),
                avatar_url: None,
            });
            let _ = update_tx.send(UiUpdate::Connected);

            // Guild list
            let _ = update_tx.send(UiUpdate::GuildsLoaded(backend.guilds.clone()));

            // Presence so status dots show (online, idle, dnd, offline)
            for (user_id, status) in [
                ("user_001", "online"),
                ("user_002", "idle"),
                ("user_003", "dnd"),
                ("user_005", "online"),
                ("test_user", "online"),
            ] {
                let _ = update_tx.send(UiUpdate::PresenceUpdated {
                    user_id: user_id.to_string(),
                    status: status.to_string(),
                });
            }

            // DM list (empty)
            let _ = update_tx.send(UiUpdate::DmChannelsLoaded(vec![]));

            while let Some(action) = action_rx.recv().await {
                match action {
                    UiAction::SelectGuild(guild_id) => {
                        let channels: Vec<ChannelInfo> = backend
                            .channels
                            .iter()
                            .filter(|c| c.guild_id.as_deref() == Some(guild_id.as_str()))
                            .cloned()
                            .collect();
                        let _ = update_tx.send(UiUpdate::ChannelsLoaded(channels));
                    }
                    UiAction::SelectChannel(channel_id, _) => {
                        let members = backend.members.clone();
                        let _ = update_tx.send(UiUpdate::MembersLoaded {
                            guild_id: backend.guilds.first().map(|g| g.id.clone()).unwrap_or_default(),
                            members,
                        });
                        let messages: Vec<MessageInfo> = backend
                            .messages
                            .iter()
                            .filter(|m| m.channel_id == channel_id)
                            .cloned()
                            .collect();
                        let _ = update_tx.send(UiUpdate::MessagesLoaded(messages));
                    }
                    UiAction::FetchUserProfile { user_id, .. } => {
                        let _ = update_tx.send(UiUpdate::UserProfileLoaded {
                            user_id,
                            profile_json: MOCK_PROFILE_JSON.to_string(),
                            raw_json: MOCK_PROFILE_RAW_JSON.to_string(),
                        });
                    }
                    _ => {}
                }
            }
        });
    });
}

/// QML-accessible test data provider (used only when running test_ui.qml; kept for compatibility)
#[allow(dead_code)]
#[derive(QObject, Default)]
struct TestDataProvider {
    base: qt_base_class!(trait QObject),

    /// JSON array of test guilds
    guilds_json: qt_property!(QString; CONST),
    /// JSON array of test channels
    channels_json: qt_property!(QString; CONST),
    /// JSON array of test messages
    messages_json: qt_property!(QString; CONST),
    /// JSON object with guildId and members array (for member list panel)
    members_json: qt_property!(QString; CONST),
    /// JSON object { channelId, participants } for voice channel (ch_003)
    voice_participants_json: qt_property!(QString; CONST),
    /// Test user name
    test_user_name: qt_property!(QString; CONST),
}

#[allow(dead_code)]
impl TestDataProvider {
    fn new(backend: &MockBackend) -> Self {
        let guilds = serde_json::to_string(
            &backend
                .guilds
                .iter()
                .map(|g| {
                    serde_json::json!({
                        "guildId": g.id,
                        "name": g.name,
                        "iconUrl": g.icon_url.as_deref().unwrap_or(""),
                        "hasUnread": g.has_unread,
                        "mentionCount": g.mention_count
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let channels = serde_json::to_string(
            &backend
                .channels
                .iter()
                .map(|c| {
                    serde_json::json!({
                        "channelId": c.id,
                        "guildId": c.guild_id.as_deref().unwrap_or(""),
                        "name": c.name,
                        "channelType": c.channel_type,
                        "position": c.position,
                        "parentId": c.parent_id.as_deref().unwrap_or(""),
                        "hasUnread": c.has_unread,
                        "mentionCount": c.mention_count,
                        "isHidden": c.is_hidden
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let messages = serde_json::to_string(
            &backend
                .messages
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "messageId": m.id,
                        "channelId": m.channel_id,
                        "authorName": m.author_name,
                        "authorId": m.author_id,
                        "authorAvatarUrl": m.author_avatar_url.as_deref().unwrap_or(""),
                        "content": m.content,
                        "timestamp": m.timestamp,
                        "isDeleted": m.is_deleted,
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
                            "me": r.me
                        })).collect::<Vec<_>>()
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let members_obj = serde_json::json!({
            "guildId": "guild_001",
            "members": backend.members.iter().map(|m| serde_json::json!({
                "memberId": m.user_id,
                "username": m.username,
                "displayName": m.display_name.as_deref().unwrap_or(""),
                "avatarUrl": m.avatar_url.as_deref().unwrap_or(""),
                "roleName": m.role_name.as_deref().unwrap_or(""),
                "roleColor": m.role_color.as_deref().unwrap_or(""),
                "publicFlags": m.public_flags.unwrap_or(0),
                "bot": m.bot.unwrap_or(false),
                "premiumType": m.premium_type.unwrap_or(0)
            })).collect::<Vec<_>>()
        });
        let members_json = serde_json::to_string(&members_obj).unwrap();

        let voice_participants = serde_json::json!({
            "channelId": "ch_003",
            "participants": [
                { "userId": "user_001", "username": "Alice", "avatarUrl": "", "selfMute": false, "selfDeaf": false, "serverMute": false, "serverDeaf": false, "speaking": false, "selfVideo": false, "selfStream": false, "suppress": false },
                { "userId": "user_002", "username": "Bob", "avatarUrl": "", "selfMute": true, "selfDeaf": false, "serverMute": false, "serverDeaf": false, "speaking": false, "selfVideo": false, "selfStream": false, "suppress": false },
                { "userId": "user_003", "username": "Charlie", "avatarUrl": "", "selfMute": false, "selfDeaf": false, "serverMute": false, "serverDeaf": false, "speaking": false, "selfVideo": true, "selfStream": false, "suppress": false }
            ]
        });
        let voice_participants_json = serde_json::to_string(&voice_participants).unwrap();

        Self {
            guilds_json: QString::from(guilds.as_str()),
            channels_json: QString::from(channels.as_str()),
            messages_json: QString::from(messages.as_str()),
            members_json: QString::from(members_json.as_str()),
            voice_participants_json: QString::from(voice_participants_json.as_str()),
            test_user_name: QString::from("TestUser"),
            ..Default::default()
        }
    }
}

fn main() {
    // Force Qt Quick Controls to use the "Basic" style (no native theming)
    std::env::set_var("QT_QUICK_CONTROLS_STYLE", "Basic");
    std::env::set_var("QT_QPA_PLATFORMTHEME", "");

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("skunkcord=info".parse().unwrap())
                .add_directive("ui_test=info".parse().unwrap()),
        )
        .init();

    let mobile_mode = std::env::args().any(|a| a == "--mobile");

    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("  Skunkcord UI Test Mode{}", if mobile_mode { " (MOBILE)" } else { "" });
    tracing::info!("  Version: {} (TEST BUILD)", env!("CARGO_PKG_VERSION"));
    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("");
    tracing::info!("This runs the UI with mock data - no Discord connection.");
    if mobile_mode {
        tracing::info!("Using mobile.qml at 390x844 (iPhone 14 Pro)");
    } else {
        tracing::info!("Uses main.qml + AppController; mock worker sends profile & presence.");
    }
    tracing::info!("");

    let backend = MockBackend::new();
    tracing::info!("Mock backend: {} guilds, {} channels, {} messages",
        backend.guilds.len(), backend.channels.len(), backend.messages.len());

    let (login_tx, _login_rx) = mpsc::channel::<LoginRequest>();
    let (update_tx, update_rx) = mpsc::channel::<UiUpdate>();
    let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel::<UiAction>();

    spawn_mock_worker(backend, update_tx, action_rx);

    let app_controller = std::cell::RefCell::new(AppController::new(login_tx, action_tx, update_rx));
    let controller_ptr = unsafe { QObjectPinned::new(&app_controller) };

    let mut engine = QmlEngine::new();
    engine.set_object_property("app".into(), controller_ptr);

    let qml_file = if mobile_mode { "mobile.qml" } else { "main.qml" };
    let qml_path = get_qml_path(qml_file);
    if !qml_path.exists() {
        eprintln!("ERROR: QML file not found at: {}", qml_path.display());
        eprintln!("Please ensure the 'qml' directory is in the same location as the executable.");
        std::process::exit(1);
    }
    engine.load_file(qml_path.to_string_lossy().to_string().into());

    engine.exec();
}

/// Get QML file path - tries multiple locations in order:
/// 1. app-qml/ (bundled package with Qt libs)
/// 2. qml/ (simple deployment package)
/// 3. Development path (CARGO_MANIFEST_DIR/src/qml)
fn get_qml_path(filename: &str) -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Try bundled package location first (app-qml)
            let bundled_path = exe_dir.join("app-qml").join(filename);
            if bundled_path.exists() {
                return bundled_path;
            }
            
            // Try simple deployment location (qml)
            let qml_path = exe_dir.join("qml").join(filename);
            if qml_path.exists() {
                return qml_path;
            }
        }
    }

    // Fall back to development path
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("qml")
        .join(filename)
}




