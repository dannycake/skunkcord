// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Storage module for session persistence
//!
//! Handles saving and loading session data to disk. Session data is encrypted at rest
//! using AES-256-GCM with a key derived from the machine ID.

mod encryption;

use crate::client::Session;
use crate::features::FeatureFlags;
use crate::proxy::ProxyConfig;
use crate::{DiscordError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Storage manager for persistent data
#[derive(Clone)]
pub struct Storage {
    /// Base directory for storage
    data_dir: PathBuf,
    /// Legacy sessions file path (plain JSON; migrated to encrypted on first read)
    sessions_path: PathBuf,
    /// Encrypted sessions file path (AES-256-GCM)
    sessions_enc_path: PathBuf,
    /// Settings file path
    settings_path: PathBuf,
}

impl Storage {
    /// Create a new storage manager
    pub fn new() -> Result<Self> {
        let project_dirs =
            ProjectDirs::from("com", "skunkcord", "Skunkcord").ok_or_else(|| {
                DiscordError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not determine data directory",
                ))
            })?;

        let data_dir = project_dirs.data_dir().to_path_buf();
        fs::create_dir_all(&data_dir)?;

        let sessions_path = data_dir.join("sessions.json");
        let sessions_enc_path = data_dir.join("sessions.enc");
        let settings_path = data_dir.join("settings.json");

        Ok(Self {
            data_dir,
            sessions_path,
            sessions_enc_path,
            settings_path,
        })
    }

    /// Create storage at a custom path
    pub fn at_path(path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&path)?;

        let sessions_path = path.join("sessions.json");
        let sessions_enc_path = path.join("sessions.enc");
        let settings_path = path.join("settings.json");

        Ok(Self {
            data_dir: path,
            sessions_path,
            sessions_enc_path,
            settings_path,
        })
    }

    /// Get the data directory path
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    /// Save a session (encrypted to disk)
    pub fn save_session(&self, session: &Session) -> Result<()> {
        let mut sessions = self.load_all_sessions()?;
        sessions.insert(session.user_id.clone(), session.clone());
        self.write_sessions_encrypted(&sessions)?;
        Ok(())
    }

    /// Load all sessions (from encrypted file, or migrate from plain JSON)
    pub fn load_all_sessions(&self) -> Result<HashMap<String, Session>> {
        if self.sessions_enc_path.exists() {
            let data = fs::read(&self.sessions_enc_path)?;
            let plain = encryption::decrypt(&data)?;
            let sessions: HashMap<String, Session> = serde_json::from_slice(&plain)?;
            return Ok(sessions);
        }
        if self.sessions_path.exists() {
            let data = fs::read_to_string(&self.sessions_path)?;
            let sessions: HashMap<String, Session> = serde_json::from_str(&data)?;
            self.write_sessions_encrypted(&sessions)?;
            let _ = fs::remove_file(&self.sessions_path);
            return Ok(sessions);
        }
        Ok(HashMap::new())
    }

    /// Write sessions map to encrypted file.
    fn write_sessions_encrypted(&self, sessions: &HashMap<String, Session>) -> Result<()> {
        let json = serde_json::to_string_pretty(sessions)?;
        let ciphertext = encryption::encrypt(json.as_bytes())?;
        fs::write(&self.sessions_enc_path, ciphertext)?;
        Ok(())
    }

    /// Load a specific session by user ID
    pub fn load_session(&self, user_id: &str) -> Result<Option<Session>> {
        let sessions = self.load_all_sessions()?;
        Ok(sessions.get(user_id).cloned())
    }

    /// Delete a session
    pub fn delete_session(&self, user_id: &str) -> Result<()> {
        let mut sessions = self.load_all_sessions()?;
        sessions.remove(user_id);
        self.write_sessions_encrypted(&sessions)?;
        Ok(())
    }

    /// Clear all sessions (removes both legacy and encrypted files)
    pub fn clear_sessions(&self) -> Result<()> {
        if self.sessions_path.exists() {
            fs::remove_file(&self.sessions_path)?;
        }
        if self.sessions_enc_path.exists() {
            fs::remove_file(&self.sessions_enc_path)?;
        }
        Ok(())
    }

    /// Save application settings
    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let json = serde_json::to_string_pretty(settings)?;
        fs::write(&self.settings_path, json)?;
        Ok(())
    }

    /// Load application settings
    pub fn load_settings(&self) -> Result<AppSettings> {
        if !self.settings_path.exists() {
            return Ok(AppSettings::default());
        }

        let data = fs::read_to_string(&self.settings_path)?;
        let settings: AppSettings = serde_json::from_str(&data)?;

        Ok(settings)
    }
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Theme (dark, light)
    pub theme: String,
    /// Window width
    pub window_width: u32,
    /// Window height
    pub window_height: u32,
    /// Start minimized
    pub start_minimized: bool,
    /// Close to tray
    pub close_to_tray: bool,
    /// Hardware acceleration
    pub hardware_acceleration: bool,
    /// Notifications enabled
    pub notifications: bool,
    /// Sound effects enabled
    pub sounds: bool,
    /// Last selected account ID
    pub last_account_id: Option<String>,
    /// Custom fingerprint settings
    pub fingerprint_settings: FingerprintSettings,
    /// Client behavior settings
    pub client_settings: ClientBehaviorSettings,
    /// Proxy settings
    #[serde(default)]
    pub proxy_settings: ProxySettings,
    /// Feature flags (safety profiles)
    #[serde(default)]
    pub feature_flags: FeatureFlags,
    /// Plugin enable state: plugin_id -> enabled (overrides feature flags for these)
    #[serde(default)]
    pub plugin_enabled: std::collections::HashMap<String, bool>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            window_width: 1280,
            window_height: 720,
            start_minimized: false,
            close_to_tray: false,
            hardware_acceleration: true,
            notifications: true,
            sounds: true,
            last_account_id: None,
            fingerprint_settings: FingerprintSettings::default(),
            client_settings: ClientBehaviorSettings::default(),
            proxy_settings: ProxySettings::default(),
            feature_flags: FeatureFlags::default(),
            plugin_enabled: std::collections::HashMap::new(),
        }
    }
}

/// Client behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientBehaviorSettings {
    /// Send typing indicators when typing
    pub send_typing_indicator: bool,
    /// Show typing indicators from others
    pub show_typing_indicators: bool,
    /// Auto-reconnect on disconnect
    pub auto_reconnect: bool,
    /// Message fetch limit per request
    pub message_fetch_limit: u8,
    /// Show message embeds
    pub show_embeds: bool,
    /// Compact message display mode
    pub compact_mode: bool,
    /// Show message timestamps
    pub show_timestamps: bool,
    /// Developer mode (show IDs)
    pub developer_mode: bool,
    /// Confirm before kicking
    pub confirm_kick: bool,
    /// Confirm before banning
    pub confirm_ban: bool,
    /// Confirm before deleting messages
    pub confirm_delete: bool,
    /// Default ban message delete duration (seconds)
    pub default_ban_delete_seconds: u32,
    /// How to display deleted messages (message logger): "strikethrough", "faded", "deleted"
    #[serde(default)]
    pub deleted_message_style: String,
}

impl Default for ClientBehaviorSettings {
    fn default() -> Self {
        Self {
            send_typing_indicator: true,
            show_typing_indicators: true,
            auto_reconnect: true,
            message_fetch_limit: 50,
            show_embeds: true,
            compact_mode: false,
            show_timestamps: true,
            developer_mode: false,
            confirm_kick: true,
            confirm_ban: true,
            confirm_delete: false,
            default_ban_delete_seconds: 0,
            deleted_message_style: "strikethrough".to_string(),
        }
    }
}

/// Fingerprint customization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintSettings {
    /// Use random fingerprint for each session
    pub randomize_each_session: bool,
    /// Browser type to emulate
    pub browser_type: BrowserType,
    /// Custom user agent (if set)
    pub custom_user_agent: Option<String>,
    /// Custom OS
    pub custom_os: Option<String>,
}

impl Default for FingerprintSettings {
    fn default() -> Self {
        Self {
            randomize_each_session: false,
            browser_type: BrowserType::Chrome,
            custom_user_agent: None,
            custom_os: None,
        }
    }
}

/// Proxy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for ProxySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "127.0.0.1".to_string(),
            port: 1080,
            username: None,
            password: None,
        }
    }
}

impl ProxySettings {
    pub fn to_proxy_config(&self) -> Option<ProxyConfig> {
        if !self.enabled {
            return None;
        }
        Some(ProxyConfig {
            enabled: true,
            host: self.host.clone(),
            port: self.port,
            username: self.username.clone(),
            password: self.password.clone(),
        })
    }
}

/// Browser types for emulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Edge,
    Safari,
}

impl BrowserType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Chrome => "Chrome",
            Self::Firefox => "Firefox",
            Self::Edge => "Edge",
            Self::Safari => "Safari",
        }
    }
}

/// Cached data for offline access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedData {
    /// Cached guilds
    pub guilds: Vec<CachedGuild>,
    /// Cached DM channels
    pub dm_channels: Vec<CachedChannel>,
    /// Cached users
    pub users: HashMap<String, CachedUser>,
    /// Last update timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedGuild {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedChannel {
    pub id: String,
    pub name: Option<String>,
    pub recipient_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
}

impl Storage {
    /// Save cached data for a user
    pub fn save_cache(&self, user_id: &str, cache: &CachedData) -> Result<()> {
        let cache_path = self.data_dir.join(format!("cache_{}.json", user_id));
        let json = serde_json::to_string(cache)?;
        fs::write(cache_path, json)?;
        Ok(())
    }

    /// Load cached data for a user
    pub fn load_cache(&self, user_id: &str) -> Result<Option<CachedData>> {
        let cache_path = self.data_dir.join(format!("cache_{}.json", user_id));

        if !cache_path.exists() {
            return Ok(None);
        }

        let data = fs::read_to_string(cache_path)?;
        let cache: CachedData = serde_json::from_str(&data)?;

        Ok(Some(cache))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::BrowserFingerprint;
    use tempfile::tempdir;

    #[test]
    fn test_storage_sessions() {
        let temp = tempdir().unwrap();
        let storage = Storage::at_path(temp.path().to_path_buf()).unwrap();

        let session = Session::new(
            "test_token".to_string(),
            "123456".to_string(),
            HashMap::new(),
            HashMap::new(),
            BrowserFingerprint::default(),
        );

        storage.save_session(&session).unwrap();

        let loaded = storage.load_session("123456").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().token, "test_token");

        storage.delete_session("123456").unwrap();
        let loaded = storage.load_session("123456").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_migration_plain_to_encrypted() {
        let temp = tempdir().unwrap();
        let path = temp.path().to_path_buf();
        let sessions_path = path.join("sessions.json");
        let sessions_enc_path = path.join("sessions.enc");

        let session = Session::new(
            "migrate_token".to_string(),
            "789".to_string(),
            HashMap::new(),
            HashMap::new(),
            BrowserFingerprint::default(),
        );
        let mut sessions = HashMap::new();
        sessions.insert("789".to_string(), session);
        let json = serde_json::to_string_pretty(&sessions).unwrap();
        fs::write(&sessions_path, json).unwrap();
        assert!(sessions_path.exists());
        assert!(!sessions_enc_path.exists());

        let storage = Storage::at_path(path).unwrap();
        let loaded = storage.load_all_sessions().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.get("789").unwrap().token, "migrate_token");
        assert!(!sessions_path.exists(), "legacy sessions.json should be removed after migration");
        assert!(sessions_enc_path.exists(), "sessions.enc should exist after migration");
    }
}
