// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! arRPC — Rich Presence / Game Activity
//!
//! Native Rust implementation of Discord's Rich Presence that enables
//! RPC functionality for games and applications. Works by:
//! 1. Running a local IPC server on Discord's standard socket paths
//! 2. Scanning running processes against a detectable games database
//! 3. Converting RPC activity data to gateway PresenceUpdate commands
//!
//! Activity from non-Discord RPC source; game data may not match Discord's detection.

pub mod process_scanner;
pub mod rpc_server;

pub use process_scanner::*;
pub use rpc_server::*;
