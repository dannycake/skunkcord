<!-- Copyright (c) Skunk Ventures LLC | Last modified: 2025-03-07 | SPDX-License-Identifier: MIT -->

# Discord Qt Client

A user-account Discord client built with Rust and Qt, featuring browser fingerprint emulation, comprehensive API coverage, voice chat with fake mute, and built-in client mod features.

## ⚠️ Legal Disclaimer

This software is for educational and personal use only. Using self-bots or automated user accounts may violate Discord's Terms of Service. Use at your own risk. **Never share your user token with anyone.**

## Features

### 🛡️ Anti-Detection & Safety

- **Feature Flags System** — Every non-vanilla feature is individually toggleable
  - **Paranoid Mode** — One-click preset: only vanilla behavior, zero detection surface
  - **Standard Mode** — Safe features on, risky features off
  - **Full Mode** — Everything enabled, accept the risk
  - Detection risk labels (None/Low/Medium/High) on every feature
- **Browser Fingerprint Emulation** — Chrome user agent, X-Super-Properties, Sec-Ch-Ua, canvas/audio hashes
- **Dynamic Build Number** — Scraped from Discord's live web app every 6 hours (no stale values)
- **Human-like Request Timing** — Random 50-300ms jitter between rapid API calls
- **Telemetry Blocking** — /science, /track, /metrics silently blocked (same as Vencord/BetterDiscord)
- **Rate Limit Retry** — Automatic retry with backoff on 429 responses (up to 3 attempts)
- **Cookie Management** — Cloudflare cookies (__dcfduid, __sdcfduid, __cfruid) properly managed
- **SOCKS5 Proxy Support** — Per-account proxy configuration with Mullvad server presets

### 💬 Core Discord Functionality

- **Messaging** — Send, receive, edit, delete, bulk delete, search, pins, replies
- **Reactions** — Add, remove, get users, delete all, per-emoji deletion
- **Threads** — Create from message or standalone, join/leave, list active/archived
- **Channels** — Create, edit, delete, permissions management
- **Guilds** — List, roles, members, bans, audit log, leave
- **Invites** — Get, accept, create, delete (channel and guild level)
- **Emoji & Stickers** — List guild emojis, sticker packs
- **User Profiles** — Profiles with connections, mutual guilds, guild context
- **Relationships** — Friends, blocks, friend requests
- **Moderation** — Kick, ban, unban, timeout, role management
- **Slash Commands** — Autocomplete, send interactions, buttons, modals
- **Read States** — Unread tracking, mention counts, mark-as-read, bulk ACK

### 🎙️ Voice Chat

- **Voice Gateway** — WebSocket connection to Discord's voice servers
- **Encryption** — Supports xchacha20, aes256-gcm, xsalsa20 modes (auto-selects best)
- **SSRC Mapping** — Track who's speaking via SSRC-to-user mapping
- **Fake Mute** ⚠️ HIGH RISK — Appear muted while still receiving audio
- **Fake Deafen** ⚠️ HIGH RISK — Appear deafened while still hearing everything
- **Recording** — Record incoming audio while fake muted/deafened

### 🔒 Security

- **Safe Link Previews** — SSRF prevention: blocks private IPs, localhost, internal hostnames
- **URL Spoofing Detection** — Catches percent-encoded domain tricks in embeds
- **Content Sanitization** — HTML entity escaping, data URI blocking
- **ClearURLs** — Strips 37+ tracking parameters (utm_*, fbclid, gclid, etc.)
- **Token Security** — Tokens never logged, stored securely

### 📝 Built-in Client Mod Features

| Feature | Risk | Description |
|---------|------|-------------|
| **Message Logger** | Medium | Track deleted/edited messages with full edit history and search |
| **Show Hidden Channels** | Medium | Display channels you can't access with lock icon |
| **ClearURLs** | None | Strip tracking params from outgoing URLs |
| **Silent Message Toggle** | None | Send messages without triggering notifications |
| **No Reply Mention** | Low | Replies don't ping by default |
| **Pin DMs** | None | Pin DM conversations to top of list |
| **Streamer Mode** | None | Auto-detect OBS, hide emails/invites |
| **Read All Notifications** | None | One-click mark everything as read |

### 📊 Markdown Rendering

Full Discord-flavored markdown → HTML:
- **bold**, *italic*, __underline__, ~~strikethrough~~
- \`inline code\`, \`\`\`code blocks\`\`\`
- > Block quotes, ### Headings
- ||Spoilers|| (click to reveal)
- @mentions, #channels, @roles
- Custom emoji `:name:` with CDN images
- Timestamps with relative formatting
- [Masked links](url)

### 🔄 Multi-Account Support

- **Quick Account Switcher** — Avatar ring in sidebar, click to swap
- **Per-Account Settings** — Different feature flags per account
- **Per-Account Proxy** — Different IP per account
- **Background Connections** — Optional: keep inactive accounts connected for notification bridging

### 🌐 Gateway

- **Real-time Events** — 40+ event types handled:
  - Messages: create, update, delete, bulk delete, reactions
  - Guilds: members, bans, roles, emojis
  - Channels: create, update, delete, pins, threads
  - Voice: state updates, server updates
  - Interactions, relationships, presence, typing
- **Auto-Reconnect** — Exponential backoff with jitter (up to 50 retries)
- **Resume** — Uses saved session_id and sequence to resume without re-identifying

### 🤖 Automated Monitoring

- **Discord API Change Detection** — Daily GitHub Action checks discord/discord-api-docs for changes, auto-creates issues
- **Build Number Auto-Update** — Every 6 hours, scrapes and updates the fallback build number
- **CI/CD Pipeline** — Runs `cargo fmt`, `cargo clippy`, `cargo test` on every push/PR

## Architecture

```
discord-qt/
├── src/
│   ├── main.rs                    # Application entry point
│   ├── lib.rs                     # Library root, error types
│   ├── build_number.rs            # Dynamic build number scraping
│   ├── client/                    # HTTP client & API (22 submodules)
│   │   ├── mod.rs                 # DiscordClient, rate limiting, telemetry blocking
│   │   ├── api.rs                 # Core REST API endpoints
│   │   ├── session.rs             # Authentication & sessions
│   │   ├── account_switcher.rs    # Multi-account management
│   │   ├── attachments.rs         # File upload (multipart)
│   │   ├── automod.rs             # Auto-moderation rules
│   │   ├── captcha_interceptor.rs # Captcha detection in HTTP responses
│   │   ├── cookies.rs             # Cloudflare cookie management
│   │   ├── forums.rs              # Forum channel posts
│   │   ├── interactions.rs        # Slash commands & components
│   │   ├── invites.rs             # Invite endpoints
│   │   ├── onboarding.rs          # Guild onboarding
│   │   ├── permissions.rs         # Permission calculator
│   │   ├── polls.rs               # Poll creation & voting
│   │   ├── reactions.rs           # Reaction endpoints
│   │   ├── read_states.rs         # Unread tracking
│   │   ├── scheduled_events.rs    # Guild scheduled events
│   │   ├── soundboard.rs          # Soundboard sounds
│   │   ├── stage.rs               # Stage channel instances
│   │   ├── threads.rs             # Thread endpoints
│   │   ├── timing.rs              # Request jitter
│   │   ├── typing.rs              # Typing indicator throttle
│   │   ├── user_settings.rs       # Full user settings
│   │   └── webhooks.rs            # Webhook management
│   ├── captcha/                   # hCaptcha Enterprise handling
│   │   ├── mod.rs                 # Detection, parsing, state machine
│   │   └── widget.rs              # Widget HTML generation with rqdata
│   ├── features/                  # Feature flags & client mod features (14 submodules)
│   │   ├── flags.rs               # FeatureFlags, presets, risk metadata
│   │   ├── message_logger.rs      # Deleted/edited message tracking
│   │   ├── message_export.rs      # Export logs to JSON/CSV
│   │   ├── show_hidden_channels.rs # Permission-based visibility
│   │   ├── clear_urls.rs          # Tracking parameter removal
│   │   ├── silent_messages.rs     # Suppress notifications flag
│   │   ├── no_reply_mention.rs    # No-ping replies
│   │   ├── pin_dms.rs             # DM pinning
│   │   ├── streamer_mode.rs       # Streaming detection & redaction
│   │   ├── emoji_picker.rs        # Unicode emoji search & recent
│   │   ├── gif_picker.rs          # Tenor GIF search
│   │   ├── notifications.rs       # Notification config & muting
│   │   └── arrpc/                 # Rich Presence (IPC + process scanner)
│   ├── fingerprint/               # Browser emulation
│   │   ├── mod.rs                 # Chrome fingerprint generation
│   │   ├── browser_data.rs        # Browser constants
│   │   └── super_properties.rs    # X-Super-Properties header
│   ├── gateway/                   # WebSocket gateway
│   │   ├── mod.rs                 # Connection, heartbeat, reconnect
│   │   ├── events.rs              # 40+ event types
│   │   └── payloads.rs            # Gateway payloads & opcodes
│   ├── rendering/                 # Message display
│   │   └── markdown.rs            # Discord markdown → HTML
│   ├── security/                  # Safety & privacy
│   │   ├── link_preview.rs        # SSRF prevention, URL safety
│   │   └── content.rs             # Sanitization, tracking removal
│   ├── voice/                     # Voice chat (5 submodules)
│   │   ├── connection.rs          # Voice connection state machine
│   │   ├── crypto.rs              # Audio encryption (nonce, packets)
│   │   ├── fake_mute.rs           # Fake mute/deafen state
│   │   ├── gateway.rs             # Voice gateway protocol
│   │   └── udp.rs                 # RTP packets, IP discovery
│   ├── proxy/                     # Proxy support
│   │   └── mod.rs                 # SOCKS5, Mullvad integration
│   ├── storage/                   # Persistence
│   │   └── mod.rs                 # Sessions, settings, cache
│   └── ui/                        # Qt/QML interface
│       ├── mod.rs                 # QObject types
│       ├── login_window.rs        # Web login & token extraction
│       ├── main_window.rs         # Main interface
│       ├── web_view.rs            # WebView controller
│       └── styles.rs              # Discord theme
├── .github/workflows/
│   ├── ci.yml                     # Build, lint, test
│   ├── api-watch.yml              # Discord API change monitor
│   └── build-number.yml           # Build number auto-updater
└── Cargo.toml
```

## Building

### Linux

#### Prerequisites

- Rust 1.83+ (1.93+ recommended)
- Qt 6 development libraries
- OpenSSL development libraries
- CMake, pkg-config, g++

#### Install Dependencies (Ubuntu/Debian)

```bash
sudo apt-get update && sudo apt-get install -y \
    qt6-base-dev qt6-declarative-dev qt6-tools-dev \
    libssl-dev pkg-config cmake g++
```

#### Build & Test

```bash
cd discord-qt

# Set Qt 6 paths (Ubuntu/Debian)
export QT_INCLUDE_PATH=/usr/include/x86_64-linux-gnu/qt6
export QT_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu
export QMAKE=/usr/bin/qmake6

# Build
cargo build --release

# Run tests (109 tests)
cargo test

# Run
DISCORD_TOKEN="your_token" cargo run
```

### Windows

For Windows builds, see **[docs/WINDOWS-BUILD.md](docs/WINDOWS-BUILD.md)** which covers:
- 🤖 Automated builds via GitHub Actions (easiest)
- 💻 Native Windows build instructions
- 🔧 Cross-compilation from Linux (advanced)
- 📦 Creating distributable packages

## Deployment

### Option 1: Standalone Bundle (Recommended)

Self-contained package with Qt libraries included - **no Qt installation required** on target system:

```bash
# Creates ~34MB package that works on ANY Linux system
./package-bundle.sh
tar -czf discord-qt-linux-standalone.tar.gz discord-qt-bundle/

# On target machine (no Qt needed!):
tar -xzf discord-qt-linux-standalone.tar.gz
cd discord-qt-bundle && ./discord_qt.sh
```

### Option 2: Minimal Package

Small package requiring Qt 6 on target system:

```bash
# Creates ~3.6MB package (requires Qt installation)
./package.sh
tar -czf discord-qt-linux.tar.gz discord-qt-release/

# On target machine (Qt 6 required):
tar -xzf discord-qt-linux.tar.gz
cd discord-qt-release && ./discord_qt
```

**Comparison:**
- **Standalone**: 34MB, works everywhere, no dependencies → **Best for distribution**
- **Minimal**: 3.6MB, requires Qt installation → Best for developers

For detailed deployment instructions, see **[docs/DEPLOYMENT.md](docs/DEPLOYMENT.md)**.

## Feature Flags

Every feature can be toggled in settings. Use presets for quick configuration:

| Preset | Description | Risk Level |
|--------|-------------|------------|
| **Paranoid** | Only vanilla behavior + safety features | None |
| **Standard** | Safe mod features enabled | Low |
| **Full** | Everything enabled | High |

See `src/features/flags.rs` for the complete list of 30 toggleable features with risk descriptions.

## Testing

244 automated tests covering:
- Feature flag presets and risk levels
- Fingerprint generation and super properties
- Session management
- Cookie handling
- Request timing
- Build number regex parsing
- hCaptcha detection, widget generation, title parsing
- Link preview safety (SSRF, spoofing)
- Content sanitization and URL tracking removal
- Message logger cache operations
- Account switcher
- Voice encryption mode selection
- Markdown rendering (15 format types)
- Show hidden channels permissions
- Silent messages, pin DMs, streamer mode
- Proxy URL formatting
- Read state management
- Permission calculator (base + channel overwrites)
- Captcha interceptor (detection, extraction, retry headers)
- Typing indicator throttle
- Multipart file upload builder
- Poll creation and results deserialization
- Voice connection state machine
- 41 integration tests via mock Discord server (wiremock)

## License

MIT License — Copyright (c) Skunk Ventures LLC. See [LICENSE](LICENSE) for details.
