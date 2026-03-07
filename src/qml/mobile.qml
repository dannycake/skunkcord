// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "Discord.js" as D
import "components" 1.0

Window {
    id: root
    visible: true
    width: 390
    height: 844
    title: isLoggedIn ? (currentGuildName || "Discord") : "Login"
    color: theme.bgBase

    // ─── Fonts ───
    FontLoader { id: ggSans; source: "https://fonts.gstatic.com/s/figtree/v9/_Xms-HUzqDCFdgfMm4S9DQ.woff2" }
    readonly property string fontFamily: ggSans.status === FontLoader.Ready ? ggSans.name : "Segoe UI, Helvetica Neue, Helvetica, Arial, sans-serif"

    // ─── Theme ───
    QtObject {
        id: theme
        readonly property color bgBase:       "#313338"
        readonly property color bgPrimary:    "#313338"
        readonly property color bgSecondary:  "#2b2d31"
        readonly property color bgTertiary:   "#1e1f22"
        readonly property color bgHover:      "#35373c"
        readonly property color bgActive:     "#404249"
        readonly property color bgElevated:   "#2b2d31"
        readonly property color bgFloating:   "#111214"
        readonly property color bgModifier:   "#ffffff08"
        readonly property color textNormal:    "#dbdee1"
        readonly property color textSecondary: "#b5bac1"
        readonly property color textMuted:     "#80848e"
        readonly property color textFaint:     "#6d6f73"
        readonly property color accent:       "#5865f2"
        readonly property color accentHover:  "#4752c4"
        readonly property color accentLight:  "#7289da"
        readonly property color accentGlow:   "#5865f230"
        readonly property color accentMuted:  "#5865f218"
        readonly property color positive:  "#23a55a"
        readonly property color warning:   "#f0b132"
        readonly property color danger:    "#f23f43"
        readonly property color info:      "#5e9eff"
        readonly property color border:        "#3f4147"
        readonly property color borderSubtle:  "#ffffff0a"
        readonly property color separator:     "#3f4147"
        readonly property color inputBg:       "#383a40"
        readonly property color channelIcon:  "#80848e"
        readonly property color headerPrimary: "#f2f3f5"
        readonly property color online:  "#23a55a"
        readonly property color idle:    "#f0b132"
        readonly property color dnd:     "#f23f43"
        readonly property color offline: "#80848e"
        readonly property color mentionBg:    "#5865f210"
        readonly property color mentionText:  "#c9cdfb"
        readonly property color mentionPillBg: "#5865f2"   // Discord blurple for @mention count badges
        readonly property color voicePositive:   "#23a55a"
        readonly property color voiceConnecting: "#f0b132"
        readonly property color voiceSpeaking:   "#23a55a"
        readonly property color voiceSpeakingGlow: "#23a55a40"
        readonly property color statsBg:     "#0a0c10"
        readonly property color statsFg:     "#40c463"
        readonly property color statsLabel:  "#6d7178"
        readonly property string monospace:  "JetBrains Mono, Fira Code, Consolas, monospace"
        // Mobile layout dimensions
        readonly property int guildBarWidth:   56
        readonly property int headerHeight:    48
        readonly property int userPanelHeight: 52
        readonly property int messageInputH:   48
        readonly property int touchTarget:     44
        readonly property int radiusSmall: 4
        readonly property int radiusMed:   8
        readonly property int radiusLarge: 12
        readonly property int radiusXl:    16
        readonly property int animFast:   100
        readonly property int animNormal: 150
        readonly property int animSlow:   250
    }

    // ─── State ───
    property bool isLoggedIn: app ? app.is_logged_in : false
    property string currentUserId: app ? app.user_id : ""
    property string currentUserName: app ? app.user_name : ""
    property string currentUserAvatar: app ? app.user_avatar : ""
    property string currentStatus: "online"
    property string connectionState: app ? app.connection_state : ""
    property string typingDisplay: app ? app.typing_display : ""
    property string typingDisplayJson: app ? app.typing_display_json : "[]"
    property int presenceVersion: app ? app.presence_version : 0

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
        try { return JSON.parse(String(typingDisplayJson || "[]")) } catch (e) { return [] }
    })()

    property string voiceConnectionState: app ? app.voice_connection_state : ""
    property string replyToMessageId: ""
    property string replyToAuthor: ""
    property string replyToAuthorColor: ""
    property string replyToContent: ""
    property bool silentMode: false
    property string currentGuildId: ""
    property string currentGuildName: ""
    property string currentChannelId: ""
    property string currentChannelName: ""
    property int currentChannelType: 0
    readonly property bool isVoiceChannel: currentChannelType === 2 || currentChannelType === 13
    onCurrentChannelIdChanged: { if (app) app.update_typing_for_channel(currentChannelId) }
    property string loginError: app ? app.error_message : ""
    property bool loginLoading: app ? app.is_loading : false
    property bool _didInitialDmRefresh: false
    property string deletedMessageStyle: "strikethrough"
    property var pluginUiData: ({})
    property var pluginToolbarButtons: []

    // Voice state
    property bool isVoiceConnected: false
    property bool isMuted: false
    property bool isDeafened: false
    property bool isFakeDeafened: false
    property string voiceChannelId: ""
    property string voiceChannelName: ""
    property string voiceGuildId: ""
    property string voiceStatsPing: "—"
    property string voiceStatsEncryption: "—"
    property string voiceStatsEndpoint: "—"
    property string voiceStatsSsrc: "—"
    property string voiceStatsPacketsSent: "0"
    property string voiceStatsPacketsReceived: "0"
    property string voiceStatsDuration: "0"

    Component.onCompleted: {
        if (app) deletedMessageStyle = app.get_deleted_message_style()
    }

    // ── Caches (same as desktop) ──
    property var channelCacheMap: ({})
    property var messageCacheMap: ({})
    property var dmChannelCache: []
    property var lastChannelPerGuild: ({})
    property var collapsedCategories: ({})
    property var fullChannelCache: ({})
    property bool showHiddenChannels: false
    property var myGuildProfile: null
    property var profilePopupTarget: null
    property bool profileLoadPending: false
    property var loadedProfileData: null
    property string loadedProfileRawJson: ""
    readonly property var emptyRepeaterModel: []

    function avatarColor(str) { return D.avatarColor(str) }

    function openUserProfile(userId, displayName, username, avatarUrl, roleColor) {
        if (!userId) return
        profilePopupTarget = { userId: userId, displayName: displayName || "", username: username || "", avatarUrl: avatarUrl || "", roleColor: roleColor || "" }
        loadedProfileData = null
        loadedProfileRawJson = ""
        profileLoadPending = true
        if (app) app.fetch_user_profile(userId, currentGuildId)
        profileDrawer.open()
    }

    function cacheCurrentChannels() {
        if (currentGuildId !== "" && channelModel.count > 0) {
            var arr = []
            for (var i = 0; i < channelModel.count; i++) {
                var it = channelModel.get(i)
                arr.push({ channelId: it.channelId, guildId: it.guildId, name: it.name,
                    channelType: it.channelType, position: it.position, parentId: it.parentId || "",
                    hasUnread: it.hasUnread, mentionCount: it.mentionCount, isHidden: it.isHidden || false })
            }
            var m = channelCacheMap; m[currentGuildId] = arr; channelCacheMap = m
        }
    }

    function cacheCurrentMessages() {
        if (currentChannelId !== "" && messageModel.count > 0) {
            var arr = []
            for (var i = 0; i < messageModel.count; i++) {
                var it = messageModel.get(i)
                arr.push({ messageId: it.messageId, channelId: it.channelId,
                    authorName: it.authorName, authorId: it.authorId, authorAvatarUrl: it.authorAvatarUrl,
                    content: it.content, timestamp: it.timestamp, isDeleted: it.isDeleted,
                    messageType: it.messageType, replyAuthorName: it.replyAuthorName,
                    replyContent: it.replyContent, replyAuthorId: it.replyAuthorId,
                    mentionsMe: it.mentionsMe, mentionEveryone: it.mentionEveryone,
                    authorRoleColor: it.authorRoleColor, authorRoleName: it.authorRoleName,
                    authorPublicFlags: it.authorPublicFlags || 0, authorBot: it.authorBot || false,
                    authorPremiumType: it.authorPremiumType || 0,
                    attachmentsJson: it.attachmentsJson, stickersJson: it.stickersJson,
                    embedsJson: it.embedsJson || "[]", contentHtml: it.contentHtml, reactions: it.reactions })
            }
            var m = messageCacheMap; m[currentChannelId] = arr; messageCacheMap = m
        }
    }

    function buildChannelList(channels, guildId) {
        if (!channels || channels.length === 0) return []
        var categories = [], uncategorized = [], byParent = {}
        for (var i = 0; i < channels.length; i++) {
            var ch = channels[i]
            if (ch.isHidden && !showHiddenChannels) continue
            if (ch.channelType === 4) categories.push(ch)
            else if (!ch.parentId || ch.parentId === "") uncategorized.push(ch)
            else { if (!byParent[ch.parentId]) byParent[ch.parentId] = []; byParent[ch.parentId].push(ch) }
        }
        uncategorized.sort(function(a, b) { return a.position - b.position })
        categories.sort(function(a, b) { return a.position - b.position })
        for (var k in byParent) byParent[k].sort(function(a, b) { return a.position - b.position })
        var collapsed = collapsedCategories[guildId] || {}
        var out = []
        for (var u = 0; u < uncategorized.length; u++) out.push(uncategorized[u])
        for (var c = 0; c < categories.length; c++) {
            var cat = categories[c]
            out.push(cat)
            if (!collapsed[cat.channelId]) {
                var kids = byParent[cat.channelId] || []
                for (var kk = 0; kk < kids.length; kk++) out.push(kids[kk])
            }
        }
        return out
    }

    function toggleCategory(guildId, categoryId) {
        var gc = collapsedCategories[guildId] || {}
        gc[categoryId] = !gc[categoryId]
        var m = collapsedCategories; m[guildId] = gc; collapsedCategories = m
        rebuildChannelModel(guildId)
    }

    function rebuildChannelModel(guildId) {
        var full = fullChannelCache[guildId]; if (!full) return
        var built = buildChannelList(full, guildId)
        channelModel.clear()
        for (var i = 0; i < built.length; i++) channelModel.append(built[i])
    }

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

    function restoreMessagesFromCache(channelId) {
        var cached = messageCacheMap[channelId]
        if (cached && cached.length > 0) {
            messageModel.clear()
            for (var i = 0; i < cached.length; i++) messageModel.append(cached[i])
            return true
        }
        return false
    }

    function segmentize(str) { return D.segmentize(str) }
    function twemojiUrl(emoji) { return D.twemojiUrl(emoji) }

    function clearReply() {
        replyToMessageId = ""; replyToAuthor = ""; replyToAuthorColor = ""; replyToContent = ""
    }

    function sendMessage() {
        if (!app || currentChannelId === "") return
        var text = messageInput.text.trim()
        if (text.length === 0) return
        if (replyToMessageId.length > 0)
            app.send_message_ex(currentChannelId, text, silentMode, replyToMessageId)
        else if (silentMode)
            app.send_message_ex(currentChannelId, text, true, "")
        else
            app.send_message(currentChannelId, text)
        messageInput.text = ""
        clearReply()
    }

    function doLogin() {
        if (!app) return
        if (app.login_mode === "token") {
            var t = tokenInput.text.trim()
            if (t.length > 0) { app.login(t); tokenInput.text = "" }
        } else {
            var e = emailInput.text.trim(), p = passwordInput.text.trim()
            if (e.length > 0 && p.length > 0) app.login_credentials(e, p)
        }
    }

    function doMfaSubmit() {
        if (!app) return
        var code = mfaInput.text.trim()
        if (code.length > 0) { app.submit_mfa_code(code); mfaInput.text = "" }
    }

    function typingSeparator(idx, total) {
        if (total <= 1) return " is typing..."
        if (idx < total - 2) return ", "
        if (idx === total - 2) return " and "
        return " are typing..."
    }

    // ─── Timer ───
    Timer {
        interval: 100; running: true; repeat: true
        onTriggered: {
            if (!app) return
            app.check_for_updates()

            var gj = app.consume_guilds()
            if (gj.length > 2) {
                try {
                    var guilds = JSON.parse(gj)
                    guildModel.clear()
                    for (var i = 0; i < guilds.length; i++) guildModel.append(guilds[i])
                    if (currentGuildId === "" && !_didInitialDmRefresh) {
                        _didInitialDmRefresh = true; app.select_guild("")
                    }
                } catch(e) {}
            }

            var cj = app.consume_channels()
            if (cj.length > 2) {
                try {
                    var channels = JSON.parse(cj)
                    var chGuildId = (channels.length > 0 && channels[0].guildId) ? channels[0].guildId : currentGuildId
                    var fMap = fullChannelCache; fMap[chGuildId] = channels; fullChannelCache = fMap
                    var cMap = channelCacheMap; cMap[chGuildId] = buildChannelList(channels, chGuildId); channelCacheMap = cMap
                    if (chGuildId === currentGuildId || currentGuildId === "") {
                        var built = buildChannelList(channels, chGuildId)
                        channelModel.clear()
                        for (var jj = 0; jj < built.length; jj++) channelModel.append(built[jj])
                    }
                } catch(e) {}
            }

            var pj = app.consume_my_profile()
            if (pj.length > 2) { try { myGuildProfile = JSON.parse(pj) } catch(e) {} }

            var dj = app.consume_dm_channels()
            if (dj.length > 2) {
                try {
                    var dms = JSON.parse(dj); dmChannelCache = dms
                    if (currentGuildId === "") {
                        dmChannelModel.clear()
                        for (var d = 0; d < dms.length; d++) dmChannelModel.append(dms[d])
                    }
                } catch(e) {}
            }

            var lmj = app.consume_loaded_messages()
            if (lmj.length > 2) {
                try {
                    var loaded = JSON.parse(lmj)
                    var msgChId = (loaded.length > 0 && loaded[0].channelId) ? loaded[0].channelId : currentChannelId
                    var mMap = messageCacheMap; mMap[msgChId] = loaded; messageCacheMap = mMap
                    if (msgChId === currentChannelId) {
                        messageModel.clear()
                        for (var l = 0; l < loaded.length; l++) messageModel.append(loaded[l])
                    }
                } catch(e) {}
            }

            var mj = app.consume_messages()
            if (mj.length > 2) {
                try {
                    var messages = JSON.parse(mj)
                    for (var k = 0; k < messages.length; k++) {
                        if (messages[k].channelId === currentChannelId) messageModel.insert(0, messages[k])
                    }
                } catch(e) {}
            }

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
                } catch(e) {}
            }

            var dj2 = app.consume_message_deletions()
            if (dj2.length > 2) {
                try {
                    var dels = JSON.parse(dj2)
                    for (var di = 0; di < dels.length; di++) {
                        if (dels[di].channelId !== currentChannelId) continue
                        var dd = dels[di]
                        if (dd.isDeleted && dd.content !== undefined) {
                            for (var mdi = 0; mdi < messageModel.count; mdi++) {
                                if (messageModel.get(mdi).messageId === dd.messageId) {
                                    messageModel.setProperty(mdi, "isDeleted", true)
                                    messageModel.setProperty(mdi, "content", dd.content || "")
                                    break
                                }
                            }
                        } else {
                            for (var mdi2 = messageModel.count - 1; mdi2 >= 0; mdi2--) {
                                if (messageModel.get(mdi2).messageId === dd.messageId) { messageModel.remove(mdi2); break }
                            }
                        }
                    }
                } catch(e) {}
            }

            var ruj = app.consume_reaction_updates()
            if (ruj.length > 2) {
                try {
                    var ru = JSON.parse(ruj)
                    for (var ri = 0; ri < ru.length; ri++) {
                        if (ru[ri].channelId === currentChannelId) {
                            for (var rmi = 0; rmi < messageModel.count; rmi++) {
                                if (messageModel.get(rmi).messageId === ru[ri].messageId) {
                                    messageModel.setProperty(rmi, "reactions", JSON.stringify(ru[ri].reactions || []))
                                    break
                                }
                            }
                        }
                    }
                } catch(e) {}
            }

            var uuj = app.consume_unread_updates()
            if (uuj.length > 2) {
                try {
                    var unreadUpdates = JSON.parse(uuj)
                    var guildsTouched = {}
                    for (var uu = 0; uu < unreadUpdates.length; uu++) {
                        var u = unreadUpdates[uu]
                        var gid = u.guildId || "", chId = u.channelId || ""
                        if (!chId) continue
                        if (gid) {
                            var chList = fullChannelCache[gid]
                            if (chList) {
                                for (var ci = 0; ci < chList.length; ci++) {
                                    if (chList[ci].channelId === chId) {
                                        chList[ci].hasUnread = !!u.hasUnread
                                        chList[ci].mentionCount = u.mentionCount || 0
                                        guildsTouched[gid] = true; break
                                    }
                                }
                            }
                        }
                    }
                    for (var gt in guildsTouched) {
                        var cl = fullChannelCache[gt]
                        if (cl) {
                            var hasUnread = false, mentionCount = 0
                            for (var cj2 = 0; cj2 < cl.length; cj2++) {
                                if (cl[cj2].hasUnread) hasUnread = true
                                mentionCount += cl[cj2].mentionCount || 0
                            }
                            for (var gi = 0; gi < guildModel.count; gi++) {
                                if (guildModel.get(gi).guildId === gt) {
                                    guildModel.setProperty(gi, "hasUnread", hasUnread)
                                    guildModel.setProperty(gi, "mentionCount", mentionCount); break
                                }
                            }
                        }
                    }
                    if (guildsTouched[currentGuildId]) rebuildChannelModel(currentGuildId)
                } catch(e) {}
            }

            var mmj = app.consume_more_messages()
            if (mmj.length > 2) {
                try {
                    var moreData = JSON.parse(mmj)
                    if (moreData.channelId === currentChannelId) {
                        var moreMsgs = moreData.messages
                        for (var mi2 = 0; mi2 < moreMsgs.length; mi2++) messageModel.append(moreMsgs[mi2])
                    }
                } catch(e) {}
            }

            var vsj = app.consume_voice_state()
            if (vsj.length > 0) {
                if (vsj.indexOf("joined:") === 0) isVoiceConnected = true
                else if (vsj === "disconnected") {
                    isVoiceConnected = false; voiceChannelName = ""; voiceChannelId = ""; voiceGuildId = ""
                    voiceParticipantModel.clear()
                }
                else if (vsj.indexOf("mute:") === 0) isMuted = (vsj === "mute:true")
                else if (vsj.indexOf("deafen:") === 0) { var deaf = (vsj === "deafen:true"); isDeafened = deaf; if (deaf) isMuted = true }
                else if (vsj.indexOf("fake_deafen:") === 0) isFakeDeafened = (vsj === "fake_deafen:true")
            }

            var vpj = app.consume_voice_participants()
            if (vpj.length > 2) {
                try {
                    var vpData = JSON.parse(vpj)
                    if (vpData.participants) {
                        voiceParticipantModel.clear()
                        for (var vpi = 0; vpi < vpData.participants.length; vpi++) voiceParticipantModel.append(vpData.participants[vpi])
                    }
                } catch(e) {}
            }

            var suj = app.consume_speaking_users()
            if (suj.length > 2) {
                try {
                    var suArr = JSON.parse(suj)
                    for (var sui = 0; sui < suArr.length; sui++) {
                        for (var spi = 0; spi < voiceParticipantModel.count; spi++) {
                            if (voiceParticipantModel.get(spi).userId === suArr[sui].userId) {
                                voiceParticipantModel.setProperty(spi, "speaking", !!suArr[sui].speaking); break
                            }
                        }
                    }
                } catch(e) {}
            }

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
                } catch(e) {}
            }

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
                                if ((_btns[_i].placement || "toolbar") === "toolbar")
                                    out.push({ pluginId: _pid, buttonId: _btns[_i].id, label: _btns[_i].label || _btns[_i].id, tooltip: _btns[_i].tooltip || "" })
                            }
                        }
                        pluginToolbarButtons = out
                    }
                } catch(e) {}
            }

            if (app && currentGuildId !== "") {
                var memj = app.consume_members()
                if (memj.length > 2) {
                    try {
                        var memberData = JSON.parse(memj)
                        if (memberData.guildId === currentGuildId && memberData.members) {
                            memberModel.clear()
                            for (var mmi = 0; mmi < memberData.members.length; mmi++) {
                                var mm = memberData.members[mmi]
                                memberModel.append({
                                    memberId: mm.memberId || "", username: mm.username || "",
                                    displayName: mm.displayName || mm.username || "", avatarUrl: mm.avatarUrl || "",
                                    roleName: mm.roleName || "", roleColor: mm.roleColor || "",
                                    publicFlags: mm.publicFlags || 0, bot: mm.bot || false, premiumType: mm.premiumType || 0
                                })
                            }
                        }
                    } catch(e) {}
                }
            }

            if (app && profileLoadPending) {
                var upj = app.consume_user_profile()
                if (upj && String(upj).length > 0) {
                    try {
                        loadedProfileData = JSON.parse(String(upj))
                        loadedProfileRawJson = app.consume_user_profile_raw() || ""
                        profileLoadPending = false
                    } catch(e) {}
                }
            }
        }
    }

    // ─── Models ───
    ListModel { id: guildModel }
    ListModel { id: channelModel }
    ListModel { id: dmChannelModel }
    ListModel { id: messageModel }
    ListModel { id: memberModel }
    ListModel { id: voiceParticipantModel }

    // ─── Navigation ───
    onIsLoggedInChanged: {
        if (isLoggedIn) navStack.replace(null, homeView)
        else navStack.replace(null, loginPage)
    }

    StackView {
        id: navStack
        anchors.fill: parent
        initialItem: loginPage
    }

    // ═══════════════════════════════════════════════════════════════
    // LOGIN PAGE
    // ═══════════════════════════════════════════════════════════════
    Component {
        id: loginPage
        Rectangle {
            color: theme.bgBase

            // MFA overlay
            Rectangle {
                anchors.fill: parent
                visible: app ? app.mfa_required : false
                color: "#dd000000"
                z: 10

                ColumnLayout {
                    anchors.centerIn: parent
                    width: parent.width - 48
                    spacing: 16

                    Text {
                        text: "Two-factor authentication"
                        color: theme.textNormal
                        font.family: fontFamily; font.pixelSize: 20; font.bold: true
                        Layout.alignment: Qt.AlignHCenter
                    }
                    Text {
                        text: "Enter the 6-digit code from your authenticator app"
                        color: theme.textMuted; font.family: fontFamily; font.pixelSize: 13
                        Layout.alignment: Qt.AlignHCenter; wrapMode: Text.WordWrap
                        Layout.fillWidth: true; horizontalAlignment: Text.AlignHCenter
                    }
                    TextField {
                        id: mfaInput
                        Layout.fillWidth: true; Layout.preferredHeight: 48
                        placeholderText: "000000"; placeholderTextColor: theme.textFaint
                        color: theme.textNormal; font.family: fontFamily; font.pixelSize: 20
                        horizontalAlignment: TextInput.AlignHCenter
                        maximumLength: 8; inputMethodHints: Qt.ImhDigitsOnly
                        background: Rectangle { color: theme.bgSecondary; radius: theme.radiusMed; border.color: mfaInput.activeFocus ? theme.accent : theme.border; border.width: mfaInput.activeFocus ? 2 : 1 }
                        Keys.onReturnPressed: doMfaSubmit()
                    }
                    Rectangle {
                        Layout.fillWidth: true; Layout.preferredHeight: 48
                        radius: theme.radiusMed; color: theme.accent
                        Text { anchors.centerIn: parent; text: "Submit"; color: "#ffffff"; font.family: fontFamily; font.pixelSize: 16; font.bold: true }
                        MouseArea { anchors.fill: parent; onClicked: doMfaSubmit() }
                    }
                }
            }

            // Login card
            Flickable {
                anchors.fill: parent
                contentHeight: loginCol.height + 64
                clip: true

                ColumnLayout {
                    id: loginCol
                    anchors.horizontalCenter: parent.horizontalCenter
                    anchors.top: parent.top
                    anchors.topMargin: 80
                    width: parent.width - 48
                    spacing: 8

                    Text { text: "Welcome back"; color: theme.textNormal; font.family: fontFamily; font.pixelSize: 24; font.bold: true; Layout.alignment: Qt.AlignHCenter }
                    Text {
                        text: (app && app.login_mode === "token") ? "Enter your token to continue" : "We're so excited to see you again!"
                        color: theme.textMuted; font.family: fontFamily; font.pixelSize: 13; Layout.alignment: Qt.AlignHCenter; Layout.bottomMargin: 16
                    }

                    Rectangle {
                        visible: loginError.length > 0; Layout.fillWidth: true
                        Layout.preferredHeight: loginErrText.implicitHeight + 20; radius: theme.radiusMed
                        color: "#f23f4318"; border.width: 1; border.color: "#f23f4330"; Layout.bottomMargin: 8
                        Text { id: loginErrText; anchors.centerIn: parent; width: parent.width - 24; text: loginError; color: theme.danger; font.family: fontFamily; font.pixelSize: 12; wrapMode: Text.WordWrap; horizontalAlignment: Text.AlignHCenter }
                    }

                    // Token form
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 8; visible: app && app.login_mode === "token"
                        Text { text: "TOKEN"; color: theme.textSecondary; font.family: fontFamily; font.pixelSize: 11; font.bold: true }
                        TextField {
                            id: tokenInput; Layout.fillWidth: true; Layout.preferredHeight: 48
                            placeholderText: "Enter your token"; placeholderTextColor: theme.textFaint
                            color: theme.textNormal; font.family: fontFamily; font.pixelSize: 14; echoMode: TextInput.Password; leftPadding: 12
                            background: Rectangle { color: theme.bgSecondary; radius: theme.radiusMed; border.color: tokenInput.activeFocus ? theme.accent : theme.border; border.width: tokenInput.activeFocus ? 2 : 1 }
                            Keys.onReturnPressed: doLogin()
                        }
                    }

                    // Credentials form
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 8; visible: app && app.login_mode !== "token"
                        Text { text: "EMAIL OR PHONE NUMBER"; color: theme.textSecondary; font.family: fontFamily; font.pixelSize: 11; font.bold: true }
                        TextField {
                            id: emailInput; Layout.fillWidth: true; Layout.preferredHeight: 48
                            placeholderText: "Email or phone number"; placeholderTextColor: theme.textFaint
                            color: theme.textNormal; font.family: fontFamily; font.pixelSize: 14; leftPadding: 12
                            background: Rectangle { color: theme.bgSecondary; radius: theme.radiusMed; border.color: emailInput.activeFocus ? theme.accent : theme.border; border.width: emailInput.activeFocus ? 2 : 1 }
                            Keys.onReturnPressed: passwordInput.forceActiveFocus()
                        }
                        Text { text: "PASSWORD"; color: theme.textSecondary; font.family: fontFamily; font.pixelSize: 11; font.bold: true; Layout.topMargin: 4 }
                        TextField {
                            id: passwordInput; Layout.fillWidth: true; Layout.preferredHeight: 48
                            placeholderText: "Password"; placeholderTextColor: theme.textFaint
                            color: theme.textNormal; font.family: fontFamily; font.pixelSize: 14; echoMode: TextInput.Password; leftPadding: 12
                            background: Rectangle { color: theme.bgSecondary; radius: theme.radiusMed; border.color: passwordInput.activeFocus ? theme.accent : theme.border; border.width: passwordInput.activeFocus ? 2 : 1 }
                            Keys.onReturnPressed: doLogin()
                        }
                    }

                    Item { Layout.preferredHeight: 8 }

                    Rectangle {
                        Layout.fillWidth: true; Layout.preferredHeight: 48; radius: theme.radiusMed
                        color: loginLoading ? theme.accentHover : theme.accent; opacity: loginLoading ? 0.7 : 1.0
                        Text { anchors.centerIn: parent; text: loginLoading ? "Connecting..." : "Log In"; color: "#ffffff"; font.family: fontFamily; font.pixelSize: 16; font.bold: true }
                        MouseArea { anchors.fill: parent; enabled: !loginLoading; onClicked: doLogin() }
                    }

                    Text {
                        Layout.topMargin: 16; Layout.alignment: Qt.AlignHCenter
                        text: (app && app.login_mode === "token") ? "Use email and password instead" : "Use token instead"
                        color: theme.accent; font.family: fontFamily; font.pixelSize: 13
                        MouseArea { anchors.fill: parent; onClicked: { if (app) app.set_login_mode(app.login_mode === "token" ? "credentials" : "token") } }
                    }
                }
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // HOME VIEW (guild strip + channels/DMs)
    // ═══════════════════════════════════════════════════════════════
    Component {
        id: homeView
        Rectangle {
            color: theme.bgBase

            RowLayout {
                anchors.fill: parent
                spacing: 0

                // Guild strip
                Rectangle {
                    Layout.preferredWidth: theme.guildBarWidth
                    Layout.fillHeight: true
                    color: theme.bgTertiary

                    ColumnLayout {
                        anchors.fill: parent; anchors.topMargin: 8; spacing: 2

                        // Home button
                        Item {
                            Layout.preferredWidth: theme.guildBarWidth; Layout.preferredHeight: 48; Layout.alignment: Qt.AlignHCenter
                            Rectangle {
                                width: 4; height: currentGuildId === "" ? 28 : 0; radius: 2
                                color: theme.textNormal; anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter; visible: currentGuildId === ""
                            }
                            GuildIcon {
                                anchors.centerIn: parent; text: "\u{2302}"; fontSize: 14; isActive: currentGuildId === ""
                                onClicked: {
                                    cacheCurrentChannels(); cacheCurrentMessages()
                                    if (currentGuildId !== "" && currentChannelId !== "") { var lcm = lastChannelPerGuild; lcm[currentGuildId] = currentChannelId; lastChannelPerGuild = lcm }
                                    currentGuildId = ""; currentGuildName = "Direct Messages"; currentChannelId = ""; currentChannelName = ""
                                    messageModel.clear()
                                    if (dmChannelCache.length > 0 && dmChannelModel.count === 0) { for (var di = 0; di < dmChannelCache.length; di++) dmChannelModel.append(dmChannelCache[di]) }
                                    if (app) app.select_guild("")
                                }
                            }
                        }

                        Rectangle { Layout.preferredHeight: 2; Layout.preferredWidth: 28; Layout.alignment: Qt.AlignHCenter; Layout.topMargin: 2; Layout.bottomMargin: 2; radius: 1; color: theme.separator }

                        ListView {
                            Layout.fillWidth: true; Layout.fillHeight: true; model: guildModel; clip: true; spacing: 2
                            delegate: Item {
                                width: ListView.view.width; height: 48
                                Rectangle {
                                    width: 4; height: currentGuildId === model.guildId ? 28 : 8; radius: 2
                                    color: theme.textNormal; anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
                                    visible: currentGuildId === model.guildId
                                }
                                GuildIcon {
                                    anchors.centerIn: parent; text: model.name ? model.name.charAt(0).toUpperCase() : "?"
                                    iconUrl: model.iconUrl || ""; isActive: currentGuildId === model.guildId
                                    onClicked: {
                                        cacheCurrentChannels(); cacheCurrentMessages()
                                        if (currentGuildId !== "" && currentChannelId !== "") { var lcm = lastChannelPerGuild; lcm[currentGuildId] = currentChannelId; lastChannelPerGuild = lcm }
                                        currentGuildId = model.guildId; currentGuildName = model.name; memberModel.clear()
                                        var hasCachedChannels = restoreChannelsFromCache(model.guildId)
                                        var lastCh = lastChannelPerGuild[model.guildId]
                                        if (lastCh && hasCachedChannels) {
                                            currentChannelId = lastCh
                                            for (var ci = 0; ci < channelModel.count; ci++) {
                                                if (channelModel.get(ci).channelId === lastCh) {
                                                    currentChannelName = channelModel.get(ci).name; currentChannelType = channelModel.get(ci).channelType; break
                                                }
                                            }
                                            restoreMessagesFromCache(lastCh)
                                        } else {
                                            currentChannelId = ""; currentChannelName = ""
                                            if (!hasCachedChannels) channelModel.clear(); messageModel.clear()
                                        }
                                        if (app) app.select_guild(model.guildId)
                                    }
                                }

                                // Unread indicator
                                DBadge {
                                    visible: (model.mentionCount || 0) > 0
                                    count: model.mentionCount || 0
                                    bgColor: theme.mentionPillBg
                                    anchors.right: parent.right; anchors.rightMargin: 2; anchors.bottom: parent.bottom; anchors.bottomMargin: 4
                                }
                            }
                        }

                        // User avatar at bottom
                        Item {
                            Layout.preferredWidth: theme.guildBarWidth; Layout.preferredHeight: 48
                            DAvatar { anchors.centerIn: parent; size: 32; imageUrl: currentUserAvatar; fallbackText: currentUserName || "?" }
                        }
                    }
                }

                // Channel / DM list
                Rectangle {
                    Layout.fillWidth: true; Layout.fillHeight: true; color: theme.bgSecondary

                    ColumnLayout {
                        anchors.fill: parent; spacing: 0

                        // Header
                        Rectangle {
                            Layout.fillWidth: true; Layout.preferredHeight: theme.headerHeight; color: theme.bgSecondary
                            Text {
                                anchors.left: parent.left; anchors.leftMargin: 12; anchors.verticalCenter: parent.verticalCenter
                                text: currentGuildId === "" ? "Direct Messages" : currentGuildName
                                color: theme.textNormal; font.family: fontFamily; font.pixelSize: 16; font.bold: true; elide: Text.ElideRight
                                width: parent.width - 60
                            }
                            // Settings gear
                            Rectangle {
                                width: 32; height: 32; radius: 16; anchors.right: parent.right; anchors.rightMargin: 8; anchors.verticalCenter: parent.verticalCenter
                                color: settingsMa.pressed ? theme.bgActive : "transparent"
                                Text { anchors.centerIn: parent; text: "\u{2699}"; font.pixelSize: 18; color: theme.textMuted }
                                MouseArea { id: settingsMa; anchors.fill: parent; onClicked: navStack.push(settingsPage) }
                            }
                            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.separator }
                        }

                        // DM list (when home selected)
                        ListView {
                            Layout.fillWidth: true; Layout.fillHeight: true; visible: currentGuildId === ""; model: dmChannelModel; clip: true
                            delegate: Rectangle {
                                width: ListView.view.width - 8; x: 4; height: 52; radius: theme.radiusSmall
                                color: currentChannelId === model.channelId ? theme.bgActive : "transparent"
                                RowLayout {
                                    anchors.fill: parent; anchors.leftMargin: 8; anchors.rightMargin: 8; spacing: 10
                                    Item {
                                        Layout.preferredWidth: 36; Layout.preferredHeight: 36
                                        DAvatar { anchors.fill: parent; size: 36; imageUrl: model.recipientAvatarUrl || ""; fallbackText: model.recipientName || "?" }
                                        Rectangle { width: 10; height: 10; radius: 5; anchors.right: parent.right; anchors.bottom: parent.bottom; border.width: 2; border.color: theme.bgSecondary; color: getStatusColor((presenceVersion, app ? app.get_user_status(model.recipientId || "") : "")) }
                                    }
                                    Text { Layout.fillWidth: true; text: model.recipientName || "Unknown"; color: theme.textNormal; font.family: fontFamily; font.pixelSize: 15; elide: Text.ElideRight }
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: {
                                        currentChannelId = model.channelId; currentChannelName = model.recipientName || "DM"; currentChannelType = 1
                                        messageModel.clear()
                                        if (!restoreMessagesFromCache(model.channelId)) { if (app) app.select_channel(model.channelId, 1) }
                                        else { if (app) app.select_channel(model.channelId, 1) }
                                        navStack.push(messagesView)
                                    }
                                }
                            }
                        }

                        // Channel list (when guild selected)
                        ListView {
                            Layout.fillWidth: true; Layout.fillHeight: true; visible: currentGuildId !== ""; model: channelModel; clip: true
                            delegate: Item {
                                width: ListView.view.width; height: model.channelType === 4 ? 32 : 44
                                // Category header
                                Rectangle {
                                    visible: model.channelType === 4; anchors.fill: parent; color: "transparent"
                                    RowLayout {
                                        anchors.left: parent.left; anchors.leftMargin: 8; anchors.right: parent.right; anchors.rightMargin: 8; anchors.verticalCenter: parent.verticalCenter; spacing: 4
                                        Text { text: "\u{25BC}"; color: theme.textFaint; font.pixelSize: 8 }
                                        Text { text: (model.name || "").toUpperCase(); color: theme.textFaint; font.family: fontFamily; font.pixelSize: 11; font.bold: true; elide: Text.ElideRight; Layout.fillWidth: true }
                                    }
                                    MouseArea { anchors.fill: parent; onClicked: toggleCategory(currentGuildId, model.channelId) }
                                }
                                // Channel item
                                Rectangle {
                                    visible: model.channelType !== 4; anchors.fill: parent; anchors.leftMargin: 4; anchors.rightMargin: 4
                                    radius: theme.radiusSmall; color: currentChannelId === model.channelId ? theme.bgActive : "transparent"
                                    RowLayout {
                                        anchors.fill: parent; anchors.leftMargin: 10; anchors.rightMargin: 8; spacing: 6
                                        Text {
                                            text: (model.channelType === 2 || model.channelType === 13) ? "\u{1F50A}" : "#"
                                            color: theme.channelIcon; font.pixelSize: 16; Layout.preferredWidth: 20
                                        }
                                        Text {
                                            Layout.fillWidth: true; text: model.name || ""; color: model.hasUnread ? theme.textNormal : theme.textMuted
                                            font.family: fontFamily; font.pixelSize: 15; font.weight: model.hasUnread ? Font.Medium : Font.Normal; elide: Text.ElideRight
                                        }
                                        DBadge { visible: (model.mentionCount || 0) > 0; count: model.mentionCount || 0; bgColor: theme.mentionPillBg }
                                    }
                                    MouseArea {
                                        anchors.fill: parent
                                        onClicked: {
                                            if (model.channelType === 4) { toggleCategory(currentGuildId, model.channelId); return }
                                            cacheCurrentMessages()
                                            currentChannelId = model.channelId; currentChannelName = model.name; currentChannelType = model.channelType
                                            messageModel.clear(); clearReply()
                                            if (!restoreMessagesFromCache(model.channelId)) { if (app) app.select_channel(model.channelId, model.channelType) }
                                            else { if (app) app.select_channel(model.channelId, model.channelType) }
                                            navStack.push(messagesView)
                                        }
                                    }
                                }
                            }
                        }

                        // Voice connection bar (shown when connected to voice)
                        Rectangle {
                            visible: isVoiceConnected || voiceConnectionState.indexOf("connecting") >= 0
                            Layout.fillWidth: true; Layout.preferredHeight: visible ? 48 : 0; color: theme.bgTertiary
                            RowLayout {
                                anchors.fill: parent; anchors.leftMargin: 12; anchors.rightMargin: 12; spacing: 8
                                Rectangle { width: 8; height: 8; radius: 4; color: voiceConnectionState === "connected" ? theme.voicePositive : theme.voiceConnecting }
                                Text { Layout.fillWidth: true; text: voiceConnectionState === "connected" ? "Voice Connected" : "Connecting..."; color: voiceConnectionState === "connected" ? theme.voicePositive : theme.voiceConnecting; font.family: fontFamily; font.pixelSize: 13; font.bold: true }
                                Rectangle {
                                    width: 32; height: 32; radius: 16; color: theme.danger
                                    Text { anchors.centerIn: parent; text: "\u{2716}"; color: "#fff"; font.pixelSize: 12 }
                                    MouseArea { anchors.fill: parent; onClicked: { if (app) app.leave_voice() } }
                                }
                            }
                        }

                        // User panel
                        Rectangle {
                            Layout.fillWidth: true; Layout.preferredHeight: theme.userPanelHeight; color: theme.bgTertiary
                            Rectangle { anchors.top: parent.top; width: parent.width; height: 1; color: theme.separator }
                            RowLayout {
                                anchors.fill: parent; anchors.leftMargin: 10; anchors.rightMargin: 10; spacing: 8
                                Item {
                                    Layout.preferredWidth: 32; Layout.preferredHeight: 32
                                    DAvatar { anchors.fill: parent; size: 32; imageUrl: currentUserAvatar; fallbackText: currentUserName || "?" }
                                    Rectangle { width: 10; height: 10; radius: 5; anchors.right: parent.right; anchors.bottom: parent.bottom; border.width: 2; border.color: theme.bgTertiary; color: getStatusColor(currentStatus) }
                                }
                                Text { Layout.fillWidth: true; text: currentUserName; color: theme.textNormal; font.family: fontFamily; font.pixelSize: 14; elide: Text.ElideRight }
                            }
                        }
                    }
                }
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // MESSAGES VIEW
    // ═══════════════════════════════════════════════════════════════
    Component {
        id: messagesView
        Rectangle {
            color: theme.bgBase

            ColumnLayout {
                anchors.fill: parent; spacing: 0

                // Header
                Rectangle {
                    Layout.fillWidth: true; Layout.preferredHeight: theme.headerHeight; color: theme.bgBase
                    RowLayout {
                        anchors.fill: parent; anchors.leftMargin: 4; anchors.rightMargin: 8; spacing: 4
                        // Back button
                        Rectangle {
                            Layout.preferredWidth: 44; Layout.preferredHeight: 44; color: backMa.pressed ? theme.bgActive : "transparent"; radius: theme.radiusSmall
                            Text { anchors.centerIn: parent; text: "\u{2190}"; color: theme.textNormal; font.pixelSize: 22 }
                            MouseArea { id: backMa; anchors.fill: parent; onClicked: navStack.pop() }
                        }
                        Text {
                            text: (currentChannelType === 1 || currentChannelType === 3) ? ("@ " + currentChannelName) : ("# " + currentChannelName)
                            color: theme.textNormal; font.family: fontFamily; font.pixelSize: 16; font.bold: true; elide: Text.ElideRight; Layout.fillWidth: true
                        }
                        // Members button (guild channels)
                        Rectangle {
                            visible: currentGuildId !== ""; Layout.preferredWidth: 36; Layout.preferredHeight: 36; radius: 18
                            color: membersBtnMa.pressed ? theme.bgActive : "transparent"
                            Text { anchors.centerIn: parent; text: "\u{1F465}"; font.pixelSize: 16 }
                            MouseArea { id: membersBtnMa; anchors.fill: parent; onClicked: navStack.push(membersView) }
                        }
                    }
                    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.separator }
                }

                // Message list
                ListView {
                    id: messageList
                    Layout.fillWidth: true; Layout.fillHeight: true; model: messageModel
                    focus: false; clip: true; verticalLayoutDirection: ListView.BottomToTop; spacing: 0
                    boundsBehavior: Flickable.StopAtBounds

                    property bool hasMoreHistory: true
                    property bool isLoadingMore: false

                    onContentYChanged: {
                        if (!hasMoreHistory || isLoadingMore) return
                        if (messageModel.count === 0) return
                        if (contentY >= contentHeight - height - 300) {
                            isLoadingMore = true
                            var oldest = messageModel.get(messageModel.count - 1)
                            if (oldest && oldest.messageId && app) app.load_more_messages(currentChannelId, oldest.messageId)
                        }
                    }

                    footer: Item {
                        width: messageList.width; focus: false
                        height: messageList.isLoadingMore ? 48 : (!messageList.hasMoreHistory && messageModel.count > 0) ? 48 : (messageList.hasMoreHistory ? 52 : 0)
                        Text { anchors.centerIn: parent; visible: messageList.isLoadingMore; text: "Loading..."; color: theme.textMuted; font.family: fontFamily; font.pixelSize: 12 }
                        Text { anchors.centerIn: parent; visible: !messageList.hasMoreHistory && messageModel.count > 0; text: currentChannelName ? ("Beginning of #" + currentChannelName) : "Beginning of conversation"; color: theme.textFaint; font.family: fontFamily; font.pixelSize: 11 }
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
                            background: Rectangle { color: "transparent"; radius: 4 }
                            contentItem: Text { text: parent.text; color: theme.textMuted; font: parent.font; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter }
                        }
                    }

                    ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded; contentItem: Rectangle { implicitWidth: 4; radius: 2; color: theme.textFaint; opacity: parent.active ? 0.6 : 0 } }

                    // Date helpers
                    function localDateStringFromTimestamp(ts) {
                        if (!ts) return ""; var d = new Date(ts); if (isNaN(d.getTime())) return ""
                        var y = d.getFullYear(), m = d.getMonth() + 1, day = d.getDate()
                        return y + "-" + (m < 10 ? "0" : "") + m + "-" + (day < 10 ? "0" : "") + day
                    }
                    function formatDateLabel(dateStr) {
                        var d = new Date(dateStr + "T00:00:00"); if (isNaN(d.getTime())) return dateStr
                        var now = new Date(); var today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
                        var yesterday = new Date(today); yesterday.setDate(today.getDate() - 1)
                        var target = new Date(d.getFullYear(), d.getMonth(), d.getDate())
                        if (target.getTime() === today.getTime()) return "Today"
                        if (target.getTime() === yesterday.getTime()) return "Yesterday"
                        var months = ["January","February","March","April","May","June","July","August","September","October","November","December"]
                        return months[d.getMonth()] + " " + d.getDate() + ", " + d.getFullYear()
                    }
                    function formatTimestamp(ts) {
                        if (!ts) return ""; var d = new Date(ts); if (isNaN(d.getTime())) return ts
                        var hrs = d.getHours(), mins = d.getMinutes(), ampm = hrs >= 12 ? "PM" : "AM"
                        hrs = hrs % 12; if (hrs === 0) hrs = 12
                        var timeStr = hrs + ":" + (mins < 10 ? "0" : "") + mins + " " + ampm
                        var now = new Date(); var today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
                        var yesterday = new Date(today); yesterday.setDate(today.getDate() - 1)
                        var msgDate = new Date(d.getFullYear(), d.getMonth(), d.getDate())
                        if (msgDate.getTime() === today.getTime()) return "Today at " + timeStr
                        if (msgDate.getTime() === yesterday.getTime()) return "Yesterday at " + timeStr
                        var mm = d.getMonth() + 1, dd = d.getDate()
                        return (mm < 10 ? "0" : "") + mm + "/" + (dd < 10 ? "0" : "") + dd + "/" + d.getFullYear() + " " + timeStr
                    }
                    function formatTimeOnly(ts) {
                        if (!ts) return ""; var d = new Date(ts); if (isNaN(d.getTime())) return ""
                        var hrs = d.getHours(), mins = d.getMinutes(), ampm = hrs >= 12 ? "PM" : "AM"
                        hrs = hrs % 12; if (hrs === 0) hrs = 12
                        return hrs + ":" + (mins < 10 ? "0" : "") + mins + " " + ampm
                    }
                    function parseJsonArray(s) { if (!s || typeof s !== "string") return []; try { var a = JSON.parse(s); return Array.isArray(a) ? a : [] } catch (e) { return [] } }
                    function parseReactions(r) { if (!r) return []; if (typeof r === "string") { try { var a = JSON.parse(r); return Array.isArray(a) ? a : [] } catch (e) { return [] } } return Array.isArray(r) ? r : [] }
                    function isDayBoundary(idx) {
                        if (idx < 0 || idx >= messageModel.count) return false
                        if (idx + 1 >= messageModel.count) return true
                        return localDateStringFromTimestamp(messageModel.get(idx).timestamp || "") !== localDateStringFromTimestamp(messageModel.get(idx + 1).timestamp || "")
                    }
                    function isCondensed(idx) {
                        if (idx < 0 || idx >= messageModel.count || isDayBoundary(idx)) return false
                        if (idx + 1 >= messageModel.count) return false
                        var curr = messageModel.get(idx), prev = messageModel.get(idx + 1)
                        if (!curr || !prev || curr.authorId !== prev.authorId) return false
                        if (curr.isDeleted || prev.isDeleted) return false
                        var t1 = curr.timestamp || "", t2 = prev.timestamp || ""
                        if (t1 === t2) return true
                        var m1 = t1.match(/:(\d{2})/), m2 = t2.match(/:(\d{2})/)
                        if (m1 && m2) { var h1 = t1.match(/(\d{1,2}):/), h2 = t2.match(/(\d{1,2}):/); if (h1 && h2 && h1[1] === h2[1]) return Math.abs(parseInt(m1[1]) - parseInt(m2[1])) <= 5 }
                        return false
                    }

                    delegate: Rectangle {
                        id: msgDelegate
                        width: messageList.width; focus: false
                        property bool condensed: messageList.isCondensed(index)
                        property bool showDayDivider: messageList.isDayBoundary(index)
                        property bool isReply: (model && model.messageType !== undefined ? model.messageType : 0) === 19
                        property bool isSystemMsg: { var t = (model && model.messageType !== undefined) ? model.messageType : 0; return t !== 0 && t !== 19 && t !== 20 && t !== 23 }
                        property bool isMentioned: model ? (model.mentionsMe || false) : false
                        property bool hasReplyPreview: isReply && (model ? (model.replyContent || "").length : 0) > 0
                        property var _reactionsCheck: model ? model.reactions : null
                        property bool hasReactions: false
                        on_ReactionsCheckChanged: hasReactions = !isSystemMsg && messageList.parseReactions(_reactionsCheck).length > 0
                        Component.onCompleted: hasReactions = !isSystemMsg && messageList.parseReactions(_reactionsCheck).length > 0

                        height: (showDayDivider ? 32 : 0)
                            + (hasReplyPreview && !condensed ? 22 : 0)
                            + (isSystemMsg ? 32 : condensed ? (compactText.implicitHeight || 14) + 4 : (msgCol.implicitHeight || 20) + 12)
                            + (hasReactions ? 28 : 0)
                        color: isMentioned ? "#faa61a12" : "transparent"

                        Rectangle { visible: isMentioned; anchors.left: parent.left; anchors.top: parent.top; anchors.bottom: parent.bottom; width: 3; color: "#FAA61A"; radius: 1 }

                        // Day divider
                        Item {
                            id: dayDivider; visible: showDayDivider; anchors.top: parent.top; anchors.left: parent.left; anchors.right: parent.right; anchors.topMargin: 4; height: visible ? 24 : 0
                            Rectangle { anchors.left: parent.left; anchors.right: parent.right; anchors.leftMargin: 12; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; height: 1; color: theme.separator }
                            Rectangle {
                                anchors.centerIn: parent; width: dayLabel.implicitWidth + 12; height: 16; color: theme.bgBase; radius: 8
                                Text { id: dayLabel; anchors.centerIn: parent; text: messageList.formatDateLabel(messageList.localDateStringFromTimestamp(model.timestamp || "")); color: theme.textMuted; font.family: fontFamily; font.pixelSize: 10; font.weight: Font.Medium }
                            }
                        }

                        // Reply bar
                        Row {
                            id: replyBar; visible: hasReplyPreview && !condensed
                            anchors.top: showDayDivider ? dayDivider.bottom : parent.top; anchors.left: parent.left; anchors.leftMargin: 52; anchors.right: parent.right; anchors.rightMargin: 12; anchors.topMargin: 4; spacing: 4
                            DAvatar { size: 14; imageUrl: model.replyAuthorAvatarUrl || ""; fallbackText: model.replyAuthorName || "?"; anchors.verticalCenter: parent.verticalCenter }
                            Text { text: model.replyAuthorName || ""; color: theme.textSecondary; font.family: fontFamily; font.pixelSize: 11; font.weight: Font.Medium; anchors.verticalCenter: parent.verticalCenter }
                            Text {
                                text: { var rc = model.replyContent || ""; return rc.length > 60 ? rc.substring(0, 60) + "..." : rc }
                                color: theme.textFaint; font.family: fontFamily; font.pixelSize: 11; elide: Text.ElideRight; anchors.verticalCenter: parent.verticalCenter
                            }
                        }

                        // System message
                        Row {
                            id: systemMsgRow; visible: isSystemMsg
                            anchors.top: showDayDivider ? dayDivider.bottom : parent.top; anchors.left: parent.left; anchors.right: parent.right; anchors.leftMargin: 12; anchors.rightMargin: 12; anchors.topMargin: 4; spacing: 8
                            Text {
                                text: {
                                    var t = model.messageType || 0
                                    if (t === 1 || t === 2) return "\u{1F465}"
                                    if (t === 3) return "\u{1F4DE}"
                                    if (t === 4) return "\u{270F}"
                                    if (t === 5) return "\u{1F5BC}"
                                    if (t === 6) return "\u{1F4CC}"
                                    if (t === 7) return "\u{2192}"
                                    if (t >= 8 && t <= 11) return "\u{1F680}"
                                    if (t === 12) return "\u{1F517}"
                                    if (t === 16 || t === 17) return "\u{26A0}"
                                    if (t === 18 || t === 21) return "\u{1F4AC}"
                                    if (t === 22) return "\u{2709}"
                                    if (t === 24) return "\u{1F6E1}"
                                    if (t === 25 || t === 26 || t === 32) return "\u{2728}"
                                    if (t === 27 || t === 28 || t === 29 || t === 31) return "\u{1F3A4}"
                                    if (t >= 36 && t <= 39) return "\u{26A0}"
                                    if (t === 44) return "\u{1F6D2}"
                                    if (t === 46) return "\u{1F4CA}"
                                    return "\u{2139}"
                                }
                                font.pixelSize: 14; color: theme.textFaint; width: 24
                            }
                            Text { text: model.content || "[System message]"; color: theme.textMuted; font.family: fontFamily; font.pixelSize: 12; font.italic: true; wrapMode: Text.WordWrap; width: parent.width - 80 }
                            Text { text: messageList.formatTimestamp(model.timestamp || ""); color: theme.textFaint; font.family: fontFamily; font.pixelSize: 10 }
                        }

                        // Full message
                        Row {
                            id: msgRow; visible: !condensed && !isSystemMsg
                            anchors.top: hasReplyPreview ? replyBar.bottom : (showDayDivider ? dayDivider.bottom : parent.top)
                            anchors.left: parent.left; anchors.right: parent.right; anchors.leftMargin: 12; anchors.rightMargin: 12; anchors.topMargin: hasReplyPreview ? 2 : 8; spacing: 10
                            DAvatar { id: msgAvatar; size: 36; imageUrl: model.authorAvatarUrl || ""; fallbackText: model.authorName || "?" }
                            Column {
                                id: msgCol; width: parent.width - msgAvatar.width - 22; spacing: 2
                                Row {
                                    spacing: 6
                                    Text { text: model.authorName || "Unknown"; color: (model.authorRoleColor && model.authorRoleColor.length > 0) ? model.authorRoleColor : theme.accent; font.family: fontFamily; font.pixelSize: 15; font.weight: Font.Medium }
                                    UserBadges { publicFlags: model.authorPublicFlags || 0; isBot: model.authorBot || false; premiumType: model.authorPremiumType || 0; badgeSize: 14 }
                                    Text { text: messageList.formatTimestamp(model.timestamp || ""); color: theme.textFaint; font.family: fontFamily; font.pixelSize: 11; anchors.verticalCenter: parent.verticalCenter }
                                }
                                Text {
                                    width: parent.width
                                    textFormat: (model.contentHtml && model.contentHtml.length > 0) ? Text.RichText : Text.PlainText
                                    text: (model.contentHtml && model.contentHtml.length > 0) ? model.contentHtml : (model.content || "")
                                    color: (model.isDeleted && deletedMessageStyle !== "faded") ? theme.danger : theme.textNormal
                                    font.family: fontFamily; font.pixelSize: 15; font.strikeout: (model.isDeleted || false) && deletedMessageStyle === "strikethrough"
                                    wrapMode: Text.WordWrap; lineHeight: 1.35; opacity: (model.isDeleted && deletedMessageStyle === "faded") ? 0.5 : 1.0
                                    onLinkActivated: Qt.openUrlExternally(link)
                                }
                                // Embeds
                                Column {
                                    id: embedsCol; width: parent.width; spacing: 6
                                    property var parsed: messageList.parseJsonArray(model ? (model.embedsJson || "[]") : "[]")
                                    Repeater {
                                        model: embedsCol.parsed
                                        delegate: Rectangle {
                                            width: embedsCol.width; implicitHeight: embedInner.implicitHeight + 12; radius: theme.radiusSmall; color: theme.bgSecondary
                                            border.width: 1; border.color: theme.border
                                            Column {
                                                id: embedInner; anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top; anchors.margins: 6; spacing: 4
                                                Text { visible: modelData.title && modelData.title.length > 0; text: modelData.title || ""; color: theme.accent; font.pixelSize: 13; font.bold: true; wrapMode: Text.WordWrap; width: parent.width - 8 }
                                                Text { visible: modelData.description && modelData.description.length > 0; text: modelData.description || ""; color: theme.textNormal; font.pixelSize: 12; wrapMode: Text.WordWrap; width: parent.width - 8; lineHeight: 1.3 }
                                                Image { visible: modelData.image && modelData.image.url; width: Math.min(parent.width - 8, 300); height: visible ? width * 0.5 : 0; source: (modelData.image && modelData.image.url) ? modelData.image.url : ""; fillMode: Image.PreserveAspectFit }
                                                Text { visible: modelData.footer && modelData.footer.text; text: modelData.footer ? modelData.footer.text : ""; color: theme.textFaint; font.pixelSize: 10 }
                                            }
                                        }
                                    }
                                }
                                // Attachments
                                Flow {
                                    id: attFlow; width: parent.width; spacing: 4
                                    property var parsed: messageList.parseJsonArray(model ? (model.attachmentsJson || "") : "")
                                    Repeater {
                                        model: attFlow.parsed
                                        delegate: Column {
                                            spacing: 2
                                            Image {
                                                visible: { var ct = (modelData.content_type || "").toLowerCase(); return ct.indexOf("image") >= 0 || /\.(png|jpe?g|gif|webp)$/.test((modelData.filename || "").toLowerCase()) }
                                                width: Math.min(200, msgCol.width); height: visible ? 120 : 0; fillMode: Image.PreserveAspectFit; source: visible ? (modelData.url || "") : ""
                                            }
                                            Text { text: modelData.filename || "file"; color: theme.accent; font.family: fontFamily; font.pixelSize: 12; elide: Text.ElideMiddle; width: 180 }
                                        }
                                    }
                                }
                            }
                        }

                        // Condensed message
                        Text {
                            id: compactText; visible: condensed && !isSystemMsg
                            anchors.left: parent.left; anchors.right: parent.right; anchors.leftMargin: 58; anchors.rightMargin: 12; anchors.top: parent.top; anchors.topMargin: 1
                            text: model.content || ""; color: (model.isDeleted && deletedMessageStyle !== "faded") ? theme.danger : theme.textNormal
                            font.family: fontFamily; font.pixelSize: 14; font.strikeout: (model.isDeleted || false) && deletedMessageStyle === "strikethrough"
                            wrapMode: Text.WordWrap; lineHeight: 1.25; opacity: (model.isDeleted && deletedMessageStyle === "faded") ? 0.5 : 1.0
                        }

                        // Reactions
                        Row {
                            visible: hasReactions; anchors.left: parent.left; anchors.leftMargin: 58; anchors.right: parent.right; anchors.rightMargin: 12
                            anchors.top: condensed ? compactText.bottom : msgRow.bottom; anchors.topMargin: 4; spacing: 4
                            Repeater {
                                model: messageList.parseReactions(msgDelegate._reactionsCheck)
                                delegate: Rectangle {
                                    width: reactionContent.width + 10; height: 22; radius: 11; color: theme.bgSecondary; border.width: 1; border.color: modelData.me ? theme.accent : theme.border
                                    Row { id: reactionContent; anchors.centerIn: parent; spacing: 3
                                        Twemoji { emoji: modelData.emoji || ""; size: 14 }
                                        Text { text: modelData.count || 0; color: theme.textMuted; font.family: fontFamily; font.pixelSize: 11; anchors.verticalCenter: parent.verticalCenter }
                                    }
                                    MouseArea {
                                        anchors.fill: parent
                                        onClicked: {
                                            if (!app) return; var emojiKey = modelData.emoji || ""
                                            if (modelData.me) app.remove_reaction(model.channelId || currentChannelId, model.messageId || "", emojiKey)
                                            else app.add_reaction(model.channelId || currentChannelId, model.messageId || "", emojiKey)
                                        }
                                    }
                                }
                            }
                        }

                        // Long press context menu
                        MouseArea {
                            anchors.fill: parent; pressAndHoldInterval: 500
                            onPressAndHold: {
                                if (isSystemMsg) return
                                contextMsgId = model.messageId || ""; contextMsgChId = model.channelId || currentChannelId
                                contextMsgContent = model.content || ""; contextMsgAuthorId = model.authorId || ""
                                contextMsgAuthorName = model.authorName || ""
                                msgContextDrawer.open()
                            }
                        }
                    }
                }

                // Typing indicator
                Item {
                    visible: !isVoiceChannel && typingDisplay.length > 0; Layout.fillWidth: true; Layout.preferredHeight: visible ? 18 : 0; Layout.leftMargin: 16
                    Text { anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter; text: typingDisplay; color: theme.textMuted; font.family: fontFamily; font.pixelSize: 11; elide: Text.ElideRight }
                }

                // Reply preview
                Rectangle {
                    visible: !isVoiceChannel && replyToMessageId.length > 0; Layout.fillWidth: true; Layout.preferredHeight: visible ? 36 : 0
                    Layout.leftMargin: 8; Layout.rightMargin: 8; radius: theme.radiusMed; color: theme.bgSecondary
                    RowLayout {
                        anchors.fill: parent; anchors.leftMargin: 10; anchors.rightMargin: 6; spacing: 4
                        Rectangle { width: 2; height: 16; radius: 1; color: theme.accent }
                        Text { text: "Replying to "; color: theme.textMuted; font.family: fontFamily; font.pixelSize: 11 }
                        Text { text: replyToAuthor; color: theme.textNormal; font.family: fontFamily; font.pixelSize: 11; font.weight: Font.Medium }
                        Item { Layout.fillWidth: true }
                        Rectangle {
                            width: 24; height: 24; radius: 12; color: "transparent"
                            Text { anchors.centerIn: parent; text: "\u{2715}"; color: theme.textFaint; font.pixelSize: 10 }
                            MouseArea { anchors.fill: parent; onClicked: clearReply() }
                        }
                    }
                }

                // Message input
                Rectangle {
                    visible: !isVoiceChannel; Layout.fillWidth: true; Layout.preferredHeight: theme.messageInputH
                    Layout.leftMargin: 8; Layout.rightMargin: 8; Layout.bottomMargin: 8; Layout.topMargin: replyToMessageId.length > 0 ? 0 : 4
                    radius: theme.radiusMed; color: theme.inputBg; border.width: messageInput.activeFocus ? 1 : 0; border.color: theme.accent

                    RowLayout {
                        anchors.fill: parent; anchors.leftMargin: 12; anchors.rightMargin: 8; spacing: 8
                        TextField {
                            id: messageInput; Layout.fillWidth: true; Layout.fillHeight: true
                            placeholderText: replyToMessageId ? ("Reply to " + replyToAuthor + "...") : currentChannelName ? "Message #" + currentChannelName : "Type a message..."
                            placeholderTextColor: theme.textFaint; color: theme.textNormal; font.family: fontFamily; font.pixelSize: 15
                            background: Item {}
                            verticalAlignment: TextInput.AlignVCenter; leftPadding: 4
                            Keys.onReturnPressed: sendMessage(); Keys.onEnterPressed: sendMessage()
                            property var lastTypingTime: 0
                            onTextChanged: { var now = Date.now(); if (text.length > 0 && now - lastTypingTime > 8000 && currentChannelId && app) { app.start_typing(currentChannelId); lastTypingTime = now } }
                        }
                        // Send button
                        Rectangle {
                            Layout.preferredWidth: 36; Layout.preferredHeight: 36; radius: 18
                            color: messageInput.text.trim().length > 0 ? theme.accent : theme.bgHover
                            Text { anchors.centerIn: parent; text: "\u{27A4}"; color: messageInput.text.trim().length > 0 ? "#ffffff" : theme.textFaint; font.pixelSize: 16 }
                            MouseArea { anchors.fill: parent; onClicked: sendMessage() }
                        }
                    }
                }
            }

            // Context menu drawer (long-press on message)
            property string contextMsgId: ""
            property string contextMsgChId: ""
            property string contextMsgContent: ""
            property string contextMsgAuthorId: ""
            property string contextMsgAuthorName: ""

            Drawer {
                id: msgContextDrawer
                width: parent.width; height: contextCol.implicitHeight + 32; edge: Qt.BottomEdge
                background: Rectangle { color: theme.bgElevated; radius: theme.radiusLarge }

                ColumnLayout {
                    id: contextCol; anchors.fill: parent; anchors.margins: 12; spacing: 4

                    Rectangle { Layout.preferredWidth: 40; Layout.preferredHeight: 4; Layout.alignment: Qt.AlignHCenter; radius: 2; color: theme.textFaint; Layout.bottomMargin: 8 }

                    Repeater {
                        model: [
                            { label: "Reply", action: "reply" },
                            { label: "Edit", action: "edit", showIf: contextMsgAuthorId === currentUserId },
                            { label: "Delete", action: "delete", showIf: contextMsgAuthorId === currentUserId },
                            { label: "Copy Text", action: "copy" }
                        ]
                        delegate: Rectangle {
                            visible: modelData.showIf !== undefined ? modelData.showIf : true
                            Layout.fillWidth: true; Layout.preferredHeight: 48; radius: theme.radiusSmall; color: ctxMa.pressed ? theme.bgHover : "transparent"
                            Text { anchors.left: parent.left; anchors.leftMargin: 16; anchors.verticalCenter: parent.verticalCenter; text: modelData.label; color: modelData.action === "delete" ? theme.danger : theme.textNormal; font.family: fontFamily; font.pixelSize: 16 }
                            MouseArea {
                                id: ctxMa; anchors.fill: parent
                                onClicked: {
                                    msgContextDrawer.close()
                                    if (modelData.action === "reply") {
                                        replyToMessageId = contextMsgId; replyToAuthor = contextMsgAuthorName; replyToContent = contextMsgContent
                                    } else if (modelData.action === "delete") {
                                        if (app) app.delete_message(contextMsgChId, contextMsgId)
                                    } else if (modelData.action === "edit") {
                                        editMsgId = contextMsgId; editMsgContent = contextMsgContent; editDrawer.open()
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Edit drawer
            property string editMsgId: ""
            property string editMsgContent: ""

            Drawer {
                id: editDrawer; width: parent.width; height: 160; edge: Qt.BottomEdge
                background: Rectangle { color: theme.bgElevated; radius: theme.radiusLarge }
                ColumnLayout {
                    anchors.fill: parent; anchors.margins: 16; spacing: 8
                    Text { text: "Edit Message"; color: theme.textNormal; font.family: fontFamily; font.pixelSize: 16; font.bold: true }
                    TextField {
                        id: editInput; Layout.fillWidth: true; Layout.preferredHeight: 44; text: editMsgContent
                        color: theme.textNormal; font.family: fontFamily; font.pixelSize: 14; leftPadding: 12
                        background: Rectangle { color: theme.inputBg; radius: theme.radiusMed; border.color: editInput.activeFocus ? theme.accent : theme.border }
                        Keys.onReturnPressed: { if (app) app.edit_message(contextMsgChId, editMsgId, editInput.text); editDrawer.close() }
                    }
                    RowLayout {
                        spacing: 8
                        Rectangle {
                            Layout.fillWidth: true; Layout.preferredHeight: 40; radius: theme.radiusMed; color: theme.bgHover
                            Text { anchors.centerIn: parent; text: "Cancel"; color: theme.textNormal; font.family: fontFamily; font.pixelSize: 14 }
                            MouseArea { anchors.fill: parent; onClicked: editDrawer.close() }
                        }
                        Rectangle {
                            Layout.fillWidth: true; Layout.preferredHeight: 40; radius: theme.radiusMed; color: theme.accent
                            Text { anchors.centerIn: parent; text: "Save"; color: "#fff"; font.family: fontFamily; font.pixelSize: 14; font.bold: true }
                            MouseArea {
                                anchors.fill: parent
                                onClicked: { if (app) app.edit_message(contextMsgChId, editMsgId, editInput.text); editDrawer.close() }
                            }
                        }
                    }
                }
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // MEMBERS VIEW
    // ═══════════════════════════════════════════════════════════════
    Component {
        id: membersView
        Rectangle {
            color: theme.bgBase
            ColumnLayout {
                anchors.fill: parent; spacing: 0
                Rectangle {
                    Layout.fillWidth: true; Layout.preferredHeight: theme.headerHeight; color: theme.bgBase
                    RowLayout {
                        anchors.fill: parent; anchors.leftMargin: 4; anchors.rightMargin: 8; spacing: 4
                        Rectangle {
                            Layout.preferredWidth: 44; Layout.preferredHeight: 44; color: memBackMa.pressed ? theme.bgActive : "transparent"; radius: theme.radiusSmall
                            Text { anchors.centerIn: parent; text: "\u{2190}"; color: theme.textNormal; font.pixelSize: 22 }
                            MouseArea { id: memBackMa; anchors.fill: parent; onClicked: navStack.pop() }
                        }
                        Text { text: "Members \u{2014} " + memberModel.count; color: theme.textMuted; font.family: fontFamily; font.pixelSize: 14; Layout.fillWidth: true }
                    }
                    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.separator }
                }
                ListView {
                    Layout.fillWidth: true; Layout.fillHeight: true; model: memberModel; clip: true; spacing: 2
                    delegate: Rectangle {
                        width: ListView.view.width - 8; x: 4; height: Math.max(48, theme.touchTarget); radius: theme.radiusSmall; color: memberRowMa.pressed ? theme.bgActive : "transparent"
                        RowLayout {
                            anchors.fill: parent; anchors.leftMargin: 8; anchors.rightMargin: 8; spacing: 10
                            Item {
                                Layout.preferredWidth: 36; Layout.preferredHeight: 36
                                DAvatar { anchors.fill: parent; size: 36; imageUrl: model.avatarUrl || ""; fallbackText: model.displayName || model.username || "?" }
                                Rectangle { width: 10; height: 10; radius: 5; anchors.right: parent.right; anchors.bottom: parent.bottom; border.width: 2; border.color: theme.bgBase; color: getStatusColor((presenceVersion, app ? app.get_user_status(model.memberId || "") : "")) }
                            }
                            Column {
                                Layout.fillWidth: true; spacing: 2
                                Row {
                                    spacing: 4
                                    Text { text: model.displayName || model.username || "Unknown"; color: (model.roleColor && model.roleColor.length > 0) ? model.roleColor : theme.textNormal; font.family: fontFamily; font.pixelSize: 15; elide: Text.ElideRight }
                                    UserBadges { publicFlags: model.publicFlags || 0; isBot: model.bot || false; premiumType: model.premiumType || 0; badgeSize: 14 }
                                }
                                Text { visible: model.roleName && model.roleName.length > 0; text: model.roleName || ""; color: theme.textFaint; font.family: fontFamily; font.pixelSize: 11 }
                            }
                        }
                        MouseArea {
                            id: memberRowMa
                            anchors.fill: parent
                            onClicked: openUserProfile(model.memberId || "", model.displayName || model.username || "", model.username || "", model.avatarUrl || "", model.roleColor || "")
                        }
                    }
                }
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // SETTINGS PAGE
    // ═══════════════════════════════════════════════════════════════
    Component {
        id: settingsPage
        Rectangle {
            color: theme.bgBase
            ColumnLayout {
                anchors.fill: parent; spacing: 0
                Rectangle {
                    Layout.fillWidth: true; Layout.preferredHeight: theme.headerHeight; color: theme.bgBase
                    RowLayout {
                        anchors.fill: parent; anchors.leftMargin: 4; anchors.rightMargin: 8; spacing: 4
                        Rectangle {
                            Layout.preferredWidth: 44; Layout.preferredHeight: 44; color: setBackMa.pressed ? theme.bgActive : "transparent"; radius: theme.radiusSmall
                            Text { anchors.centerIn: parent; text: "\u{2190}"; color: theme.textNormal; font.pixelSize: 22 }
                            MouseArea { id: setBackMa; anchors.fill: parent; onClicked: navStack.pop() }
                        }
                        Text { text: "Settings"; color: theme.textNormal; font.family: fontFamily; font.pixelSize: 18; font.bold: true; Layout.fillWidth: true }
                    }
                    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.separator }
                }
                Flickable {
                    Layout.fillWidth: true; Layout.fillHeight: true; contentHeight: settingsCol.implicitHeight + 32; clip: true
                    ColumnLayout {
                        id: settingsCol; anchors.left: parent.left; anchors.right: parent.right; anchors.margins: 16; spacing: 12

                        Item { Layout.preferredHeight: 8 }

                        // Status selector
                        Text { text: "STATUS"; color: theme.textFaint; font.family: fontFamily; font.pixelSize: 11; font.bold: true }
                        Row {
                            spacing: 8
                            Repeater {
                                model: [{ s: "online", c: theme.online }, { s: "idle", c: theme.idle }, { s: "dnd", c: theme.dnd }, { s: "invisible", c: theme.offline }]
                                delegate: Rectangle {
                                    width: 64; height: 32; radius: 16; color: currentStatus === modelData.s ? theme.bgActive : theme.bgSecondary
                                    border.width: currentStatus === modelData.s ? 1 : 0; border.color: modelData.c
                                    Row { anchors.centerIn: parent; spacing: 4
                                        Rectangle { width: 8; height: 8; radius: 4; color: modelData.c; anchors.verticalCenter: parent.verticalCenter }
                                        Text { text: modelData.s.charAt(0).toUpperCase() + modelData.s.slice(1); color: theme.textNormal; font.family: fontFamily; font.pixelSize: 11; anchors.verticalCenter: parent.verticalCenter }
                                    }
                                    MouseArea { anchors.fill: parent; onClicked: { currentStatus = modelData.s; if (app) app.set_status(modelData.s) } }
                                }
                            }
                        }

                        DSeparator {}

                        // Deleted message style
                        Text { text: "DELETED MESSAGE STYLE"; color: theme.textFaint; font.family: fontFamily; font.pixelSize: 11; font.bold: true }
                        Row {
                            spacing: 8
                            Repeater {
                                model: ["strikethrough", "faded", "deleted"]
                                delegate: Rectangle {
                                    width: delStyleText.implicitWidth + 16; height: 32; radius: 16; color: deletedMessageStyle === modelData ? theme.bgActive : theme.bgSecondary
                                    Text { id: delStyleText; anchors.centerIn: parent; text: modelData; color: theme.textNormal; font.family: fontFamily; font.pixelSize: 12 }
                                    MouseArea { anchors.fill: parent; onClicked: { deletedMessageStyle = modelData; if (app) app.set_deleted_message_style(modelData) } }
                                }
                            }
                        }

                        DSeparator {}

                        // Logout
                        Rectangle {
                            Layout.fillWidth: true; Layout.preferredHeight: 48; radius: theme.radiusMed; color: theme.danger
                            Text { anchors.centerIn: parent; text: "Log Out"; color: "#ffffff"; font.family: fontFamily; font.pixelSize: 16; font.bold: true }
                            MouseArea { anchors.fill: parent; onClicked: { if (app) app.logout(); navStack.replace(null, loginPage) } }
                        }

                        Item { Layout.preferredHeight: 32 }
                    }
                }
            }
        }
    }

    // ─── Profile Drawer (member profile from sidebar) ───
    Drawer {
        id: profileDrawer
        width: root.width
        height: Math.min(root.height * 0.92, 640)
        edge: Qt.BottomEdge
        dim: true
        background: Rectangle { color: theme.bgSecondary; radius: theme.radiusLarge }
        onClosed: { profilePopupTarget = null; loadedProfileData = null; profileLoadPending = false }

        ColumnLayout {
            anchors.fill: parent; spacing: 0
            Rectangle {
                Layout.fillWidth: true; Layout.preferredHeight: theme.headerHeight
                color: theme.bgSecondary
                RowLayout {
                    anchors.fill: parent; anchors.leftMargin: 4; anchors.rightMargin: 8
                    Rectangle {
                        Layout.preferredWidth: 44; Layout.preferredHeight: 44
                        color: profileCloseMa.pressed ? theme.bgActive : "transparent"
                        radius: theme.radiusSmall
                        Text { anchors.centerIn: parent; text: "\u{2715}"; color: theme.textNormal; font.pixelSize: 22 }
                        MouseArea { id: profileCloseMa; anchors.fill: parent; onClicked: profileDrawer.close() }
                    }
                    Text {
                        text: "Profile"
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 18
                        font.bold: true
                        Layout.fillWidth: true
                    }
                }
                Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: theme.separator }
            }
            Flickable {
                Layout.fillWidth: true
                Layout.fillHeight: true
                contentWidth: width
                contentHeight: profileCol.implicitHeight + 24
                clip: true
                Column {
                    id: profileCol
                    width: parent.width - 24
                    anchors.horizontalCenter: parent.horizontalCenter
                    spacing: 0
                    topPadding: 12

                    property var pUser: loadedProfileData && loadedProfileData.user ? loadedProfileData.user : null
                    property var pGuild: loadedProfileData && (loadedProfileData.guild_member_profile || loadedProfileData.guild_member) ? (loadedProfileData.guild_member_profile || loadedProfileData.guild_member) : null
                    property string pDisplayName: (pGuild && pGuild.nick) ? pGuild.nick : (profilePopupTarget ? profilePopupTarget.displayName : (pUser ? (pUser.global_name || pUser.username || "") : ""))
                    property string pUsername: pUser ? pUser.username : (profilePopupTarget ? profilePopupTarget.username : "")
                    property string pUserId: profilePopupTarget ? profilePopupTarget.userId : (pUser ? pUser.id : "")
                    property string pAvatarUrl: profilePopupTarget && profilePopupTarget.avatarUrl ? profilePopupTarget.avatarUrl : (pUser && pUser.avatar ? "https://cdn.discordapp.com/avatars/" + pUser.id + "/" + pUser.avatar + ".png?size=128" : "")
                    property bool pIsOwner: !!(loadedProfileData && ((loadedProfileData.guild_member_profile && loadedProfileData.guild_member_profile.is_owner) || loadedProfileData.is_owner))
                    property bool dataReady: !profileLoadPending && (loadedProfileData !== null || !profilePopupTarget)

                    Rectangle {
                        width: profileCol.width
                        height: 80
                        radius: theme.radiusMed
                        color: (pUser && pUser.accent_color != null) ? ("#" + ("000000" + (pUser.accent_color >>> 0).toString(16)).slice(-6)) : theme.accent
                        visible: true
                        Image {
                            anchors.fill: parent
                            source: (pUser && loadedProfileData && loadedProfileData.banner) ? ("https://cdn.discordapp.com/banners/" + (pUser.id || "") + "/" + loadedProfileData.banner + ".png?size=480") : ""
                            fillMode: Image.PreserveAspectCrop
                            visible: status === Image.Ready
                        }
                    }
                    Item { width: 1; height: 12 }
                    Row {
                        width: parent.width
                        spacing: 8
                        Item {
                            width: 72; height: 72
                            DAvatar {
                                anchors.fill: parent
                                size: 72
                                imageUrl: profileCol.pAvatarUrl
                                fallbackText: (profileCol.pDisplayName || "?").charAt(0).toUpperCase()
                            }
                        }
                        Column {
                            spacing: 2
                            Row { spacing: 6
                                Text {
                                    text: profileCol.pDisplayName || "Unknown"
                                    color: theme.headerPrimary
                                    font.family: fontFamily
                                    font.pixelSize: 18
                                    font.bold: true
                                }
                                Text {
                                    visible: profileCol.dataReady && profileCol.pIsOwner
                                    text: "\uD83D\uDC51"
                                    font.pixelSize: 16
                                    color: theme.headerPrimary
                                }
                            }
                            Text {
                                visible: profileCol.pUsername.length > 0
                                text: profileCol.pUsername
                                color: theme.textSecondary
                                font.family: fontFamily
                                font.pixelSize: 13
                            }
                        }
                    }
                    Item { width: 1; height: profileLoadPending ? 24 : 8 }
                    BusyIndicator {
                        width: 24; height: 24
                        running: profileLoadPending
                        anchors.horizontalCenter: parent.horizontalCenter
                    }
                    Item { visible: profileCol.dataReady && loadedProfileData && loadedProfileData.bio; width: 1; height: 12 }
                    Text {
                        visible: profileCol.dataReady && loadedProfileData && loadedProfileData.bio
                        text: loadedProfileData ? loadedProfileData.bio : ""
                        color: theme.textNormal
                        font.family: fontFamily
                        font.pixelSize: 13
                        wrapMode: Text.WordWrap
                        width: parent.width
                    }
                    Item { visible: profileCol.dataReady && profileCol.pGuild && profileCol.pGuild.joined_at; width: 1; height: 8 }
                    Text {
                        visible: profileCol.dataReady && profileCol.pGuild && profileCol.pGuild.joined_at
                        text: "Joined server: " + (profileCol.pGuild ? profileCol.pGuild.joined_at : "").substring(0, 10)
                        color: theme.textMuted
                        font.family: fontFamily
                        font.pixelSize: 12
                        width: parent.width
                    }
                    Item { visible: profileCol.dataReady && profileCol.pGuild && profileCol.pGuild.roles && profileCol.pGuild.roles.length > 0; width: 1; height: 8 }
                    Text {
                        visible: profileCol.dataReady && profileCol.pGuild && profileCol.pGuild.roles && profileCol.pGuild.roles.length > 0
                        text: "Roles: " + (profileCol.pGuild && profileCol.pGuild.roles ? profileCol.pGuild.roles.map(function(r) { return r && r.name ? r.name : r }).join(", ") : "")
                        color: theme.textMuted
                        font.family: fontFamily
                        font.pixelSize: 12
                        wrapMode: Text.WordWrap
                        width: parent.width
                    }
                    Item { visible: profileCol.dataReady && ((loadedProfileData && loadedProfileData.mutual_guilds && loadedProfileData.mutual_guilds.length > 0) || (loadedProfileData && loadedProfileData.mutual_friends && loadedProfileData.mutual_friends.length > 0)); width: 1; height: 8 }
                    Text {
                        visible: profileCol.dataReady && ((loadedProfileData && loadedProfileData.mutual_guilds && loadedProfileData.mutual_guilds.length > 0) || (loadedProfileData && loadedProfileData.mutual_friends && loadedProfileData.mutual_friends.length > 0))
                        text: (loadedProfileData && loadedProfileData.mutual_guilds && loadedProfileData.mutual_guilds.length > 0 ? "Mutual servers: " + loadedProfileData.mutual_guilds.length + " " : "") + (loadedProfileData && loadedProfileData.mutual_friends && loadedProfileData.mutual_friends.length > 0 ? "Mutual friends: " + loadedProfileData.mutual_friends.length : "")
                        color: theme.textMuted
                        font.family: fontFamily
                        font.pixelSize: 12
                        width: parent.width
                    }
                    Item { visible: profileCol.dataReady && profileCol.pGuild && profileCol.pGuild.permission_names && profileCol.pGuild.permission_names.length > 0; width: 1; height: 8 }
                    Text {
                        visible: profileCol.dataReady && profileCol.pGuild && profileCol.pGuild.permission_names && profileCol.pGuild.permission_names.length > 0
                        text: "Permissions: " + (profileCol.pGuild ? profileCol.pGuild.permission_names.join(", ") : "")
                        color: theme.textMuted
                        font.family: fontFamily
                        font.pixelSize: 11
                        wrapMode: Text.WordWrap
                        width: parent.width
                    }
                    Item { visible: profileCol.dataReady && profilePopupTarget && profilePopupTarget.userId !== currentUserId; width: 1; height: 16 }
                    Row {
                        visible: profileCol.dataReady && profilePopupTarget && profilePopupTarget.userId !== currentUserId
                        width: parent.width
                        spacing: 8
                        Rectangle {
                            width: (parent.width - 8) / 2
                            height: theme.touchTarget
                            radius: theme.radiusSmall
                            color: theme.accent
                            Text { anchors.centerIn: parent; text: "Message"; color: "#ffffff"; font.family: fontFamily; font.pixelSize: 14; font.weight: Font.Medium }
                            MouseArea {
                                anchors.fill: parent
                                onClicked: {
                                    if (app && profileCol.pUserId) app.open_dm(profileCol.pUserId)
                                    profileDrawer.close()
                                }
                            }
                        }
                        Rectangle {
                            width: (parent.width - 8) / 2
                            height: theme.touchTarget
                            radius: theme.radiusSmall
                            color: theme.bgTertiary
                            Text { anchors.centerIn: parent; text: "Add Friend"; color: theme.positive; font.family: fontFamily; font.pixelSize: 14; font.weight: Font.Medium }
                            MouseArea {
                                anchors.fill: parent
                                onClicked: {
                                    if (app && profileCol.pUsername.length > 0) app.send_friend_request(profileCol.pUsername)
                                    profileDrawer.close()
                                }
                            }
                        }
                    }
                    Item { width: 1; height: 24 }
                }
            }
        }
    }
}
