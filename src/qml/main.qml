// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

import QtQuick
import QtQuick.Window
import QtQuick.Controls
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects
import QtWebEngine
import "Discord.js" as D
import "components" 1.0

Window {
    id: root
    visible: true
    width: 1100
    height: 700
    minimumWidth: 700
    minimumHeight: 400
    title: isLoggedIn ? (currentGuildName || "Skunkcord") : "Login"
    color: theme.bgBase

    // ─── Fonts ───
    FontLoader { id: jbMono; source: "https://raw.githubusercontent.com/JetBrains/JetBrainsMono/master/fonts/ttf/JetBrainsMono-Regular.ttf" }
    readonly property string fontFamily: jbMono.status === FontLoader.Ready ? jbMono.name : "Consolas, Monaco, Courier New, monospace"

    // ─── Theme ───
    // Discord 2024/2025 dark theme — exact palette
    QtObject {
        id: theme

        // Backgrounds — Muted Violet theme
        readonly property color bgBase:       "#2a2139"
        readonly property color bgPrimary:    "#2a2139"
        readonly property color bgSecondary:  "#211a30"
        readonly property color bgTertiary:   "#191425"
        readonly property color bgHover:      "#342b44"
        readonly property color bgActive:     "#3e3450"
        readonly property color bgElevated:   "#211a30"
        readonly property color bgFloating:   "#110e1a"
        readonly property color bgModifier:   "#ffffff08"

        // Text — Discord hierarchy
        readonly property color textNormal:    "#dbdee1"
        readonly property color textSecondary: "#b5bac1"
        readonly property color textMuted:     "#80848e"
        readonly property color textFaint:     "#6d6f73"

        // Accent — Discord blurple
        readonly property color accent:       "#5865f2"
        readonly property color accentHover:  "#4752c4"
        readonly property color accentLight:  "#7289da"
        readonly property color accentGlow:   "#5865f230"
        readonly property color accentMuted:  "#5865f218"

        // Semantic
        readonly property color positive:  "#23a55a"
        readonly property color warning:   "#f0b132"
        readonly property color danger:    "#f23f43"
        readonly property color info:      "#5e9eff"

        // Borders
        readonly property color border:        "#3a3048"
        readonly property color borderSubtle:  "#ffffff0a"
        readonly property color separator:     "#3a3048"

        // Input and icons
        readonly property color inputBg:       "#302840"
        readonly property color channelIcon:  "#80848e"
        readonly property color headerPrimary: "#f2f3f5"

        // Status
        readonly property color online:  "#23a55a"
        readonly property color idle:    "#f0b132"
        readonly property color dnd:     "#f23f43"
        readonly property color offline: "#80848e"

        // Misc
        readonly property color mentionBg:    "#5865f210"
        readonly property color mentionText:  "#c9cdfb"
        readonly property color mentionPillBg: "#5865f2"   // Discord blurple for @mention count badges

        // Voice
        readonly property color voicePositive:   "#23a55a"
        readonly property color voiceConnecting: "#f0b132"
        readonly property color voiceSpeaking:   "#23a55a"
        readonly property color voiceSpeakingGlow: "#23a55a40"

        // Stats panel
        readonly property color statsBg:     "#0c0912"
        readonly property color statsFg:     "#40c463"
        readonly property color statsLabel:  "#6d7178"
        readonly property string monospace:  "JetBrains Mono, Fira Code, Consolas, monospace"

        // Layout — Discord dimensions
        readonly property int guildBarWidth:   72
        readonly property int channelBarWidth: 240
        readonly property int headerHeight:    48
        readonly property int userPanelHeight: 52
        readonly property int messageInputH:   44

        // Radii
        readonly property int radiusSmall: 4
        readonly property int radiusMed:   8
        readonly property int radiusLarge: 12
        readonly property int radiusXl:    16

        // Animations
        readonly property int animFast:   100
        readonly property int animNormal: 150
        readonly property int animSlow:   250
    }

    // ─── State (bound to app controller when available) ───
    property bool isLoggedIn: app ? app.is_logged_in : false
    property string currentUserId: app ? app.user_id : ""
    property string currentUserName: app ? app.user_name : ""
    property string currentUserAvatar: app ? app.user_avatar : ""
    property string currentStatus: "online"
    property string connectionState: app ? app.connection_state : ""
    property string typingDisplay: app ? app.typing_display : ""
    property string typingDisplayJson: app ? app.typing_display_json : "[]"
    property int presenceVersion: app ? app.presence_version : 0
    property var accountsList: []

    function getStatusColor(status) {
        if (!status || status.length === 0) return theme.textFaint
        switch (String(status)) {
            case "online": return "#43b581"
            case "idle": return "#faa61a"
            case "dnd": return "#f04747"
            case "offline": return "#747f8d"
            default: return theme.textFaint
        }
    }
    property var typingDisplayList: (function() {
        try {
            var s = String(typingDisplayJson || "[]")
            if (s.length === 0) return []
            return JSON.parse(s)
        } catch (e) { return [] }
    })()
    property string voiceConnectionState: app ? app.voice_connection_state : ""

    // Reply state
    property string replyToMessageId: ""
    property string replyToAuthor: ""
    property string replyToAuthorColor: ""
    property string replyToContent: ""

    // Silent message toggle
    property bool silentMode: false
    property string currentGuildId: ""
    property string currentGuildName: ""
    property string currentChannelId: ""
    property string currentChannelName: ""
    property int currentChannelType: 0
    // Voice/stage channels (types 2, 13) get a different view instead of messages
    readonly property bool isVoiceChannel: currentChannelType === 2 || currentChannelType === 13

    // Home sub-view when currentGuildId === "": "friends" or "dms"
    property string homeSubView: "dms"
    
    // Update typing display when channel changes
    onCurrentChannelIdChanged: {
        if (app) app.update_typing_for_channel(currentChannelId)
    }

    // Login state from app controller
    property string loginError: app ? app.error_message : ""
    property bool loginLoading: app ? app.is_loading : false

    // Track whether we've done the initial DM REST refresh after connect
    property bool _didInitialDmRefresh: false

    // Feature flags state
    property bool showProxyDialogOnStartup: true
    // UI Tweaks
    property bool showTypingInChannelList: true

    // Plugin UI — plugin_id -> { buttons, modals }; updated from consume_plugin_ui
    property var pluginUiData: ({})
    // Flattened list of toolbar buttons for Repeater: [{ pluginId, buttonId, label, tooltip }]
    property var pluginToolbarButtons: []
    // Plugin enabled states (from get_plugin_enabled_states), used in Settings
    property var pluginEnabledStates: ({})
    // Plugin list from manifests (from get_plugin_list), used in Settings
    property var pluginList: []
    // Plugin update check result (e.g. "Updates available for: x, y" or "All plugins up to date")
    property string pluginUpdateStatus: ""
    // Deleted message display style: "strikethrough", "faded", "deleted" (message logger)
    property string deletedMessageStyle: "strikethrough"

    Component.onCompleted: {
        if (app) deletedMessageStyle = app.get_deleted_message_style()
        if (showProxyDialogOnStartup && !isLoggedIn) {
            proxyConfigPopup.open()
            showProxyDialogOnStartup = false
        }
    }
    onIsLoggedInChanged: {
        if (!isLoggedIn && !proxyConfigPopup.opened) {
            showProxyDialogOnStartup = true
            proxyConfigPopup.open()
        }
    }

    // ── In-memory caches ──
    // Channels per guild (guild_id -> array of channel objects)
    property var channelCacheMap: ({})
    // Messages per channel (channel_id -> array of message objects)
    property var messageCacheMap: ({})
    // Cached DM channel list
    property var dmChannelCache: []
    // Last selected channel per guild (guild_id -> channel_id) for restoring position
    property var lastChannelPerGuild: ({})
    // Collapsed categories per guild: guildId -> { categoryId: true }
    property var collapsedCategories: ({})
    // Full channel list per guild including categories (for rebuilds)
    property var fullChannelCache: ({})
    // Show channels user cannot view (locked), with lock icon
    property bool showHiddenChannels: false
    // My profile in current guild (guildId, nick, roles) for profile popup
    property var myGuildProfile: null
    // "self" = show my profile (roles from myGuildProfile); or { displayName, username, avatarUrl } for member list
    property var profilePopupTarget: null

    // Stable empty array for Repeater models (avoids binding loops)
    readonly property var emptyRepeaterModel: []

    // Discord's default avatar color palette
    readonly property var discordAvatarColors: [
        "#5865F2", // blurple
        "#747F8D", // gray
        "#3BA55C", // green
        "#FAA61A", // yellow
        "#ED4245", // red
        "#EB459E"  // fuchsia
    ]

    // Pick a Discord avatar color from a name or ID string (delegates to Discord.js)
    function avatarColor(str) {
        return D.avatarColor(str)
    }

    // Helper: save current channel list to cache for the active guild (plain object copies to avoid stale ListModel proxies)
    function cacheCurrentChannels() {
        if (currentGuildId !== "" && channelModel.count > 0) {
            var arr = []
            for (var i = 0; i < channelModel.count; i++) {
                var it = channelModel.get(i)
                arr.push({
                    channelId: it.channelId, guildId: it.guildId,
                    name: it.name, channelType: it.channelType,
                    position: it.position, parentId: it.parentId || "",
                    hasUnread: it.hasUnread, mentionCount: it.mentionCount,
                    isHidden: it.isHidden || false
                })
            }
            var m = channelCacheMap
            m[currentGuildId] = arr
            channelCacheMap = m
        }
    }

    // Helper: save current messages to cache for the active channel (plain object copies to avoid stale ListModel proxies)
    function cacheCurrentMessages() {
        if (currentChannelId !== "" && messageModel.count > 0) {
            var arr = []
            for (var i = 0; i < messageModel.count; i++) {
                var it = messageModel.get(i)
                arr.push({
                    messageId: it.messageId, channelId: it.channelId,
                    authorName: it.authorName, authorId: it.authorId,
                    authorAvatarUrl: it.authorAvatarUrl, content: it.content,
                    timestamp: it.timestamp, isDeleted: it.isDeleted,
                    messageType: it.messageType,
                    replyAuthorName: it.replyAuthorName, replyContent: it.replyContent,
                    replyAuthorId: it.replyAuthorId,
                    mentionsMe: it.mentionsMe, mentionEveryone: it.mentionEveryone,
                    authorRoleColor: it.authorRoleColor,
                    authorRoleName: it.authorRoleName,
                    authorPublicFlags: it.authorPublicFlags || 0,
                    authorBot: it.authorBot || false,
                    authorPremiumType: it.authorPremiumType || 0,
                    attachmentsJson: it.attachmentsJson, stickersJson: it.stickersJson,
                    embedsJson: it.embedsJson || "[]",
                    contentHtml: it.contentHtml, reactions: it.reactions
                })
            }
            var m = messageCacheMap
            m[currentChannelId] = arr
            messageCacheMap = m
        }
    }

    // Build flat channel list in Discord order: uncategorized first, then each category + its children (respecting collapse and showHiddenChannels)
    function buildChannelList(channels, guildId) {
        if (!channels || channels.length === 0) return []
        var categories = []
        var uncategorized = []
        var byParent = {}
        for (var i = 0; i < channels.length; i++) {
            var ch = channels[i]
            if (ch.isHidden && !showHiddenChannels) continue
            if (ch.channelType === 4) {
                categories.push(ch)
            } else if (!ch.parentId || ch.parentId === "") {
                uncategorized.push(ch)
            } else {
                if (!byParent[ch.parentId]) byParent[ch.parentId] = []
                byParent[ch.parentId].push(ch)
            }
        }
        uncategorized.sort(function(a, b) { return a.position - b.position })
        categories.sort(function(a, b) { return a.position - b.position })
        for (var k in byParent)
            byParent[k].sort(function(a, b) { return a.position - b.position })
        var collapsed = collapsedCategories[guildId] || {}
        var out = []
        for (var u = 0; u < uncategorized.length; u++) out.push(uncategorized[u])
        for (var c = 0; c < categories.length; c++) {
            var cat = categories[c]
            out.push(cat)
            if (!collapsed[cat.channelId]) {
                var kids = byParent[cat.channelId] || []
                for (var k = 0; k < kids.length; k++) out.push(kids[k])
            }
        }
        return out
    }

    function toggleCategory(guildId, categoryId) {
        var gc = collapsedCategories[guildId] || {}
        gc[categoryId] = !gc[categoryId]
        var m = collapsedCategories
        m[guildId] = gc
        collapsedCategories = m
        rebuildChannelModel(guildId)
    }

    function rebuildChannelModel(guildId) {
        var full = fullChannelCache[guildId]
        if (!full) return
        var built = buildChannelList(full, guildId)
        channelModel.clear()
        for (var i = 0; i < built.length; i++) channelModel.append(built[i])
    }

    // Helper: restore channels from cache, returns true if cache hit
    function restoreChannelsFromCache(guildId) {
        var full = fullChannelCache[guildId]
        if (full && full.length > 0) {
            var built = buildChannelList(full, guildId)
            channelModel.clear()
            for (var i = 0; i < built.length; i++) channelModel.append(built[i])
            return true
        }
        return false
    }

    // Helper: handle link click — intercept Discord invite links, open others externally
    function handleLinkClick(link) {
        if (/discord\.gg\/|discord\.com\/invite\//.test(link)) {
            // It's an invite link — join in-app
            joinServerPopup.joinError = ""
            joinServerPopup.joinLoading = true
            joinServerPopup.open()
            joinServerPopup.inviteField.text = link
            if (app) app.join_guild_by_invite(link)
        } else {
            Qt.openUrlExternally(link)
        }
    }

    // Helper: restore messages from cache, returns true if cache hit
    function restoreMessagesFromCache(channelId) {
        var cached = messageCacheMap[channelId]
        if (cached && cached.length > 0) {
            messageModel.clear()
            for (var i = 0; i < cached.length; i++)
                messageModel.append(cached[i])
            messageList.hasMoreHistory = cached.length >= 50
            messageList.isLoadingMore = false
            return true
        }
        return false
    }

    // Timer to poll for backend updates and consume pending data
    Timer {
        interval: 100
        running: true
        repeat: true
        onTriggered: {
            if (!app) return

            app.check_for_updates()

            try {
                var aj = app.accounts_json
                accountsList = (aj && aj.length > 0) ? JSON.parse(aj) : []
            } catch (e) { accountsList = [] }

            // Consume pending guilds
            var gj = app.consume_guilds()
            if (gj.length > 2) {
                try {
                    var guilds = JSON.parse(gj)
                    guildModel.clear()
                    for (var i = 0; i < guilds.length; i++) {
                        guildModel.append(guilds[i])
                    }
                    // Auto-refresh DMs via REST on first connect (READY may
                    // only contain recipient_ids, not full user objects)
                    if (currentGuildId === "" && !root._didInitialDmRefresh) {
                        root._didInitialDmRefresh = true
                        app.select_guild("")
                    }
                } catch(e) { console.log("Guild parse error:", e) }
            }

            // Consume pending channels (from SelectGuild response)
            var cj = app.consume_channels()
            if (cj.length > 2) {
                try {
                    var channels = JSON.parse(cj)
                    var chGuildId = (channels.length > 0 && channels[0].guildId) ? channels[0].guildId : currentGuildId
                    var fMap = fullChannelCache
                    fMap[chGuildId] = channels
                    fullChannelCache = fMap
                    var cMap = channelCacheMap
                    cMap[chGuildId] = buildChannelList(channels, chGuildId)
                    channelCacheMap = cMap
                    if (chGuildId === currentGuildId || currentGuildId === "") {
                        var built = buildChannelList(channels, chGuildId)
                        channelModel.clear()
                        for (var jj = 0; jj < built.length; jj++) channelModel.append(built[jj])
                    }
                } catch(e) { console.log("Channel parse error:", e) }
            }

            var pj = app.consume_my_profile()
            if (pj.length > 2) {
                try { myGuildProfile = JSON.parse(pj) } catch(e) { }
            }

            // Consume DM channels (from SelectGuild("") response)
            var dj = app.consume_dm_channels()
            if (dj.length > 2) {
                try {
                    var dms = JSON.parse(dj)
                    // Update cache
                    dmChannelCache = dms
                    // Only update visible model if Home is selected
                    if (currentGuildId === "") {
                        dmChannelModel.clear()
                        for (var d = 0; d < dms.length; d++) {
                            dmChannelModel.append(dms[d])
                        }
                    }
                } catch(e) { console.log("DM channel parse error:", e) }
            }

            // Consume relationships (friends, pending, blocked) — real-time updates
            var relj = app.consume_relationships()
            if (relj.length > 2) {
                try {
                    var rels = JSON.parse(relj)
                    relationshipModel.clear()
                    for (var r = 0; r < rels.length; r++) {
                        var rec = rels[r]
                        var typeNum = rec.relationship_type !== undefined ? rec.relationship_type : (rec.relationshipTypeNum || 0)
                        var typeStr = "friend"
                        if (typeNum === 2) typeStr = "blocked"
                        else if (typeNum === 3) typeStr = "incoming"
                        else if (typeNum === 4) typeStr = "outgoing"
                        relationshipModel.append({
                            userId: rec.user_id || "",
                            username: rec.username || "",
                            avatarUrl: rec.avatar_url || "",
                            relationshipType: typeStr,
                            relationshipTypeNum: typeNum
                        })
                    }
                } catch(e) { console.log("Relationships parse error:", e) }
            }

            // Consume loaded messages from REST (replaces entire message list)
            var lmj = app.consume_loaded_messages()
            if (lmj.length > 2) {
                try {
                    var loaded = JSON.parse(lmj)
                    // Determine which channel these messages belong to
                    var msgChId = (loaded.length > 0 && loaded[0].channelId) ? loaded[0].channelId : currentChannelId
                    // Cache messages for this channel
                    var mMap = messageCacheMap
                    mMap[msgChId] = loaded
                    messageCacheMap = mMap
                    // Only update visible model if these messages are for the active channel
                    if (msgChId === currentChannelId) {
                        messageModel.clear()
                        // Messages from API come newest-first, ListView is BottomToTop,
                        // so we add them in order (index 0 = newest at bottom)
                        for (var l = 0; l < loaded.length; l++) {
                            messageModel.append(loaded[l])
                        }
                        // If we got fewer than 50 messages, there's no more history
                        messageList.hasMoreHistory = loaded.length >= 50
                        messageList.isLoadingMore = false
                    }
                } catch(e) { console.log("Loaded messages parse error:", e) }
            }

            // Consume new gateway messages — only add if they belong to the active channel
            var mj = app.consume_messages()
            if (mj.length > 2) {
                try {
                    var messages = JSON.parse(mj)
                    for (var k = 0; k < messages.length; k++) {
                        if (messages[k].channelId === currentChannelId) {
                            messageModel.insert(0, messages[k])
                        }
                    }
                } catch(e) { console.log("Message parse error:", e) }
            }

            // Consume message edits — update content in-place
            var ej = app.consume_message_edits()
            if (ej.length > 2) {
                try {
                    var edits = JSON.parse(ej)
                    for (var ei = 0; ei < edits.length; ei++) {
                        if (edits[ei].channelId === currentChannelId) {
                            for (var mi = 0; mi < messageModel.count; mi++) {
                                if (messageModel.get(mi).messageId === edits[ei].messageId) {
                                    messageModel.setProperty(mi, "content", edits[ei].newContent)
                                    messageModel.setProperty(mi, "contentHtml", "")
                                    break
                                }
                            }
                        }
                    }
                } catch(e) { console.log("Edit parse error:", e) }
            }

            // Consume message deletions — remove from model or show as deleted (message logger)
            var dj2 = app.consume_message_deletions()
            if (dj2.length > 2) {
                try {
                    var dels = JSON.parse(dj2)
                    for (var di = 0; di < dels.length; di++) {
                        if (dels[di].channelId !== currentChannelId) continue
                        var d = dels[di]
                        if (d.isDeleted && d.content !== undefined) {
                            // Message logger: show as deleted instead of removing
                            for (var mdi = 0; mdi < messageModel.count; mdi++) {
                                if (messageModel.get(mdi).messageId === d.messageId) {
                                    messageModel.setProperty(mdi, "isDeleted", true)
                                    messageModel.setProperty(mdi, "content", d.content || "")
                                    break
                                }
                            }
                            // If not in view, add placeholder (optional)
                        } else {
                            for (var mdi = messageModel.count - 1; mdi >= 0; mdi--) {
                                if (messageModel.get(mdi).messageId === d.messageId) {
                                    messageModel.remove(mdi)
                                    break
                                }
                            }
                        }
                    }
                } catch(e) { console.log("Delete parse error:", e) }
            }

            // Consume reaction updates — update message reactions in-place
            var ruj = app.consume_reaction_updates()
            if (ruj.length > 2) {
                try {
                    var reactionUpdates = JSON.parse(ruj)
                    for (var ru = 0; ru < reactionUpdates.length; ru++) {
                        var ruItem = reactionUpdates[ru]
                        if (ruItem.channelId === currentChannelId) {
                            for (var rmi = 0; rmi < messageModel.count; rmi++) {
                                if (messageModel.get(rmi).messageId === ruItem.messageId) {
                                    messageModel.setProperty(rmi, "reactions", JSON.stringify(ruItem.reactions || []))
                                    break
                                }
                            }
                        }
                    }
                } catch(e) { console.log("Reaction updates parse error:", e) }
            }

            // Consume unread updates — update channel and guild badges
            var uuj = app.consume_unread_updates()
            if (uuj.length > 2) {
                try {
                    var unreadUpdates = JSON.parse(uuj)
                    var guildsTouched = {}
                    for (var uu = 0; uu < unreadUpdates.length; uu++) {
                        var u = unreadUpdates[uu]
                        var gid = u.guildId || ""
                        var chId = u.channelId || ""
                        if (!chId) continue
                        if (gid) {
                            var chList = fullChannelCache[gid]
                            if (chList) {
                                for (var ci = 0; ci < chList.length; ci++) {
                                    if (chList[ci].channelId === chId) {
                                        chList[ci].hasUnread = !!u.hasUnread
                                        chList[ci].mentionCount = u.mentionCount || 0
                                        guildsTouched[gid] = true
                                        break
                                    }
                                }
                            }
                        }
                    }
                    for (var gt in guildsTouched) {
                        var chList = fullChannelCache[gt]
                        if (chList) {
                            var hasUnread = false
                            var mentionCount = 0
                            for (var cj = 0; cj < chList.length; cj++) {
                                if (chList[cj].hasUnread) hasUnread = true
                                mentionCount += chList[cj].mentionCount || 0
                            }
                            for (var gi = 0; gi < guildModel.count; gi++) {
                                if (guildModel.get(gi).guildId === gt) {
                                    guildModel.setProperty(gi, "hasUnread", hasUnread)
                                    guildModel.setProperty(gi, "mentionCount", mentionCount)
                                    break
                                }
                            }
                        }
                    }
                    if (guildsTouched[currentGuildId]) {
                        rebuildChannelModel(currentGuildId)
                    }
                } catch(e) { console.log("Unread updates parse error:", e) }
            }

            // Consume more (older) messages — append to end of model (top of visual list)
            var mmj = app.consume_more_messages()
            if (mmj.length > 2) {
                try {
                    var moreData = JSON.parse(mmj)
                    if (moreData.channelId === currentChannelId) {
                        var moreMsgs = moreData.messages
                        for (var mi2 = 0; mi2 < moreMsgs.length; mi2++) {
                            messageModel.append(moreMsgs[mi2])
                        }
                        messageList.hasMoreHistory = moreData.hasMore
                    }
                    messageList.isLoadingMore = false
                } catch(e) {
                    console.log("More messages parse error:", e)
                    messageList.isLoadingMore = false
                }
            }

            // Consume voice state changes
            var vsj = app.consume_voice_state()
            if (vsj.length > 0) {
                if (vsj.indexOf("joined:") === 0) {
                    isVoiceConnected = true
                } else if (vsj === "disconnected") {
                    isVoiceConnected = false
                    voiceChannelName = ""
                    voiceChannelId = ""
                    voiceGuildId = ""
                    voiceParticipantModel.clear()
                } else if (vsj.indexOf("mute:") === 0) {
                    isMuted = (vsj === "mute:true")
                } else if (vsj.indexOf("deafen:") === 0) {
                    var deaf = (vsj === "deafen:true")
                    isDeafened = deaf
                    if (deaf) isMuted = true
                } else if (vsj.indexOf("fake_deafen:") === 0) {
                    isFakeDeafened = (vsj === "fake_deafen:true")
                }
            }

            // Consume voice participants (only when viewing a voice channel)
            var vpj = app.consume_voice_participants()
            if (vpj.length > 2 && isVoiceChannel && currentChannelId !== "") {
                try {
                    var vpData = JSON.parse(vpj)
                    if (vpData.channelId === currentChannelId && vpData.participants) {
                        voiceParticipantModel.clear()
                        for (var vpi = 0; vpi < vpData.participants.length; vpi++) {
                            voiceParticipantModel.append(vpData.participants[vpi])
                        }
                    }
                } catch(e) { console.log("Voice participants parse error:", e) }
            }

            // Consume speaking changes — update participant cards
            var suj = app.consume_speaking_users()
            if (suj.length > 2) {
                try {
                    var suArr = JSON.parse(suj)
                    for (var sui = 0; sui < suArr.length; sui++) {
                        var uid = suArr[sui].userId
                        var sp = !!suArr[sui].speaking
                        for (var spi = 0; spi < voiceParticipantModel.count; spi++) {
                            if (voiceParticipantModel.get(spi).userId === uid) {
                                voiceParticipantModel.setProperty(spi, "speaking", sp)
                                break
                            }
                        }
                    }
                } catch(e) { console.log("Speaking parse error:", e) }
            }

            // Consume voice stats
            var vstj = app.consume_voice_stats()
            if (vstj.length > 2) {
                try {
                    var vst = JSON.parse(vstj)
                    voiceStatsPing = String(vst.pingMs != null ? vst.pingMs : "—")
                    voiceStatsEncryption = vst.encryptionMode || "—"
                    voiceStatsEndpoint = vst.endpoint || "—"
                    voiceStatsSsrc = vst.ssrc != null ? String(vst.ssrc) : "—"
                    voiceStatsPacketsSent = vst.packetsSent != null ? String(vst.packetsSent) : "0"
                    voiceStatsPacketsReceived = vst.packetsReceived != null ? String(vst.packetsReceived) : "0"
                    voiceStatsDuration = vst.connectionDurationSecs != null ? String(vst.connectionDurationSecs) : "0"
                } catch(e) { console.log("Voice stats parse error:", e) }
            }

            // Consume GIF results
            var gifj = app.consume_gifs()
            if (gifj.length > 2) {
                try {
                    var gifs = JSON.parse(gifj)
                    gifModel.clear()
                    for (var gi = 0; gi < gifs.length; gi++) {
                        gifModel.append(gifs[gi])
                    }
                    gifSearchPending = false
                } catch(e) { console.log("GIF parse error:", e); gifSearchPending = false }
            }

            // Consume sticker packs (when emoji popup is on Stickers tab)
            if (emojiPopup.visible && emojiPickerTab === 1 && app) {
                var spj = app.consume_sticker_packs()
                if (spj.length > 2) {
                    try {
                        var packs = JSON.parse(spj)
                        stickerPacksModel.clear()
                        stickerPickerModel.clear()
                        for (var pi = 0; pi < packs.length; pi++) {
                            var pack = packs[pi]
                            var stickerList = pack.stickers || []
                            stickerPacksModel.append({ packName: pack.name || "Stickers", stickers: stickerList })
                            for (var si = 0; si < stickerList.length; si++) {
                                var s = stickerList[si]
                                stickerPickerModel.append({
                                    id: s.id,
                                    name: s.name || "",
                                    url: s.url || ("https://cdn.discordapp.com/stickers/" + s.id + ".png")
                                })
                            }
                        }
                    } catch(e) { console.log("Sticker packs parse error:", e) }
                }
            }

            // Consume guild emojis (when emoji popup is on Server tab)
            if (emojiPopup.visible && emojiPickerTab === 2 && app) {
                var gej = app.consume_guild_emojis()
                if (gej.length > 2) {
                    try {
                        var emojis = JSON.parse(gej)
                        guildEmojiPickerModel.clear()
                        for (var ei = 0; ei < emojis.length; ei++) {
                            guildEmojiPickerModel.append(emojis[ei])
                        }
                    } catch(e) { console.log("Guild emojis parse error:", e) }
                }
            }

            // Consume pinned messages (when pins popup is open)
            if (pinsPopup.opened && app) {
                var pj = app.consume_pins()
                if (pj.length > 2) {
                    try {
                        var pinList = JSON.parse(pj)
                        pinsModel.clear()
                        for (var pi = 0; pi < pinList.length; pi++) {
                            pinsModel.append(pinList[pi])
                        }
                    } catch(e) { console.log("Pins parse error:", e) }
                }
            }

            // Consume member list (for current guild)
            // Consume plugin UI (buttons, modals from enabled plugins)
            var puj = app.consume_plugin_ui()
            if (puj && String(puj).length > 2) {
                try {
                    var pu = JSON.parse(String(puj))
                    if (Object.keys(pu).length > 0) {
                        pluginUiData = pu
                        var out = []
                        for (var _pid in pu) {
                            var _btns = (pu[_pid] && pu[_pid].buttons) || []
                            for (var _i = 0; _i < _btns.length; _i++) {
                                if ((_btns[_i].placement || "toolbar") === "toolbar") {
                                    out.push({ pluginId: _pid, buttonId: _btns[_i].id, label: _btns[_i].label || _btns[_i].id, tooltip: _btns[_i].tooltip || "" })
                                }
                            }
                        }
                        pluginToolbarButtons = out
                    }
                } catch(e) { }
            }

            // Plugins refreshed from disk — reload plugin list
            if (app && app.consume_plugins_refreshed()) {
                var pl = app.get_plugin_list()
                if (pl && String(pl).length > 2) {
                    try { pluginList = JSON.parse(String(pl)) } catch(e) { pluginList = [] }
                }
            }

            // Plugin updates check result
            if (app) {
                var upj = app.consume_plugin_updates()
                if (upj && String(upj).length > 2) {
                    try {
                        var updates = JSON.parse(String(upj))
                        var withUpdates = (updates || []).filter(function(u) { return u && u.has_update })
                        if (withUpdates.length > 0) {
                            pluginUpdateStatus = "Updates available for: " + withUpdates.map(function(u) { return u.plugin_id }).join(", ")
                        } else if ((updates || []).length > 0) {
                            pluginUpdateStatus = "All plugins up to date"
                        }
                    } catch(e) { }
                }
            }

            if (app && currentGuildId !== "") {
                var mj = app.consume_members()
                if (mj.length > 2) {
                    try {
                        var memberData = JSON.parse(mj)
                        if (memberData.guildId === currentGuildId && memberData.members) {
                            memberModel.clear()
                            for (var mi = 0; mi < memberData.members.length; mi++) {
                                var m = memberData.members[mi]
                                memberModel.append({
                                    memberId: m.memberId || "",
                                    username: m.username || "",
                                    displayName: m.displayName || m.username || "",
                                    avatarUrl: m.avatarUrl || "",
                                    roleName: m.roleName || "",
                                    roleColor: m.roleColor || "",
                                    publicFlags: m.publicFlags || 0,
                                    bot: m.bot || false,
                                    premiumType: m.premiumType || 0
                                })
                            }
                        }
                    } catch(e) { console.log("Members parse error:", e) }
                }
            }

            // Consume join-guild result (from "Join Server" popup or inline chat invite)
            if (app) {
                var jgr = app.consume_join_guild_result()
                if (jgr && String(jgr).length > 2) {
                    try {
                        var joinResult = JSON.parse(String(jgr))
                        if (joinResult.success && joinResult.guild) {
                            var gid = joinResult.guild.guildId || ""
                            var already = false
                            for (var gi = 0; gi < guildModel.count; gi++) {
                                if (guildModel.get(gi).guildId === gid) { already = true; break }
                            }
                            if (!already) guildModel.append(joinResult.guild)
                            joinServerPopup.joinError = ""
                            joinServerPopup.joinLoading = false
                            joinServerPopup.close()
                        } else if (!joinResult.success && joinResult.error) {
                            joinServerPopup.joinError = joinResult.error
                            joinServerPopup.joinLoading = false
                        }
                    } catch(e) { console.log("Join guild result parse error:", e) }
                }
            }

            // Consume RPC invites from browser handoff
            if (app) {
                var rpcInv = app.consume_rpc_invite()
                if (rpcInv && String(rpcInv).length > 0) {
                    joinServerPopup.inviteField.text = String(rpcInv)
                    joinServerPopup.joinError = ""
                    joinServerPopup.joinLoading = true
                    joinServerPopup.open()
                    app.join_guild_by_invite(String(rpcInv))
                }
            }
        }
    }

    // ─── Models ───
    ListModel { id: guildModel }
    ListModel { id: channelModel }
    ListModel { id: dmChannelModel }
    ListModel { id: messageModel }
    property int emojiPickerTab: 0  // 0=Emoji, 1=Stickers, 2=Server
    ListModel { id: stickerPickerModel }
    ListModel { id: stickerPacksModel }
    ListModel { id: guildEmojiPickerModel }
    ListModel { id: memberModel }
    ListModel { id: relationshipModel }
    ListModel { id: pinsModel }
    ListModel { id: voiceParticipantModel }

    property string voiceStatsPing: "—"
    property string voiceStatsEncryption: "—"
    property string voiceStatsEndpoint: "—"
    property string voiceStatsSsrc: "—"
    property string voiceStatsPacketsSent: "0"
    property string voiceStatsPacketsReceived: "0"
    property string voiceStatsDuration: "0"

    // ─── Main Layout ───
    RowLayout {
        anchors.fill: parent
        spacing: 0

        // ══════════ Guild Sidebar ══════════
        Rectangle {
            Layout.preferredWidth: theme.guildBarWidth
            Layout.fillHeight: true
            color: theme.bgTertiary

            ColumnLayout {
                anchors.fill: parent
                anchors.topMargin: 12
                spacing: 2

                // Home / DMs button
                Item {
                    Layout.preferredWidth: theme.guildBarWidth
                    Layout.preferredHeight: 52
                    Layout.alignment: Qt.AlignHCenter

                    // Pill indicator
                    Rectangle {
                        width: 4
                        height: currentGuildId === "" ? 36 : 0
                        radius: 2
                        color: theme.textNormal
                        anchors.left: parent.left
                        anchors.verticalCenter: parent.verticalCenter
                        visible: currentGuildId === ""
                        Behavior on height { NumberAnimation { duration: theme.animNormal; easing.type: Easing.OutCubic } }
                    }

                    GuildIcon {
                        anchors.centerIn: parent
                        text: "\u{2302}"
                        fontSize: 16
                        isActive: currentGuildId === ""
                        onClicked: {
                            // Save current state before switching
                            cacheCurrentChannels()
                            cacheCurrentMessages()
                            if (currentGuildId !== "" && currentChannelId !== "") {
                                var lcm = lastChannelPerGuild
                                lcm[currentGuildId] = currentChannelId
                                lastChannelPerGuild = lcm
                            }

                            currentGuildId = ""
                            currentGuildName = "Direct Messages"
                            currentChannelId = ""
                            currentChannelName = ""
                            messageModel.clear()
                            messageList.hasMoreHistory = true
                            messageList.isLoadingMore = false

                            // Restore DM list from cache if available
                            if (dmChannelCache.length > 0 && dmChannelModel.count === 0) {
                                for (var di = 0; di < dmChannelCache.length; di++)
                                    dmChannelModel.append(dmChannelCache[di])
                            }

                            if (app) app.select_guild("")
                        }
                    }
                }

                // Separator
                Rectangle {
                    Layout.preferredHeight: 2
                    Layout.preferredWidth: 32
                    Layout.alignment: Qt.AlignHCenter
                    Layout.topMargin: 4
                    Layout.bottomMargin: 4
                    radius: 1
                    color: theme.separator
                }

                // Guild list
                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    model: guildModel
                    clip: true
                    spacing: 2

                    delegate: Item {
                        width: ListView.view.width
                        height: 52

                        // Active pill
                        Rectangle {
                            width: 4
                            height: currentGuildId === model.guildId ? 36 :
                                    guildMouseArea.containsMouse ? 20 : 8
                            radius: 2
                            color: theme.textNormal
                            anchors.left: parent.left
                            anchors.verticalCenter: parent.verticalCenter
                            visible: currentGuildId === model.guildId || guildMouseArea.containsMouse
                            Behavior on height { NumberAnimation { duration: theme.animNormal; easing.type: Easing.OutCubic } }
                        }

                        GuildIcon {
                            anchors.centerIn: parent
                            text: model.name ? model.name.charAt(0).toUpperCase() : "?"
                            iconUrl: model.iconUrl || ""
                            isActive: currentGuildId === model.guildId
                            onClicked: {
                                // Save current state before switching
                                cacheCurrentChannels()
                                cacheCurrentMessages()
                                if (currentGuildId !== "" && currentChannelId !== "") {
                                    var lcm = lastChannelPerGuild
                                    lcm[currentGuildId] = currentChannelId
                                    lastChannelPerGuild = lcm
                                }

                                currentGuildId = model.guildId
                                currentGuildName = model.name
                                memberModel.clear()

                                // Try to restore channels from cache (instant)
                                var hasCachedChannels = restoreChannelsFromCache(model.guildId)

                                // Restore last selected channel in this guild
                                var lastCh = lastChannelPerGuild[model.guildId]
                                if (lastCh && hasCachedChannels) {
                                    currentChannelId = lastCh
                                    // Find channel name from cached model
                                    for (var ci = 0; ci < channelModel.count; ci++) {
                                        if (channelModel.get(ci).channelId === lastCh) {
                                            currentChannelName = channelModel.get(ci).name
                                            currentChannelType = channelModel.get(ci).channelType
                                            break
                                        }
                                    }
                                    // Restore messages for that channel too
                                    restoreMessagesFromCache(lastCh)
                                } else {
                                    currentChannelId = ""
                                    currentChannelName = ""
                                    if (!hasCachedChannels) channelModel.clear()
                                    messageModel.clear()
                                    messageList.hasMoreHistory = true
                                    messageList.isLoadingMore = false
                                }

                                // Always ask backend (will update cache when response arrives)
                                if (app) app.select_guild(model.guildId)
                            }
                        }

                        MouseArea {
                            id: guildMouseArea
                            anchors.fill: parent
                            hoverEnabled: true
                            propagateComposedEvents: true
                            acceptedButtons: Qt.NoButton
                        }
                    }
                }

                Item { Layout.fillHeight: true }

                // Divider above Join Server (at bottom of server list)
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 2
                    Layout.leftMargin: 12
                    Layout.rightMargin: 12
                    Layout.topMargin: 4
                    Layout.bottomMargin: 4
                    radius: 1
                    color: theme.separator
                }

                // Join Server button (+) — bottom of guild bar
                Item {
                    Layout.preferredWidth: theme.guildBarWidth
                    Layout.preferredHeight: 52
                    Layout.alignment: Qt.AlignHCenter

                    GuildIcon {
                        anchors.centerIn: parent
                        text: "+"
                        fontSize: 22
                        isActive: false
                        onClicked: {
                            joinServerPopup.joinError = ""
                            joinServerPopup.joinLoading = false
                            joinServerPopup.open()
                        }
                    }
                }
            }
        }

        // ══════════ Channel Sidebar ══════════
        Rectangle {
            Layout.preferredWidth: theme.channelBarWidth
            Layout.fillHeight: true
            color: theme.bgPrimary
            visible: isLoggedIn

            ColumnLayout {
                anchors.fill: parent
                spacing: 0

                // Guild name header (clickable when a server is selected — opens server context menu)
                    Rectangle {
                        id: guildHeaderRect
                        Layout.fillWidth: true
                        Layout.preferredHeight: theme.headerHeight
                        color: serverHeaderMa.containsMouse && currentGuildId !== "" ? theme.bgHover : theme.bgPrimary
                        z: 1
                        Behavior on color { ColorAnimation { duration: theme.animFast } }

                    Text {
                        anchors.left: parent.left
                        anchors.leftMargin: 16
                        anchors.verticalCenter: parent.verticalCenter
                        text: currentGuildName || "Direct Messages"
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 14
                        font.bold: true
                        elide: Text.ElideRight
                        width: parent.width - 80
                    }
                    MouseArea {
                        id: serverHeaderMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: currentGuildId !== "" ? Qt.PointingHandCursor : Qt.ArrowCursor
                        acceptedButtons: Qt.LeftButton | Qt.RightButton
                        onClicked: {
                            if (currentGuildId !== "") {
                                var pos = guildHeaderRect.mapToItem(root, 0, guildHeaderRect.height)
                                openServerContextMenu(pos.x, pos.y)
                            }
                        }
                    }
                    // Show hidden channels toggle (eye icon when guild selected)
                    Rectangle {
                        visible: currentGuildId !== ""
                        anchors.right: parent.right
                        anchors.rightMargin: 8
                        anchors.verticalCenter: parent.verticalCenter
                        width: 28
                        height: 28
                        radius: 4
                        color: showHiddenMa.containsMouse ? theme.bgHover : "transparent"
                        Text {
                            anchors.centerIn: parent
                            text: "\u{1F441}"
                            color: showHiddenChannels ? theme.textNormal : theme.textFaint
                            font.pixelSize: 14
                        }
                        MouseArea {
                            id: showHiddenMa
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                showHiddenChannels = !showHiddenChannels
                                rebuildChannelModel(currentGuildId)
                            }
                        }
                    }

                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: parent.width; height: 1
                        color: theme.separator

                        // Gradient shadow below header
                        Rectangle {
                            anchors.top: parent.bottom
                            width: parent.width
                            height: 4
                            gradient: Gradient {
                                GradientStop { position: 0.0; color: "#00000030" }
                                GradientStop { position: 1.0; color: "transparent" }
                            }
                        }
                    }
                }

                // Channel list (guild channels)
                ListView {
                    id: channelListView
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.topMargin: 8
                    model: channelModel
                    clip: true
                    spacing: 1
                    boundsBehavior: Flickable.StopAtBounds
                    visible: currentGuildId !== ""

                    ScrollBar.vertical: ScrollBar {
                        policy: ScrollBar.AsNeeded
                        contentItem: Rectangle {
                            implicitWidth: 4
                            radius: 2
                            color: theme.textFaint
                            opacity: parent.active ? 0.8 : 0.0
                            Behavior on opacity { NumberAnimation { duration: theme.animSlow } }
                        }
                        background: Item {}
                    }

                    delegate: Item {
                        width: channelListView.width - 16
                        x: 8
                        height: model.channelType === 4 ? 28 : (34 + (showVoiceParticipantsHere ? (voiceParticipantModel.count * 18 + 8) : 0))

                        property bool showVoiceParticipantsHere: (model.channelType === 2 || model.channelType === 13) && model.channelId === voiceChannelId && voiceParticipantModel.count > 0
                        property bool isCategory: model.channelType === 4
                        property bool isHiddenChannel: model.isHidden || false
                        property bool hasParent: (model.parentId && model.parentId !== "")

                        // Category header row
                        Rectangle {
                            visible: isCategory
                            anchors.fill: parent
                            color: categoryMa.containsMouse ? theme.bgHover : "transparent"
                            Behavior on color { ColorAnimation { duration: theme.animFast } }
                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: 8
                                anchors.rightMargin: 8
                                spacing: 4
                                Text {
                                    text: "\u25B8"
                                    color: theme.textFaint
                                    font.pixelSize: 10
                                    rotation: (collapsedCategories[currentGuildId] || {})[model.channelId] ? 0 : 90
                                    Layout.alignment: Qt.AlignVCenter
                                }
                                Text {
                                    text: (model.name || "category").toUpperCase()
                                    color: theme.textFaint
                                    font.family: fontFamily
                                    font.pixelSize: 11
                                    font.weight: Font.Bold
                                    font.letterSpacing: 0.5
                                    Layout.fillWidth: true
                                    elide: Text.ElideRight
                                    Layout.alignment: Qt.AlignVCenter
                                }
                            }
                            MouseArea {
                                id: categoryMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: toggleCategory(currentGuildId, model.channelId)
                            }
                        }

                        // Channel row (text, voice, etc.)
                        Rectangle {
                            visible: !isCategory
                            anchors.fill: parent
                            radius: theme.radiusSmall
                            color: currentChannelId === model.channelId ? theme.bgActive :
                                   model.hasUnread ? Qt.rgba(1,1,1,0.03) :
                                   channelMa.containsMouse ? theme.bgHover : "transparent"
                            Behavior on color { ColorAnimation { duration: theme.animFast } }

                            RowLayout {
                                anchors.top: parent.top
                                anchors.left: parent.left
                                anchors.right: parent.right
                                height: 34
                                anchors.leftMargin: hasParent ? 24 : 8
                                anchors.rightMargin: 8
                                spacing: 6

                                Item {
                                    Layout.preferredWidth: 20
                                    Layout.preferredHeight: 20
                                    Text {
                                        anchors.centerIn: parent
                                        text: {
                                            if (isHiddenChannel) return "\u{1F512}"
                                            if (model.channelType === 2) return "\u{1F50A}"
                                            if (model.channelType === 13) return "\u{1F3A4}"
                                            if (model.channelType === 5) return "\u{1F4E2}"
                                            if (model.channelType === 10 || model.channelType === 11 || model.channelType === 12) return "\u{1F4CB}"
                                            if (model.channelType === 15) return "\u{1F4C1}"
                                            if (model.channelType === 16) return "\u{1F5BC}"
                                            return "#"
                                        }
                                        color: currentChannelId === model.channelId ? theme.textSecondary :
                                               model.hasUnread ? theme.textSecondary : (isHiddenChannel ? theme.textFaint : theme.channelIcon)
                                        font.family: fontFamily
                                        font.pixelSize: (model.channelType === 2 || model.channelType === 13) ? 15 : 16
                                        font.bold: true
                                    }
                                }
                                Text {
                                    Layout.fillWidth: true
                                    text: model.name || "unknown"
                                    color: currentChannelId === model.channelId ? theme.textNormal :
                                           model.hasUnread ? theme.textNormal :
                                           channelMa.containsMouse ? theme.textSecondary : (isHiddenChannel ? theme.textFaint : theme.textMuted)
                                    font.family: fontFamily
                                    font.pixelSize: 15
                                    font.weight: model.hasUnread ? Font.Bold : Font.Normal
                                    elide: Text.ElideRight
                                    Behavior on color { ColorAnimation { duration: theme.animFast } }
                                }
                                // Typing indicator for other channels
                                Text {
                                    visible: showTypingInChannelList && currentChannelId !== model.channelId && 
                                            app && app.get_typing_in_channel(model.channelId).length > 0
                                    text: "..."
                                    color: theme.textMuted
                                    font.family: fontFamily
                                    font.pixelSize: 13
                                    font.weight: Font.Bold
                                }
                                Rectangle {
                                    visible: model.hasUnread && currentChannelId !== model.channelId
                                    width: 6; height: 6; radius: 3
                                    color: theme.textNormal
                                }
                                Rectangle {
                                    visible: (model.mentionCount || 0) > 0
                                    Layout.preferredWidth: Math.max(16, mentionLabel.implicitWidth + 8)
                                    Layout.preferredHeight: 16
                                    radius: 8
                                    color: theme.mentionPillBg
                                    Text {
                                        id: mentionLabel
                                        anchors.centerIn: parent
                                        text: model.mentionCount || ""
                                        color: "#ffffff"
                                        font.family: fontFamily
                                        font.pixelSize: 10
                                        font.weight: Font.Bold
                                    }
                                }
                            }

                            Column {
                                anchors.top: parent.top
                                anchors.topMargin: 34
                                anchors.left: parent.left
                                anchors.right: parent.right
                                anchors.leftMargin: hasParent ? 44 : 28
                                anchors.rightMargin: 8
                                anchors.bottomMargin: 6
                                visible: showVoiceParticipantsHere
                                spacing: 2
                                Repeater {
                                    model: voiceParticipantModel
                                    delegate: Row {
                                        spacing: 4
                                        height: 16
                                        Rectangle {
                                            width: 6
                                            height: 6
                                            radius: 3
                                            color: model.speaking ? theme.voiceSpeaking : theme.textFaint
                                            anchors.verticalCenter: parent.verticalCenter
                                            visible: true
                                        }
                                        Text {
                                            text: model.username || "Unknown"
                                            color: theme.textMuted
                                            font.pixelSize: 11
                                            elide: Text.ElideRight
                                            width: parent.parent.width - 20
                                            anchors.verticalCenter: parent.verticalCenter
                                        }
                                    }
                                }
                            }

                            MouseArea {
                                id: channelMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: isHiddenChannel ? Qt.ArrowCursor : Qt.PointingHandCursor
                                onClicked: {
                                    if (isHiddenChannel) return
                                    if (model.channelType === 2 || model.channelType === 13) {
                                        voiceChannelName = model.name
                                        voiceChannelId = model.channelId
                                        voiceGuildId = currentGuildId
                                        isVoiceConnected = true
                                        isMuted = false
                                        isDeafened = false
                                        if (app) app.join_voice(currentGuildId, model.channelId)
                                    } else {
                                        cacheCurrentMessages()
                                        currentChannelId = model.channelId
                                        currentChannelName = model.name
                                        currentChannelType = model.channelType
                                        if (!restoreMessagesFromCache(model.channelId)) {
                                            messageModel.clear()
                                            messageList.hasMoreHistory = true
                                            messageList.isLoadingMore = false
                                        }
                                        if (app) app.select_channel(model.channelId, model.channelType)
                                    }
                                }
                            }
                        }
                    }
                }

                // Friends / DM list (shown when Home is selected)
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    spacing: 0
                    visible: currentGuildId === ""

                    // Tab row: Friends | Direct Messages
                    Row {
                        Layout.fillWidth: true
                        Layout.leftMargin: 12
                        Layout.rightMargin: 8
                        Layout.topMargin: 6
                        Layout.preferredHeight: 28
                        spacing: 4

                        Rectangle {
                            width: 70
                            height: 24
                            radius: theme.radiusSmall
                            color: homeSubView === "friends" ? theme.bgActive : (friendsTabMa.containsMouse ? theme.bgHover : "transparent")
                            Text {
                                anchors.centerIn: parent
                                text: "Friends"
                                color: homeSubView === "friends" ? theme.textNormal : theme.textMuted
                                font.family: fontFamily
                                font.pixelSize: 12
                                font.weight: homeSubView === "friends" ? Font.DemiBold : Font.Normal
                            }
                            MouseArea {
                                id: friendsTabMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: homeSubView = "friends"
                            }
                        }
                        Rectangle {
                            width: 100
                            height: 24
                            radius: theme.radiusSmall
                            color: homeSubView === "dms" ? theme.bgActive : (dmsTabMa.containsMouse ? theme.bgHover : "transparent")
                            Text {
                                anchors.centerIn: parent
                                text: "Direct Messages"
                                color: homeSubView === "dms" ? theme.textNormal : theme.textMuted
                                font.family: fontFamily
                                font.pixelSize: 12
                                font.weight: homeSubView === "dms" ? Font.DemiBold : Font.Normal
                            }
                            MouseArea {
                                id: dmsTabMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: homeSubView = "dms"
                            }
                        }
                    }

                    // ─── Friends tab: full list is in main view ───
                    ColumnLayout {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        spacing: 0
                        visible: homeSubView === "friends"

                        Item {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            Text {
                                anchors.centerIn: parent
                                text: "Friends list is in the\nmain view \u2192"
                                color: theme.textMuted
                                font.family: fontFamily
                                font.pixelSize: 12
                                horizontalAlignment: Text.AlignHCenter
                            }
                        }

                        ListView {
                            id: relationshipListView
                            visible: relationshipModel.count > 0
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            model: relationshipModel
                            clip: true
                            spacing: 1
                            boundsBehavior: Flickable.StopAtBounds

                            ScrollBar.vertical: ScrollBar {
                                policy: ScrollBar.AsNeeded
                                contentItem: Rectangle {
                                    implicitWidth: 4
                                    radius: 2
                                    color: theme.textFaint
                                    opacity: parent.active ? 0.8 : 0.0
                                    Behavior on opacity { NumberAnimation { duration: theme.animSlow } }
                                }
                                background: Item {}
                            }

                            delegate: Rectangle {
                                width: relationshipListView.width - 16
                                x: 8
                                height: 44
                                radius: theme.radiusSmall
                                color: relRowMa.containsMouse ? theme.bgHover : "transparent"
                                focus: false

                                RowLayout {
                                    anchors.fill: parent
                                    anchors.leftMargin: 8
                                    anchors.rightMargin: 8
                                    spacing: 10

                                    Item {
                                        Layout.preferredWidth: 32
                                        Layout.preferredHeight: 32
                                        Layout.alignment: Qt.AlignVCenter
                                        DAvatar {
                                            anchors.fill: parent
                                            size: 32
                                            imageUrl: model.avatarUrl || ""
                                            fallbackText: (model.username || "").charAt(0).toUpperCase()
                                        }
                                    }

                                    ColumnLayout {
                                        Layout.fillWidth: true
                                        spacing: 0
                                        Text {
                                            text: model.username || "—"
                                            color: theme.textNormal
                                            font.family: fontFamily
                                            font.pixelSize: 13
                                            font.weight: Font.Medium
                                            elide: Text.ElideRight
                                            Layout.maximumWidth: parent.parent.width - 120
                                        }
                                        Text {
                                            text: model.relationshipType === "friend" ? "Friend" :
                                                  model.relationshipType === "incoming" ? "Incoming request" :
                                                  model.relationshipType === "outgoing" ? "Outgoing request" : "Blocked"
                                            color: theme.textMuted
                                            font.family: fontFamily
                                            font.pixelSize: 11
                                            visible: text.length > 0
                                        }
                                    }

                                    Row {
                                        spacing: 4
                                        Layout.alignment: Qt.AlignRight | Qt.AlignVCenter

                                        Rectangle {
                                            visible: model.relationshipType === "friend"
                                            width: visible ? 56 : 0
                                            height: 26
                                            radius: 4
                                            color: msgFriendMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                            Text { anchors.centerIn: parent; text: "Message"; color: theme.positive; font.pixelSize: 11; font.family: fontFamily }
                                            MouseArea {
                                                id: msgFriendMa
                                                anchors.fill: parent
                                                hoverEnabled: true
                                                cursorShape: Qt.PointingHandCursor
                                                onClicked: { if (app && model.userId) app.open_dm(model.userId) }
                                            }
                                        }
                                        Rectangle {
                                            visible: model.relationshipType === "incoming"
                                            width: visible ? 56 : 0
                                            height: 26
                                            radius: 4
                                            color: acceptFriendMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                            Text { anchors.centerIn: parent; text: "Accept"; color: theme.positive; font.pixelSize: 11; font.family: fontFamily }
                                            MouseArea {
                                                id: acceptFriendMa
                                                anchors.fill: parent
                                                hoverEnabled: true
                                                cursorShape: Qt.PointingHandCursor
                                                onClicked: { if (app && model.userId) app.accept_friend_request(model.userId) }
                                            }
                                        }
                                        Rectangle {
                                            visible: model.relationshipType === "incoming" || model.relationshipType === "friend" || model.relationshipType === "outgoing"
                                            width: visible ? (model.relationshipType === "incoming" ? 48 : 52) : 0
                                            height: 26
                                            radius: 4
                                            color: removeRelMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                            Text {
                                                anchors.centerIn: parent
                                                text: model.relationshipType === "incoming" ? "Reject" : (model.relationshipType === "outgoing" ? "Cancel" : "Remove")
                                                color: theme.textMuted
                                                font.pixelSize: 11
                                                font.family: fontFamily
                                            }
                                            MouseArea {
                                                id: removeRelMa
                                                anchors.fill: parent
                                                hoverEnabled: true
                                                cursorShape: Qt.PointingHandCursor
                                                onClicked: { if (app && model.userId) app.remove_relationship(model.userId) }
                                            }
                                        }
                                        Rectangle {
                                            visible: model.relationshipType === "friend"
                                            width: visible ? 40 : 0
                                            height: 26
                                            radius: 4
                                            color: blockUserMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                            Text { anchors.centerIn: parent; text: "Block"; color: theme.danger; font.pixelSize: 11; font.family: fontFamily }
                                            MouseArea {
                                                id: blockUserMa
                                                anchors.fill: parent
                                                hoverEnabled: true
                                                cursorShape: Qt.PointingHandCursor
                                                onClicked: { if (app && model.userId) app.block_user(model.userId) }
                                            }
                                        }
                                        Rectangle {
                                            visible: model.relationshipType === "blocked"
                                            width: visible ? 52 : 0
                                            height: 26
                                            radius: 4
                                            color: unblockMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                            Text { anchors.centerIn: parent; text: "Unblock"; color: theme.textMuted; font.pixelSize: 11; font.family: fontFamily }
                                            MouseArea {
                                                id: unblockMa
                                                anchors.fill: parent
                                                hoverEnabled: true
                                                cursorShape: Qt.PointingHandCursor
                                                onClicked: { if (app && model.userId) app.remove_relationship(model.userId) }
                                            }
                                        }
                                    }
                                }

                                MouseArea {
                                    id: relRowMa
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    acceptedButtons: Qt.RightButton
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: if (mouse.button === Qt.RightButton) openUserProfile(model.userId, model.username, model.username, model.avatarUrl, "")
                                }
                            }
                        }

                        Item {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            visible: relationshipModel.count === 0
                            Text {
                                anchors.centerIn: parent
                                text: "No friends yet.\nAdd someone by username above."
                                color: theme.textMuted
                                font.family: fontFamily
                                font.pixelSize: 12
                                horizontalAlignment: Text.AlignHCenter
                            }
                        }
                    }

                    // "DIRECT MESSAGES" section heading (when DMs tab)
                    Item {
                        visible: homeSubView === "dms"
                        Layout.fillWidth: true
                        Layout.preferredHeight: 28
                        Layout.topMargin: 8
                        Layout.leftMargin: 16
                        Layout.rightMargin: 8

                        Text {
                            anchors.left: parent.left
                            anchors.verticalCenter: parent.verticalCenter
                            text: "DIRECT MESSAGES"
                            color: theme.textFaint
                            font.family: fontFamily
                            font.pixelSize: 10
                            font.weight: Font.Bold
                            font.letterSpacing: 0.5
                        }
                    }

                    // Empty state when no DMs loaded (DMs tab only)
                    Item {
                        visible: homeSubView === "dms" && dmChannelModel.count === 0
                        Layout.fillWidth: true
                        Layout.fillHeight: true

                        ColumnLayout {
                            anchors.centerIn: parent
                            spacing: 8
                            width: parent.width - 32

                            Text {
                                Layout.alignment: Qt.AlignHCenter
                                text: "\u{1F4AC}"
                                font.pixelSize: 28
                                color: theme.textFaint
                            }
                            Text {
                                Layout.alignment: Qt.AlignHCenter
                                text: "No direct messages"
                                color: theme.textMuted
                                font.family: fontFamily
                                font.pixelSize: 12
                                font.weight: Font.Medium
                            }
                            Text {
                                Layout.alignment: Qt.AlignHCenter
                                text: "Your DMs will appear here once loaded."
                                color: theme.textFaint
                                font.family: fontFamily
                                font.pixelSize: 10
                                wrapMode: Text.WordWrap
                                horizontalAlignment: Text.AlignHCenter
                                Layout.maximumWidth: parent.width
                            }
                        }
                    }

                    ListView {
                        id: dmListView
                        visible: homeSubView === "dms" && dmChannelModel.count > 0
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        model: dmChannelModel
                        clip: true
                        spacing: 1
                        boundsBehavior: Flickable.StopAtBounds

                        ScrollBar.vertical: ScrollBar {
                            policy: ScrollBar.AsNeeded
                            contentItem: Rectangle {
                                implicitWidth: 4
                                radius: 2
                                color: theme.textFaint
                                opacity: parent.active ? 0.8 : 0.0
                                Behavior on opacity { NumberAnimation { duration: theme.animSlow } }
                            }
                            background: Item {}
                        }

                        delegate: Rectangle {
                            width: dmListView.width - 16
                            x: 8
                            height: 34
                            radius: theme.radiusSmall
                            color: currentChannelId === model.channelId ? theme.bgActive :
                                   dmMa.containsMouse ? theme.bgHover : "transparent"
                            Behavior on color { ColorAnimation { duration: theme.animFast } }

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: 8
                                anchors.rightMargin: 8
                                spacing: 10

                                Item {
                                    Layout.preferredWidth: 32
                                    Layout.preferredHeight: 32
                                    MouseArea {
                                        anchors.fill: parent
                                        acceptedButtons: Qt.RightButton
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: openUserProfile(model.recipientId || "", model.recipientName || "", model.recipientName || "", model.recipientAvatarUrl || "", "")
                                    }
                                    DAvatar {
                                        anchors.fill: parent
                                        size: 32
                                        imageUrl: model.recipientAvatarUrl || ""
                                        fallbackText: model.recipientName || model.recipientId || "?"
                                    }
                                    Rectangle {
                                        width: 10; height: 10; radius: 5
                                        anchors.right: parent.right
                                        anchors.bottom: parent.bottom
                                        border.width: 2
                                        border.color: theme.bgSecondary
                                        color: getStatusColor((presenceVersion, app ? app.get_user_status(model.recipientId || "") : ""))
                                    }
                                }

                                Text {
                                    Layout.fillWidth: true
                                    text: model.recipientName || "Unknown"
                                    color: currentChannelId === model.channelId ? theme.textNormal :
                                           dmMa.containsMouse ? theme.textSecondary : theme.textMuted
                                    font.family: fontFamily
                                    font.pixelSize: 15
                                    elide: Text.ElideRight
                                    Behavior on color { ColorAnimation { duration: theme.animFast } }
                                }
                                // Typing indicator for DM channels
                                Text {
                                    visible: showTypingInChannelList && currentChannelId !== model.channelId && 
                                            app && app.get_typing_in_channel(model.channelId).length > 0
                                    text: "..."
                                    color: theme.textMuted
                                    font.family: fontFamily
                                    font.pixelSize: 13
                                    font.weight: Font.Bold
                                }
                            }

                            MouseArea {
                                id: dmMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    cacheCurrentMessages()
                                    currentChannelId = model.channelId
                                    currentChannelName = model.recipientName
                                    currentChannelType = model.channelType || 1
                                    if (!restoreMessagesFromCache(model.channelId)) {
                                        messageModel.clear()
                                        messageList.hasMoreHistory = true
                                        messageList.isLoadingMore = false
                                    }
                                    if (app) app.select_channel(model.channelId, model.channelType || 1)
                                }
                            }
                        }
                    }
                }

                // ─── Voice Panel ───
                Rectangle {
                    id: voicePanelInline
                    Layout.fillWidth: true
                    Layout.preferredHeight: isVoiceConnected ? 96 : 0
                    color: theme.bgSecondary
                    clip: true
                    visible: isVoiceConnected

                    Behavior on Layout.preferredHeight { NumberAnimation { duration: theme.animSlow; easing.type: Easing.OutCubic } }

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 10
                        spacing: 4

                        RowLayout {
                            spacing: 6
                            Rectangle {
                                width: 8
                                height: 8
                                radius: 4
                                color: voiceConnectionState === "connected" ? theme.voicePositive :
                                       (voiceConnectionState.indexOf("failed") >= 0 ? theme.danger : theme.voiceConnecting)
                            }
                            Text {
                                text: voiceConnectionState === "connected" ? "Connected" :
                                      voiceConnectionState.indexOf("failed") >= 0 ? "Failed" : "Connecting..."
                                color: voiceConnectionState === "connected" ? theme.voicePositive :
                                       (voiceConnectionState.indexOf("failed") >= 0 ? theme.danger : theme.voiceConnecting)
                                font.family: fontFamily
                                font.pixelSize: 11
                                font.weight: Font.Medium
                            }
                            Item { Layout.fillWidth: true }
                            Text {
                                text: voiceConnectionState === "connected" && voiceStatsPing !== "—" ? voiceStatsPing + " ms" : ""
                                color: theme.textMuted
                                font.pixelSize: 10
                                visible: text !== ""
                            }
                        }
                        Text {
                            text: voiceChannelName || "Voice Channel"
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 10
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                        }

                        RowLayout {
                            spacing: 8
                            Layout.alignment: Qt.AlignHCenter

                            VoiceButton {
                                icon: isMuted ? "🔇" : "🎤"
                                isActive: isMuted
                                activeColor: theme.danger
                                onClicked: {
                                    isMuted = !isMuted
                                    if (app && isVoiceConnected) app.toggle_mute()
                                }
                            }
                            VoiceButton {
                                icon: isDeafened ? "🔇" : "🔊"
                                isActive: isDeafened
                                activeColor: theme.danger
                                onClicked: {
                                    isDeafened = !isDeafened
                                    if (app && isVoiceConnected) app.toggle_deafen()
                                }
                            }
                            VoiceButton {
                                icon: "👻"
                                isActive: isFakeMuted
                                activeColor: theme.warning
                                visible: app && app.is_plugin_enabled("fake-mute")
                                onClicked: {
                                    isFakeMuted = !isFakeMuted
                                    if (app && isVoiceConnected) app.toggle_fake_mute()
                                }
                            }
                            VoiceButton {
                                icon: "🎧"
                                isActive: isFakeDeafened
                                activeColor: theme.warning
                                visible: app && app.is_plugin_enabled("fake-deafen")
                                onClicked: {
                                    isFakeDeafened = !isFakeDeafened
                                    if (app && isVoiceConnected) app.toggle_fake_deafen()
                                }
                            }
                            VoiceButton {
                                icon: "✕"
                                isActive: true
                                activeColor: theme.danger
                                isDestructive: true
                                onClicked: {
                                    if (app) app.leave_voice()
                                    isVoiceConnected = false
                                    voiceChannelName = ""
                                    voiceChannelId = ""
                                    voiceGuildId = ""
                                    isMuted = false
                                    isDeafened = false
                                    isFakeMuted = false
                                    isFakeDeafened = false
                                }
                            }
                        }
                    }
                }

                // User panel at bottom
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: theme.userPanelHeight
                    color: theme.bgSecondary

                    Rectangle { anchors.top: parent.top; width: parent.width; height: 1; color: theme.separator }

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 10
                        anchors.rightMargin: 10
                        spacing: 10

                        // User avatar with status (click opens profile popup)
                        Item {
                            width: 32; height: 32
                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    profilePopupTarget = "self"
                                    profilePopup.open()
                                }
                            }
                            DAvatar {
                                size: 32
                                imageUrl: currentUserAvatar || ""
                                fallbackText: currentUserName || "?"
                                showStatus: true
                                statusColor: getStatusColor((presenceVersion, app ? (app.get_user_status(currentUserId).length > 0 ? app.get_user_status(currentUserId) : currentStatus) : currentStatus))
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 0

                            Text {
                                text: currentUserName || ""
                                color: theme.textNormal
                                font.family: fontFamily
                                font.pixelSize: 12
                                font.bold: true
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }
                            Text {
                                text: currentStatus === "online" ? "Online" :
                                      currentStatus === "idle" ? "Idle" :
                                      currentStatus === "dnd" ? "Do Not Disturb" : "Offline"
                                color: theme.textMuted
                                font.family: fontFamily
                                font.pixelSize: 10
                            }
                        }

                        // Plugin toolbar buttons (from enabled plugins)
                        Repeater {
                            model: pluginToolbarButtons
                            delegate: Rectangle {
                                width: 26; height: 26; radius: theme.radiusSmall
                                color: pluginBtnMa.containsMouse ? theme.bgHover : "transparent"
                                Text {
                                    anchors.centerIn: parent
                                    text: modelData.label.charAt(0).toUpperCase()
                                    color: pluginBtnMa.containsMouse ? theme.textSecondary : theme.textFaint
                                    font.pixelSize: 12
                                    font.family: fontFamily
                                }
                                ToolTip.visible: pluginBtnMa.containsMouse && modelData.tooltip
                                ToolTip.text: modelData.tooltip
                                ToolTip.delay: 500
                                MouseArea {
                                    id: pluginBtnMa
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        if (app) app.plugin_button_clicked(modelData.pluginId, modelData.buttonId)
                                        // If plugin has a modal for this button, show it
                                        var data = pluginUiData || {}
                                        var modals = (data[modelData.pluginId] && data[modelData.pluginId].modals) || []
                                        for (var j = 0; j < modals.length; j++) {
                                            if (modals[j].id === modelData.buttonId + "_modal" || (modelData.buttonId === "export" && modals[j].id === "export_modal")) {
                                                pluginModalPopup.show(modelData.pluginId, modals[j])
                                                break
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // Account switcher (when multiple accounts)
                        Rectangle {
                            visible: accountsList.length > 1
                            width: 26; height: 26; radius: theme.radiusSmall
                            color: accountSwitchMa.containsMouse ? theme.bgHover : "transparent"
                            Text {
                                anchors.centerIn: parent
                                text: "\u{21C4}"
                                color: accountSwitchMa.containsMouse ? theme.textSecondary : theme.textFaint
                                font.pixelSize: 14
                                font.family: fontFamily
                            }
                            ToolTip.visible: accountSwitchMa.containsMouse
                            ToolTip.text: "Switch account (Ctrl+1–9)"
                            ToolTip.delay: 500
                            MouseArea {
                                id: accountSwitchMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: accountSwitcherPopup.open()
                            }
                        }
                        // Settings gear
                        Rectangle {
                            width: 26; height: 26; radius: theme.radiusSmall
                            color: gearMa.containsMouse ? theme.bgHover : "transparent"

                            Text {
                                anchors.centerIn: parent
                                text: "\u{2699}"
                                color: gearMa.containsMouse ? theme.textSecondary : theme.textFaint
                                font.pixelSize: 16
                            }
                            MouseArea {
                                id: gearMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: settingsPopup.open()
                            }
                        }
                    }
                }
            }
        }

        // ══════════ Main Content Area ══════════
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: theme.bgBase

            ColumnLayout {
                anchors.fill: parent
                spacing: 0
                visible: isLoggedIn

                // Error toast (when logged in — API errors, send failures, etc.)
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: loginError.length > 0 ? 40 : 0
                    visible: loginError.length > 0
                    color: "#f23f4320"
                    border.width: 1
                    border.color: theme.danger
                    radius: theme.radiusSmall
                    Layout.leftMargin: 16
                    Layout.rightMargin: 16
                    Layout.topMargin: 8
                    Layout.bottomMargin: 4
                    opacity: loginError.length > 0 ? 1 : 0
                    Behavior on Layout.preferredHeight { NumberAnimation { duration: theme.animFast } }
                    Behavior on opacity { NumberAnimation { duration: theme.animFast } }

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 12
                        anchors.rightMargin: 12
                        spacing: 8
                        Text {
                            text: "\u{26A0}"
                            color: theme.danger
                            font.pixelSize: 14
                        }
                        Text {
                            Layout.fillWidth: true
                            text: loginError
                            color: theme.textNormal
                            font.family: fontFamily
                            font.pixelSize: 12
                            elide: Text.ElideRight
                        }
                        Text {
                            text: "\u{2715}"
                            color: theme.textFaint
                            font.pixelSize: 12
                        }
                        MouseArea {
                            width: 24
                            height: 24
                            anchors.verticalCenter: parent.verticalCenter
                            onClicked: if (app) app.clear_error()
                        }
                    }
                    Timer {
                        running: loginError.length > 0
                        interval: 5000
                        repeat: false
                        onTriggered: if (app) app.clear_error()
                    }
                }


                // Channel header — with gradient shadow
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: theme.headerHeight
                    color: theme.bgBase
                    z: 1

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 16
                        anchors.rightMargin: 16
                        spacing: 8

                        Text {
                            text: {
                                if (currentChannelType === 2) return "\u{1F50A}"      // Voice
                                if (currentChannelType === 13) return "\u{1F3A4}"     // Stage
                                if (currentChannelType === 5) return "\u{1F4E2}"      // Announcement
                                if (currentChannelType === 15) return "\u{1F4C1}"     // Forum
                                return "#"                                             // Text / default
                            }
                            color: theme.textFaint
                            font.family: fontFamily
                            font.pixelSize: isVoiceChannel ? 16 : 20
                            font.bold: true
                            visible: currentChannelName !== ""
                        }
                        Text {
                            text: (currentGuildId === "" && homeSubView === "friends") ? "Friends" : (currentChannelName || "Select a channel")
                            color: (currentChannelName || (currentGuildId === "" && homeSubView === "friends")) ? theme.headerPrimary : theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 16
                            font.weight: Font.DemiBold
                        }

                        Item { Layout.fillWidth: true }

                        // Connection status indicator
                        Rectangle {
                            visible: connectionState === "disconnected" || connectionState === "reconnecting"
                            width: connStatusText.implicitWidth + 16
                            height: 22
                            radius: 11
                            color: connectionState === "reconnecting" ? theme.warning + "20" : theme.danger + "20"

                            Text {
                                id: connStatusText
                                anchors.centerIn: parent
                                text: connectionState === "reconnecting" ? "Reconnecting..." : "Disconnected"
                                color: connectionState === "reconnecting" ? theme.warning : theme.danger
                                font.family: fontFamily
                                font.pixelSize: 10
                                font.weight: Font.Medium
                            }
                        }

                        // Pins button (text channels and DMs only)
                        Rectangle {
                            width: 26; height: 26; radius: theme.radiusSmall
                            color: pinsMa.containsMouse ? theme.bgHover : "transparent"
                            visible: currentChannelId !== "" && !isVoiceChannel

                            Text {
                                anchors.centerIn: parent
                                text: "\u{1F4CC}"
                                color: pinsMa.containsMouse ? theme.textSecondary : theme.textFaint
                                font.pixelSize: 14
                            }
                            MouseArea {
                                id: pinsMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    if (app && currentChannelId) {
                                        app.open_pins(currentChannelId)
                                        pinsPopup.open()
                                    }
                                }
                            }
                            ToolTip {
                                visible: pinsMa.containsMouse
                                text: "Pinned messages"
                                delay: 500
                            }
                        }

                        // Mark All Read button
                        Rectangle {
                            width: 26; height: 26; radius: theme.radiusSmall
                            color: markReadMa.containsMouse ? theme.bgHover : "transparent"
                            visible: currentGuildId !== ""

                            Text {
                                anchors.centerIn: parent
                                text: "\u{2713}"
                                color: markReadMa.containsMouse ? theme.textSecondary : theme.textFaint
                                font.pixelSize: 14
                                font.weight: Font.Bold
                            }
                            MouseArea {
                                id: markReadMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: { if (app) app.mark_all_read() }
                            }
                            ToolTip {
                                visible: markReadMa.containsMouse
                                text: "Mark All as Read"
                                delay: 500
                            }
                        }
                    }

                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: parent.width; height: 1
                        color: theme.separator

                        // Gradient shadow below header
                        Rectangle {
                            anchors.top: parent.bottom
                            width: parent.width
                            height: 6
                            gradient: Gradient {
                                GradientStop { position: 0.0; color: "#00000025" }
                                GradientStop { position: 1.0; color: "transparent" }
                            }
                        }
                    }
                }

                // ── Voice Channel View (shown instead of messages for voice/stage) ──
                Item {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    visible: isVoiceChannel && currentChannelId !== ""

                    ColumnLayout {
                        anchors.fill: parent
                        spacing: 12

                        // Connection banner
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 44
                            visible: isVoiceConnected || voiceConnectionState.indexOf("connecting") >= 0 || voiceConnectionState.indexOf("discovering") >= 0 || voiceConnectionState.indexOf("selecting") >= 0
                            color: theme.bgSecondary
                            radius: theme.radiusSmall
                            border.width: 1
                            border.color: theme.border

                            RowLayout {
                                anchors.fill: parent
                                anchors.margins: 10
                                spacing: 8
                                Rectangle {
                                    width: 10
                                    height: 10
                                    radius: 5
                                    color: voiceConnectionState === "connected" ? theme.voicePositive :
                                           (voiceConnectionState.indexOf("failed") >= 0 ? theme.danger : theme.voiceConnecting)
                                    Layout.alignment: Qt.AlignVCenter
                                }
                                Text {
                                    text: voiceConnectionState === "connected" ? "Connected" :
                                          voiceConnectionState.indexOf("failed") >= 0 ? "Connection failed" :
                                          "Connecting..."
                                    color: theme.textNormal
                                    font.pixelSize: 12
                                    font.weight: Font.Medium
                                    Layout.alignment: Qt.AlignVCenter
                                }
                            }
                        }

                        // Join button when not in voice
                        Item {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 80
                            visible: !isVoiceConnected && voiceConnectionState !== "connected"
                            ColumnLayout {
                                anchors.centerIn: parent
                                spacing: 8
                                Text {
                                    text: currentChannelType === 13 ? "\u{1F3A4}" : "\u{1F50A}"
                                    font.pixelSize: 36
                                    color: theme.textMuted
                                    Layout.alignment: Qt.AlignHCenter
                                }
                                Rectangle {
                                    Layout.preferredWidth: joinVoiceBtnLabel.implicitWidth + 32
                                    Layout.preferredHeight: 36
                                    Layout.alignment: Qt.AlignHCenter
                                    radius: theme.radiusMed
                                    color: joinVoiceBtnMa.containsMouse ? theme.accentHover : theme.accent
                                    Text {
                                        id: joinVoiceBtnLabel
                                        anchors.centerIn: parent
                                        text: currentChannelType === 13 ? "Join Stage" : "Join Voice"
                                        color: "#ffffff"
                                        font.pixelSize: 13
                                        font.weight: Font.Medium
                                    }
                                    MouseArea {
                                        id: joinVoiceBtnMa
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            voiceChannelName = currentChannelName
                                            voiceChannelId = currentChannelId
                                            voiceGuildId = currentGuildId
                                            isVoiceConnected = true
                                            isMuted = false
                                            isDeafened = false
                                            if (app) app.join_voice(currentGuildId, currentChannelId)
                                        }
                                    }
                                }
                            }
                        }

                        // Participant grid
                        GridView {
                            id: voiceParticipantGrid
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            visible: isVoiceConnected || voiceParticipantModel.count > 0
                            cellWidth: 150
                            cellHeight: 170
                            clip: true
                            model: voiceParticipantModel
                            delegate: VoiceParticipantCard {
                                width: voiceParticipantGrid.cellWidth - 10
                                height: voiceParticipantGrid.cellHeight - 10
                                participantName: model.username || "Unknown"
                                avatarUrl: model.avatarUrl || ""
                                isSpeaking: !!model.speaking
                                isMuted: !!(model.selfMute || model.serverMute)
                                isDeafened: !!(model.selfDeaf || model.serverDeaf)
                                isVideo: !!model.selfVideo
                                isStreaming: !!model.selfStream
                            }
                        }

                        // Stats panel (developer)
                        VoiceStatsPanel {
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignBottom
                            visible: isVoiceConnected && voiceConnectionState === "connected"
                            pingMs: voiceStatsPing
                            encryptionMode: voiceStatsEncryption
                            endpoint: voiceStatsEndpoint
                            ssrc: voiceStatsSsrc
                            packetsSent: voiceStatsPacketsSent
                            packetsReceived: voiceStatsPacketsReceived
                            connectionDurationSecs: voiceStatsDuration
                        }
                    }
                }

                // Full-page Friends view (Home + Friends tab — like Discord client)
                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    spacing: 0
                    visible: currentGuildId === "" && homeSubView === "friends"

                    Item {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 48
                        Layout.leftMargin: 16
                        Layout.rightMargin: 16
                        Layout.topMargin: 8
                        RowLayout {
                            anchors.fill: parent
                            spacing: 12
                            TextField {
                                id: addFriendUsernameFieldMain
                                Layout.fillWidth: true
                                Layout.preferredHeight: 36
                                placeholderText: "Enter username to add a friend"
                                color: theme.textNormal
                                font.family: fontFamily
                                font.pixelSize: 14
                                background: Rectangle {
                                    color: theme.inputBg
                                    radius: theme.radiusSmall
                                    border.width: 0
                                }
                                onAccepted: {
                                    var un = (addFriendUsernameFieldMain.text || "").trim()
                                    if (un.length > 0 && app) {
                                        app.send_friend_request(un)
                                        addFriendUsernameFieldMain.text = ""
                                    }
                                }
                            }
                            Rectangle {
                                Layout.preferredWidth: 80
                                Layout.preferredHeight: 36
                                radius: theme.radiusSmall
                                color: sendFriendMainMa.containsMouse ? theme.accentHover : theme.accent
                                Text {
                                    anchors.centerIn: parent
                                    text: "Send"
                                    color: "#ffffff"
                                    font.family: fontFamily
                                    font.pixelSize: 13
                                    font.weight: Font.Medium
                                }
                                MouseArea {
                                    id: sendFriendMainMa
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        var un = (addFriendUsernameFieldMain.text || "").trim()
                                        if (un.length > 0 && app) {
                                            app.send_friend_request(un)
                                            addFriendUsernameFieldMain.text = ""
                                        }
                                    }
                                }
                            }
                        }
                    }

                    ListView {
                        id: friendsPageList
                        visible: relationshipModel.count > 0
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        Layout.leftMargin: 16
                        Layout.rightMargin: 16
                        Layout.bottomMargin: 16
                        model: relationshipModel
                        clip: true
                        spacing: 2
                        boundsBehavior: Flickable.StopAtBounds
                        focus: false

                        ScrollBar.vertical: ScrollBar {
                            policy: ScrollBar.AsNeeded
                            contentItem: Rectangle {
                                implicitWidth: 6
                                radius: 3
                                color: theme.textFaint
                                opacity: parent.active ? 0.8 : 0.0
                                Behavior on opacity { NumberAnimation { duration: theme.animSlow } }
                            }
                            background: Item {}
                        }

                        delegate: Rectangle {
                            width: friendsPageList.width - 24
                            x: 12
                            height: 56
                            radius: theme.radiusMed
                            color: friendsRowMa.containsMouse ? theme.bgHover : "transparent"
                            focus: false

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: 12
                                anchors.rightMargin: 12
                                spacing: 14

                                Item {
                                    Layout.preferredWidth: 40
                                    Layout.preferredHeight: 40
                                    Layout.alignment: Qt.AlignVCenter
                                    DAvatar {
                                        anchors.fill: parent
                                        size: 40
                                        imageUrl: model.avatarUrl || ""
                                        fallbackText: (model.username || "").charAt(0).toUpperCase()
                                    }
                                }

                                ColumnLayout {
                                    Layout.fillWidth: true
                                    spacing: 2
                                    Text {
                                        text: model.username || "—"
                                        color: theme.textNormal
                                        font.family: fontFamily
                                        font.pixelSize: 15
                                        font.weight: Font.Medium
                                        elide: Text.ElideRight
                                        Layout.fillWidth: true
                                    }
                                    Text {
                                        text: model.relationshipType === "friend" ? "Friend" :
                                              model.relationshipType === "incoming" ? "Incoming request" :
                                              model.relationshipType === "outgoing" ? "Outgoing request" : "Blocked"
                                        color: theme.textMuted
                                        font.family: fontFamily
                                        font.pixelSize: 12
                                        visible: text.length > 0
                                    }
                                }

                                Row {
                                    spacing: 8
                                    Layout.alignment: Qt.AlignRight | Qt.AlignVCenter

                                    Rectangle {
                                        visible: model.relationshipType === "friend"
                                        width: visible ? 72 : 0
                                        height: 32
                                        radius: theme.radiusSmall
                                        color: msgFriendMainMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                        Text { anchors.centerIn: parent; text: "Message"; color: theme.positive; font.pixelSize: 12; font.family: fontFamily }
                                        MouseArea {
                                            id: msgFriendMainMa
                                            anchors.fill: parent
                                            hoverEnabled: true
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: { if (app && model.userId) app.open_dm(model.userId) }
                                        }
                                    }
                                    Rectangle {
                                        visible: model.relationshipType === "incoming"
                                        width: visible ? 72 : 0
                                        height: 32
                                        radius: theme.radiusSmall
                                        color: acceptFriendMainMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                        Text { anchors.centerIn: parent; text: "Accept"; color: theme.positive; font.pixelSize: 12; font.family: fontFamily }
                                        MouseArea {
                                            id: acceptFriendMainMa
                                            anchors.fill: parent
                                            hoverEnabled: true
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: { if (app && model.userId) app.accept_friend_request(model.userId) }
                                        }
                                    }
                                    Rectangle {
                                        visible: model.relationshipType === "incoming" || model.relationshipType === "friend" || model.relationshipType === "outgoing"
                                        width: visible ?  (model.relationshipType === "incoming" ? 64 : 76) : 0
                                        height: 32
                                        radius: theme.radiusSmall
                                        color: removeRelMainMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                        Text {
                                            anchors.centerIn: parent
                                            text: model.relationshipType === "incoming" ? "Reject" : (model.relationshipType === "outgoing" ? "Cancel" : "Remove")
                                            color: theme.textMuted
                                            font.pixelSize: 12
                                            font.family: fontFamily
                                        }
                                        MouseArea {
                                            id: removeRelMainMa
                                            anchors.fill: parent
                                            hoverEnabled: true
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: { if (app && model.userId) app.remove_relationship(model.userId) }
                                        }
                                    }
                                    Rectangle {
                                        visible: model.relationshipType === "friend"
                                        width: visible ? 56 : 0
                                        height: 32
                                        radius: theme.radiusSmall
                                        color: blockUserMainMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                        Text { anchors.centerIn: parent; text: "Block"; color: theme.danger; font.pixelSize: 12; font.family: fontFamily }
                                        MouseArea {
                                            id: blockUserMainMa
                                            anchors.fill: parent
                                            hoverEnabled: true
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: { if (app && model.userId) app.block_user(model.userId) }
                                        }
                                    }
                                    Rectangle {
                                        visible: model.relationshipType === "blocked"
                                        width: visible ? 68 : 0
                                        height: 32
                                        radius: theme.radiusSmall
                                        color: unblockMainMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                        Text { anchors.centerIn: parent; text: "Unblock"; color: theme.textMuted; font.pixelSize: 12; font.family: fontFamily }
                                        MouseArea {
                                            id: unblockMainMa
                                            anchors.fill: parent
                                            hoverEnabled: true
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: { if (app && model.userId) app.remove_relationship(model.userId) }
                                        }
                                    }
                                }
                            }

                            MouseArea {
                                id: friendsRowMa
                                anchors.fill: parent
                                hoverEnabled: true
                                acceptedButtons: Qt.RightButton
                                cursorShape: Qt.PointingHandCursor
                                onClicked: if (mouse.button === Qt.RightButton) openUserProfile(model.userId, model.username, model.username, model.avatarUrl, "")
                            }
                        }
                    }

                    Item {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        visible: relationshipModel.count === 0
                        Text {
                            anchors.centerIn: parent
                            text: "No friends yet.\nAdd someone by username above."
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 14
                            horizontalAlignment: Text.AlignHCenter
                        }
                    }
                }

                // Empty state when no channel selected and not on Friends page
                Item {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    visible: currentChannelId === "" && !(currentGuildId === "" && homeSubView === "friends")

                    Text {
                        anchors.centerIn: parent
                        text: "Select a channel"
                        color: theme.textMuted
                        font.family: fontFamily
                        font.pixelSize: 14
                    }
                }

                // Messages (only when a channel is selected)
                ListView {
                    id: messageList
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    visible: !isVoiceChannel && currentChannelId !== ""
                    model: messageModel
                    focus: false
                    clip: true
                    verticalLayoutDirection: ListView.BottomToTop
                    spacing: 0
                    boundsBehavior: Flickable.StopAtBounds

                    property bool hasMoreHistory: true
                    property bool isLoadingMore: false

                    // Detect scroll near top to trigger loading more messages
                    onContentYChanged: {
                        if (!hasMoreHistory || isLoadingMore) return
                        if (messageModel.count === 0) return
                        // In BottomToTop layout, contentY increases as user scrolls up
                        if (contentY >= contentHeight - height - 300) {
                            isLoadingMore = true
                            var oldest = messageModel.get(messageModel.count - 1)
                            if (oldest && oldest.messageId && app) {
                                app.load_more_messages(currentChannelId, oldest.messageId)
                            }
                        }
                    }

                    // Footer = visual top (BottomToTop). Shows loading or beginning indicator
                    footer: Item {
                        width: messageList.width
                        height: messageList.isLoadingMore ? 48 :
                                (!messageList.hasMoreHistory && messageModel.count > 0) ? 64 : (messageList.hasMoreHistory ? 56 : 0)
                        focus: false

                        // Loading spinner text
                        Text {
                            anchors.centerIn: parent
                            visible: messageList.isLoadingMore
                            text: "Loading older messages\u{2026}"
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 12
                        }

                        // Beginning of channel indicator
                        Column {
                            anchors.centerIn: parent
                            visible: !messageList.hasMoreHistory && messageModel.count > 0
                            spacing: 4

                            Rectangle {
                                width: 120
                                height: 1
                                color: theme.separator
                                anchors.horizontalCenter: parent.horizontalCenter
                            }
                            Text {
                                text: currentChannelName ? ("This is the beginning of #" + currentChannelName) : "Beginning of conversation"
                                color: theme.textFaint
                                font.family: fontFamily
                                font.pixelSize: 11
                                anchors.horizontalCenter: parent.horizontalCenter
                            }
                        }

                        // Load older messages button (visible when more history available and not loading)
                        Button {
                            anchors.centerIn: parent
                            visible: messageList.hasMoreHistory && !messageList.isLoadingMore
                            flat: true
                            text: "Load older messages"
                            font.family: fontFamily
                            font.pixelSize: 12
                            onClicked: {
                                messageList.isLoadingMore = true
                                var oldest = messageModel.get(messageModel.count - 1)
                                if (oldest && oldest.messageId && app) {
                                    app.load_more_messages(currentChannelId, oldest.messageId)
                                } else {
                                    messageList.isLoadingMore = false
                                }
                            }
                            background: Rectangle {
                                color: parent.hovered ? theme.bgModifier : "transparent"
                                radius: theme.radiusSmall
                            }
                            contentItem: Text {
                                text: parent.text
                                color: theme.textMuted
                                font: parent.font
                                horizontalAlignment: Text.AlignHCenter
                                verticalAlignment: Text.AlignVCenter
                            }
                        }
                    }

                    ScrollBar.vertical: ScrollBar {
                        policy: ScrollBar.AsNeeded
                        contentItem: Rectangle {
                            implicitWidth: 6
                            radius: 3
                            color: theme.textFaint
                            opacity: parent.active ? 0.6 : 0.0
                            Behavior on opacity { NumberAnimation { duration: theme.animSlow } }
                        }
                        background: Item {}
                    }

                    // Extract the date portion (YYYY-MM-DD) from an ISO timestamp string (UTC)
                    function dateFromTimestamp(ts) {
                        if (!ts) return ""
                        var m = ts.match(/^(\d{4}-\d{2}-\d{2})/)
                        return m ? m[1] : ts.substring(0, 10)
                    }

                    // Get local date string (YYYY-MM-DD) from a timestamp so day dividers match user's date
                    function localDateStringFromTimestamp(ts) {
                        if (!ts) return ""
                        var d = new Date(ts)
                        if (isNaN(d.getTime())) return ""
                        var y = d.getFullYear()
                        var m = d.getMonth() + 1
                        var day = d.getDate()
                        return y + "-" + (m < 10 ? "0" : "") + m + "-" + (day < 10 ? "0" : "") + day
                    }

                    // Format a date string (YYYY-MM-DD) for display: "February 13, 2025" / "Today" / "Yesterday"
                    function formatDateLabel(dateStr) {
                        var d = new Date(dateStr + "T00:00:00")
                        if (isNaN(d.getTime())) return dateStr
                        var now = new Date()
                        var today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
                        var yesterday = new Date(today); yesterday.setDate(today.getDate() - 1)
                        var target = new Date(d.getFullYear(), d.getMonth(), d.getDate())
                        if (target.getTime() === today.getTime()) return "Today"
                        if (target.getTime() === yesterday.getTime()) return "Yesterday"
                        var months = ["January","February","March","April","May","June",
                                      "July","August","September","October","November","December"]
                        return months[d.getMonth()] + " " + d.getDate() + ", " + d.getFullYear()
                    }

                    // Format an ISO timestamp into a friendly display string:
                    //   Today at 10:30 PM, Yesterday at 3:15 AM, 02/13/2025 10:30 PM
                    function formatTimestamp(ts) {
                        if (!ts) return ""
                        var d = new Date(ts)
                        if (isNaN(d.getTime())) return ts
                        // Build 12-hour time string
                        var hrs = d.getHours()
                        var mins = d.getMinutes()
                        var ampm = hrs >= 12 ? "PM" : "AM"
                        hrs = hrs % 12
                        if (hrs === 0) hrs = 12
                        var timeStr = hrs + ":" + (mins < 10 ? "0" : "") + mins + " " + ampm
                        // Compare dates
                        var now = new Date()
                        var today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
                        var yesterday = new Date(today); yesterday.setDate(today.getDate() - 1)
                        var msgDate = new Date(d.getFullYear(), d.getMonth(), d.getDate())
                        if (msgDate.getTime() === today.getTime())
                            return "Today at " + timeStr
                        if (msgDate.getTime() === yesterday.getTime())
                            return "Yesterday at " + timeStr
                        var mm = d.getMonth() + 1
                        var dd = d.getDate()
                        return (mm < 10 ? "0" : "") + mm + "/" + (dd < 10 ? "0" : "") + dd + "/" + d.getFullYear() + " " + timeStr
                    }

                    // Format just the time portion: "10:30 PM"
                    function formatTimeOnly(ts) {
                        if (!ts) return ""
                        var d = new Date(ts)
                        if (isNaN(d.getTime())) {
                            var m = ts.match(/\d{1,2}:\d{2}/)
                            return m ? m[0] : ""
                        }
                        var hrs = d.getHours()
                        var mins = d.getMinutes()
                        var ampm = hrs >= 12 ? "PM" : "AM"
                        hrs = hrs % 12
                        if (hrs === 0) hrs = 12
                        return hrs + ":" + (mins < 10 ? "0" : "") + mins + " " + ampm
                    }

                    // Parse JSON array string for stickers/attachments; returns [] on error or empty
                    function parseJsonArray(s) {
                        if (!s || typeof s !== "string") return []
                        try {
                            var a = JSON.parse(s)
                            return Array.isArray(a) ? a : []
                        } catch (e) { return [] }
                    }
                    function parseReactions(r) {
                        if (!r) return []
                        if (typeof r === "string") {
                            try { var a = JSON.parse(r); return Array.isArray(a) ? a : [] } catch (e) { return [] }
                        }
                        return Array.isArray(r) ? r : []
                    }

                    // Returns true if this message is the first of its day (should show a day divider above it).
                    // In BottomToTop, the message visually above is index+1 (older). Use local date so dividers match user timezone.
                    function isDayBoundary(idx) {
                        if (idx < 0 || idx >= messageModel.count) return false
                        var prevIdx = idx + 1
                        if (prevIdx >= messageModel.count) return true // oldest loaded msg always shows date
                        var currDate = localDateStringFromTimestamp(messageModel.get(idx).timestamp || "")
                        var prevDate = localDateStringFromTimestamp(messageModel.get(prevIdx).timestamp || "")
                        return currDate !== prevDate
                    }

                    // Helper: determine if this message should show compact (condensed) style.
                    // In BottomToTop, index 0 = newest. The "previous" message visually above
                    // is index+1 (older). We condense if same author and close timestamps.
                    function isCondensed(idx) {
                        if (idx < 0 || idx >= messageModel.count) return false
                        // Never condense across day boundaries
                        if (isDayBoundary(idx)) return false
                        // The message visually above this one is idx+1 (older)
                        var prevIdx = idx + 1
                        if (prevIdx >= messageModel.count) return false
                        var curr = messageModel.get(idx)
                        var prev = messageModel.get(prevIdx)
                        if (!curr || !prev) return false
                        if (curr.authorId !== prev.authorId) return false
                        // Don't condense deleted messages
                        if (curr.isDeleted || prev.isDeleted) return false
                        // Simple timestamp proximity: if timestamps share the same
                        // HH:MM prefix or are very similar, condense.
                        // For a more robust check, we'd parse ISO timestamps.
                        // Quick heuristic: if both timestamps contain "at HH:MM" and
                        // the minute difference is <=5, condense.
                        var t1 = curr.timestamp || ""
                        var t2 = prev.timestamp || ""
                        // If timestamps are identical prefix (same minute), condense
                        if (t1 === t2) return true
                        // Try to extract minutes for comparison
                        var m1 = t1.match(/:(\d{2})/)
                        var m2 = t2.match(/:(\d{2})/)
                        if (m1 && m2) {
                            // Same hour prefix check
                            var h1 = t1.match(/(\d{1,2}):/)
                            var h2 = t2.match(/(\d{1,2}):/)
                            if (h1 && h2 && h1[1] === h2[1]) {
                                var diff = Math.abs(parseInt(m1[1]) - parseInt(m2[1]))
                                return diff <= 5
                            }
                        }
                        // If timestamps are similar strings, condense
                        return false
                    }

                    delegate: Rectangle {
                        id: msgDelegate
                        width: messageList.width
                        focus: false

                        property bool condensed: messageList.isCondensed(index)
                        property bool showDayDivider: messageList.isDayBoundary(index)
                        property bool isReply: (model && model.messageType !== undefined ? model.messageType : 0) === 19
                        property bool isSystemMsg: {
                            var t = (model && model.messageType !== undefined) ? model.messageType : 0
                            return t !== 0 && t !== 19 && t !== 20 && t !== 23
                        }
                        property bool isMentioned: model ? (model.mentionsMe || false) : false
                        property bool isMentionEveryone: model ? (model.mentionEveryone || false) : false
                        property bool hasReplyPreview: isReply && (model ? (model.replyContent || "").length : 0) > 0
                        property var _reactionsCheck: model ? model.reactions : null
                        property bool hasReactions: false
                        on_ReactionsCheckChanged: hasReactions = !isSystemMsg && messageList.parseReactions(_reactionsCheck).length > 0
                        Component.onCompleted: hasReactions = !isSystemMsg && messageList.parseReactions(_reactionsCheck).length > 0

                        // Height: day divider + reply bar + message content + reactions row (use 0 for undefined to avoid binding loop)
                        height: (showDayDivider ? 36 : 0)
                                + (hasReplyPreview && !condensed ? (replyBar.height || 0) + 2 : 0)
                                + (isSystemMsg ? (systemMsgRow.implicitHeight || 0) + 8
                                    : condensed ? (msgContentCompact.implicitHeight || 0) + 2
                                    : (msgRow.implicitHeight || 0) + 16)
                                + (hasReactions ? 30 : 0)
                        color: isMentioned
                               ? "#5865f212"
                               : msgHoverMa.containsMouse ? theme.bgModifier : "transparent"
                        Behavior on color { ColorAnimation { duration: theme.animFast } }

                        // Left mention accent bar
                        Rectangle {
                            visible: msgDelegate.isMentioned
                            anchors.left: parent.left
                            anchors.top: parent.top
                            anchors.bottom: parent.bottom
                            width: 3
                            color: theme.accent
                            radius: 1
                        }

                        MouseArea {
                            id: msgHoverMa
                            anchors.fill: parent
                            hoverEnabled: true
                            acceptedButtons: Qt.NoButton
                        }

                        MouseArea {
                            anchors.fill: parent
                            acceptedButtons: Qt.RightMouseButton
                            onClicked: {
                                if (!model || msgDelegate.isSystemMsg) return
                                var g = msgDelegate.mapToGlobal(mouse.x, mouse.y)
                                var localPos = msgContextMenu.parent.mapFromGlobal(g.x, g.y)
                                root.openMessageContextMenu(model.messageId || "", model.channelId || "", model.authorName || "", model.authorId || "", model.content || "", model.authorRoleColor || "", model.authorAvatarUrl || "", (localPos && localPos.x !== undefined) ? localPos.x : 0, (localPos && localPos.y !== undefined) ? localPos.y : 0)
                            }
                        }

                        // ── Day divider ──
                        Item {
                            id: dayDivider
                            visible: msgDelegate.showDayDivider
                            anchors.top: parent.top
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.topMargin: 8
                            height: visible ? 28 : 0

                            Rectangle {
                                anchors.left: parent.left
                                anchors.right: parent.right
                                anchors.leftMargin: 16
                                anchors.rightMargin: 16
                                anchors.verticalCenter: parent.verticalCenter
                                height: 1
                                color: theme.separator
                            }

                            Rectangle {
                                anchors.centerIn: parent
                                width: dayLabel.implicitWidth + 16
                                height: 18
                                color: theme.bgBase
                                radius: 9

                                Text {
                                    id: dayLabel
                                    anchors.centerIn: parent
                                    text: messageList.formatDateLabel(
                                        messageList.localDateStringFromTimestamp(model.timestamp || "")
                                    )
                                    color: theme.textMuted
                                    font.family: fontFamily
                                    font.pixelSize: 10
                                    font.weight: Font.Medium
                                }
                            }
                        }

                        // ── Reply reference bar ──
                        Item {
                            id: replyBar
                            visible: msgDelegate.hasReplyPreview && !msgDelegate.condensed
                            anchors.top: msgDelegate.showDayDivider ? dayDivider.bottom : parent.top
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.leftMargin: 64
                            anchors.rightMargin: 16
                            anchors.topMargin: 6
                            height: visible ? (replyRow.implicitHeight || 0) + 4 : 0

                            Row {
                                id: replyRow
                                spacing: 6
                                anchors.verticalCenter: parent.verticalCenter

                                // Reply connector line
                                Item {
                                    width: 24
                                    height: 12
                                    anchors.verticalCenter: parent.verticalCenter

                                    // Vertical portion
                                    Rectangle {
                                        width: 2; height: 10
                                        anchors.left: parent.left
                                        anchors.leftMargin: 11
                                        anchors.top: parent.top
                                        color: theme.textFaint
                                        opacity: 0.4
                                        radius: 1
                                    }
                                    // Horizontal portion
                                    Rectangle {
                                        width: 10; height: 2
                                        anchors.left: parent.left
                                        anchors.leftMargin: 11
                                        anchors.bottom: parent.bottom
                                        color: theme.textFaint
                                        opacity: 0.4
                                        radius: 1
                                    }
                                }

                                DAvatar {
                                    size: 16
                                    imageUrl: model.replyAuthorAvatarUrl || ""
                                    fallbackText: model.replyAuthorName || "?"
                                    anchors.verticalCenter: parent.verticalCenter
                                }

                                Text {
                                    text: model.replyAuthorName || "Unknown"
                                    color: (model.replyAuthorRoleColor && model.replyAuthorRoleColor.length > 0) ? model.replyAuthorRoleColor : theme.textNormal
                                    font.family: fontFamily
                                    font.pixelSize: 11
                                    font.weight: Font.Medium
                                    anchors.verticalCenter: parent.verticalCenter
                                }

                                Text {
                                    text: {
                                        var rc = model.replyContent || ""
                                        if (rc.length > 80) rc = rc.substring(0, 80) + "..."
                                        return rc
                                    }
                                    color: theme.textMuted
                                    font.family: fontFamily
                                    font.pixelSize: 11
                                    elide: Text.ElideRight
                                    anchors.verticalCenter: parent.verticalCenter
                                    maximumLineCount: 1
                                }
                            }
                        }

                        // ── System message (joins, boosts, pins, etc.) ──
                        RowLayout {
                            id: systemMsgRow
                            visible: msgDelegate.isSystemMsg
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.leftMargin: 16
                            anchors.rightMargin: 16
                            anchors.top: msgDelegate.showDayDivider ? dayDivider.bottom : parent.top
                            anchors.topMargin: 4
                            spacing: 8

                            // System icon placeholder (arrow or info icon area)
                            Item {
                                Layout.preferredWidth: 36
                                Layout.preferredHeight: 20
                                Layout.alignment: Qt.AlignTop

                                Text {
                                    anchors.centerIn: parent
                                    text: {
                                        var t = model.messageType || 0
                                        if (t === 1 || t === 2) return "\u{1F465}"  // group add/remove
                                        if (t === 3) return "\u{1F4DE}"   // call
                                        if (t === 4) return "\u{270F}"    // pencil (channel name)
                                        if (t === 5) return "\u{1F5BC}"   // frame (channel icon)
                                        if (t === 6) return "\u{1F4CC}"   // pin
                                        if (t === 7) return "\u{2192}"    // join arrow
                                        if (t >= 8 && t <= 11) return "\u{1F680}"  // boost
                                        if (t === 12) return "\u{1F517}"  // link (follow)
                                        if (t === 16 || t === 17) return "\u{26A0}"   // discovery warning
                                        if (t === 18 || t === 21) return "\u{1F4AC}"  // thread
                                        if (t === 22) return "\u{2709}"   // envelope (invite reminder)
                                        if (t === 24) return "\u{1F6E1}"  // shield (automod)
                                        if (t === 25 || t === 26 || t === 32) return "\u{2728}"  // sparkles (premium)
                                        if (t === 27 || t === 28 || t === 29 || t === 31) return "\u{1F3A4}"  // mic (stage)
                                        if (t >= 36 && t <= 39) return "\u{26A0}"   // incident warning
                                        if (t === 44) return "\u{1F6D2}"  // shopping cart (purchase)
                                        if (t === 46) return "\u{1F4CA}"  // bar chart (poll)
                                        return "\u{2139}"                // info (default)
                                    }
                                    font.pixelSize: 14
                                    color: theme.textFaint
                                }
                            }

                            Text {
                                Layout.fillWidth: true
                                text: model.content || "[System message]"
                                color: theme.textMuted
                                font.family: fontFamily
                                font.pixelSize: 12
                                font.italic: true
                                wrapMode: Text.WordWrap
                                lineHeight: 1.3
                            }

                            Text {
                                text: messageList.formatTimestamp(model.timestamp || "")
                                color: theme.textFaint
                                font.family: fontFamily
                                font.pixelSize: 10
                                Layout.alignment: Qt.AlignTop
                            }
                        }

                        // ── Full message layout (avatar + name + timestamp + content) ──
                        RowLayout {
                            id: msgRow
                            visible: !msgDelegate.condensed && !msgDelegate.isSystemMsg
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.leftMargin: 16
                            anchors.rightMargin: 16
                            anchors.top: msgDelegate.hasReplyPreview ? replyBar.bottom
                                       : msgDelegate.showDayDivider ? dayDivider.bottom : parent.top
                            anchors.topMargin: msgDelegate.hasReplyPreview ? 2 : 17
                            spacing: 16

                            Item {
                                Layout.preferredWidth: 40
                                Layout.preferredHeight: 40
                                Layout.alignment: Qt.AlignTop
                                MouseArea {
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: openUserProfile(model.authorId || "", model.authorName || "", model.authorName || "", model.authorAvatarUrl || "", model.authorRoleColor || "")
                                }
                                DAvatar {
                                    anchors.fill: parent
                                    size: 40
                                    imageUrl: model.authorAvatarUrl || ""
                                    fallbackText: model.authorName || "?"
                                }
                                Rectangle {
                                    width: 12; height: 12; radius: 6
                                    anchors.right: parent.right
                                    anchors.bottom: parent.bottom
                                    border.width: 2
                                    border.color: theme.bgPrimary
                                    color: getStatusColor((presenceVersion, app ? app.get_user_status(model.authorId || "") : ""))
                                }
                            }

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 3

                                RowLayout {
                                    spacing: 8
                                    Item {
                                        Layout.preferredHeight: authorNameText.implicitHeight
                                        Layout.preferredWidth: authorNameText.implicitWidth
                                        MouseArea {
                                            anchors.fill: parent
                                            hoverEnabled: true
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: openUserProfile(model.authorId || "", model.authorName || "", model.authorName || "", model.authorAvatarUrl || "", model.authorRoleColor || "")
                                        }
                                        Text {
                                            id: authorNameText
                                            text: model.authorName || "Unknown"
                                            color: (model.authorRoleColor && model.authorRoleColor.length > 0) ? model.authorRoleColor : theme.accent
                                            font.family: fontFamily
                                            font.pixelSize: 16
                                            font.weight: Font.Medium
                                        }
                                    }
                                    UserBadges {
                                        publicFlags: model.authorPublicFlags || 0
                                        isBot: model.authorBot || false
                                        premiumType: model.authorPremiumType || 0
                                        badgeSize: 16
                                    }
                                    RolePill {
                                        roleName: model.authorRoleName || ""
                                        roleColor: model.authorRoleColor || ""
                                        fontSize: 10
                                    }
                                    // @mention badge when user is mentioned
                                    Rectangle {
                                        visible: msgDelegate.isMentioned
                                        width: mentionBadgeText.implicitWidth + 10
                                        height: 16
                                        radius: 8
                                        color: theme.accentMuted

                                        Text {
                                            id: mentionBadgeText
                                            anchors.centerIn: parent
                                            text: msgDelegate.isMentionEveryone ? "@everyone" : "@mention"
                                            color: theme.accent
                                            font.family: fontFamily
                                            font.pixelSize: 9
                                            font.bold: true
                                        }
                                    }
                                    Text {
                                        text: messageList.formatTimestamp(model.timestamp || "")
                                        color: theme.textFaint
                                        font.family: fontFamily
                                        font.pixelSize: 12
                                    }
                                    // Deleted badge (only when style is "deleted")
                                    Rectangle {
                                        visible: (model.isDeleted || false) && (root.deletedMessageStyle === "deleted")
                                        width: delLabel.width + 10
                                        height: 16; radius: 8
                                        color: "#f23f4318"

                                        Text {
                                            id: delLabel
                                            anchors.centerIn: parent
                                            text: "DELETED"
                                            color: theme.danger
                                            font.family: fontFamily
                                            font.pixelSize: 9
                                            font.bold: true
                                        }
                                    }
                                }

                                Text {
                                    Layout.fillWidth: true
                                    textFormat: (model.contentHtml && model.contentHtml.length > 0) ? Text.RichText : Text.PlainText
                                    text: (model.contentHtml && model.contentHtml.length > 0) ? model.contentHtml : (model.content || "")
                                    color: (model.isDeleted && (root.deletedMessageStyle === "strikethrough" || root.deletedMessageStyle === "deleted")) ? theme.danger : theme.textNormal
                                    font.family: fontFamily
                                    font.pixelSize: 16
                                    font.strikeout: (model.isDeleted || false) && (root.deletedMessageStyle === "strikethrough" || root.deletedMessageStyle === "deleted")
                                    wrapMode: Text.WordWrap
                                    lineHeight: 1.375
                                    opacity: (model.isDeleted && root.deletedMessageStyle === "faded") ? 0.5 : 1.0
                                    onLinkActivated: handleLinkClick(link)
                                }

                                // Stickers row
                                Row {
                                    id: stickersRow
                                    property string stickersJsonSource: model ? (model.stickersJson || "") : ""
                                    property var stickersParsed: []
                                    onStickersJsonSourceChanged: stickersParsed = messageList.parseJsonArray(stickersJsonSource)
                                    Component.onCompleted: stickersParsed = messageList.parseJsonArray(stickersJsonSource)
                                    visible: stickersParsed.length > 0
                                    Layout.topMargin: 4
                                    spacing: 4
                                    Repeater {
                                        model: stickersRow.stickersParsed
                                        delegate: Image {
                                            width: 80
                                            height: 80
                                            fillMode: Image.PreserveAspectFit
                                            source: modelData.url || ""
                                            smooth: true
                                            asynchronous: true
                                            mipmap: true
                                        }
                                    }
                                }

                                // Embeds row (link previews, rich embeds)
                                Column {
                                    id: embedsRow
                                    property string embedsJsonSource: model ? (model.embedsJson || "[]") : "[]"
                                    property var embedsParsed: []
                                    onEmbedsJsonSourceChanged: embedsParsed = messageList.parseJsonArray(embedsJsonSource)
                                    Component.onCompleted: embedsParsed = messageList.parseJsonArray(embedsJsonSource)
                                    visible: embedsParsed.length > 0
                                    Layout.topMargin: 4
                                    Layout.fillWidth: true
                                    spacing: 8
                                    Repeater {
                                        model: embedsRow.embedsParsed
                                        delegate: Rectangle {
                                            width: parent ? Math.max(200, parent.width - 16) : 380
                                            implicitHeight: embedCol.implicitHeight + 16
                                            radius: theme.radiusSmall
                                            color: theme.bgSecondary
                                            border.width: 1
                                            border.color: (modelData.color ? ("#" + ("000000" + (modelData.color >>> 0).toString(16)).slice(-6)) : theme.border) || theme.border
                                            Column {
                                                id: embedCol
                                                anchors.left: parent.left
                                                anchors.right: parent.right
                                                anchors.top: parent.top
                                                anchors.margins: 8
                                                spacing: 6
                                                Item { width: 1; height: 1 }
                                                Row {
                                                    visible: modelData.author && (modelData.author.name || modelData.author.icon_url)
                                                    spacing: 8
                                                    Image {
                                                        width: 24
                                                        height: 24
                                                        visible: modelData.author && modelData.author.icon_url
                                                        source: modelData.author ? modelData.author.icon_url : ""
                                                        fillMode: Image.PreserveAspectFit
                                                    }
                                                    Text {
                                                        text: modelData.author ? modelData.author.name : ""
                                                        color: theme.textNormal
                                                        font.pixelSize: 12
                                                        font.bold: true
                                                    }
                                                }
                                                Text {
                                                    visible: modelData.title && modelData.title.length > 0
                                                    text: modelData.url ? ("<a href=\"" + modelData.url + "\">" + modelData.title + "</a>") : (modelData.title || "")
                                                    textFormat: modelData.url ? Text.RichText : Text.PlainText
                                                    color: theme.accent
                                                    font.pixelSize: 14
                                                    font.bold: true
                                                    wrapMode: Text.WordWrap
                                                    width: parent.width - 16
                                                    linkColor: theme.accent
                                                    onLinkActivated: handleLinkClick(link)
                                                }
                                                Text {
                                                    visible: modelData.description && modelData.description.length > 0
                                                    text: modelData.description || ""
                                                    color: theme.textNormal
                                                    font.pixelSize: 13
                                                    wrapMode: Text.WordWrap
                                                    width: parent.width - 16
                                                    lineHeight: 1.35
                                                }
                                                Column {
                                                    visible: modelData.fields && modelData.fields.length > 0
                                                    width: parent.width - 16
                                                    spacing: 8
                                                    Repeater {
                                                        model: modelData.fields || []
                                                        delegate: Column {
                                                            width: embedCol.width - 16
                                                            spacing: 2
                                                            Text {
                                                                text: modelData.name || ""
                                                                color: theme.textFaint
                                                                font.pixelSize: 11
                                                                font.bold: true
                                                                wrapMode: Text.WordWrap
                                                                width: parent.width
                                                            }
                                                            Text {
                                                                text: modelData.value || ""
                                                                color: theme.textNormal
                                                                font.pixelSize: 12
                                                                wrapMode: Text.WordWrap
                                                                width: parent.width
                                                            }
                                                        }
                                                    }
                                                }
                                                Row {
                                                    visible: modelData.thumbnail && modelData.thumbnail.url
                                                    spacing: 8
                                                    Image {
                                                        width: 80
                                                        height: 80
                                                        source: modelData.thumbnail ? modelData.thumbnail.url : ""
                                                        fillMode: Image.PreserveAspectFit
                                                    }
                                                }
                                                Image {
                                                    visible: modelData.image && modelData.image.url
                                                    width: Math.min(400, modelData.image.width || 400)
                                                    height: modelData.image && modelData.image.url ? (modelData.image.height || 200) : 0
                                                    source: modelData.image ? modelData.image.url : ""
                                                    fillMode: Image.PreserveAspectFit
                                                }
                                                Text {
                                                    visible: modelData.footer && modelData.footer.text
                                                    text: modelData.footer ? modelData.footer.text : ""
                                                    color: theme.textFaint
                                                    font.pixelSize: 10
                                                }
                                                Item { width: 1; height: 1 }
                                            }
                                            MouseArea {
                                                anchors.fill: parent
                                                cursorShape: modelData.url ? Qt.PointingHandCursor : Qt.ArrowCursor
                                                onClicked: { if (modelData.url) Qt.openUrlExternally(modelData.url) }
                                            }
                                        }
                                    }
                                }

                                // Attachments row (thumbnails for images, link for others)
                                Flow {
                                    id: attachmentsRow
                                    property string attachmentsJsonSource: model ? (model.attachmentsJson || "") : ""
                                    property var attachmentsParsed: []
                                    onAttachmentsJsonSourceChanged: attachmentsParsed = messageList.parseJsonArray(attachmentsJsonSource)
                                    Component.onCompleted: attachmentsParsed = messageList.parseJsonArray(attachmentsJsonSource)
                                    visible: attachmentsParsed.length > 0
                                    Layout.topMargin: 4
                                    Layout.fillWidth: true
                                    spacing: 6
                                    Repeater {
                                        model: attachmentsRow.attachmentsParsed
                                        delegate: Item {
                                            width: attachmentDelegateContent.width
                                            height: attachmentDelegateContent.height
                                            property bool isImage: {
                                                var ct = (modelData.content_type || "").toLowerCase()
                                                var fn = (modelData.filename || "").toLowerCase()
                                                return ct.indexOf("image") >= 0 || /\.(png|jpe?g|gif|webp)$/.test(fn)
                                            }
                                            Row {
                                                id: attachmentDelegateContent
                                                spacing: 6
                                                Image {
                                                    width: 72
                                                    height: 72
                                                    visible: attachmentDelegateContent.parent.isImage
                                                    fillMode: Image.PreserveAspectFit
                                                    source: visible ? (modelData.url || modelData.proxy_url || "") : ""
                                                    smooth: true
                                                    asynchronous: true
                                                }
                                                Column {
                                                    spacing: 2
                                                    Text {
                                                        text: modelData.filename || "file"
                                                        color: theme.accent
                                                        font.family: fontFamily
                                                        font.pixelSize: 12
                                                        elide: Text.ElideMiddle
                                                        maximumLineCount: 1
                                                        width: 200
                                                    }
                                                    Text {
                                                        visible: modelData.size
                                                        text: (modelData.size / 1024).toFixed(1) + " KB"
                                                        color: theme.textFaint
                                                        font.family: fontFamily
                                                        font.pixelSize: 10
                                                    }
                                                }
                                            }
                                            MouseArea {
                                                anchors.fill: parent
                                                cursorShape: Qt.PointingHandCursor
                                                onClicked: Qt.openUrlExternally(modelData.url || modelData.proxy_url || "")
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // ── Condensed message layout (content only, aligned with full messages) ──
                        Item {
                            id: msgContentCompact
                            visible: msgDelegate.condensed && !msgDelegate.isSystemMsg
                            anchors.left: parent.left
                            anchors.right: parent.right
                            // 16 (left margin) + 40 (avatar) + 16 (spacing) = 72
                            anchors.leftMargin: 72
                            anchors.rightMargin: 16
                            anchors.top: parent.top
                            anchors.topMargin: 1
                            implicitHeight: compactText.implicitHeight

                            // Hover timestamp (shown instead of avatar space)
                            Text {
                                visible: msgHoverMa.containsMouse
                                anchors.right: parent.left
                                anchors.rightMargin: 8
                                anchors.top: parent.top
                                text: messageList.formatTimeOnly(model.timestamp || "")
                                color: theme.textFaint
                                font.family: fontFamily
                                font.pixelSize: 10
                            }

                            Text {
                                id: compactText
                                anchors.left: parent.left
                                anchors.right: parent.right
                                text: model.content || ""
                                color: (model.isDeleted && (root.deletedMessageStyle === "strikethrough" || root.deletedMessageStyle === "deleted")) ? theme.danger : theme.textNormal
                                font.family: fontFamily
                                font.pixelSize: 14
                                font.strikeout: (model.isDeleted || false) && (root.deletedMessageStyle === "strikethrough" || root.deletedMessageStyle === "deleted")
                                wrapMode: Text.WordWrap
                                lineHeight: 1.25
                                opacity: (model.isDeleted && root.deletedMessageStyle === "faded") ? 0.5 : 1.0
                            }
                        }

                        Item {
                            id: reactionsRow
                            visible: msgDelegate.hasReactions
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.leftMargin: 72
                            anchors.rightMargin: 16
                            anchors.top: msgDelegate.condensed ? msgContentCompact.bottom : msgRow.bottom
                            anchors.topMargin: 4
                            height: visible ? 26 : 0
                            property string msgMessageId: model ? (model.messageId || "") : ""
                            property string msgChannelId: model ? (model.channelId || "") : ""
                            property var reactionsSource: model ? model.reactions : null
                            property var reactionsParsed: []
                            onReactionsSourceChanged: reactionsParsed = messageList.parseReactions(reactionsSource)
                            Component.onCompleted: reactionsParsed = messageList.parseReactions(reactionsSource)

                            Row {
                                spacing: 4
                                Repeater {
                                    model: reactionsRow.reactionsParsed
                                    delegate: Rectangle {
                                        width: reactionPillContent.width + 12
                                        height: 22
                                        radius: 11
                                        color: reactionPillMa.containsMouse ? theme.bgActive : theme.bgSecondary
                                        border.width: 1
                                        border.color: modelData.me ? theme.accent : theme.border

                                        Row {
                                            id: reactionPillContent
                                            anchors.centerIn: parent
                                            spacing: 4
                                            Twemoji {
                                                emoji: modelData.emoji || ""
                                                size: 14
                                            }
                                            Text {
                                                text: modelData.count || 0
                                                color: theme.textMuted
                                                font.family: fontFamily
                                                font.pixelSize: 11
                                                anchors.verticalCenter: parent.verticalCenter
                                            }
                                        }
                                        MouseArea {
                                            id: reactionPillMa
                                            anchors.fill: parent
                                            hoverEnabled: true
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: {
                                                if (!app || !reactionsRow.msgChannelId || !reactionsRow.msgMessageId) return
                                                var emojiKey = modelData.emoji || ""
                                                if (modelData.me) {
                                                    app.remove_reaction(reactionsRow.msgChannelId, reactionsRow.msgMessageId, emojiKey)
                                                } else {
                                                    app.add_reaction(reactionsRow.msgChannelId, reactionsRow.msgMessageId, emojiKey)
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        Rectangle {
                            id: msgActionBtn
                            visible: (msgHoverMa.containsMouse || msgContextMenu.visible) && !msgDelegate.isSystemMsg
                            anchors.right: parent.right
                            anchors.top: parent.top
                            anchors.rightMargin: 14
                            anchors.topMargin: 4
                            width: 26; height: 24
                            radius: theme.radiusSmall
                            color: msgActionMa.containsMouse || msgContextMenu.visible
                                   ? theme.bgActive : theme.bgElevated

                            Behavior on color { ColorAnimation { duration: 60 } }

                            Text {
                                anchors.centerIn: parent
                                text: "\u{22EF}"
                                color: theme.textMuted
                                font.pixelSize: 14
                            }
                            MouseArea {
                                id: msgActionMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    var globalPos = msgActionBtn.mapToGlobal(0, msgActionBtn.height + 4)
                                    var localPos = msgContextMenu.parent.mapFromGlobal(globalPos.x, globalPos.y)
                                    var menuW = 190
                                    root.openMessageContextMenu(model.messageId || "", model.channelId || "", model.authorName || "", model.authorId || "", model.content || "", model.authorRoleColor || "", model.authorAvatarUrl || "", (localPos && localPos.x !== undefined) ? (localPos.x - menuW + msgActionBtn.width) : 0, (localPos && localPos.y !== undefined) ? localPos.y : 0)
                                }
                            }
                        }
                    }
                }

                // Typing indicator (role-colored names when typing_display_json available)
                Item {
                    visible: !isVoiceChannel && typingDisplay.length > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: visible ? 18 : 0
                    Layout.leftMargin: 20

                    RowLayout {
                        anchors.fill: parent
                        spacing: 6

                        Text {
                            text: "\u{2022}\u{2022}\u{2022}"
                            color: theme.textMuted
                            font.pixelSize: 10
                            font.weight: Font.Bold
                        }
                        Row {
                            visible: typingDisplayList && typingDisplayList.length > 0
                            spacing: 0
                            Layout.fillWidth: true
                            Repeater {
                                model: typingDisplayList || []
                                delegate: Row {
                                    spacing: 0
                                    Text {
                                        text: modelData ? modelData.name : ""
                                        color: (modelData && modelData.roleColor && modelData.roleColor.length > 0) ? modelData.roleColor : theme.textMuted
                                        font.family: fontFamily
                                        font.pixelSize: 11
                                    }
                                    Text {
                                        text: root.typingSeparator(index, (root.typingDisplayList && root.typingDisplayList.length) ? root.typingDisplayList.length : 0)
                                        color: theme.textMuted
                                        font.family: fontFamily
                                        font.pixelSize: 11
                                    }
                                }
                            }
                        }
                        Text {
                            visible: !typingDisplayList || typingDisplayList.length === 0
                            text: typingDisplay
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 11
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                        }
                    }
                }

                // Reply preview bar (shown when replying)
                Rectangle {
                    visible: !isVoiceChannel && replyToMessageId.length > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: visible ? 34 : 0
                    Layout.leftMargin: 16
                    Layout.rightMargin: 16
                    Layout.topMargin: 2
                    radius: theme.radiusMed
                    color: theme.bgSecondary

                    Behavior on Layout.preferredHeight { NumberAnimation { duration: theme.animFast } }

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 12
                        anchors.rightMargin: 8
                        spacing: 6

                        Rectangle {
                            width: 2; height: 16; radius: 1
                            color: theme.accent
                        }

                        Text {
                            text: "Replying to "
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 11
                        }
                        Text {
                            text: replyToAuthor
                            color: (replyToAuthorColor && replyToAuthorColor.length > 0) ? replyToAuthorColor : theme.textNormal
                            font.family: fontFamily
                            font.pixelSize: 11
                            font.weight: Font.Medium
                        }
                        Text {
                            Layout.fillWidth: true
                            text: replyToContent
                            color: theme.textFaint
                            font.family: fontFamily
                            font.pixelSize: 11
                            elide: Text.ElideRight
                        }

                        // Close reply
                        Rectangle {
                            width: 20; height: 20; radius: 10
                            color: closeReplyMa.containsMouse ? theme.bgHover : "transparent"

                            Text {
                                anchors.centerIn: parent
                                text: "\u{2715}"
                                color: theme.textFaint
                                font.pixelSize: 10
                            }
                            MouseArea {
                                id: closeReplyMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: clearReply()
                            }
                        }
                    }
                }

                // Message input area (hidden for voice channels)
                Rectangle {
                    visible: !isVoiceChannel
                    Layout.fillWidth: true
                    Layout.preferredHeight: theme.messageInputH
                    Layout.leftMargin: 16
                    Layout.rightMargin: 16
                    Layout.bottomMargin: 16
                    Layout.topMargin: replyToMessageId.length > 0 ? 0 : 4
                    radius: theme.radiusMed
                    color: theme.inputBg
                    border.width: messageInput.activeFocus ? 1 : 0
                    border.color: theme.accent

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 12
                        anchors.rightMargin: 12
                        spacing: 10

                        Item {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            clip: true

                            TextField {
                                id: messageInput
                                anchors.fill: parent
                                placeholderText: replyToMessageId ? ("Reply to " + replyToAuthor + "...") :
                                                                 currentChannelName ? "Message #" + currentChannelName : "Type a message..."
                                placeholderTextColor: theme.textFaint
                                color: "transparent"
                                font.family: fontFamily
                                font.pixelSize: 16
                                leftPadding: 10
                                rightPadding: 10
                                topPadding: 8
                                bottomPadding: 8
                                background: Item {}
                                verticalAlignment: TextInput.AlignVCenter
                                Keys.onReturnPressed: sendMessage()
                                Keys.onEnterPressed: sendMessage()
                                Keys.onEscapePressed: clearReply()

                                // Trigger typing indicator
                                property var lastTypingTime: 0
                                onTextChanged: {
                                    var now = Date.now()
                                    if (text.length > 0 && now - lastTypingTime > 8000 && currentChannelId && app) {
                                        app.start_typing(currentChannelId)
                                        lastTypingTime = now
                                    }
                                }
                            }

                            // Twemoji overlay: shows same text with emoji as images (non-interactive)
                            Row {
                                id: messageInputOverlay
                                anchors.left: parent.left
                                anchors.leftMargin: 10
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.right: parent.right
                                anchors.rightMargin: 10
                                spacing: 0
                                clip: true
                                enabled: false

                                Repeater {
                                    model: segmentize(messageInput.text)
                                    delegate: Item {
                                        width: modelData.type === "text" ? textSeg.implicitWidth : 20
                                        height: 20
                                        Text {
                                            id: textSeg
                                            visible: modelData.type === "text"
                                            anchors.verticalCenter: parent.verticalCenter
                                            text: modelData.type === "text" ? modelData.value : ""
                                            color: theme.textNormal
                                            font.family: fontFamily
                                            font.pixelSize: 16
                                        }
                                        Twemoji {
                                            id: emojiSeg
                                            visible: modelData.type === "emoji"
                                            anchors.centerIn: parent
                                            emoji: modelData.type === "emoji" ? modelData.value : ""
                                            size: 18
                                        }
                                    }
                                }
                            }
                        }

                        // Silent mode toggle
                        Rectangle {
                            width: 28; height: 28; radius: theme.radiusSmall
                            color: silentMode ? theme.accentMuted :
                                   silentBtnMa.containsMouse ? theme.bgHover : "transparent"
                            border.width: silentMode ? 1 : 0
                            border.color: theme.accent
                            Behavior on color { ColorAnimation { duration: theme.animFast } }

                            Text {
                                anchors.centerIn: parent
                                text: "\u{1F515}"
                                color: silentMode ? theme.accent :
                                       silentBtnMa.containsMouse ? theme.textSecondary : theme.textFaint
                                font.pixelSize: 14
                                Behavior on color { ColorAnimation { duration: theme.animFast } }
                            }
                            MouseArea {
                                id: silentBtnMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: silentMode = !silentMode
                            }

                            ToolTip {
                                visible: silentBtnMa.containsMouse
                                text: silentMode ? "Silent mode ON" : "Silent mode (no notifications)"
                                delay: 500
                            }
                        }

                        // Emoji button
                        Rectangle {
                            width: 28; height: 28; radius: theme.radiusSmall
                            color: emojiBtnMa.containsMouse ? theme.bgHover : "transparent"
                            Behavior on color { ColorAnimation { duration: theme.animFast } }

                            Text {
                                anchors.centerIn: parent
                                text: "\u{263A}"
                                color: emojiBtnMa.containsMouse ? theme.textSecondary : theme.textFaint
                                font.pixelSize: 18
                                Behavior on color { ColorAnimation { duration: theme.animFast } }
                            }
                            MouseArea {
                                id: emojiBtnMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: emojiPopup.open()
                            }
                        }
                    }
                }
            }

            // ══════════ Login Screen ══════════
            Rectangle {
                anchors.fill: parent
                visible: !isLoggedIn
                color: theme.bgBase

                // MFA overlay (when credential login requires MFA code)
                Rectangle {
                    anchors.fill: parent
                    visible: app ? app.mfa_required : false
                    color: "#dd000000"
                    z: 10

                    Rectangle {
                        anchors.centerIn: parent
                        width: 360
                        height: Math.max(300, mfaCol.implicitHeight + 56)
                        radius: theme.radiusLarge
                        color: theme.bgPrimary
                        border.width: 1
                        border.color: theme.border

                        ColumnLayout {
                            id: mfaCol
                            anchors.top: parent.top
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.topMargin: 28
                            anchors.leftMargin: 24
                            anchors.rightMargin: 24
                            anchors.bottomMargin: 28
                            spacing: 16

                            // Header: title + close
                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 12

                                Text {
                                    text: "Two-factor authentication"
                                    color: theme.textNormal
                                    font.family: fontFamily
                                    font.pixelSize: 18
                                    font.bold: true
                                }
                                Item { Layout.fillWidth: true }
                                Rectangle {
                                    width: 28
                                    height: 28
                                    radius: 14
                                    color: closeMfaMa.containsMouse ? theme.bgHover : "transparent"
                                    Layout.alignment: Qt.AlignVCenter

                                    Text {
                                        anchors.centerIn: parent
                                        text: "\u2715"
                                        color: theme.textFaint
                                        font.pixelSize: 14
                                    }
                                    MouseArea {
                                        id: closeMfaMa
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: app.cancel_mfa()
                                    }
                                }
                            }

                            Text {
                                text: "Enter the 6-digit code from your authenticator app"
                                color: theme.textMuted
                                font.family: fontFamily
                                font.pixelSize: 13
                                Layout.fillWidth: true
                                wrapMode: Text.WordWrap
                                Layout.topMargin: -4
                                Layout.bottomMargin: 4
                            }
                            TextField {
                                id: mfaInput
                                Layout.fillWidth: true
                                Layout.preferredHeight: 44
                                placeholderText: "000000"
                                placeholderTextColor: theme.textFaint
                                color: theme.textNormal
                                font.family: fontFamily
                                font.pixelSize: 18
                                maximumLength: 8
                                inputMethodHints: Qt.ImhDigitsOnly
                                background: Rectangle {
                                    color: theme.bgBase
                                    radius: theme.radiusMed
                                    border.color: mfaInput.activeFocus ? theme.accent : theme.border
                                    border.width: mfaInput.activeFocus ? 2 : 1
                                }
                                Keys.onReturnPressed: doMfaSubmit()
                            }
                            Rectangle {
                                Layout.fillWidth: true
                                Layout.preferredHeight: 44
                                radius: theme.radiusMed
                                color: mfaBtnMa.containsMouse ? theme.accentLight : theme.accent
                                Text {
                                    anchors.centerIn: parent
                                    text: "Submit"
                                    color: "#ffffff"
                                    font.family: fontFamily
                                    font.pixelSize: 14
                                    font.bold: true
                                }
                                MouseArea {
                                    id: mfaBtnMa
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: doMfaSubmit()
                                }
                            }
                        }
                    }
                }

                // Login card — credentials or token
                Rectangle {
                    anchors.centerIn: parent
                    width: 400
                    height: loginCardCol.implicitHeight + 56
                    radius: theme.radiusLarge
                    color: theme.bgPrimary
                    border.width: 1
                    border.color: theme.border

                    ColumnLayout {
                        id: loginCardCol
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.margins: 32
                        spacing: 8

                        Text {
                            text: "Welcome back"
                            color: theme.textNormal
                            font.family: fontFamily
                            font.pixelSize: 22
                            font.bold: true
                            Layout.alignment: Qt.AlignHCenter
                        }

                        Text {
                            text: (app && app.login_mode === "token") ? "Enter your token to continue" : "We're so excited to see you again!"
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 12
                            Layout.alignment: Qt.AlignHCenter
                            Layout.bottomMargin: 16
                        }

                        // Error message
                        Rectangle {
                            visible: loginError.length > 0
                            Layout.fillWidth: true
                            Layout.preferredHeight: loginErrorText.implicitHeight + 20
                            radius: theme.radiusMed
                            color: "#f23f4318"
                            border.width: 1
                            border.color: "#f23f4330"
                            Layout.bottomMargin: 8

                            Text {
                                id: loginErrorText
                                anchors.centerIn: parent
                                width: parent.width - 24
                                text: loginError
                                color: theme.danger
                                font.family: fontFamily
                                font.pixelSize: 12
                                wrapMode: Text.WordWrap
                                horizontalAlignment: Text.AlignHCenter
                            }
                        }

                        // ─── Credentials form ───
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 8
                            visible: app && app.login_mode !== "token"

                            Text {
                                text: "EMAIL OR PHONE NUMBER"
                                color: theme.textSecondary
                                font.family: fontFamily
                                font.pixelSize: 11
                                font.bold: true
                                Layout.bottomMargin: 2
                            }
                            TextField {
                                id: emailInput
                                Layout.fillWidth: true
                                Layout.preferredHeight: 44
                                placeholderText: "Email or phone number"
                                placeholderTextColor: theme.textFaint
                                color: theme.textNormal
                                font.family: fontFamily
                                font.pixelSize: 14
                                leftPadding: 12
                                background: Rectangle {
                                    color: theme.bgBase
                                    radius: theme.radiusMed
                                    border.color: emailInput.activeFocus ? theme.accent : theme.border
                                    border.width: emailInput.activeFocus ? 2 : 1
                                }
                                Keys.onReturnPressed: passwordInput.forceActiveFocus()
                            }
                            Text {
                                text: "PASSWORD"
                                color: theme.textSecondary
                                font.family: fontFamily
                                font.pixelSize: 11
                                font.bold: true
                                Layout.bottomMargin: 2
                                Layout.topMargin: 4
                            }
                            TextField {
                                id: passwordInput
                                Layout.fillWidth: true
                                Layout.preferredHeight: 44
                                placeholderText: "Password"
                                placeholderTextColor: theme.textFaint
                                color: theme.textNormal
                                font.family: fontFamily
                                font.pixelSize: 14
                                echoMode: TextInput.Password
                                leftPadding: 12
                                background: Rectangle {
                                    color: theme.bgBase
                                    radius: theme.radiusMed
                                    border.color: passwordInput.activeFocus ? theme.accent : theme.border
                                    border.width: passwordInput.activeFocus ? 2 : 1
                                }
                                Keys.onReturnPressed: doLogin()
                            }
                        }

                        // ─── Token form ───
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 8
                            visible: app && app.login_mode === "token"

                            Text {
                                text: "TOKEN"
                                color: theme.textSecondary
                                font.family: fontFamily
                                font.pixelSize: 11
                                font.bold: true
                                Layout.bottomMargin: 2
                            }
                            TextField {
                                id: tokenInput
                                Layout.fillWidth: true
                                Layout.preferredHeight: 44
                                placeholderText: "Enter your token"
                                placeholderTextColor: theme.textFaint
                                color: theme.textNormal
                                font.family: fontFamily
                                font.pixelSize: 14
                                echoMode: TextInput.Password
                                leftPadding: 12
                                background: Rectangle {
                                    color: theme.bgBase
                                    radius: theme.radiusMed
                                    border.color: tokenInput.activeFocus ? theme.accent : theme.border
                                    border.width: tokenInput.activeFocus ? 2 : 1
                                    Behavior on border.color { ColorAnimation { duration: theme.animFast } }
                                    Behavior on border.width { NumberAnimation { duration: theme.animFast } }
                                }
                                Keys.onReturnPressed: doLogin()
                            }
                        }

                        Item { Layout.preferredHeight: 4 }

                        // Login button
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 42
                            radius: theme.radiusMed
                            color: loginBtnMa.pressed ? theme.accentHover :
                                   loginBtnMa.containsMouse ? theme.accentLight :
                                   (loginLoading ? theme.accentHover : theme.accent)
                            opacity: loginLoading ? 0.7 : 1.0

                            Behavior on color { ColorAnimation { duration: theme.animFast } }
                            Behavior on opacity { NumberAnimation { duration: theme.animFast } }

                            Text {
                                anchors.centerIn: parent
                                text: loginLoading ? "Connecting..." : "Log In"
                                color: "#ffffff"
                                font.family: fontFamily
                                font.pixelSize: 14
                                font.bold: true
                            }

                            MouseArea {
                                id: loginBtnMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                enabled: !loginLoading
                                onClicked: doLogin()
                            }
                        }

                        // Mode toggle
                        Text {
                            Layout.topMargin: 12
                            Layout.alignment: Qt.AlignHCenter
                            text: (app && app.login_mode === "token") ? "Use email and password instead" : "Use token instead"
                            color: theme.accent
                            font.family: fontFamily
                            font.pixelSize: 12
                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    if (app) app.set_login_mode(app.login_mode === "token" ? "credentials" : "token")
                                }
                            }
                        }
                    }
                }
            }
        }

        // ══════════ Member List (right sidebar) ══════════
        Rectangle {
            Layout.preferredWidth: 240
            Layout.minimumWidth: 240
            Layout.maximumWidth: 240
            Layout.fillHeight: true
            visible: currentGuildId !== "" && currentChannelId !== ""
            color: theme.bgPrimary

            ColumnLayout {
                anchors.fill: parent
                spacing: 0

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: theme.headerHeight
                    color: theme.bgPrimary

                    Text {
                        anchors.left: parent.left
                        anchors.leftMargin: 12
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Members — " + memberModel.count
                        color: theme.textMuted
                        font.family: fontFamily
                        font.pixelSize: 12
                    }

                    Rectangle {
                        anchors.bottom: parent.bottom
                        width: parent.width
                        height: 1
                        color: theme.separator
                    }
                }

                ListView {
                    id: memberListView
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    model: memberModel
                    clip: true
                    focus: false
                    spacing: 2
                    boundsBehavior: Flickable.StopAtBounds

                    ScrollBar.vertical: ScrollBar {
                        policy: ScrollBar.AsNeeded
                        contentItem: Rectangle {
                            implicitWidth: 4
                            radius: 2
                            color: theme.textFaint
                            opacity: parent.active ? 0.8 : 0.0
                            Behavior on opacity { NumberAnimation { duration: theme.animSlow } }
                        }
                        background: Item {}
                    }

                    delegate: Rectangle {
                        width: memberListView.width - 8
                        x: 4
                        height: 42
                        radius: theme.radiusSmall
                        color: memberMa.containsMouse ? theme.bgHover : "transparent"
                        focus: false

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 8
                            anchors.rightMargin: 8
                            spacing: 8

                            Item {
                                Layout.preferredWidth: 32
                                Layout.preferredHeight: 32
                                DAvatar {
                                    anchors.fill: parent
                                    size: 32
                                    imageUrl: model.avatarUrl || ""
                                    fallbackText: model.displayName || model.username || "?"
                                }
                                Rectangle {
                                    width: 10; height: 10; radius: 5
                                    anchors.right: parent.right
                                    anchors.bottom: parent.bottom
                                    border.width: 2
                                    border.color: theme.bgPrimary
                                    color: getStatusColor((presenceVersion, app ? app.get_user_status(model.memberId || "") : ""))
                                }
                            }

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2

                                RowLayout {
                                    spacing: 4
                                    Text {
                                        text: model.displayName ? model.displayName : (model.username || "Unknown")
                                        color: (model.roleColor && model.roleColor.length > 0) ? model.roleColor : theme.textNormal
                                        font.family: fontFamily
                                        font.pixelSize: 14
                                        elide: Text.ElideRight
                                        Layout.fillWidth: true
                                    }
                                    UserBadges {
                                        publicFlags: model.publicFlags || 0
                                        isBot: model.bot || false
                                        premiumType: model.premiumType || 0
                                        badgeSize: 14
                                    }
                                    RolePill {
                                        roleName: model.roleName || ""
                                        roleColor: model.roleColor || ""
                                        fontSize: 9
                                    }
                                }
                            }
                        }

                        MouseArea {
                            id: memberMa
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                profilePopupTarget = {
                                    userId: model.memberId || "",
                                    displayName: model.displayName || model.username || "",
                                    username: model.username || "",
                                    avatarUrl: model.avatarUrl || "",
                                    roleColor: model.roleColor || ""
                                }
                                profilePopup.open()
                            }
                        }
                    }
                }
            }
        }
    }

    // ══════════ Join Server Popup ══════════
    Popup {
        id: joinServerPopup
        anchors.centerIn: parent
        width: 440
        height: joinServerContent.implicitHeight + 48
        modal: true
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
        padding: 0
        focus: false

        property alias inviteField: joinInviteInput
        property string joinError: ""
        property bool joinLoading: false

        onOpened: {
            joinInviteInput.text = ""
            joinError = ""
            joinLoading = false
            joinInviteInput.forceActiveFocus()
        }

        background: Rectangle {
            color: theme.bgPrimary
            radius: theme.radiusLarge
            border.color: theme.border
            border.width: 1
        }

        Overlay.modal: Rectangle {
            color: "#000000aa"
        }

        contentItem: ColumnLayout {
            id: joinServerContent
            spacing: 0

            // Header
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 56
                color: theme.bgSecondary
                radius: theme.radiusLarge

                // Bottom-clip the radius (flat bottom edge)
                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: theme.radiusLarge
                    color: theme.bgSecondary
                }

                Text {
                    anchors.centerIn: parent
                    text: "Join a Server"
                    color: theme.textNormal
                    font.family: fontFamily
                    font.pixelSize: 20
                    font.weight: Font.DemiBold
                }
            }

            // Body
            ColumnLayout {
                Layout.fillWidth: true
                Layout.margins: 24
                spacing: 12

                Text {
                    text: "INVITE LINK"
                    color: joinServerPopup.joinError.length > 0 ? theme.danger : theme.textMuted
                    font.family: fontFamily
                    font.pixelSize: 11
                    font.weight: Font.Bold
                    font.letterSpacing: 0.5
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 44
                    radius: theme.radiusSmall
                    color: theme.bgTertiary
                    border.color: joinInviteInput.activeFocus ? theme.accent : theme.border
                    border.width: 1

                    TextInput {
                        id: joinInviteInput
                        anchors.fill: parent
                        anchors.leftMargin: 12
                        anchors.rightMargin: 12
                        verticalAlignment: TextInput.AlignVCenter
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 14
                        clip: true
                        selectByMouse: true
                        onAccepted: {
                            if (text.trim().length > 0 && !joinServerPopup.joinLoading) {
                                joinServerPopup.joinError = ""
                                joinServerPopup.joinLoading = true
                                if (app) app.join_guild_by_invite(text.trim())
                            }
                        }
                    }

                    Text {
                        anchors.left: parent.left
                        anchors.leftMargin: 12
                        anchors.verticalCenter: parent.verticalCenter
                        visible: joinInviteInput.text.length === 0 && !joinInviteInput.activeFocus
                        text: "https://discord.gg/hTKzmak"
                        color: theme.textFaint
                        font.family: fontFamily
                        font.pixelSize: 14
                    }
                }

                // Hint
                Text {
                    text: "Enter an invite link or code"
                    color: theme.textMuted
                    font.family: fontFamily
                    font.pixelSize: 12
                    Layout.topMargin: -4
                }

                // Error display
                Rectangle {
                    visible: joinServerPopup.joinError.length > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: joinErrorText.implicitHeight + 16
                    radius: theme.radiusMed
                    color: "#f23f4318"
                    border.width: 1
                    border.color: "#f23f4340"

                    Text {
                        id: joinErrorText
                        anchors.centerIn: parent
                        width: parent.width - 24
                        text: joinServerPopup.joinError
                        color: theme.danger
                        font.family: fontFamily
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        horizontalAlignment: Text.AlignHCenter
                    }
                }
            }

            // Footer with buttons
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 68
                color: theme.bgSecondary
                radius: theme.radiusLarge

                // Top-clip the radius (flat top edge)
                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.top: parent.top
                    height: theme.radiusLarge
                    color: theme.bgSecondary
                }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 24
                    anchors.rightMargin: 24

                    Item { Layout.fillWidth: true }

                    // Cancel
                    Text {
                        text: "Cancel"
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 14
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: joinServerPopup.close()
                        }
                    }

                    Item { Layout.preferredWidth: 12 }

                    // Join button
                    Rectangle {
                        Layout.preferredWidth: 96
                        Layout.preferredHeight: 38
                        radius: theme.radiusSmall
                        color: joinServerPopup.joinLoading ? theme.accentMuted : (joinJoinMa.containsMouse ? theme.accentHover : theme.accent)
                        opacity: joinInviteInput.text.trim().length === 0 ? 0.5 : 1.0

                        Text {
                            anchors.centerIn: parent
                            text: joinServerPopup.joinLoading ? "Joining..." : "Join Server"
                            color: "#ffffff"
                            font.family: fontFamily
                            font.pixelSize: 14
                            font.weight: Font.DemiBold
                        }

                        MouseArea {
                            id: joinJoinMa
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            hoverEnabled: true
                            onClicked: {
                                var code = joinInviteInput.text.trim()
                                if (code.length > 0 && !joinServerPopup.joinLoading) {
                                    joinServerPopup.joinError = ""
                                    joinServerPopup.joinLoading = true
                                    if (app) app.join_guild_by_invite(code)
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ══════════ Proxy Configuration Popup ══════════
    Popup {
        id: proxyConfigPopup
        anchors.centerIn: parent
        width: 520
        height: Math.min(680, root.height - 80)
        modal: true
        closePolicy: Popup.NoAutoClose
        padding: 0
        focus: false

        property var mullvadCountries: []
        property string selectedMode: "mullvad"
        property string selectedCountry: ""
        property string selectedCity: ""

        onOpened: {
            loadProxySettings()
            if (app) app.load_mullvad_servers()
        }

        function loadProxySettings() {
            if (!app) return
            var settingsJson = app.get_proxy_settings()
            if (settingsJson && String(settingsJson).length > 0) {
                var settings = JSON.parse(String(settingsJson))
                if (!settings.enabled) {
                    selectedMode = "disabled"
                } else {
                    selectedMode = settings.mode || "mullvad"
                    selectedCountry = settings.mullvad_country || ""
                    selectedCity = settings.mullvad_city || ""
                    customHostInput.text = settings.custom_host || "127.0.0.1"
                    customPortInput.text = String(settings.custom_port || 1080)
                }
            }
        }

        function saveAndConnect() {
            if (!app) return
            var enabled = selectedMode !== "disabled"
            var mode = selectedMode === "mullvad" ? "mullvad" : "custom"
            var host = customHostInput.text ? customHostInput.text : "127.0.0.1"
            var port = customPortInput.text ? (parseInt(customPortInput.text) || 1080) : 1080
            app.set_proxy_settings(enabled, mode, selectedCountry, selectedCity, "", host, port)
            proxyConfigPopup.close()
        }

        background: Rectangle {
            color: theme.bgPrimary
            radius: theme.radiusLarge
            border.color: theme.border
            border.width: 1
        }

        Overlay.modal: Rectangle {
            color: "#000000dd"
        }

        contentItem: ColumnLayout {
            spacing: 0

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 56
                color: theme.bgSecondary
                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 24
                    anchors.rightMargin: 24
                    Text {
                        text: "\u{1F512} Proxy Configuration"
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 18
                        font.weight: Font.DemiBold
                    }
                    Item { Layout.fillWidth: true }
                }
            }

            Flickable {
                Layout.fillWidth: true
                Layout.fillHeight: true
                contentHeight: proxyContent.implicitHeight
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ScrollBar {
                    policy: ScrollBar.AsNeeded
                    contentItem: Rectangle {
                        implicitWidth: 4
                        radius: 2
                        color: theme.textFaint
                        opacity: parent.active ? 0.6 : 0.0
                    }
                }
                ColumnLayout {
                    id: proxyContent
                    width: parent.width
                    spacing: 20
                    Item { height: 4 }

                    ColumnLayout {
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        Layout.fillWidth: true
                        spacing: 8
                        Text {
                            text: "PROXY MODE"
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 11
                            font.bold: true
                        }
                        RowLayout {
                            spacing: 8
                            Repeater {
                                model: [
                                    { name: "Mullvad", value: "mullvad", icon: "\u{1F6E1}" },
                                    { name: "Custom", value: "custom", icon: "\u{2699}" },
                                    { name: "Disabled", value: "disabled", icon: "\u{1F6AB}" }
                                ]
                                delegate: Rectangle {
                                    Layout.preferredWidth: 150
                                    Layout.preferredHeight: 56
                                    radius: theme.radiusMed
                                    color: proxyConfigPopup.selectedMode === modelData.value ? theme.bgActive : theme.bgSecondary
                                    border.color: proxyConfigPopup.selectedMode === modelData.value ? theme.accent : "transparent"
                                    border.width: 2
                                    ColumnLayout {
                                        anchors.centerIn: parent
                                        spacing: 2
                                        Text {
                                            text: modelData.icon
                                            font.pixelSize: 18
                                            Layout.alignment: Qt.AlignHCenter
                                        }
                                        Text {
                                            text: modelData.name
                                            color: proxyConfigPopup.selectedMode === modelData.value ? theme.accent : theme.textNormal
                                            font.family: fontFamily
                                            font.pixelSize: 12
                                            font.weight: Font.Medium
                                            Layout.alignment: Qt.AlignHCenter
                                        }
                                    }
                                    MouseArea {
                                        anchors.fill: parent
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: proxyConfigPopup.selectedMode = modelData.value
                                    }
                                }
                            }
                        }
                    }

                    Rectangle { Layout.fillWidth: true; Layout.leftMargin: 24; Layout.rightMargin: 24; height: 1; color: theme.separator }

                    ColumnLayout {
                        visible: proxyConfigPopup.selectedMode === "mullvad"
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        Layout.fillWidth: true
                        spacing: 12
                        Text {
                            text: "MULLVAD LOCATION"
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 11
                            font.bold: true
                        }
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 44
                            radius: theme.radiusMed
                            color: theme.bgSecondary
                            border.width: 1
                            border.color: countryCombo.activeFocus ? theme.accent : theme.border
                            ComboBox {
                                id: countryCombo
                                anchors.fill: parent
                                anchors.margins: 1
                                model: proxyConfigPopup.mullvadCountries
                                textRole: "name"
                                currentIndex: {
                                    var list = proxyConfigPopup.mullvadCountries
                                    var sel = proxyConfigPopup.selectedCountry
                                    for (var i = 0; i < list.length; i++) {
                                        if (list[i].code === sel) return i
                                    }
                                    return 0
                                }
                                onActivated: function(index) {
                                    if (index >= 0 && index < proxyConfigPopup.mullvadCountries.length) {
                                        proxyConfigPopup.selectedCountry = proxyConfigPopup.mullvadCountries[index].code
                                        proxyConfigPopup.selectedCity = ""
                                    }
                                }
                                background: Rectangle { color: "transparent" }
                                contentItem: Text {
                                    text: (countryCombo.currentIndex >= 0 && countryCombo.currentIndex < proxyConfigPopup.mullvadCountries.length)
                                          ? proxyConfigPopup.mullvadCountries[countryCombo.currentIndex].name : "Select Country..."
                                    color: theme.textNormal
                                    font.family: fontFamily
                                    font.pixelSize: 13
                                    verticalAlignment: Text.AlignVCenter
                                    leftPadding: 12
                                }
                            }
                        }
                        Rectangle {
                            visible: proxyConfigPopup.selectedCountry !== ""
                            Layout.fillWidth: true
                            Layout.preferredHeight: 44
                            radius: theme.radiusMed
                            color: theme.bgSecondary
                            border.width: 1
                            border.color: cityCombo.activeFocus ? theme.accent : theme.border
                            ComboBox {
                                id: cityCombo
                                anchors.fill: parent
                                anchors.margins: 1
                                model: {
                                    if (proxyConfigPopup.selectedCountry === "") return []
                                    for (var i = 0; i < proxyConfigPopup.mullvadCountries.length; i++) {
                                        if (proxyConfigPopup.mullvadCountries[i].code === proxyConfigPopup.selectedCountry)
                                            return proxyConfigPopup.mullvadCountries[i].cities || []
                                    }
                                    return []
                                }
                                textRole: "name"
                                onActivated: function(index) {
                                    var cities = cityCombo.model
                                    if (index >= 0 && index < cities.length)
                                        proxyConfigPopup.selectedCity = cities[index].code
                                }
                                background: Rectangle { color: "transparent" }
                                contentItem: Text {
                                    text: {
                                        var m = cityCombo.model
                                        if (cityCombo.currentIndex >= 0 && cityCombo.currentIndex < m.length)
                                            return m[cityCombo.currentIndex].name
                                        return "Any City (Auto)"
                                    }
                                    color: theme.textNormal
                                    font.family: fontFamily
                                    font.pixelSize: 13
                                    verticalAlignment: Text.AlignVCenter
                                    leftPadding: 12
                                }
                            }
                        }
                        Text {
                            text: "Select a country and optionally a city. Leaving city blank will automatically select the best server."
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 11
                            wrapMode: Text.WordWrap
                            Layout.fillWidth: true
                        }
                    }

                    ColumnLayout {
                        visible: proxyConfigPopup.selectedMode === "custom"
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        Layout.fillWidth: true
                        spacing: 12
                        Text {
                            text: "CUSTOM SOCKS5 PROXY"
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 11
                            font.bold: true
                        }
                        RowLayout {
                            spacing: 8
                            Layout.fillWidth: true
                            Rectangle {
                                Layout.fillWidth: true
                                Layout.preferredHeight: 44
                                radius: theme.radiusMed
                                color: theme.bgSecondary
                                border.width: 1
                                border.color: theme.border
                                TextField {
                                    id: customHostInput
                                    anchors.fill: parent
                                    anchors.margins: 1
                                    placeholderText: "127.0.0.1"
                                    color: theme.textNormal
                                    font.family: fontFamily
                                    font.pixelSize: 13
                                    leftPadding: 12
                                    background: Rectangle { color: "transparent" }
                                }
                            }
                            Rectangle {
                                Layout.preferredWidth: 100
                                Layout.preferredHeight: 44
                                radius: theme.radiusMed
                                color: theme.bgSecondary
                                border.width: 1
                                border.color: theme.border
                                TextField {
                                    id: customPortInput
                                    anchors.fill: parent
                                    anchors.margins: 1
                                    placeholderText: "1080"
                                    color: theme.textNormal
                                    font.family: fontFamily
                                    font.pixelSize: 13
                                    leftPadding: 12
                                    validator: IntValidator { bottom: 1; top: 65535 }
                                    background: Rectangle { color: "transparent" }
                                }
                            }
                        }
                    }

                    Rectangle {
                        Layout.leftMargin: 24
                        Layout.rightMargin: 24
                        Layout.fillWidth: true
                        Layout.preferredHeight: infoText.implicitHeight + 24
                        radius: theme.radiusMed
                        color: theme.bgSecondary
                        border.color: theme.border
                        border.width: 1
                        Text {
                            id: infoText
                            anchors.fill: parent
                            anchors.margins: 12
                            text: {
                                if (proxyConfigPopup.selectedMode === "mullvad")
                                    return "Mullvad SOCKS5 proxies require an active Mullvad VPN connection. Make sure you are connected to Mullvad VPN before using this option."
                                if (proxyConfigPopup.selectedMode === "custom")
                                    return "Enter your SOCKS5 proxy address and port. All Discord traffic will be routed through this proxy."
                                return "Connecting without a proxy may expose your real IP address."
                            }
                            color: theme.textSecondary
                            font.family: fontFamily
                            font.pixelSize: 12
                            wrapMode: Text.WordWrap
                        }
                    }
                    Item { height: 8 }
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 70
                color: theme.bgSecondary
                RowLayout {
                    anchors.centerIn: parent
                    spacing: 12
                    Rectangle {
                        Layout.preferredWidth: 180
                        Layout.preferredHeight: 44
                        radius: theme.radiusMed
                        color: connectMa.containsMouse ? theme.accentHover : theme.accent
                        Behavior on color { ColorAnimation { duration: theme.animFast } }
                        Text {
                            anchors.centerIn: parent
                            text: "Connect"
                            color: "#ffffff"
                            font.family: fontFamily
                            font.pixelSize: 14
                            font.weight: Font.DemiBold
                        }
                        MouseArea {
                            id: connectMa
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: proxyConfigPopup.saveAndConnect()
                        }
                    }
                }
            }
        }

        Timer {
            interval: 100
            running: proxyConfigPopup.opened
            repeat: true
            onTriggered: {
                if (!app) return
                var serversJson = app.consume_mullvad_servers()
                if (serversJson && String(serversJson).length > 0) {
                    proxyConfigPopup.mullvadCountries = JSON.parse(String(serversJson))
                }
            }
        }
    }

    // ══════════ Profile Popup (self or member) ══════════
    Popup {
        id: profilePopup
        anchors.centerIn: parent
        width: 340
        height: Math.min(profileOuterColumn.implicitHeight, root.height - 60)
        modal: true
        padding: 0
        focus: false
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
        property bool profileLoadPending: false
        property var loadedProfileData: null
        property string loadedProfileRawJson: ""

        // Helpers scoped to this popup
        readonly property bool isSelf: profilePopupTarget === "self"
        readonly property bool isOther: profilePopupTarget && profilePopupTarget !== "self" && profilePopupTarget.userId && profilePopupTarget.userId !== currentUserId
        readonly property var pUser: loadedProfileData && loadedProfileData.user ? loadedProfileData.user : null
        readonly property var pGuild: loadedProfileData && (loadedProfileData.guild_member_profile || loadedProfileData.guild_member) ? (loadedProfileData.guild_member_profile || loadedProfileData.guild_member) : null
        readonly property string pDisplayName: {
            if (isSelf) return (myGuildProfile && myGuildProfile.nick ? myGuildProfile.nick : currentUserName) || ""
            if (pGuild && pGuild.nick) return pGuild.nick
            if (profilePopupTarget && profilePopupTarget.displayName) return profilePopupTarget.displayName
            if (pUser) return pUser.global_name || pUser.username || ""
            return ""
        }
        readonly property string pUsername: {
            if (isSelf) return currentUserName || ""
            if (pUser) return pUser.username || ""
            if (profilePopupTarget) return profilePopupTarget.username || ""
            return ""
        }
        readonly property string pAvatarUrl: {
            if (isSelf) return currentUserAvatar || ""
            if (profilePopupTarget && profilePopupTarget.avatarUrl) return profilePopupTarget.avatarUrl
            if (pUser && pUser.avatar) return "https://cdn.discordapp.com/avatars/" + pUser.id + "/" + pUser.avatar + ".png?size=128"
            return ""
        }
        readonly property string pUserId: {
            if (isSelf) return currentUserId
            if (profilePopupTarget && profilePopupTarget.userId) return profilePopupTarget.userId
            if (pUser) return pUser.id || ""
            return ""
        }
        readonly property string pRoleColor: (profilePopupTarget && profilePopupTarget.roleColor && profilePopupTarget.roleColor.length > 0) ? profilePopupTarget.roleColor : ""
        readonly property string pBannerUrl: {
            var bannerHash = pUser ? pUser.banner : (loadedProfileData ? loadedProfileData.banner : "")
            if (bannerHash && bannerHash.length > 0 && pUserId.length > 0)
                return "https://cdn.discordapp.com/banners/" + pUserId + "/" + bannerHash + ".png?size=480"
            return ""
        }
        readonly property color pBannerColor: {
            var ac = pUser ? pUser.accent_color : (loadedProfileData ? loadedProfileData.accent_color : null)
            if (ac != null && ac !== undefined) return "#" + ("000000" + (ac >>> 0).toString(16)).slice(-6)
            if (pRoleColor.length > 0) return pRoleColor
            return theme.accent
        }
        readonly property bool hasBio: loadedProfileData && loadedProfileData.bio && loadedProfileData.bio.length > 0
        readonly property bool hasNote: loadedProfileData && loadedProfileData.note && loadedProfileData.note.length > 0
        readonly property bool hasRoles: (isSelf && myGuildProfile && myGuildProfile.roles && myGuildProfile.roles.length > 0) || (pGuild && pGuild.roles && pGuild.roles.length > 0)
        readonly property var rolesList: isSelf && myGuildProfile && myGuildProfile.roles ? myGuildProfile.roles : (pGuild && pGuild.roles ? pGuild.roles : [])
        readonly property bool hasConnectedAccounts: loadedProfileData && loadedProfileData.connected_accounts && loadedProfileData.connected_accounts.length > 0
        readonly property bool hasMutualGuilds: loadedProfileData && loadedProfileData.mutual_guilds && loadedProfileData.mutual_guilds.length > 0
        readonly property bool hasMutualFriends: loadedProfileData && loadedProfileData.mutual_friends && loadedProfileData.mutual_friends.length > 0
        readonly property var mutualFriendsList: loadedProfileData && loadedProfileData.mutual_friends ? loadedProfileData.mutual_friends : []
        readonly property bool pIsOwner: !!(loadedProfileData && ((loadedProfileData.guild_member_profile && loadedProfileData.guild_member_profile.is_owner) || loadedProfileData.is_owner))
        readonly property bool hasPermissions: profilePopup.pIsOwner || (profilePopup.pGuild && profilePopup.pGuild.permission_names && profilePopup.pGuild.permission_names.length > 0)
        readonly property var permissionNamesList: profilePopup.pIsOwner ? ["Server Owner"] : (profilePopup.pGuild && profilePopup.pGuild.permission_names ? profilePopup.pGuild.permission_names : [])
        readonly property string profileRelationshipType: {
            if (!profilePopup.pUserId || !relationshipModel.count) return "none"
            for (var i = 0; i < relationshipModel.count; i++)
                if (relationshipModel.get(i).userId === profilePopup.pUserId)
                    return relationshipModel.get(i).relationshipType || "none"
            return "none"
        }
        // True when profile data sections should be shown (self always ready; other users: after load)
        readonly property bool dataReady: isSelf || (!profileLoadPending && loadedProfileData !== null)

        onOpened: {
            if (profilePopupTarget && profilePopupTarget !== "self" && profilePopupTarget.userId && app) {
                profileLoadPending = true
                loadedProfileData = null
                loadedProfileRawJson = ""
                app.fetch_user_profile(profilePopupTarget.userId, currentGuildId)
            }
        }
        onClosed: {
            profilePopupTarget = null
            profileLoadPending = false
            loadedProfileData = null
            loadedProfileRawJson = ""
        }

        enter: Transition { NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: theme.animFast } }
        exit: Transition { NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; duration: theme.animFast } }

        background: Rectangle {
            color: theme.bgSecondary
            radius: theme.radiusLarge
            border.color: theme.border
            border.width: 1
        }

        Overlay.modal: Rectangle { color: "#000000cc" }

        contentItem: Flickable {
            id: profileFlick
            contentWidth: width
            contentHeight: profileOuterColumn.implicitHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

            Column {
                id: profileOuterColumn
                width: profileFlick.width

                // ── Banner ──
                Rectangle {
                    id: profileBanner
                    width: parent.width
                    height: 120
                    radius: theme.radiusLarge
                    color: profilePopup.pBannerColor
                    clip: true

                    // Only round top corners: overlay bottom to hide bottom radius
                    Rectangle {
                        width: parent.width
                        height: theme.radiusLarge
                        anchors.bottom: parent.bottom
                        color: parent.color
                        visible: !bannerImg.visible
                    }

                    Image {
                        id: bannerImg
                        anchors.fill: parent
                        source: profilePopup.pBannerUrl
                        fillMode: Image.PreserveAspectCrop
                        visible: status === Image.Ready
                        smooth: true
                        asynchronous: true
                    }

                    // Bottom-cover for rounded corners when image is showing
                    Rectangle {
                        width: parent.width
                        height: theme.radiusLarge
                        anchors.bottom: parent.bottom
                        color: theme.bgSecondary
                        visible: bannerImg.visible
                    }
                }

                // ── Avatar overlap zone ──
                Item {
                    width: parent.width
                    height: 52 // avatar extends 40px above, we show 52px below banner for the ring bottom + spacing

                    // Avatar with ring
                    Rectangle {
                        id: avatarRing
                        width: 88; height: 88
                        radius: 44
                        color: theme.bgSecondary
                        x: 16
                        y: -40

                        Rectangle {
                            anchors.centerIn: parent
                            width: 80; height: 80; radius: 40
                            color: theme.bgTertiary
                            clip: true

                            Image {
                                id: profilePopupAvatarImg
                                anchors.fill: parent
                                source: profilePopup.pAvatarUrl
                                fillMode: Image.PreserveAspectCrop
                                visible: status === Image.Ready
                                smooth: true
                                asynchronous: true
                            }
                            Text {
                                anchors.centerIn: parent
                                visible: !profilePopupAvatarImg.visible
                                text: (profilePopup.pDisplayName || "?").charAt(0).toUpperCase()
                                color: "#ffffff"
                                font.pixelSize: 28
                                font.bold: true
                            }
                        }

                        // Status dot
                        Rectangle {
                            width: 20; height: 20; radius: 10
                            anchors.right: parent.right
                            anchors.bottom: parent.bottom
                            anchors.rightMargin: 2
                            anchors.bottomMargin: 2
                            color: theme.bgSecondary

                            Rectangle {
                                anchors.centerIn: parent
                                width: 14; height: 14; radius: 7
                                color: getStatusColor((presenceVersion, app ? app.get_user_status(profilePopup.pUserId) : ""))
                            }
                        }
                    }

                    // Badges row (top-right of avatar zone)
                    Row {
                        anchors.right: parent.right
                        anchors.rightMargin: 16
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 4
                        visible: profilePopup.pUser !== null

                        UserBadges {
                            publicFlags: profilePopup.pUser ? (profilePopup.pUser.public_flags || 0) : 0
                            isBot: profilePopup.pUser ? (profilePopup.pUser.bot || false) : false
                            premiumType: profilePopup.pUser ? (profilePopup.pUser.premium_type || 0) : (loadedProfileData ? (loadedProfileData.premium_type || 0) : 0)
                            badgeSize: 18
                        }
                    }
                }

                // ── Card body ──
                Item {
                    width: parent.width
                    implicitHeight: cardBodyRect.implicitHeight

                    Rectangle {
                        id: cardBodyRect
                        x: 12
                        width: parent.width - 24
                        color: theme.bgPrimary
                        radius: theme.radiusMed
                        implicitHeight: cardBody.implicitHeight + 24

                    Column {
                        id: cardBody
                        width: parent.width - 24
                        anchors.horizontalCenter: parent.horizontalCenter
                        anchors.top: parent.top
                        anchors.topMargin: 12
                        spacing: 0

                        // ── Display Name (with server owner crown) ──
                        Row {
                            width: parent.width
                            spacing: 6
                            layoutDirection: Qt.LeftToRight

                            Text {
                                text: profilePopup.pDisplayName
                                color: profilePopup.pRoleColor.length > 0 ? profilePopup.pRoleColor : theme.headerPrimary
                                font.family: fontFamily
                                font.pixelSize: 20
                                font.bold: true
                                elide: Text.ElideRight
                                width: parent.width - crownWrapper.width - (crownWrapper.width > 0 ? 6 : 0)
                            }
                            Item {
                                id: crownWrapper
                                width: profilePopup.dataReady && profilePopup.pIsOwner ? 24 : 0
                                height: 24
                                Text {
                                    id: crownLabel
                                    visible: profilePopup.dataReady && profilePopup.pIsOwner
                                    text: "\uD83D\uDC51"
                                    color: theme.headerPrimary
                                    font.pixelSize: 18
                                    anchors.centerIn: parent
                                }
                            }
                        }

                        // ── Username ──
                        Text {
                            visible: profilePopup.pUsername.length > 0 && profilePopup.pUsername !== profilePopup.pDisplayName
                            text: profilePopup.pUsername
                            color: theme.textSecondary
                            font.family: fontFamily
                            font.pixelSize: 14
                            elide: Text.ElideRight
                            width: parent.width
                            topPadding: 2
                        }

                        // ── Loading indicator (only while fetching, stable size) ──
                        Item {
                            visible: profilePopup.profileLoadPending && !profilePopup.isSelf
                            width: parent.width; height: visible ? 40 : 0
                            BusyIndicator {
                                width: 24; height: 24
                                running: parent.visible
                                anchors.centerIn: parent
                            }
                        }

                        // ═══════════════════════════════════════════
                        // All data sections below — only shown when dataReady
                        // ═══════════════════════════════════════════

                        // ── About Me ──
                        Item { visible: profilePopup.dataReady && profilePopup.hasBio; width: 1; height: 12 }
                        Rectangle {
                            visible: profilePopup.dataReady && profilePopup.hasBio
                            width: parent.width; height: 1; color: theme.borderSubtle
                        }
                        Item { visible: profilePopup.dataReady && profilePopup.hasBio; width: 1; height: 10 }
                        Text {
                            visible: profilePopup.dataReady && profilePopup.hasBio
                            text: "ABOUT ME"
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 12
                            font.weight: Font.Bold
                            font.letterSpacing: 0.5
                            width: parent.width
                        }
                        Item { visible: profilePopup.dataReady && profilePopup.hasBio; width: 1; height: 6 }
                        Text {
                            visible: profilePopup.dataReady && profilePopup.hasBio
                            text: loadedProfileData ? (loadedProfileData.bio || "") : ""
                            color: theme.textNormal
                            font.family: fontFamily
                            font.pixelSize: 13
                            wrapMode: Text.WordWrap
                            width: parent.width
                            lineHeight: 1.25
                        }

                        // ── Member Since ──
                        Item { visible: profilePopup.dataReady && loadedProfileData && loadedProfileData.created_at; width: 1; height: 12 }
                        Rectangle {
                            visible: profilePopup.dataReady && loadedProfileData && loadedProfileData.created_at
                            width: parent.width; height: 1; color: theme.borderSubtle
                        }
                        Item { visible: profilePopup.dataReady && loadedProfileData && loadedProfileData.created_at; width: 1; height: 10 }
                        Text {
                            visible: profilePopup.dataReady && loadedProfileData && loadedProfileData.created_at
                            text: "MEMBER SINCE"
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 12
                            font.weight: Font.Bold
                            font.letterSpacing: 0.5
                            width: parent.width
                        }
                        Item { visible: profilePopup.dataReady && loadedProfileData && loadedProfileData.created_at; width: 1; height: 8 }

                        // Discord join date
                        Row {
                            visible: profilePopup.dataReady && loadedProfileData && loadedProfileData.created_at
                            spacing: 8
                            width: parent.width
                            height: visible ? 18 : 0

                            Rectangle {
                                width: 16; height: 16; radius: 8
                                color: theme.accent
                                anchors.verticalCenter: parent.verticalCenter
                                Text {
                                    anchors.centerIn: parent
                                    text: "D"; color: "#ffffff"
                                    font.pixelSize: 9; font.bold: true
                                }
                            }
                            Text {
                                text: formatProfileDate(loadedProfileData ? loadedProfileData.created_at : "")
                                color: theme.textNormal
                                font.family: fontFamily
                                font.pixelSize: 13
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }

                        Item { visible: profilePopup.dataReady && profilePopup.pGuild && profilePopup.pGuild.joined_at; width: 1; height: 6 }

                        // Server join date
                        Row {
                            visible: profilePopup.dataReady && profilePopup.pGuild && profilePopup.pGuild.joined_at
                            spacing: 8
                            width: parent.width
                            height: visible ? 18 : 0

                            Rectangle {
                                width: 16; height: 16; radius: 8
                                color: theme.bgTertiary
                                anchors.verticalCenter: parent.verticalCenter
                                Text {
                                    anchors.centerIn: parent
                                    text: "S"; color: theme.textMuted
                                    font.pixelSize: 9; font.bold: true
                                }
                            }
                            Text {
                                text: formatProfileDate(profilePopup.pGuild ? profilePopup.pGuild.joined_at : "")
                                color: theme.textNormal
                                font.family: fontFamily
                                font.pixelSize: 13
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }

                        // ── Roles ──
                        Item { visible: profilePopup.dataReady && profilePopup.hasRoles; width: 1; height: 12 }
                        Rectangle {
                            visible: profilePopup.dataReady && profilePopup.hasRoles
                            width: parent.width; height: 1; color: theme.borderSubtle
                        }
                        Item { visible: profilePopup.dataReady && profilePopup.hasRoles; width: 1; height: 10 }
                        Text {
                            visible: profilePopup.dataReady && profilePopup.hasRoles
                            text: "ROLES"
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 12
                            font.weight: Font.Bold
                            font.letterSpacing: 0.5
                            width: parent.width
                        }
                        Item { visible: profilePopup.dataReady && profilePopup.hasRoles; width: 1; height: 6 }
                        Flow {
                            visible: profilePopup.dataReady && profilePopup.hasRoles
                            width: parent.width
                            spacing: 4
                            Repeater {
                                model: profilePopup.rolesList
                                delegate: Rectangle {
                                    property string rName: modelData ? (modelData.name || String(modelData)) : ""
                                    property string rColor: modelData && modelData.color ? modelData.color : ""
                                    width: rpLabel.implicitWidth + roleDot.width + 12
                                    height: 24
                                    radius: 4
                                    color: theme.bgSecondary

                                    Row {
                                        anchors.centerIn: parent
                                        spacing: 4

                                        Rectangle {
                                            id: roleDot
                                            width: 10; height: 10; radius: 5
                                            color: rColor.length > 0 ? rColor : theme.textFaint
                                            anchors.verticalCenter: parent.verticalCenter
                                        }
                                        Text {
                                            id: rpLabel
                                            text: rName
                                            color: theme.textNormal
                                            font.family: fontFamily
                                            font.pixelSize: 12
                                            anchors.verticalCenter: parent.verticalCenter
                                        }
                                    }
                                }
                            }
                        }

                        // ── Permissions (guild context) ──
                        Item { visible: profilePopup.dataReady && profilePopup.hasPermissions; width: 1; height: 12 }
                        Rectangle {
                            visible: profilePopup.dataReady && profilePopup.hasPermissions
                            width: parent.width; height: 1; color: theme.borderSubtle
                        }
                        Item { visible: profilePopup.dataReady && profilePopup.hasPermissions; width: 1; height: 10 }
                        Text {
                            visible: profilePopup.dataReady && profilePopup.hasPermissions
                            text: "PERMISSIONS"
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 12
                            font.weight: Font.Bold
                            font.letterSpacing: 0.5
                            width: parent.width
                        }
                        Item { visible: profilePopup.dataReady && profilePopup.hasPermissions; width: 1; height: 6 }
                        Flow {
                            visible: profilePopup.dataReady && profilePopup.hasPermissions
                            width: parent.width
                            spacing: 4
                            Repeater {
                                model: profilePopup.permissionNamesList
                                delegate: Rectangle {
                                    width: permLabel.implicitWidth + 12
                                    height: 22
                                    radius: 4
                                    color: theme.bgSecondary
                                    Text {
                                        id: permLabel
                                        anchors.centerIn: parent
                                        text: modelData || ""
                                        color: theme.textNormal
                                        font.family: fontFamily
                                        font.pixelSize: 11
                                    }
                                }
                            }
                        }

                        // ── Connected Accounts ──
                        Item { visible: profilePopup.dataReady && profilePopup.hasConnectedAccounts; width: 1; height: 12 }
                        Rectangle {
                            visible: profilePopup.dataReady && profilePopup.hasConnectedAccounts
                            width: parent.width; height: 1; color: theme.borderSubtle
                        }
                        Item { visible: profilePopup.dataReady && profilePopup.hasConnectedAccounts; width: 1; height: 10 }
                        Text {
                            visible: profilePopup.dataReady && profilePopup.hasConnectedAccounts
                            text: "CONNECTIONS"
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 12
                            font.weight: Font.Bold
                            font.letterSpacing: 0.5
                            width: parent.width
                        }
                        Item { visible: profilePopup.dataReady && profilePopup.hasConnectedAccounts; width: 1; height: 6 }
                        Flow {
                            visible: profilePopup.dataReady && profilePopup.hasConnectedAccounts
                            width: parent.width
                            spacing: 6
                            Repeater {
                                model: loadedProfileData && loadedProfileData.connected_accounts ? loadedProfileData.connected_accounts : []
                                delegate: Rectangle {
                                    width: connRow.implicitWidth + 16
                                    height: 28
                                    radius: 4
                                    color: theme.bgSecondary

                                    Row {
                                        id: connRow
                                        anchors.centerIn: parent
                                        spacing: 6

                                        Rectangle {
                                            width: connTypeLabel.implicitWidth + 8
                                            height: 16; radius: 3
                                            color: theme.bgTertiary
                                            anchors.verticalCenter: parent.verticalCenter

                                            Text {
                                                id: connTypeLabel
                                                anchors.centerIn: parent
                                                text: {
                                                    var t = (modelData.type || "").toLowerCase()
                                                    return t.charAt(0).toUpperCase() + t.slice(1)
                                                }
                                                color: theme.textMuted
                                                font.family: fontFamily
                                                font.pixelSize: 9
                                                font.weight: Font.Bold
                                            }
                                        }

                                        Text {
                                            text: (modelData.name || modelData.type || "Unknown")
                                            color: theme.textNormal
                                            font.family: fontFamily
                                            font.pixelSize: 12
                                            anchors.verticalCenter: parent.verticalCenter
                                        }
                                    }
                                }
                            }
                        }

                        // ── Mutual Servers ──
                        Item { visible: profilePopup.dataReady && profilePopup.hasMutualGuilds; width: 1; height: 12 }
                        Rectangle {
                            visible: profilePopup.dataReady && profilePopup.hasMutualGuilds
                            width: parent.width; height: 1; color: theme.borderSubtle
                        }
                        Item { visible: profilePopup.dataReady && profilePopup.hasMutualGuilds; width: 1; height: 10 }
                        Text {
                            visible: profilePopup.dataReady && profilePopup.hasMutualGuilds
                            text: "MUTUAL SERVERS — " + (loadedProfileData && loadedProfileData.mutual_guilds ? loadedProfileData.mutual_guilds.length : 0)
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 12
                            font.weight: Font.Bold
                            font.letterSpacing: 0.5
                            width: parent.width
                        }

                        // ── Mutual Friends ──
                        Item { visible: profilePopup.dataReady && profilePopup.hasMutualFriends; width: 1; height: 12 }
                        Rectangle {
                            visible: profilePopup.dataReady && profilePopup.hasMutualFriends
                            width: parent.width; height: 1; color: theme.borderSubtle
                        }
                        Item { visible: profilePopup.dataReady && profilePopup.hasMutualFriends; width: 1; height: 10 }
                        Text {
                            visible: profilePopup.dataReady && profilePopup.hasMutualFriends
                            text: "MUTUAL FRIENDS — " + (loadedProfileData && loadedProfileData.mutual_friends ? loadedProfileData.mutual_friends.length : 0)
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 12
                            font.weight: Font.Bold
                            font.letterSpacing: 0.5
                            width: parent.width
                        }
                        Item { visible: profilePopup.dataReady && profilePopup.hasMutualFriends; width: 1; height: 6 }
                        Flow {
                            visible: profilePopup.dataReady && profilePopup.hasMutualFriends
                            width: parent.width
                            spacing: 6
                            Repeater {
                                model: profilePopup.mutualFriendsList
                                delegate: Rectangle {
                                    width: mfText.implicitWidth + 24 + 6 + 16
                                    height: 28
                                    radius: 4
                                    color: theme.bgSecondary
                                    Row {
                                        anchors.centerIn: parent
                                        spacing: 6
                                        Item {
                                            width: 24; height: 24
                                            anchors.verticalCenter: parent.verticalCenter
                                            DAvatar {
                                                anchors.fill: parent
                                                size: 24
                                                imageUrl: modelData.avatar ? ("https://cdn.discordapp.com/avatars/" + (modelData.id || "") + "/" + modelData.avatar + ".png?size=48") : ""
                                                fallbackText: (modelData.global_name || modelData.username || "?").charAt(0).toUpperCase()
                                            }
                                        }
                                        Text {
                                            id: mfText
                                            text: modelData.global_name || modelData.username || "Unknown"
                                            color: theme.textNormal
                                            font.family: fontFamily
                                            font.pixelSize: 12
                                            anchors.verticalCenter: parent.verticalCenter
                                        }
                                    }
                                }
                            }
                        }

                        // ── Note ──
                        Item { visible: profilePopup.dataReady && profilePopup.hasNote; width: 1; height: 12 }
                        Rectangle {
                            visible: profilePopup.dataReady && profilePopup.hasNote
                            width: parent.width; height: 1; color: theme.borderSubtle
                        }
                        Item { visible: profilePopup.dataReady && profilePopup.hasNote; width: 1; height: 10 }
                        Text {
                            visible: profilePopup.dataReady && profilePopup.hasNote
                            text: "NOTE"
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 12
                            font.weight: Font.Bold
                            font.letterSpacing: 0.5
                            width: parent.width
                        }
                        Item { visible: profilePopup.dataReady && profilePopup.hasNote; width: 1; height: 4 }
                        Text {
                            visible: profilePopup.dataReady && profilePopup.hasNote
                            text: loadedProfileData ? (loadedProfileData.note || "") : ""
                            color: theme.textNormal
                            font.family: fontFamily
                            font.pixelSize: 13
                            wrapMode: Text.WordWrap
                            width: parent.width
                        }

                        // ── Action buttons (relationship-aware) ──
                        Item { visible: profilePopup.dataReady && profilePopup.isOther; width: 1; height: 16 }
                        Column {
                            visible: profilePopup.dataReady && profilePopup.isOther
                            width: parent.width
                            spacing: 8

                            Row {
                                width: parent.width - 0
                                spacing: 8
                                // Message (when not blocked)
                                Rectangle {
                                    visible: profilePopup.profileRelationshipType !== "blocked"
                                    width: visible ? (parent.width - 8) / 2 : 0
                                    height: 36
                                    radius: theme.radiusSmall
                                    color: profileMsgMa.containsMouse ? theme.accentHover : theme.accent
                                    Behavior on color { ColorAnimation { duration: theme.animFast } }
                                    Text {
                                        anchors.centerIn: parent
                                        text: "Message"
                                        color: "#ffffff"
                                        font.family: fontFamily
                                        font.pixelSize: 13
                                        font.weight: Font.Medium
                                    }
                                    MouseArea {
                                        id: profileMsgMa
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            if (app && profilePopup.pUserId) app.open_dm(profilePopup.pUserId)
                                            profilePopup.close()
                                        }
                                    }
                                }
                                // Add Friend (when none)
                                Rectangle {
                                    visible: profilePopup.profileRelationshipType === "none"
                                    width: visible ? (parent.width - 8) / 2 : 0
                                    height: 36
                                    radius: theme.radiusSmall
                                    color: addFriendMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                    Behavior on color { ColorAnimation { duration: theme.animFast } }
                                    Text {
                                        anchors.centerIn: parent
                                        text: "Add Friend"
                                        color: theme.positive
                                        font.family: fontFamily
                                        font.pixelSize: 13
                                        font.weight: Font.Medium
                                    }
                                    MouseArea {
                                        id: addFriendMa
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            if (app && profilePopup.pUsername.length > 0)
                                                app.send_friend_request(profilePopup.pUsername)
                                            profilePopup.close()
                                        }
                                    }
                                }
                                // Pending (Outgoing) — Cancel
                                Rectangle {
                                    visible: profilePopup.profileRelationshipType === "outgoing"
                                    width: visible ? (parent.width - 8) / 2 : 0
                                    height: 36
                                    radius: theme.radiusSmall
                                    color: cancelOutgoingMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                    Behavior on color { ColorAnimation { duration: theme.animFast } }
                                    Text {
                                        anchors.centerIn: parent
                                        text: "Cancel Request"
                                        color: theme.textMuted
                                        font.family: fontFamily
                                        font.pixelSize: 13
                                        font.weight: Font.Medium
                                    }
                                    MouseArea {
                                        id: cancelOutgoingMa
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            if (app && profilePopup.pUserId) app.remove_relationship(profilePopup.pUserId)
                                            profilePopup.close()
                                        }
                                    }
                                }
                                // Incoming — Accept
                                Rectangle {
                                    visible: profilePopup.profileRelationshipType === "incoming"
                                    width: visible ? (parent.width - 8) / 2 : 0
                                    height: 36
                                    radius: theme.radiusSmall
                                    color: acceptIncomingMa.containsMouse ? theme.accentHover : theme.accent
                                    Behavior on color { ColorAnimation { duration: theme.animFast } }
                                    Text {
                                        anchors.centerIn: parent
                                        text: "Accept"
                                        color: "#ffffff"
                                        font.family: fontFamily
                                        font.pixelSize: 13
                                        font.weight: Font.Medium
                                    }
                                    MouseArea {
                                        id: acceptIncomingMa
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            if (app && profilePopup.pUserId) app.accept_friend_request(profilePopup.pUserId)
                                            profilePopup.close()
                                        }
                                    }
                                }
                                // Friend — Remove
                                Rectangle {
                                    visible: profilePopup.profileRelationshipType === "friend"
                                    width: visible ? (parent.width - 8) / 2 : 0
                                    height: 36
                                    radius: theme.radiusSmall
                                    color: removeFriendMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                    Behavior on color { ColorAnimation { duration: theme.animFast } }
                                    Text {
                                        anchors.centerIn: parent
                                        text: "Remove"
                                        color: theme.textMuted
                                        font.family: fontFamily
                                        font.pixelSize: 13
                                        font.weight: Font.Medium
                                    }
                                    MouseArea {
                                        id: removeFriendMa
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            if (app && profilePopup.pUserId) app.remove_relationship(profilePopup.pUserId)
                                            profilePopup.close()
                                        }
                                    }
                                }
                                // Incoming — Reject (same row as Accept)
                                Rectangle {
                                    visible: profilePopup.profileRelationshipType === "incoming"
                                    width: visible ? (parent.width - 8) / 2 : 0
                                    height: 36
                                    radius: theme.radiusSmall
                                    color: rejectIncomingMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                    Behavior on color { ColorAnimation { duration: theme.animFast } }
                                    Text {
                                        anchors.centerIn: parent
                                        text: "Reject"
                                        color: theme.textMuted
                                        font.family: fontFamily
                                        font.pixelSize: 13
                                        font.weight: Font.Medium
                                    }
                                    MouseArea {
                                        id: rejectIncomingMa
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            if (app && profilePopup.pUserId) app.remove_relationship(profilePopup.pUserId)
                                            profilePopup.close()
                                        }
                                    }
                                }
                            }

                            Row {
                                width: parent.width - 0
                                spacing: 8
                                visible: profilePopup.profileRelationshipType === "blocked" || profilePopup.profileRelationshipType !== "blocked"
                                // Block (when not blocked)
                                Rectangle {
                                    visible: profilePopup.profileRelationshipType !== "blocked"
                                    width: visible ? (parent.width - 8) / 2 : 0
                                    height: 36
                                    radius: theme.radiusSmall
                                    color: blockUserMa.containsMouse ? theme.bgHover : theme.danger
                                    Behavior on color { ColorAnimation { duration: theme.animFast } }
                                    Text {
                                        anchors.centerIn: parent
                                        text: "Block"
                                        color: "#ffffff"
                                        font.family: fontFamily
                                        font.pixelSize: 13
                                        font.weight: Font.Medium
                                    }
                                    MouseArea {
                                        id: blockUserMa
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            if (app && profilePopup.pUserId) app.block_user(profilePopup.pUserId)
                                            profilePopup.close()
                                        }
                                    }
                                }
                                // Unblock (when blocked)
                                Rectangle {
                                    visible: profilePopup.profileRelationshipType === "blocked"
                                    width: visible ? (parent.width - 8) / 2 : 0
                                    height: 36
                                    radius: theme.radiusSmall
                                    color: unblockUserMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                    Behavior on color { ColorAnimation { duration: theme.animFast } }
                                    Text {
                                        anchors.centerIn: parent
                                        text: "Unblock"
                                        color: theme.textMuted
                                        font.family: fontFamily
                                        font.pixelSize: 13
                                        font.weight: Font.Medium
                                    }
                                    MouseArea {
                                        id: unblockUserMa
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            if (app && profilePopup.pUserId) app.remove_relationship(profilePopup.pUserId)
                                            profilePopup.close()
                                        }
                                    }
                                }
                            }
                        }

                        // ── Developer ──
                        Item { visible: profilePopup.dataReady; width: 1; height: 12 }
                        Rectangle { visible: profilePopup.dataReady; width: parent.width; height: 1; color: theme.borderSubtle }
                        Item { visible: profilePopup.dataReady; width: 1; height: 10 }
                        Text {
                            visible: profilePopup.dataReady
                            text: "DEVELOPER"
                            color: theme.textFaint
                            font.family: fontFamily
                            font.pixelSize: 10
                            font.weight: Font.Bold
                            font.letterSpacing: 0.5
                        }
                        Item { visible: profilePopup.dataReady; width: 1; height: 6 }
                        Row {
                            visible: profilePopup.dataReady
                            width: parent.width
                            spacing: 8

                            Text {
                                text: "ID: " + profilePopup.pUserId
                                color: theme.textMuted
                                font.family: fontFamily
                                font.pixelSize: 11
                                font.letterSpacing: 0.3
                                width: parent.width - copyIdBtn.width - 8
                                elide: Text.ElideMiddle
                                anchors.verticalCenter: parent.verticalCenter
                            }
                            Rectangle {
                                id: copyIdBtn
                                width: 50; height: 22
                                radius: 4
                                color: copyIdMa.containsMouse ? theme.bgHover : theme.bgTertiary
                                Text { anchors.centerIn: parent; text: "Copy"; color: theme.textMuted; font.pixelSize: 10; font.family: fontFamily }
                                MouseArea {
                                    id: copyIdMa
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        if (profilePopup.pUserId && app) app.copy_to_clipboard(profilePopup.pUserId)
                                    }
                                }
                            }
                        }
                        Item { visible: profilePopup.dataReady && loadedProfileRawJson.length > 0; width: 1; height: 4 }
                        Rectangle {
                            visible: profilePopup.dataReady && loadedProfileRawJson.length > 0
                            width: parent.width
                            height: 26
                            radius: theme.radiusSmall
                            color: copyRawMa.containsMouse ? theme.bgHover : theme.bgTertiary
                            Text {
                                anchors.centerIn: parent
                                text: "Copy Raw JSON"
                                color: theme.textMuted
                                font.family: fontFamily
                                font.pixelSize: 11
                            }
                            MouseArea {
                                id: copyRawMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    if (loadedProfileRawJson.length > 0 && app) app.copy_to_clipboard(loadedProfileRawJson)
                                }
                            }
                        }

                        // Bottom padding
                        Item { width: 1; height: 4 }
                    }
                    } // close cardBodyRect Rectangle
                } // close wrapping Item

                // Bottom spacing after card
                Item { width: 1; height: 12 }
            }
        }

        Timer {
            interval: 200
            running: profilePopup.opened && profilePopup.profileLoadPending && app
            repeat: true
            onTriggered: {
                if (!app || !profilePopup.profileLoadPending) return
                var json = app.consume_user_profile()
                if (json && String(json).length > 0) {
                    try {
                        profilePopup.loadedProfileData = JSON.parse(String(json))
                        profilePopup.loadedProfileRawJson = app.consume_user_profile_raw() || ""
                        profilePopup.profileLoadPending = false
                    } catch (e) {}
                }
            }
        }
    }

    // ══════════ Plugin Modal Popup ══════════
    Popup {
        id: pluginModalPopup
        anchors.centerIn: parent
        width: 360
        modal: true
        padding: 20
        property string _pluginId: ""
        property var _modalDef: null
        property var _fieldValues: ({})

        function show(pluginId, modalDef) {
            _pluginId = pluginId
            _modalDef = modalDef
            _fieldValues = {}
            if (modalDef && modalDef.fields) {
                for (var i = 0; i < modalDef.fields.length; i++) {
                    _fieldValues[modalDef.fields[i].key] = ""
                }
            }
            open()
        }

        background: Rectangle {
            color: theme.bgPrimary
            radius: theme.radiusLarge
            border.color: theme.border
            border.width: 1
        }

        contentItem: ColumnLayout {
            spacing: 16
            Text {
                text: pluginModalPopup._modalDef ? pluginModalPopup._modalDef.title : ""
                color: theme.textNormal
                font.family: fontFamily
                font.pixelSize: 16
                font.bold: true
                Layout.fillWidth: true
            }
            Repeater {
                id: pluginModalFieldsRepeater
                model: pluginModalPopup._modalDef && pluginModalPopup._modalDef.fields ? pluginModalPopup._modalDef.fields : []
                delegate: ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4
                    property var fieldDef: modelData
                    Text {
                        text: fieldDef.label + (fieldDef.required ? " *" : "")
                        color: theme.textMuted
                        font.family: fontFamily
                        font.pixelSize: 12
                    }
                    TextField {
                        Layout.fillWidth: true
                        placeholderText: fieldDef.placeholder || ""
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 14
                        background: Rectangle {
                            color: theme.inputBg
                            radius: theme.radiusSmall
                            border.color: theme.border
                            border.width: 1
                        }
                        onTextChanged: {
                            var v = {}
                            for (var k in pluginModalPopup._fieldValues) v[k] = pluginModalPopup._fieldValues[k]
                            v[fieldDef.key] = text
                            pluginModalPopup._fieldValues = v
                        }
                    }
                }
            }
            RowLayout {
                Layout.fillWidth: true
                Layout.topMargin: 8
                spacing: 8
                Item { Layout.fillWidth: true }
                Button {
                    text: "Cancel"
                    font.family: fontFamily
                    onClicked: pluginModalPopup.close()
                    background: Rectangle {
                        color: theme.bgHover
                        radius: theme.radiusSmall
                    }
                    contentItem: Text {
                        text: parent.text
                        color: theme.textNormal
                        font: parent.font
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
                Button {
                    text: "Submit"
                    font.family: fontFamily
                    onClicked: {
                        if (app && pluginModalPopup._pluginId && pluginModalPopup._modalDef) {
                            app.plugin_modal_submitted(pluginModalPopup._pluginId, pluginModalPopup._modalDef.id, JSON.stringify(pluginModalPopup._fieldValues))
                        }
                        pluginModalPopup.close()
                    }
                    background: Rectangle {
                        color: theme.accent
                        radius: theme.radiusSmall
                    }
                    contentItem: Text {
                        text: parent.text
                        color: "white"
                        font: parent.font
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
        }
    }

    // ══════════ Settings Popup ══════════
    Popup {
        id: settingsPopup
        anchors.centerIn: parent
        width: 480
        height: Math.min(600, root.height - 80)
        modal: true
        padding: 0
        focus: false

        onOpened: {
            if (app) {
                var j = app.get_plugin_enabled_states()
                if (j && String(j).length > 2) {
                    try { pluginEnabledStates = JSON.parse(String(j)) } catch(e) { }
                }
                var pl = app.get_plugin_list()
                if (pl && String(pl).length > 2) {
                    try { pluginList = JSON.parse(String(pl)) } catch(e) { pluginList = [] }
                }
                pluginUpdateStatus = ""
                deletedMessageStyle = app.get_deleted_message_style()
            }
        }

        enter: Transition {
            NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: theme.animFast }
        }
        exit: Transition {
            NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; duration: theme.animFast }
        }

        background: Rectangle {
            color: theme.bgPrimary
            radius: theme.radiusLarge
            border.color: theme.border
            border.width: 1
        }

        Overlay.modal: Rectangle {
            color: "#000000cc"
        }

        contentItem: ColumnLayout {
            spacing: 16

            // Inner margins via anchors on contentItem
            Item {
                Layout.fillWidth: true
                Layout.preferredHeight: 1
            }

            // Header
            RowLayout {
                Layout.fillWidth: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24

                Text {
                    text: "Settings"
                    color: theme.textNormal
                    font.family: fontFamily
                    font.pixelSize: 16
                    font.weight: Font.Medium
                }
                Item { Layout.fillWidth: true }
                Rectangle {
                    width: 24; height: 24; radius: 12
                    color: closeSettingsMa.containsMouse ? theme.bgHover : "transparent"

                    Text {
                        anchors.centerIn: parent
                        text: "\u{2715}"
                        color: theme.textFaint
                        font.pixelSize: 12
                    }
                    MouseArea {
                        id: closeSettingsMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: settingsPopup.close()
                    }
                }
            }

            Rectangle { Layout.fillWidth: true; Layout.leftMargin: 24; Layout.rightMargin: 24; height: 1; color: theme.separator }

            // Toggle settings
            Text {
                text: "FEATURES"
                color: theme.textMuted
                font.family: fontFamily
                font.pixelSize: 11
                font.bold: true
                Layout.leftMargin: 24
            }

            Flickable {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                contentHeight: settingsColumn.implicitHeight
                clip: true
                boundsBehavior: Flickable.StopAtBounds

                ScrollBar.vertical: ScrollBar {
                    policy: ScrollBar.AsNeeded
                    contentItem: Rectangle {
                        implicitWidth: 4
                        radius: 2
                        color: theme.textFaint
                        opacity: parent.active ? 0.6 : 0.0
                        Behavior on opacity { NumberAnimation { duration: theme.animSlow } }
                    }
                    background: Item {}
                }

                Column {
                    id: settingsColumn
                    width: parent.width
                    spacing: 2

                    SettingToggle { width: settingsColumn.width; label: "Block Telemetry"; checked: true }
                    SettingToggle { width: settingsColumn.width; label: "Request Jitter"; checked: true }
                    SettingToggle { width: settingsColumn.width; label: "Safe Link Previews"; checked: true }
                    SettingToggle { 
                        width: settingsColumn.width
                        label: "Typing in Channel List"
                        checked: showTypingInChannelList
                        onToggled: showTypingInChannelList = !showTypingInChannelList
                    }
                    Text {
                        width: settingsColumn.width
                        text: "PLUGINS"
                        color: theme.textMuted
                        font.family: fontFamily
                        font.pixelSize: 11
                        font.bold: true
                        topPadding: 12
                        bottomPadding: 4
                        visible: pluginList.length > 0
                    }
                    Row {
                        width: settingsColumn.width
                        spacing: 8
                        visible: pluginList.length > 0
                        Rectangle {
                            width: refreshPluginsLabel.implicitWidth + 16
                            height: 28
                            radius: theme.radiusSmall
                            color: refreshPluginsMa.containsMouse ? theme.bgHover : "transparent"
                            Text {
                                id: refreshPluginsLabel
                                anchors.centerIn: parent
                                text: "Refresh from disk"
                                color: theme.textSecondary
                                font.family: fontFamily
                                font.pixelSize: 11
                            }
                            MouseArea {
                                id: refreshPluginsMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: { if (app) app.refresh_plugins() }
                            }
                        }
                        Rectangle {
                            width: checkUpdatesLabel.implicitWidth + 16
                            height: 28
                            radius: theme.radiusSmall
                            color: checkUpdatesMa.containsMouse ? theme.bgHover : "transparent"
                            Text {
                                id: checkUpdatesLabel
                                anchors.centerIn: parent
                                text: "Check for updates"
                                color: theme.textSecondary
                                font.family: fontFamily
                                font.pixelSize: 11
                            }
                            MouseArea {
                                id: checkUpdatesMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: { if (app) app.check_plugin_updates() }
                            }
                        }
                    }
                    Text {
                        width: settingsColumn.width
                        text: pluginUpdateStatus
                        color: theme.textFaint
                        font.family: fontFamily
                        font.pixelSize: 10
                        wrapMode: Text.WordWrap
                        visible: pluginUpdateStatus.length > 0
                    }
                    Repeater {
                        model: pluginList
                        delegate: SettingToggle {
                            width: settingsColumn.width
                            label: modelData.name || modelData.id
                            checked: pluginEnabledStates[modelData.id] || false
                            onToggled: {
                                var pid = modelData.id
                                var v = !(pluginEnabledStates[pid] || false)
                                if (app) app.set_plugin_enabled(pid, v)
                                var c = {}; for (var k in pluginEnabledStates) c[k] = pluginEnabledStates[k]; c[pid] = v; pluginEnabledStates = c
                            }
                        }
                    }
                    // Message Logger: deleted message display style (when plugin enabled)
                    Row {
                        width: settingsColumn.width
                        spacing: 12
                        visible: pluginEnabledStates["message-logger"] || false
                        topPadding: 8
                        bottomPadding: 4
                        Text {
                            width: 140
                            anchors.verticalCenter: parent.verticalCenter
                            text: "Deleted message style"
                            color: theme.textSecondary
                            font.family: fontFamily
                            font.pixelSize: 13
                        }
                        Row {
                            spacing: 4
                            Repeater {
                                model: ["strikethrough", "faded", "deleted"]
                                delegate: Rectangle {
                                    width: delStyleLabel.implicitWidth + 16
                                    height: 28
                                    radius: theme.radiusSmall
                                    color: (root.deletedMessageStyle === modelData) ? theme.accent : (delStyleMa.containsMouse ? theme.bgHover : "transparent")
                                    border.width: root.deletedMessageStyle === modelData ? 1 : 0
                                    border.color: theme.accent
                                    Text {
                                        id: delStyleLabel
                                        anchors.centerIn: parent
                                        text: modelData.charAt(0).toUpperCase() + modelData.slice(1)
                                        color: root.deletedMessageStyle === modelData ? "#ffffff" : theme.textSecondary
                                        font.family: fontFamily
                                        font.pixelSize: 12
                                    }
                                    MouseArea {
                                        id: delStyleMa
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            if (app) {
                                                app.set_deleted_message_style(modelData)
                                                root.deletedMessageStyle = modelData
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Rectangle { Layout.fillWidth: true; Layout.leftMargin: 24; Layout.rightMargin: 24; height: 1; color: theme.separator }

            // Footer actions
            RowLayout {
                Layout.leftMargin: 24
                Layout.rightMargin: 24
                Layout.bottomMargin: 8
                spacing: 10
                Item { Layout.fillWidth: true }

                // Logout button
                Rectangle {
                    width: logoutLabel.implicitWidth + 24
                    height: 34
                    radius: theme.radiusMed
                    color: logoutMa.containsMouse ? "#f23f4320" : "transparent"
                    border.color: theme.danger
                    border.width: 1
                    Behavior on color { ColorAnimation { duration: theme.animFast } }

                    Text {
                        id: logoutLabel
                        anchors.centerIn: parent
                        text: "Log Out"
                        color: theme.danger
                        font.family: fontFamily
                        font.pixelSize: 12
                        font.bold: true
                    }
                    MouseArea {
                        id: logoutMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (app) app.logout()
                            settingsPopup.close()
                        }
                    }
                }

                // Done button
                Rectangle {
                    width: doneLabel.implicitWidth + 24
                    height: 34
                    radius: theme.radiusMed
                    color: doneMa.containsMouse ? theme.accentHover : theme.accent
                    Behavior on color { ColorAnimation { duration: theme.animFast } }

                    Text {
                        id: doneLabel
                        anchors.centerIn: parent
                        text: "Done"
                        color: "#ffffff"
                        font.family: fontFamily
                        font.pixelSize: 12
                        font.bold: true
                    }
                    MouseArea {
                        id: doneMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: settingsPopup.close()
                    }
                }
            }

            Item {
                Layout.fillWidth: true
                Layout.preferredHeight: 1
            }
        }
    }

    // ══════════ Functions ══════════
    function doLogin() {
        if (!app) return
        if (app.login_mode === "token") {
            if (tokenInput.text.trim().length > 0)
                app.login(tokenInput.text.trim())
        } else {
            if (emailInput.text.trim().length > 0 && passwordInput.text.trim().length > 0)
                app.login_credentials(emailInput.text.trim(), passwordInput.text.trim())
        }
    }

    function doMfaSubmit() {
        if (app && mfaInput.text.trim().length > 0) {
            app.submit_mfa_code(mfaInput.text.trim())
            mfaInput.text = ""
        }
    }

    function sendMessage() {
        if (messageInput.text.trim().length > 0 && currentChannelId) {
            if (app) {
                if (replyToMessageId || silentMode) {
                    app.send_message_ex(currentChannelId, messageInput.text.trim(), silentMode, replyToMessageId)
                } else {
                    app.send_message(currentChannelId, messageInput.text.trim())
                }
            }
            messageInput.text = ""
            clearReply()
        }
    }

    function clearReply() {
        replyToMessageId = ""
        replyToAuthor = ""
        replyToAuthorColor = ""
        replyToContent = ""
    }

    function openMessageContextMenu(messageId, channelId, authorName, authorId, content, authorRoleColor, authorAvatarUrl, localX, localY) {
        msgContextMenu.targetMessageId = messageId
        msgContextMenu.targetChannelId = channelId
        msgContextMenu.targetAuthorName = authorName
        msgContextMenu.targetAuthorId = authorId
        msgContextMenu.targetContent = content
        msgContextMenu.targetAuthorRoleColor = authorRoleColor || ""
        msgContextMenu.targetAuthorAvatarUrl = authorAvatarUrl || ""
        var x = Number(localX)
        var y = Number(localY)
        msgContextMenu.x = (isFinite(x) ? x : 0)
        msgContextMenu.y = (isFinite(y) ? y : 0)
        msgContextMenu.open()
    }

    function openServerContextMenu(localX, localY) {
        var x = Number(localX)
        var y = Number(localY)
        serverContextMenu.x = (isFinite(x) ? x : 0)
        serverContextMenu.y = (isFinite(y) ? y : 0)
        serverContextMenu.open()
    }

    function startReply(messageId, authorName, content, authorRoleColor) {
        replyToMessageId = messageId
        replyToAuthor = authorName
        replyToAuthorColor = authorRoleColor || ""
        replyToContent = content.length > 80 ? content.substring(0, 80) + "..." : content
        messageInput.forceActiveFocus()
    }

    function typingSeparator(idx, total) {
        if (total === 1) return idx === 0 ? " is typing..." : ""
        if (total === 2) return idx === 0 ? " and " : " are typing..."
        if (idx === 0) return ", "
        if (idx === 1) return ", and " + (total - 2) + " others are typing..."
        return ""
    }

    function formatProfileDate(isoString) {
        if (!isoString) return ""
        // Strip fractional seconds (Qt JS engine can't parse e.g. .420000+00:00)
        var clean = String(isoString).replace(/\.\d+/, "")
        var months = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"]
        var d = new Date(clean)
        if (isNaN(d.getTime())) return String(isoString).substring(0, 10) // fallback: "2020-11-16"
        return months[d.getMonth()] + " " + d.getDate() + ", " + d.getFullYear()
    }

    function openUserProfile(userId, displayName, username, avatarUrl, roleColor) {
        if (!userId) return
        profilePopupTarget = {
            userId: userId,
            displayName: displayName || "",
            username: username || "",
            avatarUrl: avatarUrl || "",
            roleColor: roleColor || ""
        }
        profilePopup.open()
    }

    // Convert Unicode emoji to Twemoji CDN image URL
    function twemojiUrl(emoji) { return D.twemojiUrl(emoji) }
    function segmentize(str) { return D.segmentize(str) }

    // ─── Emoji Picker Data ───
    property int emojiCat: 0
    property var emojiCats: [
        { icon: "\u{1F600}", label: "Smileys", emoji: [
            "\u{1F600}","\u{1F603}","\u{1F604}","\u{1F601}","\u{1F606}","\u{1F605}","\u{1F923}","\u{1F602}","\u{1F642}","\u{1F643}",
            "\u{1F609}","\u{1F60A}","\u{1F607}","\u{1F970}","\u{1F60D}","\u{1F929}","\u{1F618}","\u{1F617}","\u{1F61A}","\u{1F619}",
            "\u{1F972}","\u{1F60B}","\u{1F61B}","\u{1F61C}","\u{1F92A}","\u{1F61D}","\u{1F911}","\u{1F917}","\u{1F92D}","\u{1F92B}",
            "\u{1F914}","\u{1F910}","\u{1F928}","\u{1F610}","\u{1F611}","\u{1F636}","\u{1FAE0}","\u{1F60F}","\u{1F612}","\u{1F644}",
            "\u{1F62C}","\u{1F60C}","\u{1F614}","\u{1F62A}","\u{1F924}","\u{1F634}","\u{1F637}","\u{1F912}","\u{1F915}","\u{1F922}",
            "\u{1F92E}","\u{1F974}","\u{1F635}","\u{1F92F}","\u{1F920}","\u{1F973}","\u{1F978}","\u{1F60E}","\u{1F913}","\u{1F9D0}",
            "\u{1F615}","\u{1F61F}","\u{2639}","\u{1F62E}","\u{1F632}","\u{1F633}","\u{1F97A}","\u{1F626}","\u{1F628}","\u{1F630}",
            "\u{1F625}","\u{1F622}","\u{1F62D}","\u{1F631}","\u{1F616}","\u{1F623}","\u{1F61E}","\u{1F613}","\u{1F629}","\u{1F62B}",
            "\u{1F971}","\u{1F624}","\u{1F621}","\u{1F620}","\u{1F92C}","\u{1F608}","\u{1F47F}","\u{1F480}","\u{1F4A9}","\u{1F921}"
        ]},
        { icon: "\u{1F44B}", label: "People", emoji: [
            "\u{1F44B}","\u{1F91A}","\u{1F590}","\u{270B}","\u{1F596}","\u{1F44C}","\u{1F90C}","\u{1F90F}","\u{270C}","\u{1F91E}",
            "\u{1F91F}","\u{1F918}","\u{1F919}","\u{1F448}","\u{1F449}","\u{1F446}","\u{1F447}","\u{261D}","\u{1F44D}","\u{1F44E}",
            "\u{270A}","\u{1F44A}","\u{1F91B}","\u{1F91C}","\u{1F44F}","\u{1F64C}","\u{1F450}","\u{1F932}","\u{1F64F}","\u{270D}",
            "\u{1F4AA}","\u{1F9BE}","\u{1F9BF}","\u{1F9B5}","\u{1F9B6}","\u{1F442}","\u{1F9BB}","\u{1F443}","\u{1F9E0}","\u{1FAC0}",
            "\u{1F440}","\u{1F441}","\u{1F445}","\u{1F444}","\u{1F48B}","\u{1FAC2}","\u{1F464}","\u{1F465}","\u{1F5E3}","\u{1F476}"
        ]},
        { icon: "\u{1F43B}", label: "Nature", emoji: [
            "\u{1F436}","\u{1F431}","\u{1F42D}","\u{1F439}","\u{1F430}","\u{1F98A}","\u{1F43B}","\u{1F43C}","\u{1F428}","\u{1F42F}",
            "\u{1F981}","\u{1F42E}","\u{1F437}","\u{1F438}","\u{1F435}","\u{1F648}","\u{1F649}","\u{1F64A}","\u{1F412}","\u{1F414}",
            "\u{1F427}","\u{1F426}","\u{1F424}","\u{1F986}","\u{1F985}","\u{1F989}","\u{1F987}","\u{1F43A}","\u{1F417}","\u{1F434}",
            "\u{1F984}","\u{1F41D}","\u{1F41B}","\u{1F98B}","\u{1F40C}","\u{1F41E}","\u{1F41C}","\u{1F338}","\u{1F490}","\u{1F339}",
            "\u{1F940}","\u{1F33A}","\u{1F33B}","\u{1F33C}","\u{1F337}","\u{1F331}","\u{1F332}","\u{1F333}","\u{1F334}","\u{1F335}"
        ]},
        { icon: "\u{1F355}", label: "Food", emoji: [
            "\u{1F34E}","\u{1F350}","\u{1F34A}","\u{1F34B}","\u{1F34C}","\u{1F349}","\u{1F347}","\u{1F353}","\u{1FAD0}","\u{1F348}",
            "\u{1F352}","\u{1F351}","\u{1F96D}","\u{1F34D}","\u{1F965}","\u{1F95D}","\u{1F345}","\u{1F346}","\u{1F951}","\u{1F966}",
            "\u{1F336}","\u{1F33D}","\u{1F955}","\u{1F954}","\u{1F950}","\u{1F35E}","\u{1F956}","\u{1F968}","\u{1F9C0}","\u{1F95A}",
            "\u{1F373}","\u{1F95E}","\u{1F9C7}","\u{1F953}","\u{1F354}","\u{1F35F}","\u{1F355}","\u{1F32D}","\u{1F96A}","\u{1F32E}",
            "\u{1F32F}","\u{1F959}","\u{1F9C6}","\u{1F958}","\u{1F35D}","\u{1F35C}","\u{1F372}","\u{1F35B}","\u{1F363}","\u{1F371}"
        ]},
        { icon: "\u{26BD}", label: "Activities", emoji: [
            "\u{26BD}","\u{1F3C0}","\u{1F3C8}","\u{26BE}","\u{1F94E}","\u{1F3BE}","\u{1F3D0}","\u{1F3C9}","\u{1F94F}","\u{1F3B1}",
            "\u{1F3D3}","\u{1F3F8}","\u{1F3D2}","\u{1F3D1}","\u{1F94D}","\u{1F3CF}","\u{1F945}","\u{26F3}","\u{1F3F9}","\u{1F3A3}",
            "\u{1F94A}","\u{1F94B}","\u{1F3BD}","\u{1F6F9}","\u{1F6F7}","\u{26F8}","\u{1F94C}","\u{1F3BF}","\u{26F7}","\u{1F3C2}",
            "\u{1F3CB}","\u{1F938}","\u{1F93A}","\u{26F9}","\u{1F3AE}","\u{1F579}","\u{1F3B2}","\u{265F}","\u{1F3AF}","\u{1F3B3}",
            "\u{1F3AA}","\u{1F3AD}","\u{1F3A8}","\u{1F3AC}","\u{1F3A4}","\u{1F3A7}","\u{1F3BC}","\u{1F3B5}","\u{1F3B6}","\u{1F3B9}"
        ]},
        { icon: "\u{1F697}", label: "Travel", emoji: [
            "\u{1F697}","\u{1F695}","\u{1F699}","\u{1F68C}","\u{1F3CE}","\u{1F693}","\u{1F691}","\u{1F692}","\u{1F690}","\u{1F6FB}",
            "\u{1F69A}","\u{1F69B}","\u{1F69C}","\u{1F3CD}","\u{1F6F5}","\u{1F6B2}","\u{1F6F4}","\u{1F6FA}","\u{1F694}","\u{2708}",
            "\u{1F6EB}","\u{1F6EC}","\u{1F6E9}","\u{1F4BA}","\u{1F680}","\u{1F6F8}","\u{1F681}","\u{26F5}","\u{1F6A4}","\u{1F6F3}",
            "\u{26F4}","\u{1F6A2}","\u{2693}","\u{1F5FC}","\u{1F5FD}","\u{1F5FF}","\u{1F3F0}","\u{1F3EF}","\u{1F3DF}","\u{1F30D}",
            "\u{1F30E}","\u{1F30F}","\u{1F30B}","\u{1F5FB}","\u{1F3D4}","\u{26F0}","\u{1F3D5}","\u{1F3D6}","\u{1F3DC}","\u{1F3DD}"
        ]},
        { icon: "\u{1F4A1}", label: "Objects", emoji: [
            "\u{231A}","\u{1F4F1}","\u{1F4BB}","\u{2328}","\u{1F5A5}","\u{1F5A8}","\u{1F5B1}","\u{1F5B2}","\u{1F4BE}","\u{1F4BF}",
            "\u{1F4C0}","\u{1F4FC}","\u{1F4F7}","\u{1F4F8}","\u{1F4F9}","\u{1F3A5}","\u{1F4FD}","\u{1F39E}","\u{1F4DE}","\u{260E}",
            "\u{1F4DF}","\u{1F4E0}","\u{1F4FA}","\u{1F4FB}","\u{1F399}","\u{1F39A}","\u{1F39B}","\u{1F9ED}","\u{23F1}","\u{23F0}",
            "\u{1F50B}","\u{1F50C}","\u{1F4A1}","\u{1F526}","\u{1F56F}","\u{1F9EF}","\u{1F5D1}","\u{1F6D2}","\u{1F6CD}","\u{1F381}",
            "\u{1F388}","\u{1F3CF}","\u{1F380}","\u{1FA84}","\u{1F52E}","\u{1F9FF}","\u{1F3AE}","\u{1F579}","\u{1F4E6}","\u{1F4E8}"
        ]},
        { icon: "\u{2764}", label: "Hearts", emoji: [
            "\u{2764}","\u{1F9E1}","\u{1F49B}","\u{1F49A}","\u{1F499}","\u{1F49C}","\u{1F5A4}","\u{1F90D}","\u{1F90E}","\u{1F494}",
            "\u{2763}","\u{1F495}","\u{1F49E}","\u{1F493}","\u{1F497}","\u{1F496}","\u{1F498}","\u{1F49D}","\u{1F49F}","\u{2665}",
            "\u{1F525}","\u{2728}","\u{1F4AF}","\u{2B50}","\u{1F31F}","\u{1F4AB}","\u{1F4A5}","\u{1F4A2}","\u{1F4A4}","\u{1F4AC}",
            "\u{1F5EF}","\u{1F4AD}","\u{1F6A9}","\u{1F3F4}","\u{1F3F3}","\u{1F3C1}","\u{2705}","\u{274C}","\u{2B55}","\u{1F6D1}",
            "\u{26A0}","\u{1F534}","\u{1F7E0}","\u{1F7E1}","\u{1F7E2}","\u{1F535}","\u{1F7E3}","\u{26AB}","\u{26AA}","\u{1F7E4}"
        ]}
    ]

    // ══════════ Custom Components ══════════

    // Guild sidebar icon — with glow on hover
    component GuildIcon: Rectangle {
        property string text: ""
        property string iconUrl: ""
        property bool isActive: false
        property int fontSize: 14
        signal clicked()

        width: 44; height: 44
        Layout.alignment: Qt.AlignHCenter
        radius: isActive ? 14 : 22
        color: iconUrl && guildIconImg.status === Image.Ready
               ? "transparent"
               : isActive ? theme.accent
               : guildIconMa.containsMouse ? theme.accentLight : theme.bgSecondary
        clip: true

        Behavior on radius { NumberAnimation { duration: theme.animNormal; easing.type: Easing.OutCubic } }
        Behavior on color { ColorAnimation { duration: theme.animNormal } }

        // Glow ring on hover
        Rectangle {
            anchors.fill: parent
            anchors.margins: -3
            radius: parent.radius + 3
            color: "transparent"
            border.width: guildIconMa.containsMouse && !isActive ? 2 : 0
            border.color: theme.accentGlow
            Behavior on border.width { NumberAnimation { duration: theme.animFast } }
        }

        // Guild icon image
        Image {
            id: guildIconImg
            anchors.fill: parent
            source: iconUrl || ""
            sourceSize: Qt.size(88, 88)
            fillMode: Image.PreserveAspectCrop
            visible: iconUrl && status === Image.Ready
            smooth: true
            cache: true
        }

        // Fallback: first letter when no icon
        Text {
            anchors.centerIn: parent
            text: parent.text
            color: isActive || guildIconMa.containsMouse ? "#ffffff" : theme.textSecondary
            font.family: fontFamily
            font.pixelSize: parent.fontSize
            font.bold: true
            visible: !iconUrl || guildIconImg.status !== Image.Ready
            Behavior on color { ColorAnimation { duration: theme.animFast } }
        }

        MouseArea {
            id: guildIconMa
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: parent.clicked()
        }
    }

    // Setting toggle row
    component SettingToggle: Item {
        property string label: ""
        property bool checked: false
        signal toggled()

        // width set externally; height is fixed
        height: 36

        Rectangle {
            anchors.fill: parent
            radius: theme.radiusSmall
            color: "transparent"
        }

        Text {
            id: toggleLabel
            anchors.left: parent.left
            anchors.leftMargin: 8
            anchors.right: toggleSwitch.left
            anchors.rightMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            text: label
            color: theme.textSecondary
            font.family: fontFamily
            font.pixelSize: 13
            elide: Text.ElideRight
        }

        // Toggle switch — compact
        Rectangle {
            id: toggleSwitch
            width: 38; height: 20; radius: 10
            anchors.right: parent.right
            anchors.rightMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            color: checked ? theme.accent : theme.bgTertiary

            Behavior on color { ColorAnimation { duration: theme.animFast } }

            Rectangle {
                width: 16; height: 16; radius: 8
                anchors.verticalCenter: parent.verticalCenter
                x: checked ? parent.width - width - 2 : 2
                color: checked ? "#ffffff" : theme.textFaint

                Behavior on x { NumberAnimation { duration: theme.animFast; easing.type: Easing.OutCubic } }
                Behavior on color { ColorAnimation { duration: theme.animFast } }
            }
        }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: {
                checked = !checked
                toggled()
            }
        }
    }

    // Twemoji image with Unicode fallback while loading
    component Twemoji: Item {
        property string emoji: ""
        property int size: 22

        width: size; height: size

        Image {
            id: twImg
            anchors.fill: parent
            source: emoji !== "" ? twemojiUrl(emoji) : ""
            sourceSize.width: parent.width * 2
            sourceSize.height: parent.height * 2
            smooth: true
            asynchronous: true
            fillMode: Image.PreserveAspectFit
            visible: status === Image.Ready
        }
        // Fallback to native Unicode while image loads or on error
        Text {
            anchors.centerIn: parent
            text: emoji
            font.pixelSize: parent.size - 4
            visible: twImg.status !== Image.Ready
        }
    }

    // Icon button
    component IconButton: Rectangle {
        property string icon: ""
        property int fontSize: 16
        property bool usesTwemoji: false
        signal clicked()

        width: 26; height: 26; radius: theme.radiusSmall
        color: iconBtnMa.containsMouse ? theme.bgHover : "transparent"

        Text {
            anchors.centerIn: parent
            text: parent.icon
            font.pixelSize: parent.fontSize
            color: theme.textFaint
        }

        MouseArea {
            id: iconBtnMa
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: parent.clicked()
        }
    }

    // Voice control button
    component VoiceButton: Rectangle {
        property string icon: ""
        property bool isActive: false
        property color activeColor: theme.danger
        property bool isDestructive: false
        signal clicked()

        width: 26; height: 26; radius: 13
        color: isActive || isDestructive ? activeColor :
               voiceBtnMa.containsMouse ? theme.bgHover : theme.bgTertiary

        Text {
            anchors.centerIn: parent
            text: parent.icon
            color: "#ffffff"
            font.pixelSize: 12
        }

        MouseArea {
            id: voiceBtnMa
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: parent.clicked()
        }
    }

    // Voice participant card for the grid
    component VoiceParticipantCard: Rectangle {
        id: participantCardRoot
        property string participantName: ""
        property string avatarUrl: ""
        property bool isSpeaking: false
        property bool isMuted: false
        property bool isDeafened: false
        property bool isVideo: false
        property bool isStreaming: false

        width: 140
        height: 160
        radius: theme.radiusMed
        color: isSpeaking ? theme.bgTertiary : theme.bgSecondary
        border.width: isSpeaking ? 2 : 0
        property real borderOpacity: 1.0
        border.color: Qt.rgba(
            theme.voiceSpeaking.r, theme.voiceSpeaking.g, theme.voiceSpeaking.b,
            borderOpacity
        )
        SequentialAnimation on borderOpacity {
            running: participantCardRoot.isSpeaking
            loops: Animation.Infinite
            NumberAnimation { to: 0.9; duration: theme.animNormal; easing.type: Easing.InOutQuad }
            NumberAnimation { to: 0.4; duration: theme.animNormal; easing.type: Easing.InOutQuad }
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 10
            spacing: 6

            Item {
                Layout.preferredWidth: 64
                Layout.preferredHeight: 64
                Layout.alignment: Qt.AlignHCenter

                Rectangle {
                    anchors.fill: parent
                    radius: width / 2
                    color: "transparent"
                    border.width: participantCardRoot.isSpeaking ? 3 : 0
                    border.color: theme.voiceSpeaking
                    opacity: participantCardRoot.isSpeaking ? 0.9 : 0

                    SequentialAnimation on opacity {
                        running: participantCardRoot.isSpeaking
                        loops: Animation.Infinite
                        NumberAnimation { to: 0.5; duration: theme.animSlow; easing.type: Easing.InOutQuad }
                        NumberAnimation { to: 1; duration: theme.animSlow; easing.type: Easing.InOutQuad }
                    }
                }

                DAvatar {
                    anchors.centerIn: parent
                    size: 60
                    imageUrl: participantCardRoot.avatarUrl || ""
                    fallbackText: participantCardRoot.participantName || "?"
                }
            }

            Text {
                Layout.fillWidth: true
                text: participantCardRoot.participantName || "Unknown"
                color: theme.textNormal
                font.pixelSize: 13
                font.weight: Font.Medium
                elide: Text.ElideRight
                horizontalAlignment: Text.AlignHCenter
            }

            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: 4
                Text { text: participantCardRoot.isMuted ? "\u{1F507}" : ""; font.pixelSize: 10; color: theme.textMuted; visible: participantCardRoot.isMuted }
                Text { text: participantCardRoot.isDeafened ? "\u{1F508}" : ""; font.pixelSize: 10; color: theme.textMuted; visible: participantCardRoot.isDeafened }
                Text { text: participantCardRoot.isVideo ? "\u{1F4F9}" : ""; font.pixelSize: 10; color: theme.info; visible: participantCardRoot.isVideo }
                Text { text: participantCardRoot.isStreaming ? "\u{1F3AC}" : ""; font.pixelSize: 10; color: theme.accent; visible: participantCardRoot.isStreaming }
            }
        }
    }

    // Developer-style voice stats panel
    component VoiceStatsPanel: Rectangle {
        property string pingMs: "—"
        property string encryptionMode: "—"
        property string endpoint: "—"
        property string ssrc: "—"
        property string packetsSent: "0"
        property string packetsReceived: "0"
        property string connectionDurationSecs: "0"

        width: 280
        height: contentCol.implicitHeight + 16
        radius: theme.radiusSmall
        color: theme.statsBg
        border.width: 1
        border.color: theme.border

        Column {
            id: contentCol
            anchors.fill: parent
            anchors.margins: 8
            spacing: 4

            Text {
                text: "Voice connection"
                font.family: theme.monospace
                font.pixelSize: 10
                color: theme.statsLabel
            }
            Text {
                text: "  endpoint: " + endpoint
                font.family: theme.monospace
                font.pixelSize: 11
                color: theme.statsFg
            }
            Text {
                text: "  encryption: " + encryptionMode
                font.family: theme.monospace
                font.pixelSize: 11
                color: theme.statsFg
            }
            Text {
                text: "  ssrc: " + ssrc
                font.family: theme.monospace
                font.pixelSize: 11
                color: theme.statsFg
            }
            Item { height: 6 }
            Text {
                text: "Network"
                font.family: theme.monospace
                font.pixelSize: 10
                color: theme.statsLabel
            }
            Text {
                text: "  ping: " + pingMs + " ms"
                font.family: theme.monospace
                font.pixelSize: 11
                color: theme.statsFg
            }
            Text {
                text: "  sent: " + packetsSent + "  recv: " + packetsReceived
                font.family: theme.monospace
                font.pixelSize: 11
                color: theme.statsFg
            }
            Item { height: 6 }
            Text {
                text: "Uptime"
                font.family: theme.monospace
                font.pixelSize: 10
                color: theme.statsLabel
            }
            Text {
                text: "  " + connectionDurationSecs + " s"
                font.family: theme.monospace
                font.pixelSize: 11
                color: theme.statsFg
            }
        }
    }

    // ══════════ Voice State ══════════
    property bool isVoiceConnected: false
    property string voiceChannelName: ""
    property string voiceChannelId: ""
    property string voiceGuildId: ""
    property bool isMuted: false
    property bool isDeafened: false
    property bool isFakeMuted: false
    property bool isFakeDeafened: false

    // ══════════ Message Context Menu ══════════
    Popup {
        id: msgContextMenu
        width: 190
        height: Math.max(140, contextMenuCol.implicitHeight + 14)
        padding: 0
        modal: false
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

        onOpened: {
            var px = Math.max(0, Math.min(msgContextMenu.x, root.width - msgContextMenu.width))
            var py = Math.max(0, Math.min(msgContextMenu.y, root.height - msgContextMenu.height))
            msgContextMenu.x = px
            msgContextMenu.y = py
        }

        property string targetMessageId: ""
        property string targetChannelId: ""
        property string targetAuthorName: ""
        property string targetAuthorId: ""
        property string targetAuthorRoleColor: ""
        property string targetAuthorAvatarUrl: ""
        property string targetContent: ""

        enter: Transition {
            NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: theme.animFast }
            NumberAnimation { property: "scale"; from: 0.95; to: 1.0; duration: theme.animFast }
        }
        exit: Transition {
            NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; duration: theme.animFast }
            NumberAnimation { property: "scale"; from: 1.0; to: 0.95; duration: theme.animFast }
        }

        background: Rectangle {
            color: theme.bgElevated
            radius: theme.radiusMed
            border.color: theme.border
            border.width: 1
        }

            ColumnLayout {
                id: contextMenuCol
                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.topMargin: 6
                anchors.bottomMargin: 6
                anchors.leftMargin: 8
                anchors.rightMargin: 8
                spacing: 2

            // Profile (view user profile)
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                radius: theme.radiusSmall
                color: profileMa.pressed ? theme.bgActive :
                       profileMa.containsMouse ? theme.bgHover : "transparent"
                Behavior on color { ColorAnimation { duration: theme.animFast } }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    spacing: 8

                    Text {
                        text: "\u{1F464}"
                        color: theme.textFaint
                        font.pixelSize: 13
                    }
                    Text {
                        text: "Profile"
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 13
                        Layout.fillWidth: true
                    }
                }
                MouseArea {
                    id: profileMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        openUserProfile(msgContextMenu.targetAuthorId, msgContextMenu.targetAuthorName, msgContextMenu.targetAuthorName, msgContextMenu.targetAuthorAvatarUrl, msgContextMenu.targetAuthorRoleColor)
                        msgContextMenu.close()
                    }
                }
            }

            // Message (open DM, only for other users)
            Rectangle {
                visible: msgContextMenu.targetAuthorId && msgContextMenu.targetAuthorId !== currentUserId
                Layout.fillWidth: true
                Layout.preferredHeight: visible ? 32 : 0
                radius: theme.radiusSmall
                color: messageDmMa.pressed ? theme.bgActive :
                       messageDmMa.containsMouse ? theme.bgHover : "transparent"
                Behavior on color { ColorAnimation { duration: theme.animFast } }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    spacing: 8

                    Text {
                        text: "\u{2709}"
                        color: theme.textFaint
                        font.pixelSize: 13
                    }
                    Text {
                        text: "Message"
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 13
                        Layout.fillWidth: true
                    }
                }
                MouseArea {
                    id: messageDmMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (app) app.open_dm(msgContextMenu.targetAuthorId)
                        msgContextMenu.close()
                    }
                }
            }

            // Reply
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                radius: theme.radiusSmall
                color: replyMa.pressed ? theme.bgActive :
                       replyMa.containsMouse ? theme.bgHover : "transparent"
                Behavior on color { ColorAnimation { duration: theme.animFast } }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    spacing: 8

                    Text {
                        text: "\u{21A9}"
                        color: theme.textFaint
                        font.pixelSize: 13
                    }
                    Text {
                        text: "Reply"
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 13
                        Layout.fillWidth: true
                    }
                }
                MouseArea {
                    id: replyMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        startReply(msgContextMenu.targetMessageId, msgContextMenu.targetAuthorName, msgContextMenu.targetContent, msgContextMenu.targetAuthorRoleColor)
                        msgContextMenu.close()
                    }
                }
            }

            // Edit Message (only shown for own messages)
            Rectangle {
                visible: msgContextMenu.targetAuthorId === currentUserId
                Layout.fillWidth: true
                Layout.preferredHeight: visible ? 32 : 0
                radius: theme.radiusSmall
                color: editMa.pressed ? theme.bgActive :
                       editMa.containsMouse ? theme.bgHover : "transparent"
                Behavior on color { ColorAnimation { duration: theme.animFast } }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    spacing: 8

                    Text {
                        text: "\u{270F}"
                        color: theme.textFaint
                        font.pixelSize: 13
                    }
                    Text {
                        text: "Edit Message"
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 13
                        Layout.fillWidth: true
                    }
                }
                MouseArea {
                    id: editMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        editPopup.targetChannelId = msgContextMenu.targetChannelId
                        editPopup.targetMessageId = msgContextMenu.targetMessageId
                        editPopup.editText = msgContextMenu.targetContent
                        editPopup.open()
                        msgContextMenu.close()
                    }
                }
            }

            // Add Reaction
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                radius: theme.radiusSmall
                color: reactMa.pressed ? theme.bgActive :
                       reactMa.containsMouse ? theme.bgHover : "transparent"
                Behavior on color { ColorAnimation { duration: theme.animFast } }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    spacing: 8

                    Text {
                        text: "\u{1F600}"
                        color: theme.textFaint
                        font.pixelSize: 13
                    }
                    Text {
                        text: "Add Reaction"
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 13
                        Layout.fillWidth: true
                    }
                }
                MouseArea {
                    id: reactMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        reactionPickerPopup.targetChannelId = msgContextMenu.targetChannelId
                        reactionPickerPopup.targetMessageId = msgContextMenu.targetMessageId
                        reactionPickerPopup.open()
                        msgContextMenu.close()
                    }
                }
            }

            // Copy Message Text
            Rectangle {
                visible: msgContextMenu.targetContent && msgContextMenu.targetContent.length > 0
                Layout.fillWidth: true
                Layout.preferredHeight: visible ? 32 : 0
                radius: theme.radiusSmall
                color: copyTextMa.pressed ? theme.bgActive :
                       copyTextMa.containsMouse ? theme.bgHover : "transparent"
                Behavior on color { ColorAnimation { duration: theme.animFast } }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    spacing: 8

                    Text {
                        text: "\u{1F4CB}"
                        color: theme.textFaint
                        font.pixelSize: 13
                    }
                    Text {
                        text: "Copy Message Text"
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 13
                        Layout.fillWidth: true
                    }
                }
                MouseArea {
                    id: copyTextMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (app && msgContextMenu.targetContent) app.copy_to_clipboard(msgContextMenu.targetContent)
                        msgContextMenu.close()
                    }
                }
            }

            // Copy Message Link
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                radius: theme.radiusSmall
                color: copyLinkMa.pressed ? theme.bgActive :
                       copyLinkMa.containsMouse ? theme.bgHover : "transparent"
                Behavior on color { ColorAnimation { duration: theme.animFast } }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    spacing: 8

                    Text {
                        text: "\u{1F517}"
                        color: theme.textFaint
                        font.pixelSize: 13
                    }
                    Text {
                        text: "Copy Message Link"
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 13
                        Layout.fillWidth: true
                    }
                }
                MouseArea {
                    id: copyLinkMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (app) app.copy_message_link(msgContextMenu.targetChannelId, currentGuildId, msgContextMenu.targetMessageId)
                        msgContextMenu.close()
                    }
                }
            }

            // Pin Message
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                radius: theme.radiusSmall
                color: pinMa.pressed ? theme.bgActive :
                       pinMa.containsMouse ? theme.bgHover : "transparent"
                Behavior on color { ColorAnimation { duration: theme.animFast } }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    spacing: 8

                    Text {
                        text: "\u{1F4CC}"
                        color: theme.textFaint
                        font.pixelSize: 13
                    }
                    Text {
                        text: "Pin Message"
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 13
                        Layout.fillWidth: true
                    }
                }
                MouseArea {
                    id: pinMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (app) app.pin_message(msgContextMenu.targetChannelId, msgContextMenu.targetMessageId)
                        msgContextMenu.close()
                    }
                }
            }

            // Separator before destructive action (only when Delete is shown)
            Rectangle {
                visible: msgContextMenu.targetAuthorId === currentUserId
                Layout.fillWidth: true
                Layout.preferredHeight: 1
                Layout.topMargin: 2
                Layout.bottomMargin: 2
                color: theme.separator
            }

            // Delete Message (only own messages)
            Rectangle {
                visible: msgContextMenu.targetAuthorId === currentUserId
                Layout.fillWidth: true
                Layout.preferredHeight: visible ? 32 : 0
                radius: theme.radiusSmall
                color: deleteMa.pressed ? Qt.darker(theme.danger, 1.4) :
                       deleteMa.containsMouse ? Qt.rgba(theme.danger.r, theme.danger.g, theme.danger.b, 0.12) : "transparent"
                Behavior on color { ColorAnimation { duration: theme.animFast } }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    spacing: 8

                    Text {
                        text: "\u{1F5D1}"
                        color: theme.danger
                        font.pixelSize: 13
                    }
                    Text {
                        text: "Delete Message"
                        color: theme.danger
                        font.family: fontFamily
                        font.pixelSize: 13
                        Layout.fillWidth: true
                    }
                }
                MouseArea {
                    id: deleteMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (app) app.delete_message(msgContextMenu.targetChannelId, msgContextMenu.targetMessageId)
                        msgContextMenu.close()
                    }
                }
            }
        }
    }

    // ══════════ Server Context Menu (guild name header) ══════════
    Popup {
        id: serverContextMenu
        width: 220
        height: Math.max(100, serverContextMenuCol.implicitHeight + 14)
        padding: 0
        modal: false
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

        onOpened: {
            var px = Math.max(0, Math.min(serverContextMenu.x, root.width - serverContextMenu.width))
            var py = Math.max(0, Math.min(serverContextMenu.y, root.height - serverContextMenu.height))
            serverContextMenu.x = px
            serverContextMenu.y = py
        }

        enter: Transition {
            NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: theme.animFast }
            NumberAnimation { property: "scale"; from: 0.95; to: 1.0; duration: theme.animFast }
        }
        exit: Transition {
            NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; duration: theme.animFast }
            NumberAnimation { property: "scale"; from: 1.0; to: 0.95; duration: theme.animFast }
        }

        background: Rectangle {
            color: theme.bgElevated
            radius: theme.radiusMed
            border.color: theme.border
            border.width: 1
        }

        ColumnLayout {
            id: serverContextMenuCol
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.topMargin: 6
            anchors.bottomMargin: 6
            anchors.leftMargin: 8
            anchors.rightMargin: 8
            spacing: 2

            // Mute Server / Unmute Server
            Rectangle {
                id: serverMuteRow
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                radius: theme.radiusSmall
                color: serverMuteMa.pressed ? theme.bgActive :
                       serverMuteMa.containsMouse ? theme.bgHover : "transparent"
                Behavior on color { ColorAnimation { duration: theme.animFast } }

                property bool isMuted: {
                    if (!app || !currentGuildId) return false
                    try {
                        var ids = JSON.parse(app.mutedGuildIdsJson || "[]")
                        return ids.indexOf(currentGuildId) >= 0
                    } catch (e) { return false }
                }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    spacing: 8

                    Text {
                        text: serverMuteRow.isMuted ? "\u{1F507}" : "\u{1F50A}"
                        color: theme.textFaint
                        font.pixelSize: 13
                    }
                    Text {
                        text: serverMuteRow.isMuted ? "Unmute Server" : "Mute Server"
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 13
                        Layout.fillWidth: true
                    }
                }
                MouseArea {
                    id: serverMuteMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (app && currentGuildId) {
                            if (serverMuteRow.isMuted) app.unmute_guild(currentGuildId)
                            else app.mute_guild(currentGuildId)
                        }
                        serverContextMenu.close()
                    }
                }
            }

            // Leave Server
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                radius: theme.radiusSmall
                color: serverLeaveMa.pressed ? Qt.darker(theme.danger, 1.4) :
                       serverLeaveMa.containsMouse ? Qt.rgba(theme.danger.r, theme.danger.g, theme.danger.b, 0.12) : "transparent"
                Behavior on color { ColorAnimation { duration: theme.animFast } }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    spacing: 8

                    Text {
                        text: "\u{1F4A5}"
                        color: theme.danger
                        font.pixelSize: 13
                    }
                    Text {
                        text: "Leave Server"
                        color: theme.danger
                        font.family: fontFamily
                        font.pixelSize: 13
                        Layout.fillWidth: true
                    }
                }
                MouseArea {
                    id: serverLeaveMa
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        serverContextMenu.close()
                        leaveServerConfirmPopup.open()
                    }
                }
            }
        }
    }

    // Leave Server confirmation
    Popup {
        id: leaveServerConfirmPopup
        anchors.centerIn: parent
        width: 340
        height: leaveServerConfirmCol.implicitHeight + 32
        padding: 16
        modal: true
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

        background: Rectangle {
            color: theme.bgElevated
            radius: theme.radiusMed
            border.color: theme.border
            border.width: 1
        }

        ColumnLayout {
            id: leaveServerConfirmCol
            width: parent.width - 32
            spacing: 16

            Text {
                Layout.fillWidth: true
                text: "Leave \u201C" + (currentGuildName || "Server") + "\u201D?"
                color: theme.textNormal
                font.family: fontFamily
                font.pixelSize: 16
                wrapMode: Text.WordWrap
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 8
                layoutDirection: Qt.RightToLeft

                Button {
                    text: "Cancel"
                    flat: true
                    font.family: fontFamily
                    font.pixelSize: 14
                    onClicked: leaveServerConfirmPopup.close()
                    contentItem: Text {
                        text: parent.text
                        color: theme.textNormal
                        font: parent.font
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                    background: Rectangle {
                        color: parent.hovered ? theme.bgHover : "transparent"
                        radius: 4
                    }
                }
                Button {
                    text: "Leave Server"
                    font.family: fontFamily
                    font.pixelSize: 14
                    onClicked: {
                        if (app && currentGuildId) {
                            app.leave_guild(currentGuildId)
                            app.select_guild("")
                        }
                        leaveServerConfirmPopup.close()
                    }
                    contentItem: Text {
                        text: parent.text
                        color: theme.danger
                        font: parent.font
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                    background: Rectangle {
                        color: parent.hovered ? Qt.rgba(theme.danger.r, theme.danger.g, theme.danger.b, 0.2) : "transparent"
                        radius: 4
                    }
                }
            }
        }
    }

    // ══════════ Edit Message Popup ══════════
    Popup {
        id: editPopup
        anchors.centerIn: parent
        width: 480
        height: 200
        modal: true

        property string targetChannelId: ""
        property string targetMessageId: ""
        property string editText: ""

        enter: Transition {
            NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: theme.animFast }
        }
        exit: Transition {
            NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; duration: theme.animFast }
        }

        background: Rectangle {
            color: theme.bgPrimary
            radius: theme.radiusLarge
            border.color: theme.border
            border.width: 1
        }

        Overlay.modal: Rectangle { color: "#000000aa" }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 20
            spacing: 12

            Text {
                text: "Edit Message"
                color: theme.textNormal
                font.family: fontFamily
                font.pixelSize: 15
                font.weight: Font.Medium
            }

            TextField {
                id: editInput
                Layout.fillWidth: true
                Layout.preferredHeight: 44
                text: editPopup.editText
                color: theme.textNormal
                font.family: fontFamily
                font.pixelSize: 14
                leftPadding: 12
                background: Rectangle {
                    color: theme.bgBase
                    radius: theme.radiusMed
                    border.color: editInput.activeFocus ? theme.accent : theme.border
                    border.width: editInput.activeFocus ? 2 : 1
                }
                Keys.onReturnPressed: {
                    if (editInput.text.trim().length > 0 && app) {
                        app.edit_message(editPopup.targetChannelId, editPopup.targetMessageId, editInput.text.trim())
                    }
                    editPopup.close()
                }
                Keys.onEscapePressed: editPopup.close()
            }

            RowLayout {
                Layout.alignment: Qt.AlignRight
                spacing: 10

                Rectangle {
                    width: cancelEditLabel.implicitWidth + 24
                    height: 34
                    radius: theme.radiusMed
                    color: cancelEditMa.containsMouse ? theme.bgHover : theme.bgSecondary
                    Text {
                        id: cancelEditLabel
                        anchors.centerIn: parent
                        text: "Cancel"
                        color: theme.textNormal
                        font.family: fontFamily; font.pixelSize: 12
                    }
                    MouseArea {
                        id: cancelEditMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: editPopup.close()
                    }
                }
                Rectangle {
                    width: saveEditLabel.implicitWidth + 24
                    height: 34
                    radius: theme.radiusMed
                    color: saveEditMa.containsMouse ? theme.accentHover : theme.accent
                    Text {
                        id: saveEditLabel
                        anchors.centerIn: parent
                        text: "Save"
                        color: "#ffffff"
                        font.family: fontFamily; font.pixelSize: 12; font.bold: true
                    }
                    MouseArea {
                        id: saveEditMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (editInput.text.trim().length > 0 && app) {
                                app.edit_message(editPopup.targetChannelId, editPopup.targetMessageId, editInput.text.trim())
                            }
                            editPopup.close()
                        }
                    }
                }
            }
        }

        onOpened: {
            editInput.text = editText
            editInput.forceActiveFocus()
            editInput.cursorPosition = editInput.text.length
        }
    }

    // ══════════ Reaction Picker Popup ══════════
    Popup {
        id: reactionPickerPopup
        anchors.centerIn: parent
        width: 320
        height: 260
        modal: false

        property string targetChannelId: ""
        property string targetMessageId: ""

        enter: Transition {
            NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: theme.animFast }
        }
        exit: Transition {
            NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; duration: theme.animFast }
        }

        background: Rectangle {
            color: theme.bgElevated
            radius: theme.radiusMed
            border.color: theme.border
            border.width: 1
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 10
            spacing: 6

            Text {
                text: "Add Reaction"
                color: theme.textNormal
                font.family: fontFamily
                font.pixelSize: 13
                font.weight: Font.Medium
            }

            // Quick reaction row
            RowLayout {
                Layout.fillWidth: true
                spacing: 4
                Repeater {
                    model: ["\u{1F44D}", "\u{2764}", "\u{1F602}", "\u{1F622}", "\u{1F621}", "\u{1F440}", "\u{1F525}", "\u{2705}"]
                    delegate: Rectangle {
                        width: 34; height: 34; radius: theme.radiusSmall
                        color: quickReactMa.containsMouse ? theme.bgHover : "transparent"
                        Twemoji {
                            anchors.centerIn: parent
                            emoji: modelData
                            size: 22
                        }
                        MouseArea {
                            id: quickReactMa
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (app) app.add_reaction(reactionPickerPopup.targetChannelId, reactionPickerPopup.targetMessageId, modelData)
                                reactionPickerPopup.close()
                            }
                        }
                    }
                }
            }

            Rectangle { Layout.fillWidth: true; height: 1; color: theme.separator }

            // Full emoji grid (reusing same data)
            GridView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                cellWidth: 36
                cellHeight: 36
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                model: emojiCats[0] ? emojiCats[0].emoji : []

                delegate: Rectangle {
                    width: 34; height: 34; radius: theme.radiusSmall
                    color: reactGridMa.containsMouse ? theme.bgHover : "transparent"
                    Twemoji {
                        anchors.centerIn: parent
                        emoji: modelData
                        size: 20
                    }
                    MouseArea {
                        id: reactGridMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (app) app.add_reaction(reactionPickerPopup.targetChannelId, reactionPickerPopup.targetMessageId, modelData)
                            reactionPickerPopup.close()
                        }
                    }
                }
            }
        }
    }

    // ══════════ Emoji Picker Popup ══════════
    Popup {
        id: emojiPopup
        x: parent.width - 400
        y: parent.height - 440
        width: 380
        height: 420
        modal: false

        enter: Transition {
            NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: theme.animFast }
        }
        exit: Transition {
            NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; duration: theme.animFast }
        }

        background: Rectangle {
            color: theme.bgElevated
            radius: theme.radiusMed
            border.color: theme.border
            border.width: 1
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 0
            spacing: 0

            // ── Top-level tabs: Emoji | Stickers | Server ──
            Row {
                Layout.fillWidth: true
                Layout.preferredHeight: 36
                Layout.leftMargin: 8
                Layout.rightMargin: 8
                Layout.topMargin: 6
                spacing: 4
                Repeater {
                    model: ["Emoji", "Stickers", "Server"]
                    delegate: Rectangle {
                        width: 70
                        height: 28
                        radius: theme.radiusSmall
                        color: emojiPickerTab === index ? theme.bgActive :
                               emojiPickerTabMa.containsMouse ? theme.bgHover : "transparent"
                        Text {
                            anchors.centerIn: parent
                            text: modelData
                            color: theme.textNormal
                            font.family: fontFamily
                            font.pixelSize: 12
                        }
                        MouseArea {
                            id: emojiPickerTabMa
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                emojiPickerTab = index
                                if (index === 1 && app) app.load_sticker_packs()
                                if (index === 2 && app) app.load_guild_emojis(currentGuildId || "")
                            }
                        }
                    }
                }
            }

            // ── Header: Category tabs (only for Emoji tab) ──
            Rectangle {
                visible: emojiPickerTab === 0
                Layout.fillWidth: true
                Layout.preferredHeight: 44
                color: theme.bgFloating
                radius: theme.radiusMed

                // Only round the top corners
                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: theme.radiusMed
                    color: parent.color
                }

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 8
                    anchors.rightMargin: 8
                    spacing: 2

                    Repeater {
                        model: emojiCats
                        delegate: Rectangle {
                            Layout.preferredWidth: 32
                            Layout.preferredHeight: 32
                            radius: theme.radiusSmall
                            color: emojiCat === index ? theme.bgActive :
                                   emojiTabMa.containsMouse ? theme.bgHover : "transparent"

                            Twemoji {
                                anchors.centerIn: parent
                                emoji: modelData.icon
                                size: 18
                                opacity: emojiCat === index ? 1.0 : 0.4
                            }

                            MouseArea {
                                id: emojiTabMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: emojiCat = index
                            }
                        }
                    }
                }
            }

            Rectangle { Layout.fillWidth: true; height: 1; color: theme.separator; visible: emojiPickerTab === 0 }

            // ── Emoji tab content ──
            Item {
                visible: emojiPickerTab === 0
                Layout.fillWidth: true
                Layout.fillHeight: true
                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 44
                        Layout.leftMargin: 10
                        Layout.rightMargin: 10
                        Layout.topMargin: 8
                        color: "transparent"
                        TextField {
                            id: emojiSearch
                            anchors.fill: parent
                            placeholderText: "Search emoji..."
                            placeholderTextColor: theme.textFaint
                            color: theme.textNormal
                            font.family: fontFamily
                            font.pixelSize: 13
                            leftPadding: 32
                            background: Rectangle {
                                color: theme.bgBase
                                radius: theme.radiusMed
                                border.width: emojiSearch.activeFocus ? 1 : 0
                                border.color: theme.accent
                            }
                            Text {
                                anchors.left: parent.left
                                anchors.leftMargin: 10
                                anchors.verticalCenter: parent.verticalCenter
                                text: "🔍"
                                font.pixelSize: 13
                                opacity: 0.5
                            }
                        }
                    }
                    Text {
                        Layout.leftMargin: 14
                        Layout.topMargin: 8
                        Layout.bottomMargin: 4
                        text: emojiCats[emojiCat] ? emojiCats[emojiCat].label.toUpperCase() : ""
                        color: theme.textMuted
                        font.family: fontFamily
                        font.pixelSize: 11
                        font.bold: true
                    }
                    GridView {
                        id: emojiGrid
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        Layout.leftMargin: 8
                        Layout.rightMargin: 8
                        Layout.bottomMargin: 8
                        cellWidth: 40
                        cellHeight: 40
                        clip: true
                        boundsBehavior: Flickable.StopAtBounds
                        model: emojiCats[emojiCat] ? emojiCats[emojiCat].emoji : []
                        ScrollBar.vertical: ScrollBar {
                            policy: ScrollBar.AsNeeded
                            contentItem: Rectangle {
                                implicitWidth: 4; radius: 2
                                color: theme.textFaint
                                opacity: parent.active ? 0.6 : 0.0
                                Behavior on opacity { NumberAnimation { duration: theme.animSlow } }
                            }
                            background: Item {}
                        }
                        delegate: Rectangle {
                            width: 38; height: 38; radius: theme.radiusSmall
                            color: emojiCellMa.containsMouse ? theme.bgHover : "transparent"
                            Twemoji {
                                anchors.centerIn: parent
                                emoji: modelData
                                size: 26
                            }
                            MouseArea {
                                id: emojiCellMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    messageInput.text += modelData
                                    emojiPopup.close()
                                }
                            }
                        }
                    }
                }
            }

            // ── Stickers tab content (grouped by pack with pack name) ──
            Flickable {
                visible: emojiPickerTab === 1
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                contentWidth: width
                contentHeight: stickerPacksColumn.implicitHeight
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ScrollBar {
                    policy: ScrollBar.AsNeeded
                    contentItem: Rectangle {
                        implicitWidth: 4; radius: 2
                        color: theme.textFaint
                        opacity: parent.active ? 0.6 : 0.0
                        Behavior on opacity { NumberAnimation { duration: theme.animSlow } }
                    }
                    background: Item {}
                }
                Column {
                    id: stickerPacksColumn
                    width: parent.width - 8
                    leftPadding: 8
                    rightPadding: 8
                    bottomPadding: 8
                    spacing: 12
                    Repeater {
                        model: stickerPacksModel
                        delegate: Column {
                            width: stickerPacksColumn.width - 16
                            spacing: 6
                            Text {
                                text: ((model && model.packName) ? model.packName : "Stickers").toUpperCase()
                                color: theme.textMuted
                                font.family: fontFamily
                                font.pixelSize: 11
                                font.bold: true
                            }
                            Flow {
                                width: parent.width
                                spacing: 4
                                Repeater {
                                    model: (model && model.stickers) ? model.stickers : []
                                    delegate: Rectangle {
                                        width: 68
                                        height: 68
                                        radius: theme.radiusSmall
                                        color: stickerCellMa.containsMouse ? theme.bgHover : "transparent"
                                        Image {
                                            anchors.centerIn: parent
                                            width: 64
                                            height: 64
                                            fillMode: Image.PreserveAspectFit
                                            source: modelData.url || ""
                                            smooth: true
                                            asynchronous: true
                                        }
                                        MouseArea {
                                            id: stickerCellMa
                                            anchors.fill: parent
                                            hoverEnabled: true
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: {
                                                if (app && currentChannelId) {
                                                    app.send_message_with_options(
                                                        currentChannelId,
                                                        messageInput.text.trim(),
                                                        silentMode,
                                                        replyToMessageId,
                                                        JSON.stringify([modelData.id]),
                                                        "[]"
                                                    )
                                                    messageInput.text = ""
                                                    clearReply()
                                                }
                                                emojiPopup.close()
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── Server emojis tab content ──
            ColumnLayout {
                visible: emojiPickerTab === 2
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 0
                Text {
                    Layout.leftMargin: 8
                    Layout.rightMargin: 8
                    Layout.topMargin: 8
                    Layout.bottomMargin: 4
                    text: (currentGuildName || "Server").toUpperCase()
                    color: theme.textMuted
                    font.family: fontFamily
                    font.pixelSize: 11
                    font.bold: true
                }
                GridView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.leftMargin: 8
                    Layout.rightMargin: 8
                    Layout.bottomMargin: 8
                    cellWidth: 40
                    cellHeight: 40
                    clip: true
                    model: guildEmojiPickerModel
                ScrollBar.vertical: ScrollBar {
                    policy: ScrollBar.AsNeeded
                    contentItem: Rectangle {
                        implicitWidth: 4; radius: 2
                        color: theme.textFaint
                        opacity: parent.active ? 0.6 : 0.0
                        Behavior on opacity { NumberAnimation { duration: theme.animSlow } }
                    }
                    background: Item {}
                }
                delegate: Rectangle {
                    width: 38
                    height: 38
                    radius: theme.radiusSmall
                    color: serverEmojiCellMa.containsMouse ? theme.bgHover : "transparent"
                    Image {
                        anchors.centerIn: parent
                        width: 28
                        height: 28
                        fillMode: Image.PreserveAspectFit
                        source: model.url || ""
                        smooth: true
                        asynchronous: true
                    }
                    MouseArea {
                        id: serverEmojiCellMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            var tag = (model.animated ? "<a:" : "<:") + (model.name || "") + ":" + (model.id || "") + ">"
                            messageInput.text += tag
                            emojiPopup.close()
                        }
                    }
                }
            }
            }
        }
    }

    // Open captcha popup when backend requests it
    Connections {
        target: app
        function onCaptcha_visibleChanged() {
            if (app && app.captcha_visible)
                captchaPopup.open()
        }
    }

    // ══════════ Captcha Popup ══════════
    Popup {
        id: captchaPopup
        anchors.centerIn: parent
        width: 400
        height: 420
        modal: true
        closePolicy: Popup.NoAutoClose

        onOpened: {
            if (app && app.captcha_html)
                captchaWebView.loadHtml(app.captcha_html, "https://discord.com/")
        }

        enter: Transition {
            NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: theme.animFast }
        }
        exit: Transition {
            NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; duration: theme.animFast }
        }

        background: Rectangle {
            color: theme.bgPrimary
            radius: theme.radiusLarge
            border.color: theme.border
            border.width: 1
        }

        Overlay.modal: Rectangle {
            color: "#000000cc"
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 24
            spacing: 16

            Text {
                text: "Verification Required"
                color: theme.textNormal
                font.family: fontFamily
                font.pixelSize: 18
                font.bold: true
                Layout.alignment: Qt.AlignHCenter
            }

            Text {
                text: "Please complete the captcha below"
                color: theme.textSecondary
                font.family: fontFamily
                font.pixelSize: 13
                Layout.alignment: Qt.AlignHCenter
            }

            // Captcha widget (hCaptcha loads in WebEngineView; reports solution via document.title)
            WebEngineView {
                id: captchaWebView
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.minimumHeight: 200
                onTitleChanged: {
                    if (title.indexOf("CAPTCHA_SOLVED:") === 0) {
                        var token = title.substring(14)
                        if (app) app.submit_captcha(token)
                        captchaPopup.close()
                    }
                }
            }

            RowLayout {
                Layout.alignment: Qt.AlignRight
                spacing: 10

                Rectangle {
                    width: cancelCaptchaLabel.implicitWidth + 24
                    height: 34
                    radius: theme.radiusMed
                    color: cancelCaptchaMa.containsMouse ? theme.bgHover : theme.bgSecondary
                    Behavior on color { ColorAnimation { duration: theme.animFast } }

                    Text {
                        id: cancelCaptchaLabel
                        anchors.centerIn: parent
                        text: "Cancel"
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 12
                    }
                    MouseArea {
                        id: cancelCaptchaMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: captchaPopup.close()
                    }
                }
            }
        }
    }

    // ══════════ GIF Picker Popup ══════════
    ListModel { id: gifModel }
    property bool gifSearchPending: false
    property var gifSearchTimer: null

    Popup {
        id: gifPopup
        x: parent.width - 400
        y: parent.height - 440
        width: 380
        height: 420
        modal: false

        enter: Transition {
            NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: theme.animFast }
        }
        exit: Transition {
            NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; duration: theme.animFast }
        }

        onOpened: {
            // Load trending on open
            if (gifModel.count === 0 && app) {
                gifSearchPending = true
                app.search_gifs("")
            }
        }

        background: Rectangle {
            color: theme.bgElevated
            radius: theme.radiusMed
            border.color: theme.border
            border.width: 1
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 12
            spacing: 10

            // Header
            RowLayout {
                Layout.fillWidth: true
                Text {
                    text: "GIFs"
                    color: theme.textNormal
                    font.family: fontFamily
                    font.pixelSize: 15
                    font.bold: true
                }
                Item { Layout.fillWidth: true }
                Text {
                    text: "Powered by Tenor"
                    color: theme.textFaint
                    font.family: fontFamily
                    font.pixelSize: 9
                }
            }

            // Search
            TextField {
                id: gifSearch
                Layout.fillWidth: true
                Layout.preferredHeight: 34
                placeholderText: "Search GIFs..."
                placeholderTextColor: theme.textFaint
                color: theme.textNormal
                font.family: fontFamily
                font.pixelSize: 13
                leftPadding: 12
                background: Rectangle {
                    color: theme.bgBase
                    radius: theme.radiusMed
                    border.width: gifSearch.activeFocus ? 1 : 0
                    border.color: theme.accent
                }

                onTextChanged: {
                    gifSearchDebounce.restart()
                }

                Timer {
                    id: gifSearchDebounce
                    interval: 400
                    repeat: false
                    onTriggered: {
                        if (app) {
                            gifSearchPending = true
                            app.search_gifs(gifSearch.text)
                        }
                    }
                }
            }

            Rectangle { Layout.fillWidth: true; height: 1; color: theme.separator }

            // Trending categories
            RowLayout {
                Layout.fillWidth: true
                spacing: 6
                Repeater {
                    model: ["Trending", "Agree", "Aww", "Dance", "Facepalm", "Hug", "OMG"]
                    delegate: Rectangle {
                        width: catText.implicitWidth + 16; height: 26; radius: 13
                        color: gifCatMa.containsMouse ? theme.bgActive : theme.bgSecondary
                        Behavior on color { ColorAnimation { duration: theme.animFast } }

                        Text {
                            id: catText
                            anchors.centerIn: parent
                            text: modelData
                            color: gifCatMa.containsMouse ? theme.textNormal : theme.textSecondary
                            font.family: fontFamily
                            font.pixelSize: 11
                            Behavior on color { ColorAnimation { duration: theme.animFast } }
                        }

                        MouseArea {
                            id: gifCatMa
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                gifSearch.text = modelData === "Trending" ? "" : modelData
                            }
                        }
                    }
                }
            }

            // GIF grid
            GridView {
                id: gifGrid
                Layout.fillWidth: true
                Layout.fillHeight: true
                cellWidth: 170
                cellHeight: 130
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                model: gifModel

                ScrollBar.vertical: ScrollBar {
                    policy: ScrollBar.AsNeeded
                    contentItem: Rectangle {
                        implicitWidth: 4; radius: 2
                        color: theme.textFaint
                        opacity: parent.active ? 0.6 : 0.0
                        Behavior on opacity { NumberAnimation { duration: theme.animSlow } }
                    }
                    background: Item {}
                }

                delegate: Rectangle {
                    width: 166; height: 126
                    radius: theme.radiusSmall
                    color: gifItemMa.containsMouse ? theme.bgHover : theme.bgSecondary
                    clip: true

                    Image {
                        anchors.fill: parent
                        anchors.margins: 2
                        source: model.previewUrl || model.gifUrl || ""
                        fillMode: Image.PreserveAspectCrop
                        smooth: true
                        asynchronous: true

                        Rectangle {
                            anchors.fill: parent
                            color: gifItemMa.containsMouse ? "#00000040" : "transparent"
                        }
                    }

                    // Loading placeholder
                    Text {
                        anchors.centerIn: parent
                        text: "..."
                        color: theme.textFaint
                        visible: !parent.children[0].status || parent.children[0].status !== Image.Ready
                    }

                    MouseArea {
                        id: gifItemMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            // Send the GIF URL as a message
                            if (currentChannelId && app && model.gifUrl) {
                                app.send_message(currentChannelId, model.gifUrl)
                            }
                            gifPopup.close()
                        }
                    }
                }

                // Empty / loading state
                Text {
                    anchors.centerIn: parent
                    visible: gifModel.count === 0
                    text: gifSearchPending ? "Loading GIFs..." : "No results"
                    color: theme.textMuted
                    font.family: fontFamily
                    font.pixelSize: 12
                }
            }
        }
    }

    // ══════════ Quick Switcher Popup ══════════
    Popup {
        id: quickSwitcherPopup
        anchors.centerIn: parent
        width: 440
        height: 360
        modal: true

        enter: Transition {
            NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: theme.animFast }
            NumberAnimation { property: "scale"; from: 0.95; to: 1.0; duration: theme.animFast }
        }
        exit: Transition {
            NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; duration: theme.animFast }
            NumberAnimation { property: "scale"; from: 1.0; to: 0.95; duration: theme.animFast }
        }

        background: Rectangle {
            color: theme.bgFloating
            radius: theme.radiusMed
            border.color: theme.border
            border.width: 1
        }

        onOpened: {
            switcherSearch.text = ""
            switcherSearch.forceActiveFocus()
            populateSwitcherResults("")
        }

        function populateSwitcherResults(filter) {
            switcherModel.clear()
            var lf = filter.toLowerCase()

            // Add DMs
            for (var d = 0; d < dmChannelModel.count; d++) {
                var dm = dmChannelModel.get(d)
                if (lf === "" || (dm.recipientName && dm.recipientName.toLowerCase().indexOf(lf) >= 0)) {
                    switcherModel.append({
                        switchId: dm.channelId,
                        switchName: dm.recipientName || "Unknown",
                        switchIcon: "@",
                        switchType: "dm",
                        switchGuildId: "",
                        switchChannelType: dm.channelType || 1
                    })
                }
            }

            // Add guild channels
            for (var c = 0; c < channelModel.count; c++) {
                var ch = channelModel.get(c)
                if (lf === "" || (ch.name && ch.name.toLowerCase().indexOf(lf) >= 0)) {
                    var icon = "#"
                    if (ch.channelType === 2) icon = "\u{1F50A}"
                    else if (ch.channelType === 13) icon = "\u{1F3A4}"
                    else if (ch.channelType === 5) icon = "\u{1F4E2}"
                    switcherModel.append({
                        switchId: ch.channelId,
                        switchName: ch.name || "unnamed",
                        switchIcon: icon,
                        switchType: "channel",
                        switchGuildId: currentGuildId,
                        switchChannelType: ch.channelType || 0
                    })
                }
            }
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 12
            spacing: 10

            TextField {
                id: switcherSearch
                Layout.fillWidth: true
                Layout.preferredHeight: 40
                placeholderText: "Where would you like to go?"
                placeholderTextColor: theme.textFaint
                color: theme.textNormal
                font.family: fontFamily
                font.pixelSize: 14
                leftPadding: 12
                background: Rectangle {
                    color: theme.bgBase
                    radius: theme.radiusMed
                    border.width: switcherSearch.activeFocus ? 2 : 1
                    border.color: switcherSearch.activeFocus ? theme.accent : theme.border
                }

                onTextChanged: quickSwitcherPopup.populateSwitcherResults(text)

                Keys.onEscapePressed: quickSwitcherPopup.close()
                Keys.onReturnPressed: {
                    // Select first result
                    if (switcherModel.count > 0) {
                        var item = switcherModel.get(0)
                        quickSwitcherPopup.activateItem(item)
                    }
                }
            }

            Rectangle { Layout.fillWidth: true; height: 1; color: theme.separator }

            ListView {
                id: switcherList
                Layout.fillWidth: true
                Layout.fillHeight: true
                model: switcherModel
                clip: true
                spacing: 2
                boundsBehavior: Flickable.StopAtBounds

                delegate: Rectangle {
                    width: switcherList.width
                    height: 36
                    radius: theme.radiusSmall
                    color: switcherItemMa.containsMouse ? theme.bgHover : "transparent"

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 10
                        anchors.rightMargin: 10
                        spacing: 10

                        Text {
                            text: model.switchIcon
                            color: theme.textFaint
                            font.family: fontFamily
                            font.pixelSize: 14
                            font.bold: true
                            Layout.preferredWidth: 20
                            horizontalAlignment: Text.AlignHCenter
                        }
                        Text {
                            Layout.fillWidth: true
                            text: model.switchName
                            color: theme.textNormal
                            font.family: fontFamily
                            font.pixelSize: 13
                            elide: Text.ElideRight
                        }
                        Text {
                            text: model.switchType === "dm" ? "DM" : "Channel"
                            color: theme.textFaint
                            font.family: fontFamily
                            font.pixelSize: 10
                        }
                    }

                    MouseArea {
                        id: switcherItemMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: quickSwitcherPopup.activateItem(model)
                    }
                }

                Text {
                    anchors.centerIn: parent
                    visible: switcherModel.count === 0
                    text: "No results"
                    color: theme.textMuted
                    font.family: fontFamily
                    font.pixelSize: 12
                }
            }
        }

        ListModel { id: switcherModel }

        function activateItem(item) {
            if (item.switchType === "dm") {
                currentChannelId = item.switchId
                currentChannelName = item.switchName
                currentChannelType = item.switchChannelType
                messageModel.clear()
                messageList.hasMoreHistory = true
                messageList.isLoadingMore = false
                if (app) app.select_channel(item.switchId, item.switchChannelType)
            } else {
                if (item.switchChannelType === 2 || item.switchChannelType === 13) {
                    // Voice channel
                    voiceChannelName = item.switchName
                    voiceChannelId = item.switchId
                    voiceGuildId = item.switchGuildId
                    isVoiceConnected = true
                    isMuted = false
                    isDeafened = false
                    if (app) app.join_voice(item.switchGuildId, item.switchId)
                } else {
                    currentChannelId = item.switchId
                    currentChannelName = item.switchName
                    currentChannelType = item.switchChannelType
                    messageModel.clear()
                    messageList.hasMoreHistory = true
                    messageList.isLoadingMore = false
                    if (app) app.select_channel(item.switchId, item.switchChannelType)
                }
            }
            quickSwitcherPopup.close()
        }
    }

    // ══════════ Pinned Messages Popup ══════════
    Popup {
        id: pinsPopup
        anchors.centerIn: parent
        width: 480
        height: 420
        modal: true

        enter: Transition {
            NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: theme.animFast }
            NumberAnimation { property: "scale"; from: 0.95; to: 1.0; duration: theme.animFast }
        }
        exit: Transition {
            NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; duration: theme.animFast }
            NumberAnimation { property: "scale"; from: 1.0; to: 0.95; duration: theme.animFast }
        }

        background: Rectangle {
            color: theme.bgFloating
            radius: theme.radiusMed
            border.color: theme.border
            border.width: 1
        }

        onOpened: pinsModel.clear()

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 12
            spacing: 10

            RowLayout {
                Layout.fillWidth: true
                Text {
                    text: "Pinned messages"
                    color: theme.textNormal
                    font.family: fontFamily
                    font.pixelSize: 15
                    font.bold: true
                }
                Item { Layout.fillWidth: true }
                Text {
                    text: currentChannelName ? (currentChannelType === 1 ? "DM" : "#" + currentChannelName) : ""
                    color: theme.textFaint
                    font.family: fontFamily
                    font.pixelSize: 12
                    elide: Text.ElideRight
                    Layout.maximumWidth: 180
                }
            }

            Rectangle { Layout.fillWidth: true; height: 1; color: theme.separator }

            ListView {
                id: pinsList
                Layout.fillWidth: true
                Layout.fillHeight: true
                model: pinsModel
                clip: true
                spacing: 4
                boundsBehavior: Flickable.StopAtBounds

                delegate: Item {
                    width: pinsList.width
                    height: pinRow.implicitHeight + 8
                    focus: false

                    RowLayout {
                        id: pinRow
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.leftMargin: 4
                        anchors.rightMargin: 4
                        spacing: 10

                        DAvatar {
                            Layout.preferredWidth: 32
                            Layout.preferredHeight: 32
                            Layout.alignment: Qt.AlignTop
                            size: 32
                            imageUrl: model.authorAvatarUrl || ""
                            fallbackText: model.authorName || "?"
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2

                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 6
                                Text {
                                    text: model.authorName || "Unknown"
                                    color: (model.authorRoleColor && model.authorRoleColor !== "") ? model.authorRoleColor : theme.accent
                                    font.family: fontFamily
                                    font.pixelSize: 13
                                    font.bold: true
                                }
                                UserBadges {
                                    publicFlags: model.authorPublicFlags || 0
                                    isBot: model.authorBot || false
                                    premiumType: model.authorPremiumType || 0
                                    badgeSize: 14
                                }
                                RolePill {
                                    roleName: model.authorRoleName || ""
                                    roleColor: model.authorRoleColor || ""
                                    fontSize: 9
                                }
                                Text {
                                    text: model.timestamp ? (function(ts){ var d = new Date(ts); return isNaN(d.getTime()) ? ts : d.toLocaleDateString() + " " + d.toLocaleTimeString(undefined, {hour: "numeric", minute: "2-digit"}); })(model.timestamp) : ""
                                    color: theme.textFaint
                                    font.family: fontFamily
                                    font.pixelSize: 11
                                }
                                Item { Layout.fillWidth: true }
                                Rectangle {
                                    visible: currentChannelId && app
                                    width: 22; height: 22; radius: theme.radiusSmall
                                    color: pinUnpinMa.containsMouse ? theme.bgHover : "transparent"
                                    Text {
                                        anchors.centerIn: parent
                                        text: "\u{1F4CC}"
                                        color: theme.textFaint
                                        font.pixelSize: 11
                                    }
                                    MouseArea {
                                        id: pinUnpinMa
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            if (app && currentChannelId && model.messageId) {
                                                app.unpin_message(currentChannelId, model.messageId)
                                                app.open_pins(currentChannelId)
                                            }
                                        }
                                    }
                                    ToolTip {
                                        visible: pinUnpinMa.containsMouse
                                        text: "Unpin"
                                        delay: 300
                                    }
                                }
                            }

                            Text {
                                Layout.fillWidth: true
                                text: model.content || ""
                                color: theme.textNormal
                                font.family: fontFamily
                                font.pixelSize: 13
                                wrapMode: Text.WordWrap
                                maximumLineCount: 4
                                elide: Text.ElideRight
                            }
                        }
                    }
                }
            }

            Text {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignHCenter
                visible: pinsModel.count === 0
                text: "No pinned messages"
                color: theme.textMuted
                font.family: fontFamily
                font.pixelSize: 13
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }

    Popup {
        id: accountSwitcherPopup
        width: 220
        height: Math.min(320, accountSwitcherList.count * 44 + 40)
        x: root.width - width - 20
        y: root.height - height - theme.userPanelHeight - 12
        padding: 0
        background: Rectangle {
            color: theme.bgFloating
            border.color: theme.border
            radius: theme.radiusMed
        }
        contentItem: ColumnLayout {
            spacing: 0
            Text {
                Layout.topMargin: 12
                Layout.leftMargin: 12
                Layout.rightMargin: 12
                text: "Switch account"
                color: theme.textMuted
                font.family: fontFamily
                font.pixelSize: 11
                font.weight: Font.Bold
                font.letterSpacing: 0.5
            }
            ListView {
                id: accountSwitcherList
                Layout.fillWidth: true
                Layout.preferredHeight: contentHeight
                Layout.topMargin: 8
                Layout.bottomMargin: 8
                clip: true
                model: accountsList
                delegate: Rectangle {
                    width: accountSwitcherList.width - 16
                    height: 40
                    x: 8
                    radius: theme.radiusSmall
                    color: accMa.containsMouse ? theme.bgHover : (modelData.id === currentUserId ? theme.bgActive : "transparent")
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 10
                        anchors.rightMargin: 10
                        spacing: 10
                        DAvatar {
                            size: 28
                            imageUrl: ""
                            fallbackText: (modelData.name || modelData.id || "?").toString().charAt(0)
                        }
                        Text {
                            Layout.fillWidth: true
                            text: (modelData.name || modelData.id || "Account").toString()
                            color: theme.textNormal
                            font.family: fontFamily
                            font.pixelSize: 13
                            elide: Text.ElideRight
                        }
                        Text {
                            visible: modelData.id === currentUserId
                            text: "Current"
                            color: theme.textMuted
                            font.family: fontFamily
                            font.pixelSize: 11
                        }
                    }
                    MouseArea {
                        id: accMa
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (app && modelData.id !== currentUserId) app.switch_account(modelData.id)
                            accountSwitcherPopup.close()
                        }
                    }
                }
            }
        }
    }

    // ══════════ Keyboard Shortcuts ══════════
    Shortcut { sequence: "Ctrl+K"; onActivated: quickSwitcherPopup.open() }
    Shortcut { sequence: "Ctrl+E"; onActivated: emojiPopup.open() }
    Shortcut { sequence: "Ctrl+G"; onActivated: gifPopup.open() }
    Shortcut { sequence: "Escape"; onActivated: { emojiPopup.close(); gifPopup.close(); settingsPopup.close(); captchaPopup.close(); editPopup.close(); reactionPickerPopup.close(); quickSwitcherPopup.close(); pinsPopup.close(); pluginModalPopup.close(); joinServerPopup.close(); clearReply() } }
    Shortcut { sequence: "Ctrl+Shift+M"; onActivated: { isMuted = !isMuted; if (app && isVoiceConnected) app.toggle_mute() } }
    Shortcut { sequence: "Ctrl+Shift+D"; onActivated: { isDeafened = !isDeafened; if (app && isVoiceConnected) app.toggle_deafen() } }
    Shortcut { sequence: "Ctrl+Shift+S"; onActivated: silentMode = !silentMode }
    Shortcut { sequence: "Ctrl+1"; onActivated: { if (app && accountsList.length > 0) app.switch_account(accountsList[0].id) } }
    Shortcut { sequence: "Ctrl+2"; onActivated: { if (app && accountsList.length > 1) app.switch_account(accountsList[1].id) } }
    Shortcut { sequence: "Ctrl+3"; onActivated: { if (app && accountsList.length > 2) app.switch_account(accountsList[2].id) } }
    Shortcut { sequence: "Ctrl+4"; onActivated: { if (app && accountsList.length > 3) app.switch_account(accountsList[3].id) } }
    Shortcut { sequence: "Ctrl+5"; onActivated: { if (app && accountsList.length > 4) app.switch_account(accountsList[4].id) } }
    Shortcut { sequence: "Ctrl+6"; onActivated: { if (app && accountsList.length > 5) app.switch_account(accountsList[5].id) } }
    Shortcut { sequence: "Ctrl+7"; onActivated: { if (app && accountsList.length > 6) app.switch_account(accountsList[6].id) } }
    Shortcut { sequence: "Ctrl+8"; onActivated: { if (app && accountsList.length > 7) app.switch_account(accountsList[7].id) } }
    Shortcut { sequence: "Ctrl+9"; onActivated: { if (app && accountsList.length > 8) app.switch_account(accountsList[8].id) } }
}
