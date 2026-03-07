// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Keyboard shortcut registry
//!
//! Manages configurable keyboard shortcuts for the application.
//! Supports both in-window and global (push-to-talk) shortcuts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A keyboard shortcut action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyAction {
    /// Ctrl+K: Quick channel/DM switcher
    QuickSwitcher,
    /// Ctrl+Shift+M: Toggle mute
    ToggleMute,
    /// Ctrl+Shift+D: Toggle deafen
    ToggleDeafen,
    /// Escape: Close popup/modal
    ClosePopup,
    /// Up arrow in empty input: Edit last message
    EditLastMessage,
    /// Ctrl+E: Open emoji picker
    EmojiPicker,
    /// Ctrl+G: Open GIF picker
    GifPicker,
    /// Ctrl+/: Show keyboard shortcuts help
    ShowShortcuts,
    /// Alt+Up: Previous channel
    PreviousChannel,
    /// Alt+Down: Next channel
    NextChannel,
    /// Ctrl+1..9: Switch to account by index
    SwitchAccount1,
    SwitchAccount2,
    SwitchAccount3,
    SwitchAccount4,
    SwitchAccount5,
    SwitchAccount6,
    SwitchAccount7,
    SwitchAccount8,
    SwitchAccount9,
    /// Push-to-talk key (configurable, global)
    PushToTalk,
    /// Ctrl+Shift+I: Toggle dev tools / developer mode
    ToggleDevMode,
}

impl KeyAction {
    /// Get the display name for this action
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::QuickSwitcher => "Quick Switcher",
            Self::ToggleMute => "Toggle Mute",
            Self::ToggleDeafen => "Toggle Deafen",
            Self::ClosePopup => "Close Popup",
            Self::EditLastMessage => "Edit Last Message",
            Self::EmojiPicker => "Emoji Picker",
            Self::GifPicker => "GIF Picker",
            Self::ShowShortcuts => "Keyboard Shortcuts",
            Self::PreviousChannel => "Previous Channel",
            Self::NextChannel => "Next Channel",
            Self::SwitchAccount1 => "Switch to Account 1",
            Self::SwitchAccount2 => "Switch to Account 2",
            Self::SwitchAccount3 => "Switch to Account 3",
            Self::SwitchAccount4 => "Switch to Account 4",
            Self::SwitchAccount5 => "Switch to Account 5",
            Self::SwitchAccount6 => "Switch to Account 6",
            Self::SwitchAccount7 => "Switch to Account 7",
            Self::SwitchAccount8 => "Switch to Account 8",
            Self::SwitchAccount9 => "Switch to Account 9",
            Self::PushToTalk => "Push to Talk",
            Self::ToggleDevMode => "Toggle Developer Mode",
        }
    }

    /// Whether this keybind should work globally (even when window is not focused)
    pub fn is_global(&self) -> bool {
        matches!(
            self,
            Self::PushToTalk | Self::ToggleMute | Self::ToggleDeafen
        )
    }
}

/// A key combination (e.g., Ctrl+Shift+M)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCombo {
    /// The main key (as a string, e.g., "M", "Escape", "F1")
    pub key: String,
    /// Modifier keys
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyCombo {
    /// Create a simple key combo
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
        }
    }

    /// Add Ctrl modifier
    pub fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }
    /// Add Shift modifier
    pub fn shift(mut self) -> Self {
        self.shift = true;
        self
    }
    /// Add Alt modifier
    pub fn alt(mut self) -> Self {
        self.alt = true;
        self
    }

    /// Get display string (e.g., "Ctrl+Shift+M")
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.meta {
            parts.push("Meta");
        }
        parts.push(&self.key);
        parts.join("+")
    }
}

/// Keybind registry — maps actions to key combinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindRegistry {
    bindings: HashMap<KeyAction, KeyCombo>,
}

impl KeybindRegistry {
    /// Create with default keybindings
    pub fn defaults() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(KeyAction::QuickSwitcher, KeyCombo::new("K").ctrl());
        bindings.insert(KeyAction::ToggleMute, KeyCombo::new("M").ctrl().shift());
        bindings.insert(KeyAction::ToggleDeafen, KeyCombo::new("D").ctrl().shift());
        bindings.insert(KeyAction::ClosePopup, KeyCombo::new("Escape"));
        bindings.insert(KeyAction::EditLastMessage, KeyCombo::new("Up"));
        bindings.insert(KeyAction::EmojiPicker, KeyCombo::new("E").ctrl());
        bindings.insert(KeyAction::GifPicker, KeyCombo::new("G").ctrl());
        bindings.insert(KeyAction::ShowShortcuts, KeyCombo::new("/").ctrl());
        bindings.insert(KeyAction::PreviousChannel, KeyCombo::new("Up").alt());
        bindings.insert(KeyAction::NextChannel, KeyCombo::new("Down").alt());
        bindings.insert(KeyAction::SwitchAccount1, KeyCombo::new("1").ctrl());
        bindings.insert(KeyAction::SwitchAccount2, KeyCombo::new("2").ctrl());
        bindings.insert(KeyAction::SwitchAccount3, KeyCombo::new("3").ctrl());
        bindings.insert(KeyAction::ToggleDevMode, KeyCombo::new("I").ctrl().shift());

        Self { bindings }
    }

    /// Get the key combo for an action
    pub fn get(&self, action: &KeyAction) -> Option<&KeyCombo> {
        self.bindings.get(action)
    }

    /// Set a custom keybind
    pub fn set(&mut self, action: KeyAction, combo: KeyCombo) {
        self.bindings.insert(action, combo);
    }

    /// Remove a keybind
    pub fn remove(&mut self, action: &KeyAction) {
        self.bindings.remove(action);
    }

    /// Find the action for a key combo (reverse lookup)
    pub fn find_action(&self, combo: &KeyCombo) -> Option<KeyAction> {
        self.bindings
            .iter()
            .find(|(_, c)| *c == combo)
            .map(|(a, _)| *a)
    }

    /// Get all bindings
    pub fn all(&self) -> &HashMap<KeyAction, KeyCombo> {
        &self.bindings
    }

    /// Reset to defaults
    pub fn reset(&mut self) {
        *self = Self::defaults();
    }
}

impl Default for KeybindRegistry {
    fn default() -> Self {
        Self::defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_have_expected_bindings() {
        let reg = KeybindRegistry::defaults();
        let qs = reg.get(&KeyAction::QuickSwitcher).unwrap();
        assert_eq!(qs.key, "K");
        assert!(qs.ctrl);
        assert!(!qs.shift);
    }

    #[test]
    fn test_display() {
        let combo = KeyCombo::new("M").ctrl().shift();
        assert_eq!(combo.display(), "Ctrl+Shift+M");
    }

    #[test]
    fn test_custom_keybind() {
        let mut reg = KeybindRegistry::defaults();
        reg.set(KeyAction::PushToTalk, KeyCombo::new("V"));
        assert_eq!(reg.get(&KeyAction::PushToTalk).unwrap().key, "V");
    }

    #[test]
    fn test_find_action() {
        let reg = KeybindRegistry::defaults();
        let combo = KeyCombo::new("K").ctrl();
        assert_eq!(reg.find_action(&combo), Some(KeyAction::QuickSwitcher));
    }

    #[test]
    fn test_global_keybinds() {
        assert!(KeyAction::PushToTalk.is_global());
        assert!(KeyAction::ToggleMute.is_global());
        assert!(!KeyAction::QuickSwitcher.is_global());
        assert!(!KeyAction::EmojiPicker.is_global());
    }

    #[test]
    fn test_serialization() {
        let reg = KeybindRegistry::defaults();
        let json = serde_json::to_string(&reg).unwrap();
        let deser: KeybindRegistry = serde_json::from_str(&json).unwrap();
        assert!(deser.get(&KeyAction::QuickSwitcher).is_some());
    }
}
