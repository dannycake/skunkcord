// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Discord API endpoints and data structures

use super::{DiscordClient, User};
use crate::{DiscordError, Result};
use serde::{Deserialize, Serialize};

/// Guild (server) object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub owner: Option<bool>,
    pub owner_id: Option<String>,
    pub permissions: Option<String>,
    pub features: Vec<String>,
    pub approximate_member_count: Option<u64>,
    pub approximate_presence_count: Option<u64>,
}

impl Guild {
    /// Get the guild icon URL
    pub fn icon_url(&self, size: u32) -> Option<String> {
        self.icon.as_ref().map(|hash| {
            let ext = if hash.starts_with("a_") { "gif" } else { "png" };
            format!(
                "https://cdn.discordapp.com/icons/{}/{}.{}?size={}",
                self.id, hash, ext, size
            )
        })
    }
}

/// Permission overwrite on a channel (Discord API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPermissionOverwrite {
    pub id: String,
    #[serde(rename = "type")]
    pub overwrite_type: u8,
    #[serde(default)]
    pub allow: String,
    #[serde(default)]
    pub deny: String,
}

/// Channel object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    #[serde(rename = "type")]
    pub channel_type: u8,
    pub guild_id: Option<String>,
    pub position: Option<i32>,
    pub name: Option<String>,
    pub topic: Option<String>,
    pub nsfw: Option<bool>,
    pub last_message_id: Option<String>,
    pub bitrate: Option<u32>,
    pub user_limit: Option<u32>,
    pub recipients: Option<Vec<User>>,
    pub icon: Option<String>,
    pub owner_id: Option<String>,
    pub parent_id: Option<String>,
    #[serde(default)]
    pub permission_overwrites: Option<Vec<ChannelPermissionOverwrite>>,
}

// Channel type constants (Discord API)
pub const CHANNEL_TYPE_GUILD_TEXT: u8 = 0;
pub const CHANNEL_TYPE_DM: u8 = 1;
pub const CHANNEL_TYPE_GUILD_VOICE: u8 = 2;
pub const CHANNEL_TYPE_GROUP_DM: u8 = 3;
pub const CHANNEL_TYPE_GUILD_CATEGORY: u8 = 4;
pub const CHANNEL_TYPE_GUILD_ANNOUNCEMENT: u8 = 5;
pub const CHANNEL_TYPE_ANNOUNCEMENT_THREAD: u8 = 10;
pub const CHANNEL_TYPE_PUBLIC_THREAD: u8 = 11;
pub const CHANNEL_TYPE_PRIVATE_THREAD: u8 = 12;
pub const CHANNEL_TYPE_GUILD_STAGE_VOICE: u8 = 13;
// pub const CHANNEL_TYPE_GUILD_FORUM: u8 = 15;  // defined in forums.rs
// pub const CHANNEL_TYPE_GUILD_MEDIA: u8 = 16;  // defined in forums.rs

impl Channel {
    /// Check if this is a DM channel
    pub fn is_dm(&self) -> bool {
        self.channel_type == CHANNEL_TYPE_DM
    }

    /// Check if this is a group DM
    pub fn is_group_dm(&self) -> bool {
        self.channel_type == CHANNEL_TYPE_GROUP_DM
    }

    /// Check if this is a text channel
    pub fn is_text(&self) -> bool {
        self.channel_type == CHANNEL_TYPE_GUILD_TEXT
    }

    /// Check if this is a voice channel
    pub fn is_voice(&self) -> bool {
        self.channel_type == CHANNEL_TYPE_GUILD_VOICE
    }

    /// Check if this is an announcement channel
    pub fn is_announcement(&self) -> bool {
        self.channel_type == CHANNEL_TYPE_GUILD_ANNOUNCEMENT
    }

    /// Check if this is a thread (announcement, public, or private)
    pub fn is_thread(&self) -> bool {
        matches!(
            self.channel_type,
            CHANNEL_TYPE_ANNOUNCEMENT_THREAD
                | CHANNEL_TYPE_PUBLIC_THREAD
                | CHANNEL_TYPE_PRIVATE_THREAD
        )
    }

    /// Check if this is a stage voice channel
    pub fn is_stage_voice(&self) -> bool {
        self.channel_type == CHANNEL_TYPE_GUILD_STAGE_VOICE
    }

    /// Check if this is a voice or stage voice channel
    pub fn is_voice_or_stage(&self) -> bool {
        self.is_voice() || self.is_stage_voice()
    }

    /// Check if this channel type supports text messages
    pub fn supports_messages(&self) -> bool {
        matches!(
            self.channel_type,
            CHANNEL_TYPE_GUILD_TEXT
                | CHANNEL_TYPE_DM
                | CHANNEL_TYPE_GROUP_DM
                | CHANNEL_TYPE_GUILD_ANNOUNCEMENT
                | CHANNEL_TYPE_ANNOUNCEMENT_THREAD
                | CHANNEL_TYPE_PUBLIC_THREAD
                | CHANNEL_TYPE_PRIVATE_THREAD
        )
    }
}

/// Check if a given channel type supports text messages (standalone helper)
pub fn channel_type_supports_messages(channel_type: u8) -> bool {
    matches!(
        channel_type,
        CHANNEL_TYPE_GUILD_TEXT
            | CHANNEL_TYPE_DM
            | CHANNEL_TYPE_GROUP_DM
            | CHANNEL_TYPE_GUILD_ANNOUNCEMENT
            | CHANNEL_TYPE_ANNOUNCEMENT_THREAD
            | CHANNEL_TYPE_PUBLIC_THREAD
            | CHANNEL_TYPE_PRIVATE_THREAD
    )
}

/// Check if a given channel type is voice or stage (standalone helper)
pub fn channel_type_is_voice(channel_type: u8) -> bool {
    channel_type == CHANNEL_TYPE_GUILD_VOICE || channel_type == CHANNEL_TYPE_GUILD_STAGE_VOICE
}

/// Message object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel_id: String,
    pub author: Option<User>,
    pub content: String,
    pub timestamp: String,
    pub edited_timestamp: Option<String>,
    pub tts: bool,
    pub mention_everyone: bool,
    pub mentions: Vec<User>,
    /// Role IDs mentioned in this message
    #[serde(default)]
    pub mention_roles: Vec<String>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default)]
    pub embeds: Vec<Embed>,
    /// Sticker items on this message (Discord API: sticker_items)
    #[serde(default)]
    #[serde(rename = "sticker_items")]
    pub sticker_items: Vec<MessageStickerItem>,
    #[serde(rename = "type")]
    pub message_type: u8,
    /// The message this is replying to (only present for type 19 replies)
    #[serde(default)]
    pub referenced_message: Option<Box<Message>>,
    /// Flags (bitfield)
    #[serde(default)]
    pub flags: Option<u64>,
    /// Reactions on this message
    #[serde(default)]
    pub reactions: Vec<super::reactions::Reaction>,
}

/// Sticker item on a message (reduced form from Discord API sticker_items)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStickerItem {
    pub id: String,
    pub name: String,
    /// 1 = PNG, 2 = APNG, 3 = Lottie
    #[serde(rename = "format_type")]
    pub format_type: u8,
}

/// Attachment object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub filename: String,
    pub size: u64,
    pub url: String,
    pub proxy_url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub content_type: Option<String>,
}

/// Embed object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embed {
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub embed_type: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub timestamp: Option<String>,
    pub color: Option<u32>,
    pub footer: Option<EmbedFooter>,
    pub image: Option<EmbedImage>,
    pub thumbnail: Option<EmbedThumbnail>,
    pub author: Option<EmbedAuthor>,
    pub fields: Option<Vec<EmbedField>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedFooter {
    pub text: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedImage {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedThumbnail {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedAuthor {
    pub name: String,
    pub url: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    pub inline: Option<bool>,
}

/// Create message request
#[derive(Debug, Clone, Serialize)]
pub struct CreateMessage {
    pub content: Option<String>,
    pub tts: Option<bool>,
    pub embeds: Option<Vec<Embed>>,
    pub message_reference: Option<MessageReference>,
    pub nonce: Option<String>,
    pub flags: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sticker_ids: Option<Vec<String>>,
}

impl CreateMessage {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: Some(content.into()),
            tts: None,
            embeds: None,
            message_reference: None,
            nonce: Some(uuid::Uuid::new_v4().to_string()),
            flags: None,
            sticker_ids: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReference {
    pub message_id: Option<String>,
    pub channel_id: Option<String>,
    pub guild_id: Option<String>,
    pub fail_if_not_exists: Option<bool>,
}

/// Relationship (friend, blocked, etc)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    #[serde(rename = "type")]
    pub relationship_type: u8,
    pub nickname: Option<String>,
    pub user: User,
}

impl Relationship {
    /// Check if this is a friend
    pub fn is_friend(&self) -> bool {
        self.relationship_type == 1
    }

    /// Check if this is a blocked user
    pub fn is_blocked(&self) -> bool {
        self.relationship_type == 2
    }

    /// Check if this is an incoming friend request
    pub fn is_incoming_request(&self) -> bool {
        self.relationship_type == 3
    }

    /// Check if this is an outgoing friend request
    pub fn is_outgoing_request(&self) -> bool {
        self.relationship_type == 4
    }
}

/// User settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub locale: Option<String>,
    pub theme: Option<String>,
    pub status: Option<String>,
    pub custom_status: Option<CustomStatus>,
    pub guild_positions: Option<Vec<String>>,
    pub restricted_guilds: Option<Vec<String>>,
    pub friend_source_flags: Option<FriendSourceFlags>,
    pub developer_mode: Option<bool>,
    pub message_display_compact: Option<bool>,
    pub render_embeds: Option<bool>,
    pub inline_attachment_media: Option<bool>,
    pub inline_embed_media: Option<bool>,
    pub gif_auto_play: Option<bool>,
    pub render_reactions: Option<bool>,
    pub animate_emoji: Option<bool>,
    pub enable_tts_command: Option<bool>,
    pub explicit_content_filter: Option<u8>,
    pub afk_timeout: Option<u32>,
    pub timezone_offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStatus {
    pub text: Option<String>,
    pub expires_at: Option<String>,
    pub emoji_id: Option<String>,
    pub emoji_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendSourceFlags {
    pub all: Option<bool>,
    pub mutual_friends: Option<bool>,
    pub mutual_guilds: Option<bool>,
}

/// API methods for DiscordClient
impl DiscordClient {
    /// Get the current user's guilds
    pub async fn get_guilds(&self) -> Result<Vec<Guild>> {
        let response = self.get("/users/@me/guilds").await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let guilds: Vec<Guild> = serde_json::from_str(&body)?;
            Ok(guilds)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get guilds: {}",
                response.status()
            )))
        }
    }

    /// Get a specific guild
    pub async fn get_guild(&self, guild_id: &str) -> Result<Guild> {
        let response = self.get(&format!("/guilds/{}", guild_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let guild: Guild = serde_json::from_str(&body)?;
            Ok(guild)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get guild: {}",
                response.status()
            )))
        }
    }

    /// Get guild channels
    pub async fn get_guild_channels(&self, guild_id: &str) -> Result<Vec<Channel>> {
        let response = self.get(&format!("/guilds/{}/channels", guild_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let channels: Vec<Channel> = serde_json::from_str(&body)?;
            Ok(channels)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get channels: {}",
                response.status()
            )))
        }
    }

    /// Get DM channels
    pub async fn get_dm_channels(&self) -> Result<Vec<Channel>> {
        let response = self.get("/users/@me/channels").await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let channels: Vec<Channel> = serde_json::from_str(&body)?;
            Ok(channels)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get DM channels: {}",
                response.status()
            )))
        }
    }

    /// Get messages from a channel
    pub async fn get_messages(&self, channel_id: &str, limit: u8) -> Result<Vec<Message>> {
        let response = self
            .get(&format!(
                "/channels/{}/messages?limit={}",
                channel_id, limit
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let messages: Vec<Message> = serde_json::from_str(&body)?;
            Ok(messages)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get messages: {}",
                response.status()
            )))
        }
    }

    /// Send a message to a channel
    pub async fn send_message(&self, channel_id: &str, message: CreateMessage) -> Result<Message> {
        let response = self
            .post(&format!("/channels/{}/messages", channel_id), &message)
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let msg: Message = serde_json::from_str(&body)?;
            Ok(msg)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to send message: {}",
                response.status()
            )))
        }
    }

    /// Send a message with file attachments (multipart). Use send_message when no files.
    pub async fn send_message_multipart(
        &self,
        channel_id: &str,
        message: CreateMessage,
        files: &[(String, Vec<u8>, String)], // (filename, bytes, content_type)
    ) -> Result<Message> {
        if files.is_empty() {
            return self.send_message(channel_id, message).await;
        }
        let payload = serde_json::to_value(&message).map_err(|e| {
            DiscordError::Http(format!("Failed to serialize message: {}", e))
        })?;
        let form = super::attachments::build_multipart_message_with_payload(payload, files);
        let response = self
            .post_multipart(&format!("/channels/{}/messages", channel_id), form)
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let msg: Message = serde_json::from_str(&body)?;
            Ok(msg)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to send message (multipart): {}",
                response.status()
            )))
        }
    }

    /// Get the user's relationships (friends, blocked, etc)
    pub async fn get_relationships(&self) -> Result<Vec<Relationship>> {
        let response = self.get("/users/@me/relationships").await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let relationships: Vec<Relationship> = serde_json::from_str(&body)?;
            Ok(relationships)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get relationships: {}",
                response.status()
            )))
        }
    }

    /// Get the user's Discord settings (user accounts only)
    pub async fn get_user_settings(&self) -> Result<UserSettings> {
        let response = self.get("/users/@me/settings").await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let settings: UserSettings = serde_json::from_str(&body)?;
            Ok(settings)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get settings: {}",
                response.status()
            )))
        }
    }

    /// Update the user's presence/status
    pub async fn update_presence(
        &self,
        status: &str,
        custom_status: Option<CustomStatus>,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct SettingsUpdate {
            status: String,
            custom_status: Option<CustomStatus>,
        }

        let update = SettingsUpdate {
            status: status.to_string(),
            custom_status,
        };

        let response = self.patch("/users/@me/settings", &update).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to update presence: {}",
                response.status()
            )))
        }
    }

    /// Get a user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<User> {
        let response = self.get(&format!("/users/{}", user_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let user: User = serde_json::from_str(&body)?;
            Ok(user)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get user: {}",
                response.status()
            )))
        }
    }

    /// Create a DM channel with a user
    pub async fn create_dm(&self, recipient_id: &str) -> Result<Channel> {
        #[derive(Serialize)]
        struct CreateDM {
            recipient_id: String,
        }

        let response = self
            .post(
                "/users/@me/channels",
                &CreateDM {
                    recipient_id: recipient_id.to_string(),
                },
            )
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let channel: Channel = serde_json::from_str(&body)?;
            Ok(channel)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to create DM: {}",
                response.status()
            )))
        }
    }

    /// Delete a message
    pub async fn delete_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        let response = self
            .delete(&format!("/channels/{}/messages/{}", channel_id, message_id))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to delete message: {}",
                response.status()
            )))
        }
    }

    /// Get the gateway URL
    pub async fn get_gateway(&self) -> Result<String> {
        let response = self.get("/gateway").await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let gateway: serde_json::Value = serde_json::from_str(&body)?;
            gateway["url"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| DiscordError::Http("Invalid gateway response".to_string()))
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get gateway: {}",
                response.status()
            )))
        }
    }

    // ==================== User Profile ====================

    /// Get user profile (includes connections, mutual guilds, etc). Returns (profile, raw JSON body).
    pub async fn get_user_profile(&self, user_id: &str) -> Result<(UserProfile, String)> {
        let response = self
            .get(&format!(
                "/users/{}/profile?with_mutual_guilds=true&with_mutual_friends=true",
                user_id
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let profile: UserProfile = serde_json::from_str(&body)?;
            Ok((profile, body))
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get user profile: {}",
                response.status()
            )))
        }
    }

    /// Get user profile in guild context (includes roles, nickname, join date). Returns (profile, raw JSON body).
    pub async fn get_user_profile_in_guild(
        &self,
        user_id: &str,
        guild_id: &str,
    ) -> Result<(UserProfile, String)> {
        let response = self
            .get(&format!(
                "/users/{}/profile?with_mutual_guilds=true&with_mutual_friends=true&guild_id={}",
                user_id, guild_id
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let profile: UserProfile = serde_json::from_str(&body)?;
            Ok((profile, body))
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get user profile: {}",
                response.status()
            )))
        }
    }

    /// Get member roles with full role objects (combines member and guild roles)
    pub async fn get_member_roles(&self, guild_id: &str, user_id: &str) -> Result<Vec<Role>> {
        // Get the member to get their role IDs
        let member = self.get_guild_member(guild_id, user_id).await?;

        // Get all guild roles
        let all_roles = self.get_guild_roles(guild_id).await?;

        // Filter to only the member's roles and sort by position (highest first)
        let mut member_roles: Vec<Role> = all_roles
            .into_iter()
            .filter(|role| member.roles.contains(&role.id))
            .collect();

        member_roles.sort_by(|a, b| b.position.cmp(&a.position));

        Ok(member_roles)
    }

    /// Get user connections (for current user)
    pub async fn get_connections(&self) -> Result<Vec<Connection>> {
        let response = self.get("/users/@me/connections").await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let connections: Vec<Connection> = serde_json::from_str(&body)?;
            Ok(connections)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get connections: {}",
                response.status()
            )))
        }
    }

    // ==================== Guild Members ====================

    /// Get guild members
    pub async fn get_guild_members(&self, guild_id: &str, limit: u16) -> Result<Vec<GuildMember>> {
        let response = self
            .get(&format!("/guilds/{}/members?limit={}", guild_id, limit))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let members: Vec<GuildMember> = serde_json::from_str(&body)?;
            Ok(members)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get guild members: {}",
                response.status()
            )))
        }
    }

    /// Get a specific guild member
    pub async fn get_guild_member(&self, guild_id: &str, user_id: &str) -> Result<GuildMember> {
        let response = self
            .get(&format!("/guilds/{}/members/{}", guild_id, user_id))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let member: GuildMember = serde_json::from_str(&body)?;
            Ok(member)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get guild member: {}",
                response.status()
            )))
        }
    }

    /// Search guild members
    pub async fn search_guild_members(
        &self,
        guild_id: &str,
        query: &str,
        limit: u16,
    ) -> Result<Vec<GuildMember>> {
        let response = self
            .get(&format!(
                "/guilds/{}/members/search?query={}&limit={}",
                guild_id, query, limit
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let members: Vec<GuildMember> = serde_json::from_str(&body)?;
            Ok(members)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to search members: {}",
                response.status()
            )))
        }
    }

    // ==================== Roles ====================

    /// Get guild roles
    pub async fn get_guild_roles(&self, guild_id: &str) -> Result<Vec<Role>> {
        let response = self.get(&format!("/guilds/{}/roles", guild_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let roles: Vec<Role> = serde_json::from_str(&body)?;
            Ok(roles)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get roles: {}",
                response.status()
            )))
        }
    }

    /// Add role to member
    pub async fn add_member_role(
        &self,
        guild_id: &str,
        user_id: &str,
        role_id: &str,
    ) -> Result<()> {
        let response = self
            .put(
                &format!("/guilds/{}/members/{}/roles/{}", guild_id, user_id, role_id),
                &serde_json::json!({}),
            )
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to add role: {}",
                response.status()
            )))
        }
    }

    /// Remove role from member
    pub async fn remove_member_role(
        &self,
        guild_id: &str,
        user_id: &str,
        role_id: &str,
    ) -> Result<()> {
        let response = self
            .delete(&format!(
                "/guilds/{}/members/{}/roles/{}",
                guild_id, user_id, role_id
            ))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to remove role: {}",
                response.status()
            )))
        }
    }

    // ==================== Moderation ====================

    /// Kick a member from a guild
    pub async fn kick_member(
        &self,
        guild_id: &str,
        user_id: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        let endpoint = format!("/guilds/{}/members/{}", guild_id, user_id);
        let url = format!("{}/v{}{}", self.api_base, crate::API_VERSION, endpoint);
        let client = self.inner.read().await;

        let mut request = self.prepare_request(client.delete(&url)).await;

        if let Some(r) = reason {
            request = request.header("X-Audit-Log-Reason", r);
        }

        let response = request
            .send()
            .await
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to kick member: {}",
                response.status()
            )))
        }
    }

    /// Ban a member from a guild
    pub async fn ban_member(
        &self,
        guild_id: &str,
        user_id: &str,
        reason: Option<&str>,
        delete_message_seconds: Option<u32>,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct BanRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            delete_message_seconds: Option<u32>,
        }

        let body = BanRequest {
            delete_message_seconds,
        };
        let endpoint = format!("/guilds/{}/bans/{}", guild_id, user_id);
        let url = format!("{}/v{}{}", self.api_base, crate::API_VERSION, endpoint);
        let client = self.inner.read().await;

        let mut request = self
            .prepare_request(client.put(&url))
            .await
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&body)?);

        if let Some(r) = reason {
            request = request.header("X-Audit-Log-Reason", r);
        }

        let response = request
            .send()
            .await
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to ban member: {}",
                response.status()
            )))
        }
    }

    /// Unban a member from a guild
    pub async fn unban_member(&self, guild_id: &str, user_id: &str) -> Result<()> {
        let response = self
            .delete(&format!("/guilds/{}/bans/{}", guild_id, user_id))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to unban member: {}",
                response.status()
            )))
        }
    }

    /// Timeout a member (communication disabled)
    pub async fn timeout_member(
        &self,
        guild_id: &str,
        user_id: &str,
        until: Option<&str>,
        reason: Option<&str>,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct TimeoutRequest {
            communication_disabled_until: Option<String>,
        }

        let body = TimeoutRequest {
            communication_disabled_until: until.map(|s| s.to_string()),
        };

        let endpoint = format!("/guilds/{}/members/{}", guild_id, user_id);
        let url = format!("{}/v{}{}", self.api_base, crate::API_VERSION, endpoint);
        let client = self.inner.read().await;

        let mut request = self
            .prepare_request(client.patch(&url))
            .await
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&body)?);

        if let Some(r) = reason {
            request = request.header("X-Audit-Log-Reason", r);
        }

        let response = request
            .send()
            .await
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to timeout member: {}",
                response.status()
            )))
        }
    }

    /// Bulk delete messages (2-100 messages, not older than 14 days)
    pub async fn bulk_delete_messages(
        &self,
        channel_id: &str,
        message_ids: Vec<String>,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct BulkDelete {
            messages: Vec<String>,
        }

        let body = BulkDelete {
            messages: message_ids,
        };
        let response = self
            .post(
                &format!("/channels/{}/messages/bulk-delete", channel_id),
                &body,
            )
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to bulk delete: {}",
                response.status()
            )))
        }
    }

    /// Get guild bans
    pub async fn get_guild_bans(&self, guild_id: &str) -> Result<Vec<Ban>> {
        let response = self.get(&format!("/guilds/{}/bans", guild_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let bans: Vec<Ban> = serde_json::from_str(&body)?;
            Ok(bans)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get bans: {}",
                response.status()
            )))
        }
    }

    // ==================== Typing Indicator ====================

    /// Send typing indicator (lasts 10 seconds)
    pub async fn trigger_typing(&self, channel_id: &str) -> Result<()> {
        // Check settings first
        let settings = self.settings.read().await;
        if !settings.send_typing_indicator {
            return Ok(());
        }
        drop(settings);

        let response = self
            .post_empty(&format!("/channels/{}/typing", channel_id))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            // Typing indicator failures are not critical
            Ok(())
        }
    }

    // ==================== Channel Permissions ====================

    /// Get channel permissions for the current user
    pub async fn get_channel(&self, channel_id: &str) -> Result<Channel> {
        let response = self.get(&format!("/channels/{}", channel_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let channel: Channel = serde_json::from_str(&body)?;
            Ok(channel)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get channel: {}",
                response.status()
            )))
        }
    }

    /// Edit channel permissions
    pub async fn edit_channel_permissions(
        &self,
        channel_id: &str,
        overwrite_id: &str,
        allow: u64,
        deny: u64,
        perm_type: u8,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct PermissionOverwrite {
            allow: String,
            deny: String,
            #[serde(rename = "type")]
            perm_type: u8,
        }

        let body = PermissionOverwrite {
            allow: allow.to_string(),
            deny: deny.to_string(),
            perm_type,
        };

        let response = self
            .put(
                &format!("/channels/{}/permissions/{}", channel_id, overwrite_id),
                &body,
            )
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to edit permissions: {}",
                response.status()
            )))
        }
    }

    // ==================== Pins ====================

    /// Get pinned messages
    pub async fn get_pinned_messages(&self, channel_id: &str) -> Result<Vec<Message>> {
        let response = self.get(&format!("/channels/{}/pins", channel_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let messages: Vec<Message> = serde_json::from_str(&body)?;
            Ok(messages)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get pins: {}",
                response.status()
            )))
        }
    }

    /// Pin a message
    pub async fn pin_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        let response = self
            .put(
                &format!("/channels/{}/pins/{}", channel_id, message_id),
                &serde_json::json!({}),
            )
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to pin message: {}",
                response.status()
            )))
        }
    }

    /// Unpin a message
    pub async fn unpin_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        let response = self
            .delete(&format!("/channels/{}/pins/{}", channel_id, message_id))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to unpin message: {}",
                response.status()
            )))
        }
    }
}

// ==================== Additional Data Structures ====================

/// User profile with extended information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user: User,
    #[serde(default)]
    pub connected_accounts: Vec<Connection>,
    #[serde(default)]
    pub mutual_guilds: Vec<MutualGuild>,
    #[serde(default)]
    pub mutual_friends: Vec<User>,
    pub premium_since: Option<String>,
    pub premium_type: Option<u8>,
    pub premium_guild_since: Option<String>,
    pub bio: Option<String>,
    pub banner: Option<String>,
    pub accent_color: Option<u32>,
    /// Guild member info (if viewing in guild context)
    pub guild_member: Option<GuildMemberProfile>,
    /// Guild member profile (alternative field name from API)
    #[serde(default)]
    pub guild_member_profile: Option<GuildMemberProfile>,
}

impl UserProfile {
    /// Get the member's roles in the current guild context
    pub fn get_guild_roles(&self) -> Option<&Vec<String>> {
        self.guild_member
            .as_ref()
            .map(|m| &m.roles)
            .or_else(|| self.guild_member_profile.as_ref().map(|m| &m.roles))
    }

    /// Get the member's nickname in the current guild
    pub fn get_guild_nick(&self) -> Option<&str> {
        self.guild_member
            .as_ref()
            .and_then(|m| m.nick.as_deref())
            .or_else(|| {
                self.guild_member_profile
                    .as_ref()
                    .and_then(|m| m.nick.as_deref())
            })
    }

    /// Get the member's join date
    pub fn get_joined_at(&self) -> Option<&str> {
        self.guild_member
            .as_ref()
            .and_then(|m| m.joined_at.as_deref())
            .or_else(|| {
                self.guild_member_profile
                    .as_ref()
                    .and_then(|m| m.joined_at.as_deref())
            })
    }
}

/// Guild-specific member profile info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMemberProfile {
    #[serde(default)]
    pub guild_id: Option<String>,
    pub nick: Option<String>,
    pub avatar: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    pub joined_at: Option<String>,
    pub premium_since: Option<String>,
    pub pending: Option<bool>,
    pub communication_disabled_until: Option<String>,
    pub flags: Option<u32>,
}

/// Connected account (Twitch, YouTube, etc)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub connection_type: String,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub visibility: u8,
    pub friend_sync: Option<bool>,
    pub show_activity: Option<bool>,
    pub two_way_link: Option<bool>,
}

/// Mutual guild info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualGuild {
    pub id: String,
    pub nick: Option<String>,
}

/// Guild member object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMember {
    pub user: Option<User>,
    pub nick: Option<String>,
    pub avatar: Option<String>,
    pub roles: Vec<String>,
    pub joined_at: String,
    pub premium_since: Option<String>,
    pub deaf: bool,
    pub mute: bool,
    pub pending: Option<bool>,
    pub permissions: Option<String>,
    pub communication_disabled_until: Option<String>,
}

impl GuildMember {
    /// Check if member is timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(ref until) = self.communication_disabled_until {
            // Parse ISO 8601 timestamp and compare with now
            chrono::DateTime::parse_from_rfc3339(until)
                .map(|dt| dt > chrono::Utc::now())
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Get display name (nick or username)
    pub fn display_name(&self) -> String {
        self.nick.clone().unwrap_or_else(|| {
            self.user
                .as_ref()
                .map(|u| u.display_name().to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        })
    }
}

/// Role object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub color: u32,
    pub hoist: bool,
    pub icon: Option<String>,
    pub unicode_emoji: Option<String>,
    pub position: i32,
    pub permissions: String,
    pub managed: bool,
    pub mentionable: bool,
}

impl Role {
    /// Check if role has a specific permission
    pub fn has_permission(&self, permission: Permission) -> bool {
        let perms = self.permissions.parse::<u64>().unwrap_or(0);
        perms & (permission as u64) != 0
    }

    /// Get color as hex string
    pub fn color_hex(&self) -> String {
        if self.color == 0 {
            "#99aab5".to_string() // Default gray for roles without color
        } else {
            format!("#{:06x}", self.color)
        }
    }

    /// Get color as RGB tuple
    pub fn color_rgb(&self) -> (u8, u8, u8) {
        (
            ((self.color >> 16) & 0xFF) as u8,
            ((self.color >> 8) & 0xFF) as u8,
            (self.color & 0xFF) as u8,
        )
    }

    /// Check if this is the @everyone role
    pub fn is_everyone(&self) -> bool {
        // The @everyone role has the same ID as the guild
        self.name == "@everyone"
    }
}

/// Role info for display (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInfo {
    pub id: String,
    pub name: String,
    pub color: String,
    pub position: i32,
}

impl From<&Role> for RoleInfo {
    fn from(role: &Role) -> Self {
        Self {
            id: role.id.clone(),
            name: role.name.clone(),
            color: role.color_hex(),
            position: role.position,
        }
    }
}

/// Ban object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ban {
    pub user: User,
    pub reason: Option<String>,
}

/// Discord permission flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum Permission {
    CreateInstantInvite = 1 << 0,
    KickMembers = 1 << 1,
    BanMembers = 1 << 2,
    Administrator = 1 << 3,
    ManageChannels = 1 << 4,
    ManageGuild = 1 << 5,
    AddReactions = 1 << 6,
    ViewAuditLog = 1 << 7,
    PrioritySpeaker = 1 << 8,
    Stream = 1 << 9,
    ViewChannel = 1 << 10,
    SendMessages = 1 << 11,
    SendTtsMessages = 1 << 12,
    ManageMessages = 1 << 13,
    EmbedLinks = 1 << 14,
    AttachFiles = 1 << 15,
    ReadMessageHistory = 1 << 16,
    MentionEveryone = 1 << 17,
    UseExternalEmojis = 1 << 18,
    ViewGuildInsights = 1 << 19,
    Connect = 1 << 20,
    Speak = 1 << 21,
    MuteMembers = 1 << 22,
    DeafenMembers = 1 << 23,
    MoveMembers = 1 << 24,
    UseVad = 1 << 25,
    ChangeNickname = 1 << 26,
    ManageNicknames = 1 << 27,
    ManageRoles = 1 << 28,
    ManageWebhooks = 1 << 29,
    ManageEmojisAndStickers = 1 << 30,
    UseApplicationCommands = 1 << 31,
    RequestToSpeak = 1 << 32,
    ManageEvents = 1 << 33,
    ManageThreads = 1 << 34,
    CreatePublicThreads = 1 << 35,
    CreatePrivateThreads = 1 << 36,
    UseExternalStickers = 1 << 37,
    SendMessagesInThreads = 1 << 38,
    UseEmbeddedActivities = 1 << 39,
    ModerateMembers = 1 << 40,
}

impl Permission {
    /// Check if a permission value includes this permission
    pub fn check(permissions: u64, permission: Permission) -> bool {
        permissions & (permission as u64) != 0
    }

    /// Check if permissions include Administrator
    pub fn is_admin(permissions: u64) -> bool {
        Self::check(permissions, Permission::Administrator)
    }

    /// Check if user can manage messages
    pub fn can_manage_messages(permissions: u64) -> bool {
        Self::is_admin(permissions) || Self::check(permissions, Permission::ManageMessages)
    }

    /// Check if user can kick members
    pub fn can_kick(permissions: u64) -> bool {
        Self::is_admin(permissions) || Self::check(permissions, Permission::KickMembers)
    }

    /// Check if user can ban members
    pub fn can_ban(permissions: u64) -> bool {
        Self::is_admin(permissions) || Self::check(permissions, Permission::BanMembers)
    }

    /// Check if user can timeout members
    pub fn can_timeout(permissions: u64) -> bool {
        Self::is_admin(permissions) || Self::check(permissions, Permission::ModerateMembers)
    }
}

// ==================== Additional API Methods ====================

/// Edit message request
#[derive(Debug, Clone, Serialize)]
pub struct EditMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_mentions: Option<AllowedMentions>,
}

/// Allowed mentions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedMentions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replied_user: Option<bool>,
}

/// Create channel request
#[derive(Debug, Clone, Serialize)]
pub struct CreateChannel {
    pub name: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
}

/// Edit channel request
#[derive(Debug, Clone, Serialize)]
pub struct EditChannel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

/// Emoji object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildEmoji {
    pub id: String,
    pub name: Option<String>,
    pub roles: Option<Vec<String>>,
    pub user: Option<User>,
    pub require_colons: Option<bool>,
    pub managed: Option<bool>,
    pub animated: Option<bool>,
    pub available: Option<bool>,
}

/// Sticker object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sticker {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<String>,
    #[serde(rename = "type")]
    pub sticker_type: u8,
    pub format_type: u8,
    pub available: Option<bool>,
    pub guild_id: Option<String>,
    pub user: Option<User>,
    pub sort_value: Option<u32>,
}

/// Sticker pack object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickerPack {
    pub id: String,
    pub stickers: Vec<Sticker>,
    pub name: String,
    pub sku_id: String,
    pub cover_sticker_id: Option<String>,
    pub description: String,
    pub banner_asset_id: Option<String>,
}

/// Format types: 1=PNG, 2=APNG, 3=Lottie, 4=GIF. Returns the CDN URL for the sticker image.
pub fn sticker_cdn_url(id: &str, format_type: u8) -> String {
    match format_type {
        4 => format!("https://media.discordapp.net/stickers/{}.gif", id),
        3 => format!("https://cdn.discordapp.com/stickers/{}.png", id), // Lottie: fallback to PNG
        1 | 2 | _ => format!("https://cdn.discordapp.com/stickers/{}.png", id),
    }
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub user_id: Option<String>,
    pub target_id: Option<String>,
    pub action_type: u32,
    pub reason: Option<String>,
    pub changes: Option<Vec<serde_json::Value>>,
    pub options: Option<serde_json::Value>,
}

/// Audit log response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub audit_log_entries: Vec<AuditLogEntry>,
    pub users: Vec<User>,
}

/// Read state ACK request
#[derive(Debug, Clone, Serialize)]
pub struct AckBulkRequest {
    pub read_states: Vec<AckReadState>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AckReadState {
    pub channel_id: String,
    pub message_id: String,
}

impl DiscordClient {
    // ==================== Message Operations ====================

    /// Edit a message
    pub async fn edit_message(
        &self,
        channel_id: &str,
        message_id: &str,
        edit: &EditMessage,
    ) -> Result<Message> {
        let response = self
            .patch(
                &format!("/channels/{}/messages/{}", channel_id, message_id),
                edit,
            )
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let msg: Message = serde_json::from_str(&body)?;
            Ok(msg)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to edit message: {}",
                response.status()
            )))
        }
    }

    /// Get messages before a specific message
    pub async fn get_messages_before(
        &self,
        channel_id: &str,
        before: &str,
        limit: u8,
    ) -> Result<Vec<Message>> {
        let response = self
            .get(&format!(
                "/channels/{}/messages?before={}&limit={}",
                channel_id, before, limit
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let messages: Vec<Message> = serde_json::from_str(&body)?;
            Ok(messages)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get messages: {}",
                response.status()
            )))
        }
    }

    /// Get messages after a specific message
    pub async fn get_messages_after(
        &self,
        channel_id: &str,
        after: &str,
        limit: u8,
    ) -> Result<Vec<Message>> {
        let response = self
            .get(&format!(
                "/channels/{}/messages?after={}&limit={}",
                channel_id, after, limit
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let messages: Vec<Message> = serde_json::from_str(&body)?;
            Ok(messages)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get messages: {}",
                response.status()
            )))
        }
    }

    /// Search messages in a guild
    pub async fn search_guild_messages(
        &self,
        guild_id: &str,
        query: &str,
    ) -> Result<serde_json::Value> {
        let encoded = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
        let response = self
            .get(&format!(
                "/guilds/{}/messages/search?content={}",
                guild_id, encoded
            ))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let result: serde_json::Value = serde_json::from_str(&body)?;
            Ok(result)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to search messages: {}",
                response.status()
            )))
        }
    }

    /// Acknowledge (mark as read) a message
    pub async fn ack_message(&self, channel_id: &str, message_id: &str) -> Result<()> {
        let body = serde_json::json!({ "token": null });
        let response = self
            .post(
                &format!("/channels/{}/messages/{}/ack", channel_id, message_id),
                &body,
            )
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            // ACK failures are non-critical
            Ok(())
        }
    }

    /// Bulk acknowledge multiple channels
    pub async fn ack_bulk(&self, read_states: Vec<AckReadState>) -> Result<()> {
        let body = AckBulkRequest { read_states };
        let response = self.post("/read-states/ack-bulk", &body).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Ok(()) // Non-critical
        }
    }

    // ==================== Channel Management ====================

    /// Create a channel in a guild
    pub async fn create_channel(&self, guild_id: &str, request: &CreateChannel) -> Result<Channel> {
        let response = self
            .post(&format!("/guilds/{}/channels", guild_id), request)
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let channel: Channel = serde_json::from_str(&body)?;
            Ok(channel)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to create channel: {}",
                response.status()
            )))
        }
    }

    /// Edit a channel
    pub async fn edit_channel(&self, channel_id: &str, edit: &EditChannel) -> Result<Channel> {
        let response = self
            .patch(&format!("/channels/{}", channel_id), edit)
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let channel: Channel = serde_json::from_str(&body)?;
            Ok(channel)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to edit channel: {}",
                response.status()
            )))
        }
    }

    /// Delete a channel
    pub async fn delete_channel(&self, channel_id: &str) -> Result<()> {
        let response = self.delete(&format!("/channels/{}", channel_id)).await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to delete channel: {}",
                response.status()
            )))
        }
    }

    // ==================== Emoji & Stickers ====================

    /// List guild emojis
    pub async fn list_guild_emojis(&self, guild_id: &str) -> Result<Vec<GuildEmoji>> {
        let response = self.get(&format!("/guilds/{}/emojis", guild_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let emojis: Vec<GuildEmoji> = serde_json::from_str(&body)?;
            Ok(emojis)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to list emojis: {}",
                response.status()
            )))
        }
    }

    /// Get a specific guild emoji
    pub async fn get_guild_emoji(&self, guild_id: &str, emoji_id: &str) -> Result<GuildEmoji> {
        let response = self
            .get(&format!("/guilds/{}/emojis/{}", guild_id, emoji_id))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let emoji: GuildEmoji = serde_json::from_str(&body)?;
            Ok(emoji)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get emoji: {}",
                response.status()
            )))
        }
    }

    /// Get a sticker
    pub async fn get_sticker(&self, sticker_id: &str) -> Result<Sticker> {
        let response = self.get(&format!("/stickers/{}", sticker_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let sticker: Sticker = serde_json::from_str(&body)?;
            Ok(sticker)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get sticker: {}",
                response.status()
            )))
        }
    }

    /// List sticker packs
    pub async fn list_sticker_packs(&self) -> Result<Vec<StickerPack>> {
        let response = self.get("/sticker-packs").await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let wrapper: serde_json::Value = serde_json::from_str(&body)?;
            if let Some(packs) = wrapper.get("sticker_packs") {
                let packs: Vec<StickerPack> = serde_json::from_value(packs.clone())?;
                Ok(packs)
            } else {
                Ok(vec![])
            }
        } else {
            Err(DiscordError::Http(format!(
                "Failed to list sticker packs: {}",
                response.status()
            )))
        }
    }

    // ==================== User Account Operations ====================

    /// Get user notes
    pub async fn get_note(&self, user_id: &str) -> Result<Option<String>> {
        let response = self.get(&format!("/users/@me/notes/{}", user_id)).await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let note: serde_json::Value = serde_json::from_str(&body)?;
            Ok(note
                .get("note")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string()))
        } else if response.status().as_u16() == 404 {
            Ok(None)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get note: {}",
                response.status()
            )))
        }
    }

    /// Set a note on a user
    pub async fn set_note(&self, user_id: &str, note: &str) -> Result<()> {
        let body = serde_json::json!({ "note": note });
        let response = self
            .put(&format!("/users/@me/notes/{}", user_id), &body)
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to set note: {}",
                response.status()
            )))
        }
    }

    /// Send a friend request by username
    pub async fn send_friend_request(&self, username: &str) -> Result<()> {
        let body = serde_json::json!({ "username": username });
        let response = self.post("/users/@me/relationships", &body).await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to send friend request: {}",
                response.status()
            )))
        }
    }

    /// Accept a friend request
    pub async fn accept_friend_request(&self, user_id: &str) -> Result<()> {
        let body = serde_json::json!({});
        let response = self
            .put(&format!("/users/@me/relationships/{}", user_id), &body)
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to accept friend request: {}",
                response.status()
            )))
        }
    }

    /// Remove a relationship (unfriend, unblock, reject request)
    pub async fn remove_relationship(&self, user_id: &str) -> Result<()> {
        let response = self
            .delete(&format!("/users/@me/relationships/{}", user_id))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to remove relationship: {}",
                response.status()
            )))
        }
    }

    /// Block a user
    pub async fn block_user(&self, user_id: &str) -> Result<()> {
        let body = serde_json::json!({ "type": 2 }); // type 2 = blocked
        let response = self
            .put(&format!("/users/@me/relationships/{}", user_id), &body)
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to block user: {}",
                response.status()
            )))
        }
    }

    // ==================== Audit Log ====================

    /// Get guild audit log
    pub async fn get_audit_log(&self, guild_id: &str, limit: Option<u8>) -> Result<AuditLog> {
        let limit = limit.unwrap_or(50).min(100);
        let response = self
            .get(&format!("/guilds/{}/audit-logs?limit={}", guild_id, limit))
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let log: AuditLog = serde_json::from_str(&body)?;
            Ok(log)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to get audit log: {}",
                response.status()
            )))
        }
    }

    // ==================== Guild Management ====================

    /// Create a guild role
    pub async fn create_guild_role(
        &self,
        guild_id: &str,
        name: &str,
        color: Option<u32>,
        permissions: Option<&str>,
    ) -> Result<Role> {
        let mut body = serde_json::json!({ "name": name });
        if let Some(c) = color {
            body["color"] = serde_json::json!(c);
        }
        if let Some(p) = permissions {
            body["permissions"] = serde_json::json!(p);
        }
        let response = self
            .post(&format!("/guilds/{}/roles", guild_id), &body)
            .await?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| DiscordError::Http(e.to_string()))?;
            let role: Role = serde_json::from_str(&body)?;
            Ok(role)
        } else {
            Err(DiscordError::Http(format!(
                "Failed to create role: {}",
                response.status()
            )))
        }
    }

    /// Delete a guild role
    pub async fn delete_guild_role(&self, guild_id: &str, role_id: &str) -> Result<()> {
        let response = self
            .delete(&format!("/guilds/{}/roles/{}", guild_id, role_id))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to delete role: {}",
                response.status()
            )))
        }
    }

    /// Leave a guild
    pub async fn leave_guild(&self, guild_id: &str) -> Result<()> {
        let response = self
            .delete(&format!("/users/@me/guilds/{}", guild_id))
            .await?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(DiscordError::Http(format!(
                "Failed to leave guild: {}",
                response.status()
            )))
        }
    }
}
