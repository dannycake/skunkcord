// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Plugin manifest schema
//!
//! Defines plugin metadata, options with categories, and event subscriptions.

use serde::{Deserialize, Serialize};

/// Category for grouping plugin options in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OptionCategory {
    #[default]
    General,
    Display,
    Storage,
    Voice,
    Privacy,
    Advanced,
}

impl OptionCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Display => "Display",
            Self::Storage => "Storage",
            Self::Voice => "Voice",
            Self::Privacy => "Privacy",
            Self::Advanced => "Advanced",
        }
    }
}

/// Type of a plugin option
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginOptionType {
    Boolean,
    String,
    Number,
    Select,
}

/// Placement for plugin UI buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ButtonPlacement {
    /// Top toolbar (e.g. next to settings)
    #[default]
    Toolbar,
    /// Message input area (e.g. next to send)
    MessageInput,
    /// Channel header
    ChannelHeader,
}

/// A button the plugin adds to the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUiButton {
    /// Unique ID within the plugin
    pub id: String,
    /// Button label
    pub label: String,
    /// Optional tooltip
    #[serde(default)]
    pub tooltip: String,
    /// Where to place the button
    #[serde(default)]
    pub placement: ButtonPlacement,
}

/// Field type for modal inputs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModalFieldType {
    /// Single-line text
    #[default]
    Short,
    /// Multi-line paragraph
    Paragraph,
}

/// A field in a plugin modal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUiModalField {
    /// Key for the submitted value
    pub key: String,
    /// Label shown above the input
    pub label: String,
    /// Input type
    #[serde(default)]
    pub field_type: ModalFieldType,
    /// Placeholder text
    #[serde(default)]
    pub placeholder: String,
    /// Whether the field is required
    #[serde(default)]
    pub required: bool,
}

/// A modal dialog the plugin can show
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUiModal {
    /// Unique ID within the plugin
    pub id: String,
    /// Modal title
    pub title: String,
    /// Input fields
    #[serde(default)]
    pub fields: Vec<PluginUiModalField>,
}

/// Plugin UI definitions (buttons, modals) from manifest
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginUi {
    #[serde(default)]
    pub buttons: Vec<PluginUiButton>,
    #[serde(default)]
    pub modals: Vec<PluginUiModal>,
}

/// A single configurable option for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginOption {
    /// Unique key for this option
    pub key: String,
    /// Human-readable label
    pub label: String,
    /// Description/tooltip
    #[serde(default)]
    pub description: String,
    /// Option type
    #[serde(rename = "type")]
    pub option_type: PluginOptionType,
    /// Default value (as JSON-compatible string or number)
    #[serde(default)]
    pub default: serde_json::Value,
    /// Category for UI grouping
    #[serde(default)]
    pub category: OptionCategory,
    /// Choices for Select type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<String>>,
    /// Minimum value (for number type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Maximum value (for number type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

/// Plugin manifest (plugin.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique plugin ID (e.g. "message-logger")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Short description
    #[serde(default)]
    pub description: String,
    /// Semantic version
    #[serde(default = "default_version")]
    pub version: String,
    /// Author name
    #[serde(default)]
    pub author: String,
    /// Git repository URL (for user-installed plugins)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Configurable options
    #[serde(default)]
    pub options: Vec<PluginOption>,
    /// Discord gateway events this plugin subscribes to
    #[serde(default)]
    pub events: Vec<String>,
    /// Entry script (default: main.lua)
    #[serde(default = "default_entry")]
    pub entry: String,
    /// UI elements (buttons, modals) the plugin adds
    #[serde(default)]
    pub ui: PluginUi,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_entry() -> String {
    "main.lua".to_string()
}

impl PluginManifest {
    /// Get default config values from options
    pub fn default_config(&self) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        for opt in &self.options {
            obj.insert(opt.key.clone(), opt.default.clone());
        }
        serde_json::Value::Object(obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parse() {
        let json = r#"{
            "id": "message-logger",
            "name": "Message Logger",
            "description": "Track deleted and edited messages",
            "version": "1.0.0",
            "options": [
                {
                    "key": "cache_size",
                    "label": "Cache Size",
                    "type": "number",
                    "default": 10000,
                    "category": "storage",
                    "min": 100,
                    "max": 50000
                }
            ],
            "events": ["MESSAGE_CREATE", "MESSAGE_UPDATE", "MESSAGE_DELETE", "MESSAGE_DELETE_BULK"],
            "entry": "main.lua"
        }"#;
        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.id, "message-logger");
        assert_eq!(manifest.options.len(), 1);
        assert_eq!(manifest.events.len(), 4);
    }
}
