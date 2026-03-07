// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Plugin runtime — loads built-in plugins and dispatches events
//!
//! Built-in plugins (message_logger, fake_mute, fake_deafen) are implemented
//! in Rust and gated by plugin enable state. Custom Lua plugins can be added
//! when the Lua runtime is available.

use crate::plugins::manifest::PluginManifest;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A loaded plugin (built-in: Rust logic, custom: Lua when available)
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub enabled: bool,
}

impl LoadedPlugin {
    /// Create a built-in plugin from manifest
    pub fn builtin(manifest: PluginManifest, enabled: bool) -> Self {
        Self { manifest, enabled }
    }
}
