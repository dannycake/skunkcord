// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Advanced Lua plugin system
//!
//! Plugins are Lua scripts loaded from git repositories or the built-in plugins directory.
//! They can subscribe to Discord gateway events, have configurable options with categories,
//! and call into Rust APIs for features like message logging and fake mute.
//!
//! ## Plugin structure
//!
//! Each plugin lives in a directory (e.g. `plugins/message-logger/`) containing:
//! - `plugin.json` — manifest with name, id, version, options schema, event subscriptions
//! - `main.lua` — entry point that registers event handlers
//!
//! ## Options
//!
//! Options are defined in the manifest with categories. Users configure them in the
//! plugin settings UI. Plugins read config via `discord.get_config()`.

use std::sync::Arc;

pub mod api;
pub mod builtin;
pub mod hooks;
pub mod loader;
pub mod manifest;
pub mod message_logger;
pub mod runtime;

pub use builtin::builtin_manifests;
pub use hooks::{GatewayEventHooks, MessageDeleteResult, NoopHooks, SharedGatewayHooks};

/// Result of create_gateway_hooks: (hooks for bridge, optional message cache for export)
pub type GatewayHooksResult = (
    Option<Arc<dyn GatewayEventHooks>>,
    Option<std::sync::Arc<std::sync::RwLock<message_logger::MessageCache>>>,
);

/// Create gateway hooks based on enabled plugins. Called by app_runner.
/// Returns (hooks for bridge, message cache for export when message-logger enabled).
pub fn create_gateway_hooks(
    plugin_enabled: &std::collections::HashMap<String, bool>,
) -> GatewayHooksResult {
    if !plugin_enabled.get("message-logger").copied().unwrap_or(false) {
        return (None, None);
    }
    let cache_size = builtin_manifests()
        .into_iter()
        .find(|m| m.id == "message-logger")
        .and_then(|m| {
            m.options
                .iter()
                .find(|o| o.key == "cache_size")
                .and_then(|o| o.default.as_f64())
        })
        .map(|n| n as usize)
        .unwrap_or(10000)
        .clamp(100, 50000);
    let handler = message_logger::MessageLoggerHandler::new(cache_size);
    let cache = handler.cache();
    (Some(Arc::new(handler)), Some(cache))
}
pub use loader::PluginLoader;
pub use manifest::{PluginManifest, PluginOption, PluginOptionType, OptionCategory};
pub use runtime::LoadedPlugin;

/// Build plugin_id -> manifest map (builtin + user-installed from loader)
pub fn all_manifests() -> std::collections::HashMap<String, PluginManifest> {
    let mut map = std::collections::HashMap::new();
    for m in builtin_manifests() {
        map.insert(m.id.clone(), m);
    }
    if let Some(loader) = PluginLoader::new() {
        for dir in loader.discover_plugin_dirs() {
            if let Ok(m) = loader.load_manifest(&dir) {
                map.insert(m.id.clone(), m);
            }
        }
    }
    map
}

/// Ordered list of plugin manifests for UI (builtins first, then user-installed)
pub fn plugin_list_for_ui() -> Vec<PluginManifest> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for m in builtin_manifests() {
        if seen.insert(m.id.clone()) {
            out.push(m);
        }
    }
    if let Some(loader) = PluginLoader::new() {
        for dir in loader.discover_plugin_dirs() {
            if let Ok(m) = loader.load_manifest(&dir) {
                if seen.insert(m.id.clone()) {
                    out.push(m);
                }
            }
        }
    }
    out
}

/// Result of checking a single plugin for updates
#[derive(Debug, Clone, serde::Serialize)]
pub struct PluginUpdateInfo {
    pub plugin_id: String,
    pub has_update: bool,
    pub current_version: String,
}

/// Check all git-based plugins for available updates.
/// Runs synchronously (git fetch + rev-list); call from a blocking task if needed.
pub fn check_plugin_updates() -> Vec<PluginUpdateInfo> {
    let mut results = Vec::new();
    let Some(loader) = PluginLoader::new() else {
        return results;
    };
    for dir in loader.discover_plugin_dirs() {
        let Ok(manifest) = loader.load_manifest(&dir) else {
            continue;
        };
        if let Some((has_update, current)) = loader.check_plugin_updates(&dir, &manifest) {
            results.push(PluginUpdateInfo {
                plugin_id: manifest.id.clone(),
                has_update,
                current_version: current,
            });
        }
    }
    results
}
