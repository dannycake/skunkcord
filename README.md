<!-- Copyright (c) Skunk Ventures LLC | Last modified: 2025-03-07 | SPDX-License-Identifier: MIT -->

# skunkcord client
<img width="1594" height="950" alt="image" src="https://github.com/user-attachments/assets/8a8b2e6a-5714-4182-8601-dbbf56f8a372" />

a user-account discord client built with rust and qt, featuring comprehensive api coverage and built-in client mod features.

**not affiliated with discord.** this project is an independent, community-built client. discord and the discord logo are trademarks of discord inc. we are not endorsed by, sponsored by, or connected with discord inc. in any way.

## legal disclaimer

this software is for educational and personal use only. using self-bots or automated user accounts may violate discord's terms of service. use at your own risk. **never share your user token with anyone.**

## features

### client behavior

- **rate limit retry** — automatic retry with backoff on 429 responses (up to 3 attempts)
- **cookie management** — cloudflare cookies (__dcfduid, __sdcfduid, __cfruid) properly managed
- **socks5 proxy** — runtime-configurable proxy for all traffic (http + gateway websocket)

### core discord functionality

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

### security

- **safe link previews** — ssrf prevention: blocks private ips, localhost, internal hostnames
- **url spoofing detection** — catches percent-encoded domain tricks in embeds
- **content sanitization** — html entity escaping, data uri blocking
- **clearurls** — strips 37+ tracking parameters (utm_*, fbclid, gclid, etc.)
- **token security** — tokens never logged, stored securely

### built-in client mod features

| feature | description |
|---------|-------------|
| **message logger** | track deleted/edited messages with full edit history and search |
| **show hidden channels** | display channels you can't access with lock icon |
| **clearurls** | strip tracking params from outgoing urls |
| **silent message toggle** | send messages without triggering notifications |
| **no reply mention** | replies don't ping by default |
| **pin dms** | pin dm conversations to top of list |
| **read all notifications** | one-click mark everything as read |

### markdown rendering

full discord-flavored markdown to html:
- **bold**, *italic*, __underline__, ~~strikethrough~~
- \`inline code\`, \`\`\`code blocks\`\`\`
- > block quotes, ### headings
- ||spoilers|| (click to reveal)
- @mentions, #channels, @roles
- custom emoji `:name:` with cdn images
- timestamps with relative formatting
- [masked links](url)

### multi-account support

- **quick account switcher** — avatar ring in sidebar, click to swap
- **per-account settings** — different settings per account
- **per-account proxy** — different socks5 proxy per account

### gateway

- **real-time events** — 40+ event types handled:
  - messages: create, update, delete, bulk delete, reactions
  - guilds: members, bans, roles, emojis
  - channels: create, update, delete, pins, threads
  - interactions, relationships, presence, typing
- **auto-reconnect** — exponential backoff with jitter (up to 50 retries)
- **resume** — uses saved session_id and sequence to resume without re-identifying
- **proxied websocket** — gateway connections route through socks5 when proxy is enabled

### proxy

all traffic is routed through the proxy when enabled — both http rest api calls and the gateway websocket connection. configured at runtime from the settings panel:

- **socks5 only** — simple host/port/optional auth
- **runtime toggle** — enable/disable without restarting
- **auto-reconnect** — gateway reconnects through the proxy automatically when settings change

## architecture

```
skunkcord/
├── src/
│   ├── main.rs                    # application entry point
│   ├── lib.rs                     # library root, error types
│   ├── client/                    # http client & api
│   │   ├── mod.rs                 # discordclient, rate limiting
│   │   ├── api.rs                 # core rest api endpoints
│   │   ├── session.rs             # authentication & sessions
│   │   ├── account_switcher.rs    # multi-account management
│   │   ├── attachments.rs         # file upload (multipart)
│   │   ├── captcha_interceptor.rs # captcha detection in http responses
│   │   ├── cookies.rs             # cloudflare cookie management
│   │   ├── interactions.rs        # slash commands & components
│   │   ├── invites.rs             # invite endpoints
│   │   ├── permissions.rs         # permission calculator
│   │   ├── reactions.rs           # reaction endpoints
│   │   ├── read_states.rs         # unread tracking
│   │   ├── threads.rs             # thread endpoints
│   │   └── typing.rs              # typing indicator throttle
│   ├── captcha/                   # hcaptcha enterprise handling
│   │   ├── mod.rs                 # detection, parsing, state machine
│   │   └── widget.rs              # widget html generation with rqdata
│   ├── features/                  # client mod features
│   │   ├── message_logger.rs      # deleted/edited message tracking
│   │   ├── clear_urls.rs          # tracking parameter removal
│   │   ├── silent_messages.rs     # suppress notifications flag
│   │   ├── emoji_picker.rs        # unicode emoji search & recent
│   │   ├── gif_picker.rs          # tenor gif search
│   │   └── notifications.rs       # notification config & muting
│   ├── gateway/                   # websocket gateway
│   │   ├── mod.rs                 # connection, heartbeat, reconnect, socks5 proxy
│   │   ├── events.rs              # 40+ event types
│   │   └── payloads.rs            # gateway payloads & opcodes
│   ├── rendering/                 # message display
│   │   └── markdown.rs            # discord markdown to html
│   ├── security/                  # safety & privacy
│   │   ├── link_preview.rs        # ssrf prevention, url safety
│   │   └── content.rs             # sanitization, tracking removal
│   ├── proxy/                     # socks5 proxy support
│   │   └── mod.rs                 # proxy config, url generation
│   ├── storage/                   # persistence
│   │   └── mod.rs                 # sessions, settings, proxy config
│   └── ui/                        # qt/qml interface
│       └── app_controller.rs      # qml-to-rust bridge
├── .github/workflows/
│   ├── ci.yml                     # build, lint, test
│   └── api-watch.yml              # discord api change monitor
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

#### build & run

```bash
cd skunkcord

# set qt 6 paths (ubuntu/debian)
export QT_INCLUDE_PATH=/usr/include/x86_64-linux-gnu/qt6
export QT_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu
export QMAKE=/usr/bin/qmake6

# build
cargo build --release

# run tests
cargo test

# run
DISCORD_TOKEN="your_token" cargo run
```

### windows

for windows builds, see **[docs/WINDOWS-BUILD.md](docs/WINDOWS-BUILD.md)**.

## deployment

### standalone bundle (recommended)

self-contained package with qt libraries included — no qt installation required on target system:

```bash
./package-bundle.sh
tar -czf skunkcord-linux-standalone.tar.gz skunkcord-bundle/

# on target machine:
tar -xzf skunkcord-linux-standalone.tar.gz
cd skunkcord-bundle && ./skunkcord.sh
```

### minimal package

small package requiring qt 6 on target system:

```bash
./package.sh
tar -czf skunkcord-linux.tar.gz skunkcord-release/

# on target machine (qt 6 required):
tar -xzf skunkcord-linux.tar.gz
cd skunkcord-release && ./skunkcord
```

for detailed deployment instructions, see **[docs/DEPLOYMENT.md](docs/DEPLOYMENT.md)**.

## license

mit license — copyright (c) skunk ventures llc. see [LICENSE](LICENSE) for details.
