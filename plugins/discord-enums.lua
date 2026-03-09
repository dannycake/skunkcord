--- Skunkcord Plugin API — Enums and constants
--- Require this file in your plugin for enum values with intellisense:
---   local discord = require("discord-enums")

---@class discord
local discord = {}

---Gateway event names — use with discord.on(event, callback)
discord.Events = {
    MESSAGE_CREATE = "MESSAGE_CREATE",
    MESSAGE_UPDATE = "MESSAGE_UPDATE",
    MESSAGE_DELETE = "MESSAGE_DELETE",
    MESSAGE_DELETE_BULK = "MESSAGE_DELETE_BULK",
    MESSAGE_REACTION_ADD = "MESSAGE_REACTION_ADD",
    MESSAGE_REACTION_REMOVE = "MESSAGE_REACTION_REMOVE",
    MESSAGE_REACTION_REMOVE_ALL = "MESSAGE_REACTION_REMOVE_ALL",
    MESSAGE_REACTION_REMOVE_EMOJI = "MESSAGE_REACTION_REMOVE_EMOJI",
    READY = "READY",
    VOICE_STATE_UPDATE = "VOICE_STATE_UPDATE",
    VOICE_SERVER_UPDATE = "VOICE_SERVER_UPDATE",
    TYPING_START = "TYPING_START",
    PRESENCE_UPDATE = "PRESENCE_UPDATE",
    USER_UPDATE = "USER_UPDATE",
    GUILD_CREATE = "GUILD_CREATE",
    GUILD_UPDATE = "GUILD_UPDATE",
    GUILD_DELETE = "GUILD_DELETE",
    GUILD_MEMBER_ADD = "GUILD_MEMBER_ADD",
    GUILD_MEMBER_UPDATE = "GUILD_MEMBER_UPDATE",
    GUILD_MEMBER_REMOVE = "GUILD_MEMBER_REMOVE",
    CHANNEL_CREATE = "CHANNEL_CREATE",
    CHANNEL_UPDATE = "CHANNEL_UPDATE",
    CHANNEL_DELETE = "CHANNEL_DELETE",
    CHANNEL_PINS_UPDATE = "CHANNEL_PINS_UPDATE",
    THREAD_CREATE = "THREAD_CREATE",
    THREAD_UPDATE = "THREAD_UPDATE",
    THREAD_DELETE = "THREAD_DELETE",
    INTERACTION_CREATE = "INTERACTION_CREATE",
}

---Option categories for plugin.json
discord.OptionCategory = {
    General = "general",
    Display = "display",
    Storage = "storage",
    Voice = "voice",
    Privacy = "privacy",
    Advanced = "advanced",
}

---Option types for plugin.json
discord.OptionType = {
    Boolean = "boolean",
    String = "string",
    Number = "number",
    Select = "select",
}

return discord
