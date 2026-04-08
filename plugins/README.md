<!-- Copyright (c) Skunk Ventures LLC | Last modified: 2025-03-07 | SPDX-License-Identifier: MIT -->

# Skunkcord Plugins

This directory contains built-in and user-installed plugins for Skunkcord.

## Lua Intellisense (LuaLS)

For autocomplete and type checking in your editor:

1. **Copy type definitions**: Copy `discord.d.lua` into your plugin directory
2. **Use enums**: `require("discord-enums")` for `discord.Events`, `discord.OptionCategory`, etc.
3. **LuaLS config**: Add `.luarc.json` to your workspace (see this directory for an example)

The `discord` global is injected by the host at runtime; the `.lua` files provide static types for editors.

## Plugin Structure

Each plugin is a directory containing:

- **plugin.json** — Manifest with metadata, options schema, and event subscriptions
- **main.lua** — Entry script (for custom Lua plugins; built-ins use Rust)

## Built-in Plugins

These plugins ship with Skunkcord (sorted by priority):

| Plugin | Description |
|--------|-------------|
| message-logger | Track deleted and edited messages |
| blur-nsfw | Blur NSFW images/attachments until clicked |
| show-hidden-channels | Display channels you lack permission to view |
| read-all-notifications | One-click mark all channels read |
| clear-urls | Strip tracking params from URLs |
| silent-messages | Send without triggering notifications |
| pin-dms | Pin DMs to top of list |
| no-reply-mention | Replies don't ping by default |
| image-zoom | Click images to zoom |
| always-animate | Force animated avatars/emojis |
| custom-rpc | Game activity / arRPC |

## Installing Custom Plugins

Plugins can be installed from git repositories:

1. **Via UI**: Settings → Plugins → Install from URL
2. **Via CLI**: The app supports `SetPluginEnabled` and `InstallPlugin` actions

Example: Clone a plugin repo into the plugins directory:

```bash
# Plugins directory: ~/.local/share/skunkcord/Skunkcord/plugins/ (Linux)
cd ~/.local/share/skunkcord/Skunkcord/plugins/
git clone https://github.com/example/skunkcord-my-plugin
```

**Refresh from disk**: Use the "Refresh from disk" button in Settings → Plugins to re-scan the plugins directory after adding or removing plugins manually.

**Check for updates**: Use the "Check for updates" button to see if any git-based plugins have updates available. Plugins installed via `git clone` are checked against their remote.

## Plugin Manifest (plugin.json)

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "description": "What it does",
  "version": "1.0.0",
  "author": "Your Name",
  "repository": "https://github.com/you/my-plugin",
  "options": [
    {
      "key": "my_setting",
      "label": "My Setting",
      "type": "boolean",
      "default": true,
      "category": "general"
    }
  ],
  "events": ["MESSAGE_CREATE", "MESSAGE_DELETE"],
  "entry": "main.lua",
  "ui": {
    "buttons": [
      { "id": "export", "label": "Export", "tooltip": "Export data", "placement": "toolbar" }
    ],
    "modals": [
      {
        "id": "export_modal",
        "title": "Export",
        "fields": [
          { "key": "format", "label": "Format", "field_type": "short", "required": true }
        ]
      }
    ]
  }
}
```

### Option Types

- **boolean** — Toggle
- **string** — Text input
- **number** — Numeric (use `min`, `max` for range)
- **select** — Dropdown (use `choices` array)

### Option Categories

- `general`, `display`, `storage`, `privacy`, `advanced`

### Discord Events

Plugins can subscribe to: `MESSAGE_CREATE`, `MESSAGE_UPDATE`, `MESSAGE_DELETE`, `MESSAGE_DELETE_BULK`, `READY`, etc.

## Lua API (when Lua runtime is available)

```lua
-- Register event handler
discord.on("MESSAGE_DELETE", function(data)
  discord.message_cache.mark_deleted(data.id)
end)

-- Get config
local config = discord.get_config()
local cache_size = config.cache_size or 10000

-- Message cache (for message logger)
discord.message_cache.insert({ id = "...", channel_id = "...", ... })
discord.message_cache.mark_deleted(message_id)
discord.message_cache.record_edit(message_id, new_content)

-- Logging
discord.log("Debug message")
```
