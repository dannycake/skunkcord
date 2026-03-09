<!-- Copyright (c) Skunk Ventures LLC | Last modified: 2025-03-07 | SPDX-License-Identifier: MIT -->

# Skunkcord — Architecture

This document describes how the codebase is structured so developers can navigate and extend it.

## High-level overview

- **Rust backend**: Discord API client, Gateway (WebSocket), voice, REST, feature flags.
- **Qt/QML frontend**: Single-window UI (`src/qml/main.qml`) with login, guild/channel list, message list, input.
- **Bridge**: Connects backend and UI via two channels:
  - **UiAction**: QML → backend (e.g. `SelectGuild`, `SendMessage`, `SelectChannel`).
  - **UiUpdate**: Backend → QML (e.g. `GuildsLoaded`, `MessagesLoaded`, `NewMessage`, `MembersLoaded`).

The main thread runs the Qt event loop. A **worker thread** runs a Tokio runtime: Gateway, REST client, and the bridge's action handler. Data flows:

1. User clicks in QML → `AppController` method (e.g. `select_guild`) → `UiAction` sent on `action_tx` (tokio unbounded).
2. Worker receives action → `handle_ui_action()` in `bridge.rs` (and optionally Gateway/API) → one or more `UiUpdate`s sent on `update_tx` (std mpsc).
3. QML timer calls `app.check_for_updates()` → `AppController` drains `update_rx` and applies updates (e.g. pushes to ListModels or sets properties).

## Key Rust modules

| Path | Role |
|------|------|
| `src/main.rs` | Entry point: env (Qt style), logging, channel setup, spawns worker, creates `AppController`, loads `main.qml`, exposes `app` to QML. |
| `src/bridge.rs` | **Bridge**: `UiAction`, `UiUpdate`, `BackendBridge`, `BridgeCache`, `handle_ui_action()`. Translates Gateway events and API responses into UI-friendly structs; holds in-memory cache for guilds/channels/DMs/messages. |
| `src/ui/app_controller.rs` | **AppController**: QObject exposed to QML as `app`. Properties (e.g. `is_logged_in`, `user_name`), methods (e.g. `select_guild`, `send_message`, `consume_guilds`). Receives `UiUpdate`s and buffers them; QML pulls via `consume_*` and `check_for_updates()`. |
| `src/gateway/mod.rs` | Gateway WebSocket, reconnect, `GatewayCommand`, `GatewayState`. Subscribes to events and forwards to bridge. |
| `src/gateway/events.rs` | `GatewayEvent` (e.g. `Ready`, `MessageCreate`, `MessageUpdate`). |
| `src/client/mod.rs` | `DiscordClient`: REST API (guilds, channels, messages, DMs, etc.). |
| `src/client/api.rs` | API types: `Guild`, `Channel`, `Message`, `CreateMessage`, etc. |
| `src/ui/main_window.rs` | Legacy/main window helpers; primary UI is QML. |

## Thread and channel layout

- **Main thread**: Qt, QML, `AppController`. Holds `login_tx`, `action_tx`, `update_rx`.
- **Worker thread**: `run_app_with_updates()`:
  - Builds `DiscordClient`, `BackendBridge`, `Gateway`.
  - Spawns task that receives `UiAction` and runs `handle_ui_action(client, update_tx, gateway_cmd, bridge_cache)`.
  - Spawns task that forwards Gateway events into `bridge.handle_gateway_event()`.
  - Runs `gateway.connect_with_reconnect()` (blocking until disconnect).

So: **QML never calls async Rust directly**. All async work and I/O happen in the worker; the bridge turns results into `UiUpdate`s that the main thread delivers to QML via `check_for_updates()` and the `consume_*` methods.

## Bridge cache

`BridgeCache` in `bridge.rs` caches:

- Guild list, channels per guild, DM channels, messages per channel (with TTL).
- **Member list per guild** (from Gateway Op 14 Lazy Guild / `GUILD_MEMBER_LIST_UPDATE` and Op 8 `GUILD_MEMBERS_CHUNK`). Exposed to QML via `UiUpdate::MembersLoaded` and `app.consume_members()`.
- `my_user_id` (for mention detection and message context).

Used to:

- Avoid refetching when switching guild/channel (instant restore from cache; backend still asked for fresh data).
- Resolve DM recipient names from READY + user map when `recipients` is missing.
- Provide `my_user_id` to `MessageInfo::from_message_with_context()` for mentions/replies.

## Adding a new UI action

1. Add a variant to `UiAction` in `src/bridge.rs`.
2. In `handle_ui_action()` in `bridge.rs`, handle that variant (call client/gateway, then send `UiUpdate`s as needed).
3. In `AppController` (`src/ui/app_controller.rs`), add a `qt_method!` that sends that `UiAction` via `action_tx`.
4. In QML, call `app.new_method_name(...)` where appropriate.

## Adding a new UI update

1. Add a variant to `UiUpdate` in `src/bridge.rs`.
2. In the bridge (or code that holds `update_tx`), send that variant when the event occurs.
3. In `AppController`, in the `check_for_updates()` path where you process the receiver, handle the new variant (e.g. push to a buffer, set a property, or add a new `consume_*` buffer).
4. If you use a buffer, add a `consume_*` method that returns JSON for QML and clear the buffer.
5. In QML, ensure a Timer or binding calls `check_for_updates()` and, if needed, reads the new `consume_*` result and updates the relevant ListModel or property.

## UI test binary

`src/bin/ui_test.rs` builds a separate binary (`cargo run --bin ui_test`) that:

- Loads the same `main.qml` (or a test QML that mirrors it).
- Injects mock data (guilds, channels, messages) via a test backend so you can develop and test the UI without a live Discord connection.

Use it to verify layout and styling without hitting the real bridge.
