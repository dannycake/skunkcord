// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Message logger gateway event handler

use super::cache::{LoggedMessage, MessageCache};
use crate::plugins::hooks::{GatewayEventHooks, MessageDeleteResult};
use crate::gateway::{MessageCreateEvent, MessageDeleteBulkEvent, MessageDeleteEvent, MessageUpdateEvent};
use std::sync::{Arc, RwLock};

/// Message logger plugin handler — implements GatewayEventHooks
pub struct MessageLoggerHandler {
    cache: Arc<RwLock<MessageCache>>,
}

impl MessageLoggerHandler {
    pub fn new(cache_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(MessageCache::new(cache_size))),
        }
    }

    /// Get the cache (for export, Lua API, etc.)
    pub fn cache(&self) -> Arc<RwLock<MessageCache>> {
        Arc::clone(&self.cache)
    }

    fn message_to_logged(msg: &crate::client::Message, guild_id: Option<&str>) -> LoggedMessage {
        let author_name = msg
            .author
            .as_ref()
            .map(|u| u.display_name().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let author_id = msg
            .author
            .as_ref()
            .map(|u| u.id.clone())
            .unwrap_or_default();
        let attachments_json = serde_json::to_string(&msg.attachments).unwrap_or_else(|_| "[]".to_string());
        let embeds_json = serde_json::to_string(&msg.embeds).unwrap_or_else(|_| "[]".to_string());

        LoggedMessage {
            id: msg.id.clone(),
            channel_id: msg.channel_id.clone(),
            guild_id: guild_id.map(String::from),
            author_id,
            author_name,
            content: msg.content.clone(),
            attachments_json,
            embeds_json,
            timestamp: msg.timestamp.clone(),
            deleted: false,
            deleted_at: None,
            edit_history: vec![],
        }
    }
}

impl GatewayEventHooks for MessageLoggerHandler {
    fn on_message_create(&self, event: &MessageCreateEvent) {
        let logged = Self::message_to_logged(&event.message, event.guild_id.as_deref());
        if let Ok(mut c) = self.cache.write() {
            c.insert(logged);
        }
    }

    fn on_message_update(&self, event: &MessageUpdateEvent) {
        if let Some(ref content) = event.content {
            if let Ok(mut c) = self.cache.write() {
                c.record_edit(&event.id, content);
            }
        }
    }

    fn on_message_delete(&self, event: &MessageDeleteEvent) -> MessageDeleteResult {
        if let Ok(mut c) = self.cache.write() {
            if let Some(msg) = c.mark_deleted(&event.id) {
                return MessageDeleteResult::ShowAsDeleted {
                    channel_id: event.channel_id.clone(),
                    message_id: event.id.clone(),
                    content: msg.content.clone(),
                    author_name: msg.author_name.clone(),
                    author_id: msg.author_id.clone(),
                    timestamp: msg.timestamp.clone(),
                    author_avatar_url: None,
                };
            }
        }
        MessageDeleteResult::Remove
    }

    fn on_message_delete_bulk(&self, event: &MessageDeleteBulkEvent) {
        if let Ok(mut c) = self.cache.write() {
            for id in &event.ids {
                c.mark_deleted(id);
            }
        }
    }
}
