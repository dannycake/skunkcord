<!-- Copyright (c) Skunk Ventures LLC | Last modified: 2025-03-07 | SPDX-License-Identifier: MIT -->

# Contributing

## Development Setup

```bash
# Install system dependencies (Ubuntu/Debian)
sudo apt-get install -y qt6-base-dev qt6-declarative-dev qt6-tools-dev libssl-dev pkg-config cmake g++

# Set Qt 6 paths
export QT_INCLUDE_PATH=/usr/include/x86_64-linux-gnu/qt6
export QT_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu
export QMAKE=/usr/bin/qmake6

# Build and test
cargo build
cargo test
```

## Code Quality Standards

Before submitting changes, ensure:

1. **All tests pass:** `cargo test` — currently 278+ tests
2. **No warnings:** `cargo check` should produce zero warnings
3. **Formatted:** `cargo fmt --all` — CI enforces this
4. **Linted:** `cargo clippy` — no new warnings
5. **Documented:** All public items should have doc comments

## Architecture

The project is organized into the following top-level modules:

| Module | Purpose |
|--------|---------|
| `bridge` | Backend ↔ QML UI communication |
| `captcha` | hCaptcha Enterprise handling |
| `client` | HTTP client with 32 API submodules |
| `features` | Client mod features |
| `gateway` | WebSocket + health + session limits |
| `input` | Keyboard shortcuts registry |
| `mobile_ffi` | C FFI for Android/iOS |
| `rendering` | Discord markdown parser |
| `security` | SSRF prevention, content sanitization |
| `storage` | Session/settings persistence |
| `ui` | Qt/QML interface components |

## Testing

- Unit tests go in `#[cfg(test)] mod tests` within each file
- Integration tests go in `tests/` using `wiremock` mock server
- All API endpoints should have at least one integration test
- Use `DiscordClient::set_api_base()` to point at mock servers

