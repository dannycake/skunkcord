// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Plugin loader — discovers and loads plugins from directories and git repos

use crate::plugins::manifest::PluginManifest;
use crate::plugins::runtime::LoadedPlugin;
use directories::ProjectDirs;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Plugin loader that discovers plugins in the plugins directory
pub struct PluginLoader {
    plugins_dir: PathBuf,
    /// Plugin configs: plugin_id -> { key -> value }
    configs: Arc<RwLock<HashMap<String, HashMap<String, JsonValue>>>>,
    /// Enabled plugins: plugin_id -> enabled
    enabled: Arc<RwLock<HashMap<String, bool>>>,
}

impl PluginLoader {
    /// Create a loader using the standard plugins directory.
    /// On mobile (feature "mobile"), returns None — only builtin plugins are used.
    pub fn new() -> Option<Self> {
        #[cfg(feature = "mobile")]
        {
            // Mobile builds use only builtin plugins (virtual plugin system).
            // No user-installed plugins, no git, no filesystem plugin dirs.
            return None;
        }
        #[cfg(not(feature = "mobile"))]
        {
            let dirs = ProjectDirs::from("com", "skunkcord", "Skunkcord")?;
            let plugins_dir = dirs.data_dir().join("plugins");
            std::fs::create_dir_all(&plugins_dir).ok()?;
            Some(Self {
                plugins_dir,
                configs: Arc::new(RwLock::new(HashMap::new())),
                enabled: Arc::new(RwLock::new(HashMap::new())),
            })
        }
    }

    /// Create a loader with a custom plugins directory
    pub fn with_dir(plugins_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&plugins_dir).unwrap_or(());
        Self {
            plugins_dir,
            configs: Arc::new(RwLock::new(HashMap::new())),
            enabled: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the plugins directory path
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    /// Check if a plugin is enabled
    pub async fn is_enabled(&self, plugin_id: &str) -> bool {
        let enabled = self.enabled.read().await;
        enabled.get(plugin_id).copied().unwrap_or(false)
    }

    /// Set plugin enabled state
    pub async fn set_enabled(&self, plugin_id: &str, enabled: bool) {
        let mut e = self.enabled.write().await;
        e.insert(plugin_id.to_string(), enabled);
    }

    /// Discover all plugin directories (including git clones)
    pub fn discover_plugin_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let manifest_path = path.join("plugin.json");
                    if manifest_path.exists() {
                        dirs.push(path);
                    }
                }
            }
        }
        dirs
    }

    /// Load a manifest from a plugin directory
    pub fn load_manifest(&self, plugin_dir: &Path) -> Result<PluginManifest, String> {
        let manifest_path = plugin_dir.join("plugin.json");
        let json = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        serde_json::from_str(&json).map_err(|e| format!("Invalid manifest: {}", e))
    }

    /// Load a plugin
    pub fn load_plugin(
        &self,
        _plugin_dir: &Path,
        manifest: PluginManifest,
        enabled: bool,
    ) -> LoadedPlugin {
        LoadedPlugin::builtin(manifest, enabled)
    }

    /// Install a plugin from a git repository URL
    pub fn install_from_git(&self, repo_url: &str) -> Result<PathBuf, String> {
        let repo_name = repo_url
            .trim_end_matches('/')
            .split('/')
            .last()
            .unwrap_or("plugin")
            .trim_end_matches(".git");
        let target_dir = self.plugins_dir.join(repo_name);

        if target_dir.exists() {
            let status = std::process::Command::new("git")
                .arg("pull")
                .current_dir(&target_dir)
                .status()
                .map_err(|e| format!("Failed to run git pull: {}", e))?;
            if !status.success() {
                return Err("git pull failed".to_string());
            }
        } else {
            let status = std::process::Command::new("git")
                .args(["clone", "--depth", "1", repo_url])
                .arg(&target_dir)
                .status()
                .map_err(|e| format!("Failed to run git clone: {}", e))?;
            if !status.success() {
                return Err("git clone failed".to_string());
            }
        }

        if !target_dir.join("plugin.json").exists() {
            return Err(format!(
                "Plugin installed but plugin.json not found in {}",
                target_dir.display()
            ));
        }

        Ok(target_dir)
    }

    /// Save plugin config
    pub async fn save_plugin_config(
        &self,
        plugin_id: &str,
        config: HashMap<String, JsonValue>,
    ) -> Result<(), String> {
        let mut configs = self.configs.write().await;
        configs.insert(plugin_id.to_string(), config);
        Ok(())
    }

    /// Get plugin config
    pub async fn get_plugin_config(
        &self,
        plugin_id: &str,
    ) -> Option<HashMap<String, JsonValue>> {
        let configs = self.configs.read().await;
        configs.get(plugin_id).cloned()
    }

    /// Check if a plugin directory is a git repo with updates available.
    /// Returns Some((has_update, current_version)) for git repos, None for non-git.
    pub fn check_plugin_updates(&self, plugin_dir: &Path, manifest: &PluginManifest) -> Option<(bool, String)> {
        if !plugin_dir.join(".git").exists() {
            return None;
        }
        let current = manifest.version.clone();
        // Fetch from origin (ignore errors - network may be down)
        let _ = std::process::Command::new("git")
            .args(["fetch", "origin"])
            .current_dir(plugin_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        // Check if we're behind the remote (origin/HEAD points to default branch)
        let out = std::process::Command::new("git")
            .args(["rev-list", "--count", "HEAD..origin/HEAD"])
            .current_dir(plugin_dir)
            .output()
            .ok()?;
        if !out.status.success() {
            // Try origin/main and origin/master as fallbacks
            for branch in ["origin/main", "origin/master"] {
                let out2 = std::process::Command::new("git")
                    .args(["rev-list", "--count", &format!("HEAD..{}", branch)])
                    .current_dir(plugin_dir)
                    .output()
                    .ok()?;
                if out2.status.success() {
                    let s = String::from_utf8_lossy(&out2.stdout).trim().to_string();
                    let behind: u32 = s.parse().unwrap_or(0);
                    return Some((behind > 0, current));
                }
            }
            return Some((false, current));
        }
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let behind: u32 = s.parse().unwrap_or(0);
        Some((behind > 0, current))
    }
}
