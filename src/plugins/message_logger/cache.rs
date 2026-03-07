// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Message cache for the message logger plugin

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A logged message with metadata about deletions/edits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggedMessage {
    pub id: String,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub attachments_json: String,
    pub embeds_json: String,
    pub timestamp: String,
    pub deleted: bool,
    pub deleted_at: Option<String>,
    pub edit_history: Vec<MessageEdit>,
}

/// A single edit record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEdit {
    pub old_content: String,
    pub edited_at: String,
}

/// In-memory message cache
#[derive(Debug, Clone, Default)]
pub struct MessageCache {
    messages: HashMap<String, LoggedMessage>,
    order: Vec<String>,
    max_size: usize,
}

impl MessageCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: HashMap::new(),
            order: Vec::new(),
            max_size,
        }
    }

    pub fn insert(&mut self, msg: LoggedMessage) {
        let id = msg.id.clone();
        if self.messages.contains_key(&id) {
            self.messages.insert(id.clone(), msg);
            self.order.retain(|oid| oid != &id);
            self.order.push(id);
        } else {
            while self.messages.len() >= self.max_size && !self.order.is_empty() {
                if let Some(oldest) = self.order.first().cloned() {
                    if let Some(m) = self.messages.get(&oldest) {
                        if m.deleted || !m.edit_history.is_empty() {
                            self.order.remove(0);
                            self.order.push(oldest);
                            continue;
                        }
                    }
                    self.order.remove(0);
                    self.messages.remove(&oldest);
                }
            }
            self.messages.insert(id.clone(), msg);
            self.order.push(id);
        }
    }

    pub fn get(&self, message_id: &str) -> Option<&LoggedMessage> {
        self.messages.get(message_id)
    }

    pub fn mark_deleted(&mut self, message_id: &str) -> Option<&LoggedMessage> {
        if let Some(msg) = self.messages.get_mut(message_id) {
            msg.deleted = true;
            msg.deleted_at = Some(chrono::Utc::now().to_rfc3339());
            return Some(msg);
        }
        None
    }

    pub fn record_edit(&mut self, message_id: &str, new_content: &str) -> Option<&LoggedMessage> {
        if let Some(msg) = self.messages.get_mut(message_id) {
            let old_content = msg.content.clone();
            msg.edit_history.push(MessageEdit {
                old_content,
                edited_at: chrono::Utc::now().to_rfc3339(),
            });
            msg.content = new_content.to_string();
            return Some(msg);
        }
        None
    }

    pub fn deleted_in_channel(&self, channel_id: &str) -> Vec<&LoggedMessage> {
        self.messages
            .values()
            .filter(|m| m.channel_id == channel_id && m.deleted)
            .collect()
    }

    pub fn edited_in_channel(&self, channel_id: &str) -> Vec<&LoggedMessage> {
        self.messages
            .values()
            .filter(|m| m.channel_id == channel_id && !m.edit_history.is_empty())
            .collect()
    }

    pub fn logged_in_guild(&self, guild_id: &str) -> Vec<&LoggedMessage> {
        self.messages
            .values()
            .filter(|m| {
                m.guild_id.as_deref() == Some(guild_id)
                    && (m.deleted || !m.edit_history.is_empty())
            })
            .collect()
    }

    pub fn search(&self, query: &str) -> Vec<&LoggedMessage> {
        let q = query.to_lowercase();
        self.messages
            .values()
            .filter(|m| {
                (m.deleted || !m.edit_history.is_empty())
                    && (m.content.to_lowercase().contains(&q)
                        || m.edit_history.iter().any(|e| e.old_content.to_lowercase().contains(&q)))
            })
            .collect()
    }

    /// All messages in insertion order (oldest first)
    pub fn all(&self) -> Vec<&LoggedMessage> {
        self.order
            .iter()
            .filter_map(|id| self.messages.get(id))
            .collect()
    }

    pub fn total_count(&self) -> usize {
        self.messages.len()
    }

    pub fn deleted_count(&self) -> usize {
        self.messages.values().filter(|m| m.deleted).count()
    }

    pub fn edited_count(&self) -> usize {
        self.messages
            .values()
            .filter(|m| !m.edit_history.is_empty())
            .count()
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.order.clear();
    }

    pub fn clear_older_than(&mut self, days: i64) {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
        let cutoff_str = cutoff.to_rfc3339();
        self.messages.retain(|_, m| m.timestamp > cutoff_str);
        let ids: std::collections::HashSet<_> = self.messages.keys().cloned().collect();
        self.order.retain(|id| ids.contains(id));
    }
}
