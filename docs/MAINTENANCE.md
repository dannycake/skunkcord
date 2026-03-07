<!-- Copyright (c) Skunk Ventures LLC | Last modified: 2025-03-07 | SPDX-License-Identifier: MIT -->

# Discord Qt — Maintenance Guide

How to build, run, test, and make common changes.

## Build and run

```bash
cd discord-qt
cargo build
cargo build --release
cargo run
cargo run --bin ui_test
```

- **Full app**: Loads `src/qml/main.qml`, connects to Discord. Token from env `DISCORD_TOKEN`, CLI `--token`, or saved session.
- **UI test**: Same QML, mock backend. Use for layout/styling without a token.

## Tests

```bash
cargo test
```

Rust tests are next to the code. Use `ui_test` for manual UI checks.

## Key files

| Purpose | File(s) |
|---------|--------|
| Entry, Qt env, load QML | `src/main.rs` |
| Actions and updates | `src/bridge.rs` |
| QML API | `src/ui/app_controller.rs` |
| UI layout and styling | `src/qml/main.qml` |
| Gateway events | `src/gateway/events.rs` |
| REST types and client | `src/client/api.rs`, `src/client/mod.rs` |
| UI test | `src/bin/ui_test.rs` |

## Conventions

- UI and backend talk only via `UiAction` and `UiUpdate`.
- Use the `theme` object in `main.qml` for colors and spacing.
- Use local date for message day dividers (`localDateStringFromTimestamp`, `formatDateLabel`).
- Message list visible only when `currentChannelId !== ""`.
- Set `focus: false` on message list and delegates to avoid focus rectangles.

## Adding a feature (UI + backend)

1. Backend: add or use API/Gateway in `src/client` or `src/gateway`.
2. Bridge: add `UiAction`/`UiUpdate` in `src/bridge.rs`; handle in `handle_ui_action()`.
3. AppController: add method sending the action; add buffer and `consume_*` if new data.
4. QML: call new `app.*` method; in timer that calls `check_for_updates()`, parse `consume_*` and update models/properties.
5. Optionally extend `BridgeCache` for instant restore when switching guild/channel.

## UI-only changes

- Layout: edit `src/qml/main.qml` (RowLayout, ListView delegates, visibility).
- Colors/spacing: edit `theme` in `main.qml`.
- New popup: add `Popup` in `main.qml`, open/close from buttons or shortcuts.

## Debugging

- Rust: `RUST_LOG=discord_qt=debug cargo run`.
- QML: `console.log()`; Qt warnings on stderr.
- Updates not in UI: ensure Timer calls `app.check_for_updates()` and `consume_*` result is applied to the right model/property.

## More detail

- **ARCHITECTURE.md** — threads, bridge, cache, adding actions/updates.
- **UI-QML.md** — layout, theme, models, message list, where to change what.
