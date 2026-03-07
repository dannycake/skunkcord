-- Message Logger plugin
-- Tracks deleted and edited messages using the Discord Qt message cache API.
-- This plugin is also implemented natively in Rust for performance;
-- this Lua file documents the plugin API for custom plugin authors.

-- discord.on("MESSAGE_CREATE", function(data)
--   -- data contains: message (id, channel_id, content, author, etc.)
--   discord.message_cache.insert({
--     id = data.message.id,
--     channel_id = data.message.channel_id,
--     guild_id = data.guild_id,
--     author_id = data.message.author.id,
--     author_name = data.message.author.username or "Unknown",
--     content = data.message.content or "",
--     attachments_json = data.message.attachments and json.encode(data.message.attachments) or "[]",
--     embeds_json = data.message.embeds and json.encode(data.message.embeds) or "[]",
--     timestamp = data.message.timestamp or "",
--     deleted = false,
--     deleted_at = nil,
--     edit_history = {}
--   })
-- end)

-- discord.on("MESSAGE_DELETE", function(data)
--   discord.message_cache.mark_deleted(data.id)
-- end)

-- discord.on("MESSAGE_UPDATE", function(data)
--   if data.content then
--     discord.message_cache.record_edit(data.id, data.content)
--   end
-- end)

-- discord.on("MESSAGE_DELETE_BULK", function(data)
--   for _, id in ipairs(data.ids) do
--     discord.message_cache.mark_deleted(id)
--   end
-- end)
