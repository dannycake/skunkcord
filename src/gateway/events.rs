// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Gateway event types

use crate::client::{Channel, Message, Relationship, User};
use serde::{Deserialize, Serialize};

/// Events received from the Discord gateway.
///
/// Covers every documented Discord dispatch event as of Gateway v10 (Feb 2026),
/// plus undocumented user-client events (MESSAGE_ACK, RELATIONSHIP_*, etc.).
/// Events that don't need typed payloads use `serde_json::Value` for forward
/// compatibility.  Truly unknown events land in the [`Raw`] variant.
#[derive(Debug, Clone)]
pub enum GatewayEvent {
    // ── Connection lifecycle ──────────────────────────────────────────
    /// Ready event — received after successful identify
    Ready(ReadyEvent),
    /// Ready supplemental — extra data sent after READY (user-client only)
    ReadySupplemental(serde_json::Value),
    /// Sessions replace — session list updated (user-client only)
    SessionsReplace(serde_json::Value),

    // ── Messages ─────────────────────────────────────────────────────
    /// Message created
    MessageCreate(MessageCreateEvent),
    /// Message updated/edited
    MessageUpdate(MessageUpdateEvent),
    /// Message deleted
    MessageDelete(MessageDeleteEvent),
    /// Multiple messages deleted at once
    MessageDeleteBulk(MessageDeleteBulkEvent),
    /// Message acknowledged / read state (user-client only)
    MessageAck(MessageAckEvent),

    // ── Reactions ────────────────────────────────────────────────────
    /// Reaction added to a message
    MessageReactionAdd(MessageReactionAddEvent),
    /// Reaction removed from a message
    MessageReactionRemove(MessageReactionRemoveEvent),
    /// All reactions removed from a message
    MessageReactionRemoveAll(MessageReactionRemoveAllEvent),
    /// All reactions of a specific emoji removed from a message
    MessageReactionRemoveEmoji(MessageReactionRemoveEmojiEvent),

    // ── Polls ────────────────────────────────────────────────────────
    /// User voted on a poll
    MessagePollVoteAdd(MessagePollVoteEvent),
    /// User removed a vote on a poll
    MessagePollVoteRemove(MessagePollVoteEvent),

    // ── Presence & typing ────────────────────────────────────────────
    /// User's presence updated
    PresenceUpdate(PresenceUpdateEvent),
    /// User started typing in a channel
    TypingStart(TypingStartEvent),

    // ── Users ────────────────────────────────────────────────────────
    /// Current user's properties changed
    UserUpdate(User),
    /// User settings updated (user-client only)
    UserSettingsUpdate(serde_json::Value),
    /// User note updated (user-client only)
    UserNoteUpdate(UserNoteUpdateEvent),

    // ── Guilds ───────────────────────────────────────────────────────
    /// Guild created / became available / user joined
    GuildCreate(serde_json::Value),
    /// Guild was updated
    GuildUpdate(serde_json::Value),
    /// Guild became unavailable or user left
    GuildDelete(GuildDeleteEvent),

    // ── Guild members ────────────────────────────────────────────────
    /// Member joined a guild
    GuildMemberAdd(GuildMemberAddEvent),
    /// Guild member was updated
    GuildMemberUpdate(GuildMemberUpdateEvent),
    /// Member left / was kicked / banned from a guild
    GuildMemberRemove(GuildMemberRemoveEvent),
    /// Response to Request Guild Members (op 8)
    GuildMembersChunk(GuildMembersChunkEvent),
    /// Lazy guild member list update (user-client, response to op 14)
    GuildMemberListUpdate(serde_json::Value),

    // ── Guild moderation ─────────────────────────────────────────────
    /// User was banned from a guild
    GuildBanAdd(GuildBanEvent),
    /// User was unbanned from a guild
    GuildBanRemove(GuildBanEvent),
    /// Guild audit log entry was created
    GuildAuditLogEntryCreate(serde_json::Value),

    // ── Guild roles ──────────────────────────────────────────────────
    /// Guild role was created
    GuildRoleCreate(GuildRoleEvent),
    /// Guild role was updated
    GuildRoleUpdate(GuildRoleEvent),
    /// Guild role was deleted
    GuildRoleDelete(GuildRoleDeleteEvent),

    // ── Guild customisation ──────────────────────────────────────────
    /// Guild emojis were updated
    GuildEmojisUpdate(GuildEmojisUpdateEvent),
    /// Guild stickers were updated
    GuildStickersUpdate(serde_json::Value),
    /// Guild integrations were updated
    GuildIntegrationsUpdate(serde_json::Value),

    // ── Guild scheduled events ───────────────────────────────────────
    /// Scheduled event created
    GuildScheduledEventCreate(serde_json::Value),
    /// Scheduled event updated
    GuildScheduledEventUpdate(serde_json::Value),
    /// Scheduled event deleted
    GuildScheduledEventDelete(serde_json::Value),
    /// User subscribed to a scheduled event
    GuildScheduledEventUserAdd(serde_json::Value),
    /// User unsubscribed from a scheduled event
    GuildScheduledEventUserRemove(serde_json::Value),

    // ── Guild soundboard ─────────────────────────────────────────────
    /// Guild soundboard sound created
    GuildSoundboardSoundCreate(serde_json::Value),
    /// Guild soundboard sound updated
    GuildSoundboardSoundUpdate(serde_json::Value),
    /// Guild soundboard sound deleted
    GuildSoundboardSoundDelete(serde_json::Value),
    /// Guild soundboard sounds bulk update
    GuildSoundboardSoundsUpdate(serde_json::Value),
    /// Response to Request Soundboard Sounds (op 31)
    SoundboardSounds(serde_json::Value),

    // ── Channels ─────────────────────────────────────────────────────
    /// New guild channel created
    ChannelCreate(Channel),
    /// Channel was updated
    ChannelUpdate(Channel),
    /// Channel was deleted
    ChannelDelete(Channel),
    /// Message was pinned or unpinned
    ChannelPinsUpdate(ChannelPinsUpdateEvent),
    /// Channel unread state changed (user-client only)
    ChannelUnreadUpdate(serde_json::Value),

    // ── Threads ──────────────────────────────────────────────────────
    /// Thread created (or added to a private thread)
    ThreadCreate(serde_json::Value),
    /// Thread was updated
    ThreadUpdate(serde_json::Value),
    /// Thread was deleted
    ThreadDelete(serde_json::Value),
    /// Gained access to a channel — contains active threads
    ThreadListSync(serde_json::Value),
    /// Thread member for current user was updated
    ThreadMemberUpdate(serde_json::Value),
    /// Users were added to or removed from a thread
    ThreadMembersUpdate(serde_json::Value),

    // ── Relationships (user-client only) ─────────────────────────────
    /// Friend / block / request added
    RelationshipAdd(Relationship),
    /// Relationship removed
    RelationshipRemove(RelationshipRemoveEvent),

    // ── Voice ────────────────────────────────────────────────────────
    /// Someone joined/left/moved a voice channel
    VoiceStateUpdate(VoiceStateUpdateEvent),
    /// Guild voice server updated (for voice connections)
    VoiceServerUpdate(VoiceServerUpdateEvent),
    /// Voice channel effect sent (emoji/soundboard in VC)
    VoiceChannelEffectSend(serde_json::Value),

    // ── Stage instances ──────────────────────────────────────────────
    /// Stage instance created
    StageInstanceCreate(serde_json::Value),
    /// Stage instance updated
    StageInstanceUpdate(serde_json::Value),
    /// Stage instance deleted
    StageInstanceDelete(serde_json::Value),

    // ── Interactions ─────────────────────────────────────────────────
    /// User used an interaction (slash command, button, etc.)
    InteractionCreate(InteractionCreateEvent),

    // ── Invites ──────────────────────────────────────────────────────
    /// Invite to a channel was created
    InviteCreate(serde_json::Value),
    /// Invite to a channel was deleted
    InviteDelete(serde_json::Value),

    // ── Integrations ─────────────────────────────────────────────────
    /// Guild integration was created
    IntegrationCreate(serde_json::Value),
    /// Guild integration was updated
    IntegrationUpdate(serde_json::Value),
    /// Guild integration was deleted
    IntegrationDelete(serde_json::Value),

    // ── Webhooks ─────────────────────────────────────────────────────
    /// Guild channel webhook was created/updated/deleted
    WebhooksUpdate(serde_json::Value),

    // ── Auto-moderation ──────────────────────────────────────────────
    /// Auto-moderation rule was created
    AutoModerationRuleCreate(serde_json::Value),
    /// Auto-moderation rule was updated
    AutoModerationRuleUpdate(serde_json::Value),
    /// Auto-moderation rule was deleted
    AutoModerationRuleDelete(serde_json::Value),
    /// Auto-moderation rule was triggered
    AutoModerationActionExecution(serde_json::Value),

    // ── Application commands ─────────────────────────────────────────
    /// Application command permissions were updated
    ApplicationCommandPermissionsUpdate(serde_json::Value),

    // ── Entitlements (monetisation) ──────────────────────────────────
    /// Entitlement was created
    EntitlementCreate(serde_json::Value),
    /// Entitlement was updated
    EntitlementUpdate(serde_json::Value),
    /// Entitlement was deleted
    EntitlementDelete(serde_json::Value),

    // ── Subscriptions (premium apps) ─────────────────────────────────
    /// Subscription was created
    SubscriptionCreate(serde_json::Value),
    /// Subscription was updated
    SubscriptionUpdate(serde_json::Value),
    /// Subscription was deleted
    SubscriptionDelete(serde_json::Value),

    // ── Catch-all ────────────────────────────────────────────────────
    /// Any event not matched above (future-proof)
    Raw {
        event_type: String,
        data: serde_json::Value,
    },
}

/// Ready event data
///
/// Uses `serde_json::Value` for fields whose shape varies between Discord API
/// versions so that deserialization never fails on unknown/changed layouts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyEvent {
    /// API version (Discord may omit or send as number)
    #[serde(default)]
    pub v: u8,
    /// Current user
    pub user: User,
    /// DM channels
    #[serde(default)]
    pub private_channels: Vec<serde_json::Value>,
    /// Guilds (may be full or unavailable depending on Discord version)
    #[serde(default)]
    pub guilds: Vec<serde_json::Value>,
    /// Session ID for resuming
    pub session_id: String,
    /// Resume gateway URL
    pub resume_gateway_url: Option<String>,
    /// Shard info [shard_id, num_shards]
    #[serde(default)]
    pub shard: Option<[u32; 2]>,
    /// Application info
    #[serde(default)]
    pub application: Option<serde_json::Value>,
    /// Relationships (friends)
    #[serde(default)]
    pub relationships: Vec<serde_json::Value>,
    /// User settings (may be absent; Discord now uses user_settings_proto)
    #[serde(default)]
    pub user_settings: Option<serde_json::Value>,
    /// User settings proto (newer Discord format)
    #[serde(default)]
    pub user_settings_proto: Option<String>,
    /// User guild settings — may be array or object with entries
    #[serde(default)]
    pub user_guild_settings: serde_json::Value,
    /// Read states — may be array or object with entries
    #[serde(default)]
    pub read_state: serde_json::Value,
    /// Connected accounts
    #[serde(default)]
    pub connected_accounts: Vec<serde_json::Value>,
    /// Session type
    #[serde(default)]
    pub session_type: Option<String>,
    /// Auth session ID hash
    #[serde(default)]
    pub auth_session_id_hash: Option<String>,
    /// Sessions list
    #[serde(default)]
    pub sessions: Vec<serde_json::Value>,
    /// Merged members (guild member info)
    #[serde(default)]
    pub merged_members: Vec<serde_json::Value>,
    /// Users referenced in the ready payload
    #[serde(default)]
    pub users: Vec<serde_json::Value>,
    /// Analytics token
    #[serde(default)]
    pub analytics_token: Option<String>,
    /// Country code
    #[serde(default)]
    pub country_code: Option<String>,
}

impl ReadyEvent {
    /// Extract guild IDs from the guilds array (works with both full and unavailable guild formats)
    pub fn guild_ids(&self) -> Vec<String> {
        self.guilds
            .iter()
            .filter_map(|g| g.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
            .collect()
    }
}

/// Message create event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCreateEvent {
    #[serde(flatten)]
    pub message: Message,
    /// Guild ID if in a guild
    #[serde(default)]
    pub guild_id: Option<String>,
    /// Member info if in a guild
    #[serde(default)]
    pub member: Option<PartialMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialMember {
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub joined_at: Option<String>,
    #[serde(default)]
    pub nick: Option<String>,
    #[serde(default)]
    pub deaf: Option<bool>,
    #[serde(default)]
    pub mute: Option<bool>,
}

/// Message update event (partial — only changed fields are present)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageUpdateEvent {
    pub id: String,
    pub channel_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub embeds: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub edited_timestamp: Option<String>,
    #[serde(default)]
    pub author: Option<User>,
}

/// Message delete event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeleteEvent {
    pub id: String,
    pub channel_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
}

/// Presence update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdateEvent {
    pub user: PartialUser,
    #[serde(default)]
    pub guild_id: Option<String>,
    pub status: String,
    #[serde(default)]
    pub activities: Vec<ActivityData>,
    #[serde(default)]
    pub client_status: Option<ClientStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialUser {
    pub id: String,
    pub username: Option<String>,
    pub avatar: Option<String>,
    pub discriminator: Option<String>,
    pub global_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityData {
    pub name: String,
    #[serde(rename = "type")]
    pub activity_type: u8,
    pub url: Option<String>,
    pub state: Option<String>,
    pub details: Option<String>,
    pub timestamps: Option<ActivityTimestamps>,
    pub application_id: Option<String>,
    pub emoji: Option<ActivityEmoji>,
    pub party: Option<serde_json::Value>,
    pub assets: Option<ActivityAssets>,
    pub flags: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityTimestamps {
    pub start: Option<u64>,
    pub end: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEmoji {
    pub name: String,
    pub id: Option<String>,
    pub animated: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityAssets {
    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStatus {
    pub desktop: Option<String>,
    pub mobile: Option<String>,
    pub web: Option<String>,
}

/// Typing start event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingStartEvent {
    pub channel_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    pub user_id: String,
    pub timestamp: u64,
    #[serde(default)]
    pub member: Option<PartialMember>,
}

/// Guild delete event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildDeleteEvent {
    pub id: String,
    #[serde(default)]
    pub unavailable: Option<bool>,
}

/// Relationship remove event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipRemoveEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub relationship_type: u8,
}

/// Voice state update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceStateUpdateEvent {
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub channel_id: Option<String>,
    pub user_id: String,
    pub session_id: String,
    #[serde(default)]
    pub deaf: bool,
    #[serde(default)]
    pub mute: bool,
    #[serde(default)]
    pub self_deaf: bool,
    #[serde(default)]
    pub self_mute: bool,
    #[serde(default)]
    pub self_stream: Option<bool>,
    #[serde(default)]
    pub self_video: bool,
    #[serde(default)]
    pub suppress: bool,
}

/// User note update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNoteUpdateEvent {
    pub id: String,
    pub note: String,
}

/// Message delete bulk event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeleteBulkEvent {
    pub ids: Vec<String>,
    pub channel_id: String,
    pub guild_id: Option<String>,
}

/// Guild member add event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMemberAddEvent {
    pub guild_id: String,
    #[serde(default)]
    pub user: Option<User>,
    #[serde(default)]
    pub nick: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub joined_at: Option<String>,
    #[serde(default)]
    pub deaf: Option<bool>,
    #[serde(default)]
    pub mute: Option<bool>,
}

/// Guild member update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMemberUpdateEvent {
    pub guild_id: String,
    #[serde(default)]
    pub roles: Vec<String>,
    pub user: User,
    #[serde(default)]
    pub nick: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub joined_at: Option<String>,
    #[serde(default)]
    pub premium_since: Option<String>,
    #[serde(default)]
    pub communication_disabled_until: Option<String>,
}

/// Guild member remove event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMemberRemoveEvent {
    pub guild_id: String,
    pub user: User,
}

/// Guild ban add/remove event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildBanEvent {
    pub guild_id: String,
    pub user: User,
}

/// Guild role create/update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildRoleEvent {
    pub guild_id: String,
    pub role: RoleData,
}

/// Role data within guild role events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleData {
    pub id: String,
    pub name: String,
    pub color: u32,
    pub hoist: bool,
    pub position: i32,
    pub permissions: String,
    pub managed: bool,
    pub mentionable: bool,
}

/// Guild role delete event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildRoleDeleteEvent {
    pub guild_id: String,
    pub role_id: String,
}

/// Channel pins update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPinsUpdateEvent {
    pub guild_id: Option<String>,
    pub channel_id: String,
    pub last_pin_timestamp: Option<String>,
}

/// Voice server update event (critical for voice connections)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceServerUpdateEvent {
    pub token: String,
    pub guild_id: Option<String>,
    pub endpoint: Option<String>,
}

/// Message reaction add event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReactionAddEvent {
    pub user_id: String,
    pub channel_id: String,
    pub message_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub member: Option<PartialMember>,
    pub emoji: ReactionEmoji,
}

/// Message reaction remove event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReactionRemoveEvent {
    pub user_id: String,
    pub channel_id: String,
    pub message_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    pub emoji: ReactionEmoji,
}

/// Message reaction remove all event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReactionRemoveAllEvent {
    pub channel_id: String,
    pub message_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
}

/// All reactions of a specific emoji removed from a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReactionRemoveEmojiEvent {
    pub channel_id: String,
    pub message_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    pub emoji: ReactionEmoji,
}

/// Emoji used in a reaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionEmoji {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub animated: Option<bool>,
}

/// Interaction create event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionCreateEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub interaction_type: u8,
    pub data: Option<serde_json::Value>,
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub member: Option<PartialMember>,
    pub user: Option<User>,
    pub token: String,
    pub version: u8,
    pub message: Option<Message>,
}

/// Message ACK event (read state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAckEvent {
    pub channel_id: String,
    pub message_id: String,
    pub version: Option<u32>,
}

/// Thread create event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadCreateEvent {
    #[serde(flatten)]
    pub channel: Channel,
    pub newly_created: Option<bool>,
}

/// Guild emojis update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildEmojisUpdateEvent {
    pub guild_id: String,
    pub emojis: Vec<serde_json::Value>,
}

/// Guild members chunk event (response to op 8 Request Guild Members)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMembersChunkEvent {
    pub guild_id: String,
    pub members: Vec<serde_json::Value>,
    pub chunk_index: u32,
    pub chunk_count: u32,
    #[serde(default)]
    pub not_found: Vec<String>,
    #[serde(default)]
    pub presences: Vec<serde_json::Value>,
    pub nonce: Option<String>,
}

/// Message poll vote event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePollVoteEvent {
    pub user_id: String,
    pub channel_id: String,
    pub message_id: String,
    pub guild_id: Option<String>,
    pub answer_id: u32,
}
