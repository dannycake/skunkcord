// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! UI Styles and Theming
//!
//! Muted violet dark theme with JetBrains Mono font.
//! All colors here are kept in sync with the QML theme object in `src/qml/main.qml`.

/// Muted violet color palette — dark purple-tinted backgrounds for visual depth,
/// paired with Discord-inspired accent colors.
pub mod colors {
    // ── Background layers ──

    /// Base background — deep muted violet
    pub const BG_BASE: &str = "#2a2139";
    /// Primary background — dark panels
    pub const BG_PRIMARY: &str = "#2a2139";
    /// Secondary background — elevated surfaces, sidebars
    pub const BG_SECONDARY: &str = "#211a30";
    /// Tertiary background — cards, nested surfaces
    pub const BG_TERTIARY: &str = "#191425";
    /// Floating elements — popups, tooltips
    pub const BG_FLOATING: &str = "#110e1a";
    /// Elevated elements — contextual menus, action bars
    pub const BG_ELEVATED: &str = "#211a30";
    /// Hover state
    pub const BG_HOVER: &str = "#342b44";
    /// Selected / active state
    pub const BG_ACTIVE: &str = "#3e3450";
    /// Subtle white overlay for hover modifiers
    pub const BG_MODIFIER: &str = "#ffffff08";

    // ── Text hierarchy ──

    /// Primary text — off-white for reduced eye strain
    pub const TEXT_NORMAL: &str = "#dcddde";
    /// Secondary text — muted but readable
    pub const TEXT_SECONDARY: &str = "#949ba4";
    /// Muted text — less important information
    pub const TEXT_MUTED: &str = "#6d7178";
    /// Faint text — timestamps, subtle hints
    pub const TEXT_FAINT: &str = "#4e5058";
    /// Link color
    pub const TEXT_LINK: &str = "#5e9eff";

    // ── Accent colors ──

    /// Brand accent — Discord blurple
    pub const ACCENT: &str = "#5865f2";
    /// Accent hover state
    pub const ACCENT_HOVER: &str = "#4752c4";
    /// Lighter accent for secondary emphasis
    pub const ACCENT_LIGHT: &str = "#7289da";
    /// Accent glow — used for focus rings and hover glows
    pub const ACCENT_GLOW: &str = "#5865f230";
    /// Muted accent — subtle accent background tint
    pub const ACCENT_MUTED: &str = "#5865f218";

    // ── Semantic colors ──

    /// Positive / success (green)
    pub const POSITIVE: &str = "#23a55a";
    /// Warning (yellow)
    pub const WARNING: &str = "#f0b132";
    /// Danger / error (red)
    pub const DANGER: &str = "#f23f43";
    /// Info (blue)
    pub const INFO: &str = "#5e9eff";

    // ── Status colors ──

    /// Online status
    pub const STATUS_ONLINE: &str = "#23a55a";
    /// Idle status
    pub const STATUS_IDLE: &str = "#f0b132";
    /// DND status
    pub const STATUS_DND: &str = "#f23f43";
    /// Offline status
    pub const STATUS_OFFLINE: &str = "#80848e";

    // ── Borders & separators ──

    /// Primary border color
    pub const BORDER: &str = "#3a3048";
    /// Subtle border (semi-transparent white)
    pub const BORDER_SUBTLE: &str = "#ffffff0a";
    /// Separator lines
    pub const SEPARATOR: &str = "#3a3048";

    // ── Misc ──

    /// Mention highlight background
    pub const MENTION_BG: &str = "#5865f218";
    /// Mention text color
    pub const MENTION_FG: &str = "#c9cdfb";
    /// Deleted message background (for message logger)
    pub const DELETED_BG: &str = "#f23f4312";
    /// Edited message indicator
    pub const EDITED_FG: &str = "#6d7178";

    // ── Legacy aliases (for backward compatibility) ──

    /// Alias for BG_BASE
    pub const BG_PRIMARY_LEGACY: &str = BG_BASE;
    /// Input field background
    pub const INPUT_BG: &str = "#302840";
}

/// Font configuration
pub mod fonts {
    /// Primary font family — JetBrains Mono for the terminal aesthetic
    pub const PRIMARY: &str = "JetBrains Mono";
    /// Fallback fonts
    pub const FALLBACK: &str = "Consolas, Monaco, 'Courier New', monospace";
    /// Full font-family CSS string
    pub const FAMILY: &str = "'JetBrains Mono', Consolas, Monaco, 'Courier New', monospace";

    /// Font sizes — hierarchical type scale
    pub const SIZE_TINY: &str = "10px";
    pub const SIZE_SMALL: &str = "11px";
    pub const SIZE_BODY: &str = "13px";
    pub const SIZE_NORMAL: &str = "14px";
    pub const SIZE_MEDIUM: &str = "15px";
    pub const SIZE_LARGE: &str = "18px";
    pub const SIZE_HEADER: &str = "20px";
    pub const SIZE_TITLE: &str = "24px";
}

/// Layout dimensions
pub mod layout {
    /// Guild sidebar width
    pub const GUILD_BAR_WIDTH: u32 = 62;
    /// Channel sidebar width
    pub const CHANNEL_BAR_WIDTH: u32 = 240;
    /// Header bar height
    pub const HEADER_HEIGHT: u32 = 48;
    /// User panel height
    pub const USER_PANEL_HEIGHT: u32 = 52;
    /// Message input height
    pub const MESSAGE_INPUT_HEIGHT: u32 = 54;

    /// Border radius — small (buttons, tags)
    pub const RADIUS_SMALL: u32 = 4;
    /// Border radius — medium (cards, inputs)
    pub const RADIUS_MEDIUM: u32 = 8;
    /// Border radius — large (popups, dialogs)
    pub const RADIUS_LARGE: u32 = 12;
    /// Border radius — extra large (login card)
    pub const RADIUS_XL: u32 = 16;
}

/// Animation durations (milliseconds)
pub mod animation {
    /// Fast — hover states, micro-interactions
    pub const FAST: u32 = 100;
    /// Normal — state transitions, color changes
    pub const NORMAL: u32 = 150;
    /// Slow — popup entrance, scroll transitions
    pub const SLOW: u32 = 250;
}
