// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! ClearURLs — strip tracking parameters from outgoing URLs
//!
//! Removes utm_*, fbclid, gclid, and other tracking parameters from
//! URLs in outgoing messages before sending. This is purely local text
//! processing with zero API interaction — completely undetectable.

pub use crate::security::content::{clean_message_urls, strip_tracking_params};
