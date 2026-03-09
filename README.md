<!-- Copyright (c) Skunk Ventures LLC | Last modified: 2025-03-07 | SPDX-License-Identifier: MIT -->

# skunkcord client

a user-account discord client built with rust and qt, featuring browser fingerprint emulation, comprehensive api coverage, voice chat with fake mute, and built-in client mod features.

**not affiliated with discord.** this project is an independent, community-built client. discord and the discord logo are trademarks of discord inc. we are not endorsed by, sponsored by, or connected with discord inc. in any way.

## ⚠️ legal disclaimer

this software is for educational and personal use only. using self-bots or automated user accounts may violate discord's terms of service. use at your own risk. **never share your user token with anyone.**

## features

### 🛡️ anti-detection & safety

- **feature flags system** — every non-vanilla feature is individually toggleable
  - **paranoid mode** — one-click preset: only vanilla behavior, zero detection surface
  - **standard mode** — safe features on, risky features off
  - **full mode** — everything enabled, accept the risk
  - detection risk labels (none/low/medium/high) on every feature
- **browser fingerprint emulation** — chrome user agent, x-super-properties, sec-ch-ua, canvas/audio hashes
- **dynamic build number** — scraped from discord's live web app every 6 hours (no stale values)
- **human-like request timing** — random 50-300ms jitter between rapid api calls
- **telemetry blocking** — /science, /track, /metrics silently blocked (same as vencord/betterdiscord)
- **rate limit retry** — automatic retry with backoff on 429 responses (up to 3 attempts)
- **cookie management** — cloudflare cookies (__dcfduid, __sdcfduid, __cfruid) properly managed
- **socks5 proxy support** — per-account proxy configuration with mullvad server presets

### 💬 core discord functionality

- **messaging** — send, receive, edit, delete, bulk delete, search, pins, replies
- **reactions** — add, remove, get users, delete all, per-emoji deletion
- **threads** — create from message or standalone, join/leave, list active/archived
- **channels** — create, edit, delete, permissions management
- **guilds** — list, roles, members, bans, audit log, leave
- **invites** — get, accept, create, delete (channel and guild level)
- **emoji & stickers** — list guild emojis, sticker packs
- **user profiles** — profiles with connections, mutual guilds, guild context
- **relationships** — friends, blocks, friend requests
- **moderation** — kick, ban, unban, timeout, role management
- **slash commands** — autocomplete, send interactions, buttons, modals
- **read states** — unread tracking, mention counts, mark-as-read, bulk ack

### 🎙️ voice chat

- **voice gateway** — websocket connection to discord's voice servers
- **encryption** — supports xchacha20, aes256-gcm, xsalsa20 modes (auto-selects best)
- **ssrc mapping** — track who's speaking via ssrc-to-user mapping
- **fake mute** ⚠️ high risk — appear muted while still receiving audio
- **fake deafen** ⚠️ high risk — appear deafened while still hearing everything
- **recording** — record incoming audio while fake muted/deafened

### 🔒 security

- **safe link previews** — ssrf prevention: blocks private ips, localhost, internal hostnames
- **url spoofing detection** — catches percent-encoded domain tricks in embeds
- **content sanitization** — html entity escaping, data uri blocking
- **clearurls** — strips 37+ tracking parameters (utm_*, fbclid, gclid, etc.)
- **token security** — tokens never logged, stored securely

### 📝 built-in client mod features

| feature | risk | description |
|---------|------|-------------|
| **message logger** | medium | track deleted/edited messages with full edit history and search |
| **show hidden channels** | medium | display channels you can't access with lock icon |
| **clearurls** | none | strip tracking params from outgoing urls |
| **silent message toggle** | none | send messages without triggering notifications |
| **no reply mention** | low | replies don't ping by default |
| **pin dms** | none | pin dm conversations to top of list |
| **streamer mode** | none | auto-detect obs, hide emails/invites |
| **read all notifications** | none | one-click mark everything as read |

### 📊 markdown rendering

full discord-flavored markdown → html:
- **bold**, *italic*, __underline__, ~~strikethrough~~
- \`inline code\`, \`\`\`code blocks\`\`\`
- > block quotes, ### headings
- ||spoilers|| (click to reveal)
- @mentions, #channels, @roles
- custom emoji `:name:` with cdn images
- timestamps with relative formatting
- [masked links](url)

### 🔄 multi-account support

- **quick account switcher** — avatar ring in sidebar, click to swap
- **per-account settings** — different feature flags per account
- **per-account proxy** — different ip per account
- **background connections** — optional: keep inactive accounts connected for notification bridging

### 🌐 gateway

- **real-time events** — 40+ event types handled:
  - messages: create, update, delete, bulk delete, reactions
  - guilds: members, bans, roles, emojis
  - channels: create, update, delete, pins, threads
  - voice: state updates, server updates
  - interactions, relationships, presence, typing
- **auto-reconnect** — exponential backoff with jitter (up to 50 retries)
- **resume** — uses saved session_id and sequence to resume without re-identifying

### 🤖 automated monitoring

- **discord api change detection** — daily github action checks discord/discord-api-docs for changes, auto-creates issues
- **build number auto-update** — every 6 hours, scrapes and updates the fallback build number
- **ci/cd pipeline** — runs `cargo fmt`, `cargo clippy`, `cargo test` on every push/pr

## architecture

```
skunkcord/
├── src/
│   ├── main.rs                    # application entry point
│   ├── lib.rs                     # library root, error types
│   ├── build_number.rs            # dynamic build number scraping
│   ├── client/                    # http client & api (22 submodules)
│   │   ├── mod.rs                 # discordclient, rate limiting, telemetry blocking
│   │   ├── api.rs                 # core rest api endpoints
│   │   ├── session.rs             # authentication & sessions
│   │   ├── account_switcher.rs    # multi-account management
│   │   ├── attachments.rs        # file upload (multipart)
│   │   ├── automod.rs            # auto-moderation rules
│   │   ├── captcha_interceptor.rs # captcha detection in http responses
│   │   ├── cookies.rs            # cloudflare cookie management
│   │   ├── forums.rs             # forum channel posts
│   │   ├── interactions.rs      # slash commands & components
│   │   ├── invites.rs            # invite endpoints
│   │   ├── onboarding.rs         # guild onboarding
│   │   ├── permissions.rs       # permission calculator
│   │   ├── polls.rs             # poll creation & voting
│   │   ├── reactions.rs         # reaction endpoints
│   │   ├── read_states.rs       # unread tracking
│   │   ├── scheduled_events.rs  # guild scheduled events
│   │   ├── soundboard.rs        # soundboard sounds
│   │   ├── stage.rs             # stage channel instances
│   │   ├── threads.rs           # thread endpoints
│   │   ├── timing.rs            # request jitter
│   │   ├── typing.rs            # typing indicator throttle
│   │   ├── user_settings.rs    # full user settings
│   │   └── webhooks.rs          # webhook management
│   ├── captcha/                  # hcaptcha enterprise handling
│   │   ├── mod.rs               # detection, parsing, state machine
│   │   └── widget.rs            # widget html generation with rqdata
│   ├── features/                 # feature flags & client mod features (14 submodules)
│   │   ├── flags.rs             # featureflags, presets, risk metadata
│   │   ├── message_logger.rs    # deleted/edited message tracking
│   │   ├── message_export.rs    # export logs to json/csv
│   │   ├── show_hidden_channels.rs # permission-based visibility
│   │   ├── clear_urls.rs        # tracking parameter removal
│   │   ├── silent_messages.rs   # suppress notifications flag
│   │   ├── no_reply_mention.rs  # no-ping replies
│   │   ├── pin_dms.rs           # dm pinning
│   │   ├── streamer_mode.rs    # streaming detection & redaction
│   │   ├── emoji_picker.rs     # unicode emoji search & recent
│   │   ├── gif_picker.rs       # tenor gif search
│   │   ├── notifications.rs    # notification config & muting
│   │   └── arrpc/              # rich presence (ipc + process scanner)
│   ├── fingerprint/             # browser emulation
│   │   ├── mod.rs               # chrome fingerprint generation
│   │   ├── browser_data.rs     # browser constants
│   │   └── super_properties.rs # x-super-properties header
│   ├── gateway/                 # websocket gateway
│   │   ├── mod.rs               # connection, heartbeat, reconnect
│   │   ├── events.rs            # 40+ event types
│   │   └── payloads.rs          # gateway payloads & opcodes
│   ├── rendering/               # message display
│   │   └── markdown.rs         # discord markdown → html
│   ├── security/               # safety & privacy
│   │   ├── link_preview.rs     # ssrf prevention, url safety
│   │   └── content.rs         # sanitization, tracking removal
│   ├── voice/                  # voice chat (5 submodules)
│   │   ├── connection.rs      # voice connection state machine
│   │   ├── crypto.rs          # audio encryption (nonce, packets)
│   │   ├── fake_mute.rs       # fake mute/deafen state
│   │   ├── gateway.rs         # voice gateway protocol
│   │   └── udp.rs             # rtp packets, ip discovery
│   ├── proxy/                  # proxy support
│   │   └── mod.rs             # socks5, mullvad integration
│   ├── storage/               # persistence
│   │   └── mod.rs             # sessions, settings, cache
│   └── ui/                    # qt/qml interface
│       ├── mod.rs             # qobject types
│       ├── login_window.rs    # web login & token extraction
│       ├── main_window.rs    # main interface
│       ├── web_view.rs       # webview controller
│       └── styles.rs         # discord theme
├── .github/workflows/
│   ├── ci.yml                 # build, lint, test
│   ├── api-watch.yml          # discord api change monitor
│   └── build-number.yml       # build number auto-updater
└── Cargo.toml
```

## building

### linux

#### prerequisites

- rust 1.83+ (1.93+ recommended)
- qt 6 development libraries
- openssl development libraries
- cmake, pkg-config, g++

#### install dependencies (ubuntu/debian)

```bash
sudo apt-get update && sudo apt-get install -y \
    qt6-base-dev qt6-declarative-dev qt6-tools-dev \
    libssl-dev pkg-config cmake g++
```

#### build & test

```bash
cd skunkcord

# set qt 6 paths (ubuntu/debian)
export QT_INCLUDE_PATH=/usr/include/x86_64-linux-gnu/qt6
export QT_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu
export QMAKE=/usr/bin/qmake6

# build
cargo build --release

# run tests (109 tests)
cargo test

# run
DISCORD_TOKEN="your_token" cargo run
```

### windows

for windows builds, see **[docs/WINDOWS-BUILD.md](docs/WINDOWS-BUILD.md)** which covers:
- 🤖 automated builds via github actions (easiest)
- 💻 native windows build instructions
- 🔧 cross-compilation from linux (advanced)
- 📦 creating distributable packages

## deployment

### option 1: standalone bundle (recommended)

self-contained package with qt libraries included - **no qt installation required** on target system:

```bash
# creates ~34mb package that works on any linux system
./package-bundle.sh
tar -czf skunkcord-linux-standalone.tar.gz skunkcord-bundle/

# on target machine (no qt needed!):
tar -xzf skunkcord-linux-standalone.tar.gz
cd skunkcord-bundle && ./skunkcord.sh
```

### option 2: minimal package

small package requiring qt 6 on target system:

```bash
# creates ~3.6mb package (requires qt installation)
./package.sh
tar -czf skunkcord-linux.tar.gz skunkcord-release/

# on target machine (qt 6 required):
tar -xzf skunkcord-linux.tar.gz
cd skunkcord-release && ./skunkcord
```

**comparison:**
- **standalone**: 34mb, works everywhere, no dependencies → **best for distribution**
- **minimal**: 3.6mb, requires qt installation → best for developers

for detailed deployment instructions, see **[docs/DEPLOYMENT.md](docs/DEPLOYMENT.md)**.

## feature flags

every feature can be toggled in settings. use presets for quick configuration:

| preset | description | risk level |
|--------|-------------|------------|
| **paranoid** | only vanilla behavior + safety features | none |
| **standard** | safe mod features enabled | low |
| **full** | everything enabled | high |

see `src/features/flags.rs` for the complete list of 30 toggleable features with risk descriptions.

## testing

244 automated tests covering:
- feature flag presets and risk levels
- fingerprint generation and super properties
- session management
- cookie handling
- request timing
- build number regex parsing
- hcaptcha detection, widget generation, title parsing
- link preview safety (ssrf, spoofing)
- content sanitization and url tracking removal
- message logger cache operations
- account switcher
- voice encryption mode selection
- markdown rendering (15 format types)
- show hidden channels permissions
- silent messages, pin dms, streamer mode
- proxy url formatting
- read state management
- permission calculator (base + channel overwrites)
- captcha interceptor (detection, extraction, retry headers)
- typing indicator throttle
- multipart file upload builder
- poll creation and results deserialization
- voice connection state machine
- 41 integration tests via mock discord server (wiremock)

## license

mit license — copyright (c) skunk ventures llc. see [LICENSE](LICENSE) for details.
