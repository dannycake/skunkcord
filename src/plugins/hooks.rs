// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Gateway event hooks — plugin interface for intercepting events
//!
//! The bridge calls these hooks when processing gateway events. Plugins
//! register handlers; the bridge never imports plugin-specific code.

use crate::bridge::UiUpdate;
use crate::gateway::{GatewayEvent, MessageCreateEvent, MessageDeleteBulkEvent, MessageDeleteEvent, MessageUpdateEvent};
use std::sync::Arc;

/// Result of a message delete handler: either remove (default) or show with content
#[derive(Debug, Clone)]
pub enum MessageDeleteResult {
    /// Remove message from UI (default when no plugin has it)
    Remove,
    /// Show as deleted with cached content
    ShowAsDeleted {
        channel_id: String,
        message_id: String,
        content: String,
        author_name: String,
        author_id: String,
        timestamp: String,
        author_avatar_url: Option<String>,
    },
}

/// Gateway event hooks — plugins implement this to intercept events
pub trait GatewayEventHooks: Send + Sync {
    /// Called when MESSAGE_CREATE is received. Plugin can cache the message.
    fn on_message_create(&self, _event: &MessageCreateEvent) {}

    /// Called when MESSAGE_UPDATE is received. Plugin can record the edit.
    fn on_message_update(&self, _event: &MessageUpdateEvent) {}

    /// Called when MESSAGE_DELETE is received. Returns how to display in UI.
    fn on_message_delete(&self, _event: &MessageDeleteEvent) -> MessageDeleteResult {
        MessageDeleteResult::Remove
    }

    /// Called when MESSAGE_DELETE_BULK is received
    fn on_message_delete_bulk(&self, _event: &MessageDeleteBulkEvent) {}
}

/// No-op hooks when no plugins are enabled
pub struct NoopHooks;

impl GatewayEventHooks for NoopHooks {}

/// Type alias for shared hooks
pub type SharedGatewayHooks = Option<Arc<dyn GatewayEventHooks>>;
