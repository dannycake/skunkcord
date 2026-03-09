---@meta
--- Skunkcord Plugin API — type definitions for Lua Language Server (LuaLS) intellisense
--- Place this file in your plugin directory or add to workspace for autocomplete.

---@class discord
---@field Events discord.Events
---@field OptionCategory discord.OptionCategory
---@field message_cache discord.MessageCache
---@field ui discord.Ui
local discord = {}

---@class discord.Events
---Discord Gateway event names — use with discord.on()
discord.Events = {
    MESSAGE_CREATE = "MESSAGE_CREATE",
    MESSAGE_UPDATE = "MESSAGE_UPDATE",
    MESSAGE_DELETE = "MESSAGE_DELETE",
    MESSAGE_DELETE_BULK = "MESSAGE_DELETE_BULK",
    MESSAGE_REACTION_ADD = "MESSAGE_REACTION_ADD",
    MESSAGE_REACTION_REMOVE = "MESSAGE_REACTION_REMOVE",
    READY = "READY",
    VOICE_STATE_UPDATE = "VOICE_STATE_UPDATE",
    VOICE_SERVER_UPDATE = "VOICE_SERVER_UPDATE",
    TYPING_START = "TYPING_START",
    PRESENCE_UPDATE = "PRESENCE_UPDATE",
    GUILD_CREATE = "GUILD_CREATE",
    GUILD_UPDATE = "GUILD_UPDATE",
    GUILD_DELETE = "GUILD_DELETE",
    CHANNEL_CREATE = "CHANNEL_CREATE",
    CHANNEL_UPDATE = "CHANNEL_UPDATE",
    CHANNEL_DELETE = "CHANNEL_DELETE",
}

---@class discord.OptionCategory
---Categories for plugin options in settings UI
discord.OptionCategory = {
    General = "general",
    Display = "display",
    Storage = "storage",
    Voice = "voice",
    Privacy = "privacy",
    Advanced = "advanced",
}

---@class discord.MessageCache
---Message cache API (only available when message-logger plugin provides it)
discord.message_cache = {}

---Insert a message into the cache
---@param msg LoggedMessage
function discord.message_cache.insert(msg) end

---Mark a message as deleted
---@param message_id string
function discord.message_cache.mark_deleted(message_id) end

---Record an edit to a message
---@param message_id string
---@param new_content string
function discord.message_cache.record_edit(message_id, new_content) end

---Get a cached message by ID
---@param message_id string
---@return LoggedMessage|nil
function discord.message_cache.get(message_id) end

---@class LoggedMessage
---@field id string
---@field channel_id string
---@field guild_id string|nil
---@field author_id string
---@field author_name string
---@field content string
---@field attachments_json string
---@field embeds_json string
---@field timestamp string
---@field deleted boolean
---@field deleted_at string|nil
---@field edit_history MessageEdit[]

---@class MessageEdit
---@field old_content string
---@field edited_at string

---Register an event handler
---@param event_name string Event name from discord.Events
---@param callback fun(data: table)
function discord.on(event_name, callback) end

---Get full plugin config as table
---@return table<string, any>
function discord.get_config() end

---Get a single config value
---@param key string
---@return any
function discord.get_config_value(key) end

---Set fake mute state (appear muted while receiving audio)
---@param enabled boolean
function discord.set_fake_mute(enabled) end

---Set fake deafen state (appear deafened while hearing)
---@param enabled boolean
function discord.set_fake_deafen(enabled) end

---Get current fake mute state
---@return boolean
function discord.get_fake_mute() end

---Get current fake deafen state
---@return boolean
function discord.get_fake_deafen() end

---Log a debug message
---@param message string
function discord.log(message) end

---@class discord.Ui
---Plugin UI API — add buttons and modals (declare in plugin.json "ui" section)
discord.ui = {}

---Add a button (declare in plugin.json ui.buttons)
---@param id string
---@param label string
---@param tooltip? string
---@param placement? "toolbar"|"message_input"|"channel_header"
function discord.ui.add_button(id, label, tooltip, placement) end

---Add a modal (declare in plugin.json ui.modals)
---@param id string
---@param title string
---@param fields PluginModalField[]
function discord.ui.add_modal(id, title, fields) end

---Show a modal (call from button handler or elsewhere)
---@param modal_id string
function discord.ui.show_modal(modal_id) end

---@class PluginModalField
---@field key string
---@field label string
---@field field_type? "short"|"paragraph"
---@field placeholder? string
---@field required? boolean

return discord
