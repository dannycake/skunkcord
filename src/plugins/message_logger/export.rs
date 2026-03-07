// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Export logged messages to JSON/CSV

use super::LoggedMessage;
use serde::Serialize;

/// Export format
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
}

/// Export messages to JSON string
pub fn export_json(messages: &[&LoggedMessage]) -> String {
    serde_json::to_string_pretty(messages).unwrap_or_else(|_| "[]".to_string())
}

/// Export messages to CSV string
pub fn export_csv(messages: &[&LoggedMessage]) -> String {
    let mut csv = String::new();
    csv.push_str("id,channel_id,guild_id,author_id,author_name,content,timestamp,deleted,deleted_at,edit_count\n");
    for msg in messages {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            escape_csv(&msg.id),
            escape_csv(&msg.channel_id),
            escape_csv(msg.guild_id.as_deref().unwrap_or("")),
            escape_csv(&msg.author_id),
            escape_csv(&msg.author_name),
            escape_csv(&msg.content),
            escape_csv(&msg.timestamp),
            msg.deleted,
            escape_csv(msg.deleted_at.as_deref().unwrap_or("")),
            msg.edit_history.len(),
        ));
    }
    csv
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Summary statistics for the message log
#[derive(Debug, Clone, Serialize)]
pub struct LogStats {
    pub total_messages: usize,
    pub deleted_messages: usize,
    pub edited_messages: usize,
    pub unique_authors: usize,
    pub unique_channels: usize,
    pub unique_guilds: usize,
}

/// Calculate statistics from logged messages
pub fn calculate_stats(messages: &[&LoggedMessage]) -> LogStats {
    use std::collections::HashSet;
    let mut authors = HashSet::new();
    let mut channels = HashSet::new();
    let mut guilds = HashSet::new();
    let mut deleted = 0;
    let mut edited = 0;
    for msg in messages {
        authors.insert(&msg.author_id);
        channels.insert(&msg.channel_id);
        if let Some(ref gid) = msg.guild_id {
            guilds.insert(gid);
        }
        if msg.deleted {
            deleted += 1;
        }
        if !msg.edit_history.is_empty() {
            edited += 1;
        }
    }
    LogStats {
        total_messages: messages.len(),
        deleted_messages: deleted,
        edited_messages: edited,
        unique_authors: authors.len(),
        unique_channels: channels.len(),
        unique_guilds: guilds.len(),
    }
}
