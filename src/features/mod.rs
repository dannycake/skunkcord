// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Feature flags and built-in client mod features
//!
//! Every feature that touches the Discord API or deviates from vanilla client
//! behavior is individually toggleable.

pub mod arrpc;
pub mod browser_handoff;
pub mod clear_urls;
pub mod emoji_picker;
pub mod experiments;
pub mod flags;
pub mod gif_picker;
pub mod no_reply_mention;
pub mod notifications;
pub mod pin_dms;
pub mod show_hidden_channels;
pub mod silent_messages;
pub mod streamer_mode;

pub use flags::*;
