// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Message Logger plugin — track deleted and edited messages
//!
//! This plugin is entirely self-contained. The bridge never imports it directly;
//! it only receives a handler via the plugin registry when the plugin is enabled.

mod cache;
mod handler;

pub mod export;

pub use cache::{LoggedMessage, MessageCache, MessageEdit};
pub use handler::MessageLoggerHandler;
