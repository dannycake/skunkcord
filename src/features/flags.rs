// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Feature flags system.
//!
//! Every non-vanilla feature is gated behind a flag. When a flag is OFF,
//! the code path is completely dead — no extra API calls, no gateway
//! processing, no side effects.

use serde::{Deserialize, Serialize};

/// Category for grouping features in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureCategory {
    Infrastructure,
    Security,
    Voice,
    Privacy,
    QualityOfLife,
    Advanced,
}

impl FeatureCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Infrastructure => "Infrastructure",
            Self::Security => "Security & Privacy",
            Self::Voice => "Voice",
            Self::Privacy => "Privacy Features",
            Self::QualityOfLife => "Quality of Life",
            Self::Advanced => "Advanced",
        }
    }
}

/// Metadata about a single feature flag
pub struct FeatureMeta {
    /// Human-readable name
    pub name: &'static str,
    /// Description of what it does
    pub description: &'static str,
    /// Category for UI grouping
    pub category: FeatureCategory,
}

/// Central registry of every toggleable feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    // --- Infrastructure ---
    /// Fetch build number dynamically (vs hardcoded fallback)
    pub dynamic_build_number: bool,
    /// Add human-like jitter between API requests
    pub request_timing_jitter: bool,
    /// Block /science, /track, /metrics endpoints
    pub block_telemetry: bool,

    // --- Captcha ---
    /// Enable captcha auto-detection & widget popup
    pub captcha_handling: bool,

    // --- Voice ---
    /// Enable voice chat
    pub voice_chat: bool,
    /// Enable fake mute (appear muted, still receive audio)
    pub fake_mute: bool,
    /// Enable fake deafen
    pub fake_deafen: bool,

    // --- Security ---
    /// Proxy link preview fetches (vs direct fetch)
    pub safe_link_previews: bool,
    /// Strip EXIF/metadata from images before display
    pub strip_image_metadata: bool,
    /// Block data: URIs in messages
    pub block_data_uris: bool,

    // --- Client Mod Features ---
    /// arRPC game activity detection
    pub arrpc: bool,
    /// arRPC process scanning
    pub arrpc_process_scan: bool,
    /// Show channels user can't access
    pub show_hidden_channels: bool,
    /// Silent message toggle button
    pub silent_message_toggle: bool,
    /// Strip tracking params from URLs
    pub clear_urls: bool,
    /// Default no-ping on replies
    pub no_reply_mention: bool,
    /// Unlock experiments panel
    pub experiments_panel: bool,
    /// Bulk read-all notifications
    pub read_all_button: bool,
    /// Pin DMs to top
    pub pin_dms: bool,
    /// Always animate avatars/emojis
    pub always_animate: bool,
    /// Double-click to join voice
    pub voice_double_click: bool,

    // --- Client Essentials ---
    /// Keep background gateway connections for other accounts
    pub background_account_connections: bool,
    /// Slash command autocomplete & interactions
    pub slash_commands: bool,
    /// GIF picker (Tenor integration)
    pub gif_picker: bool,
    /// Desktop notifications
    pub desktop_notifications: bool,
    /// Notification sounds
    pub notification_sounds: bool,
    /// Streamer mode (auto-detect + manual)
    pub streamer_mode: bool,
    /// Global keyboard shortcuts (outside window focus)
    pub global_keybinds: bool,
}

impl FeatureFlags {
    /// PARANOID MODE: Nothing non-standard enabled.
    /// Client behaves exactly like vanilla Discord web client.
    /// Zero detection surface beyond the fingerprint itself.
    pub fn paranoid() -> Self {
        Self {
            // These make you LESS detectable, not more:
            dynamic_build_number: true,
            request_timing_jitter: true,
            captcha_handling: true,
            block_telemetry: true,
            safe_link_previews: true,
            strip_image_metadata: true,
            block_data_uris: true,
            // Normal client features (vanilla behavior):
            voice_chat: true,
            slash_commands: true,
            desktop_notifications: true,
            notification_sounds: true,
            global_keybinds: true,
            gif_picker: true,
            streamer_mode: true,
            // Everything non-vanilla OFF:
            fake_mute: false,
            fake_deafen: false,
            arrpc: false,
            arrpc_process_scan: false,
            show_hidden_channels: false,
            silent_message_toggle: false,
            clear_urls: false,
            no_reply_mention: false,
            experiments_panel: false,
            read_all_button: false,
            pin_dms: false,
            always_animate: false,
            voice_double_click: false,
            background_account_connections: false,
        }
    }

    /// Sensible defaults.
    pub fn standard() -> Self {
        Self {
            // Infrastructure (all safe)
            dynamic_build_number: true,
            request_timing_jitter: true,
            block_telemetry: true,

            // Captcha
            captcha_handling: true,

            // Voice
            voice_chat: true,
            fake_mute: false,
            fake_deafen: false,

            // Security
            safe_link_previews: true,
            strip_image_metadata: true,
            block_data_uris: true,

            // Safe mod features
            arrpc: false,
            arrpc_process_scan: false,
            show_hidden_channels: false,
            silent_message_toggle: true,
            clear_urls: true,
            no_reply_mention: false,
            experiments_panel: false,
            read_all_button: true,
            pin_dms: true,
            always_animate: false,
            voice_double_click: false,

            // Client essentials
            background_account_connections: false,
            slash_commands: true,
            gif_picker: true,
            desktop_notifications: true,
            notification_sounds: true,
            streamer_mode: true,
            global_keybinds: true,
        }
    }

    /// FULL: Everything enabled. Maximum features, higher detection surface.
    pub fn full() -> Self {
        Self {
            dynamic_build_number: true,
            request_timing_jitter: true,
            block_telemetry: true,
            captcha_handling: true,
            voice_chat: true,
            fake_mute: true,
            fake_deafen: true,
            safe_link_previews: true,
            strip_image_metadata: true,
            block_data_uris: true,
            arrpc: true,
            arrpc_process_scan: true,
            show_hidden_channels: true,
            silent_message_toggle: true,
            clear_urls: true,
            no_reply_mention: true,
            experiments_panel: true,
            read_all_button: true,
            pin_dms: true,
            always_animate: true,
            voice_double_click: true,
            background_account_connections: true,
            slash_commands: true,
            gif_picker: true,
            desktop_notifications: true,
            notification_sounds: true,
            streamer_mode: true,
            global_keybinds: true,
        }
    }

    /// Get a list of all enabled non-default flag names
    pub fn enabled_flags(&self) -> Vec<&'static str> {
        let mut flags = Vec::new();
        // Only list flags that are ON and represent non-vanilla behavior
        if self.fake_mute {
            flags.push("fake_mute");
        }
        if self.fake_deafen {
            flags.push("fake_deafen");
        }
        if self.arrpc {
            flags.push("arrpc");
        }
        if self.arrpc_process_scan {
            flags.push("arrpc_process_scan");
        }
        if self.show_hidden_channels {
            flags.push("show_hidden_channels");
        }
        if self.silent_message_toggle {
            flags.push("silent_message_toggle");
        }
        if self.clear_urls {
            flags.push("clear_urls");
        }
        if self.no_reply_mention {
            flags.push("no_reply_mention");
        }
        if self.experiments_panel {
            flags.push("experiments_panel");
        }
        if self.read_all_button {
            flags.push("read_all_button");
        }
        if self.pin_dms {
            flags.push("pin_dms");
        }
        if self.always_animate {
            flags.push("always_animate");
        }
        if self.voice_double_click {
            flags.push("voice_double_click");
        }
        if self.background_account_connections {
            flags.push("background_account_connections");
        }

        flags
    }

    /// Get metadata for all feature flags
    pub fn all_metadata() -> Vec<(&'static str, FeatureMeta)> {
        vec![
            ("dynamic_build_number", FeatureMeta {
                name: "Dynamic Build Number",
                description: "Fetch current Discord build number instead of using a stale hardcoded value",
                category: FeatureCategory::Infrastructure,
            }),
            ("request_timing_jitter", FeatureMeta {
                name: "Request Timing Jitter",
                description: "Add human-like random delays between API requests",
                category: FeatureCategory::Infrastructure,
            }),
            ("block_telemetry", FeatureMeta {
                name: "Block Telemetry",
                description: "Block /science, /track, /metrics endpoints (standard in all client mods)",
                category: FeatureCategory::Security,
            }),
            ("captcha_handling", FeatureMeta {
                name: "Captcha Handling",
                description: "Auto-detect and display hCaptcha challenges",
                category: FeatureCategory::Infrastructure,
            }),
            ("voice_chat", FeatureMeta {
                name: "Voice Chat",
                description: "Enable voice channel connections",
                category: FeatureCategory::Voice,
            }),
            ("fake_mute", FeatureMeta {
                name: "Fake Mute",
                description: "Appear muted to others while still receiving audio",
                category: FeatureCategory::Advanced,
            }),
            ("fake_deafen", FeatureMeta {
                name: "Fake Deafen",
                description: "Appear deafened to others while still receiving audio",
                category: FeatureCategory::Advanced,
            }),
            ("safe_link_previews", FeatureMeta {
                name: "Safe Link Previews",
                description: "Proxy link preview fetches to prevent IP leakage",
                category: FeatureCategory::Security,
            }),
            ("arrpc", FeatureMeta {
                name: "arRPC (Rich Presence)",
                description: "Game activity detection via local RPC server",
                category: FeatureCategory::QualityOfLife,
            }),
            ("arrpc_process_scan", FeatureMeta {
                name: "arRPC Process Scanning",
                description: "Automatically detect running games for Rich Presence",
                category: FeatureCategory::QualityOfLife,
            }),
            ("show_hidden_channels", FeatureMeta {
                name: "Show Hidden Channels",
                description: "Display channels you don't have access to view",
                category: FeatureCategory::Advanced,
            }),
            ("silent_message_toggle", FeatureMeta {
                name: "Silent Message Toggle",
                description: "Button to send messages without triggering notifications",
                category: FeatureCategory::QualityOfLife,
            }),
            ("clear_urls", FeatureMeta {
                name: "Clear URLs",
                description: "Strip tracking parameters (utm_*, fbclid, etc.) from outgoing URLs",
                category: FeatureCategory::Privacy,
            }),
            ("no_reply_mention", FeatureMeta {
                name: "No Reply Mention",
                description: "Replies don't ping the original author by default",
                category: FeatureCategory::QualityOfLife,
            }),
            ("experiments_panel", FeatureMeta {
                name: "Experiments Panel",
                description: "Unlock Discord's hidden experiments and developer features",
                category: FeatureCategory::Advanced,
            }),
            ("read_all_button", FeatureMeta {
                name: "Read All Notifications",
                description: "One-click button to mark all channels as read",
                category: FeatureCategory::QualityOfLife,
            }),
            ("pin_dms", FeatureMeta {
                name: "Pin DMs",
                description: "Pin DM conversations to the top of the list",
                category: FeatureCategory::QualityOfLife,
            }),
            ("always_animate", FeatureMeta {
                name: "Always Animate",
                description: "Force animated avatars, emojis, and stickers to always play",
                category: FeatureCategory::QualityOfLife,
            }),
            ("voice_double_click", FeatureMeta {
                name: "Double-Click Voice Join",
                description: "Require double-click to join voice channels (prevent accidents)",
                category: FeatureCategory::Voice,
            }),
            ("background_account_connections", FeatureMeta {
                name: "Background Account Connections",
                description: "Keep gateway connections alive for non-active accounts (notification bridging)",
                category: FeatureCategory::Advanced,
            }),
            ("slash_commands", FeatureMeta {
                name: "Slash Commands",
                description: "Autocomplete and send slash commands to bots",
                category: FeatureCategory::QualityOfLife,
            }),
            ("gif_picker", FeatureMeta {
                name: "GIF Picker",
                description: "Search and send GIFs via Tenor integration",
                category: FeatureCategory::QualityOfLife,
            }),
            ("desktop_notifications", FeatureMeta {
                name: "Desktop Notifications",
                description: "OS-native notification popups for messages",
                category: FeatureCategory::QualityOfLife,
            }),
            ("notification_sounds", FeatureMeta {
                name: "Notification Sounds",
                description: "Play sounds for messages, calls, and other events",
                category: FeatureCategory::QualityOfLife,
            }),
            ("streamer_mode", FeatureMeta {
                name: "Streamer Mode",
                description: "Auto-detect streaming software and hide sensitive info",
                category: FeatureCategory::Privacy,
            }),
            ("global_keybinds", FeatureMeta {
                name: "Global Keybinds",
                description: "Keyboard shortcuts that work even when the window isn't focused",
                category: FeatureCategory::QualityOfLife,
            }),
        ]
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paranoid_fake_mute_off() {
        let flags = FeatureFlags::paranoid();
        assert!(!flags.fake_mute);
        assert!(!flags.fake_deafen);
        assert!(!flags.arrpc);
        assert!(!flags.arrpc);
        assert!(!flags.show_hidden_channels);
        assert!(!flags.experiments_panel);
    }

    #[test]
    fn test_all_metadata_covers_features() {
        let meta = FeatureFlags::all_metadata();
        let keys: Vec<&str> = meta.iter().map(|(k, _)| *k).collect();
        assert!(keys.contains(&"fake_mute"));
        assert!(keys.contains(&"fake_deafen"));
        assert!(keys.contains(&"arrpc"));
        assert!(keys.contains(&"arrpc"));
        assert!(keys.contains(&"show_hidden_channels"));
    }

    #[test]
    fn test_default_is_standard() {
        let default = FeatureFlags::default();
        let standard = FeatureFlags::standard();
        // Check a few representative fields
        assert_eq!(default.fake_mute, standard.fake_mute);
        assert_eq!(default.block_telemetry, standard.block_telemetry);
        assert_eq!(default.arrpc, standard.arrpc);
        assert_eq!(default.clear_urls, standard.clear_urls);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let flags = FeatureFlags::full();
        let json = serde_json::to_string(&flags).unwrap();
        let deserialized: FeatureFlags = serde_json::from_str(&json).unwrap();
        assert_eq!(flags.fake_mute, deserialized.fake_mute);
        assert_eq!(flags.arrpc, deserialized.arrpc);
        assert_eq!(flags.block_telemetry, deserialized.block_telemetry);
    }
}
