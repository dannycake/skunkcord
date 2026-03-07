// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Plugin API — types and hooks for plugin integration
//!
//! When Lua support is enabled (--features plugins), this module exposes
//! the full Lua API. Without it, plugins are identified by manifest and
//! the bridge gates behavior by plugin enable state.

use crate::plugins::message_logger::{LoggedMessage, MessageCache};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Plugin context for built-in plugins (no Lua)
pub struct PluginContext {
    pub plugin_id: String,
    pub config: Arc<RwLock<HashMap<String, JsonValue>>>,
    pub message_cache: Arc<RwLock<MessageCache>>,
}
