// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "components" 1.0

Window {
    id: root
    visible: true
    width: 1100
    height: 700
    minimumWidth: 700
    minimumHeight: 400
    title: "[TEST MODE] Skunkcord"
    color: theme.bgBase

    // Test mode banner color
    readonly property color testBannerColor: "#ff6b35"
    readonly property color testBannerDark: "#cc5528"
    // Deleted message display style (matches main.qml; test mode uses default)
    property string deletedMessageStyle: "strikethrough"

    // ─── Fonts ───
    FontLoader { id: jbMono; source: "https://raw.githubusercontent.com/JetBrains/JetBrainsMono/master/fonts/ttf/JetBrainsMono-Regular.ttf" }
    FontLoader { id: jbMonoBold; source: "https://raw.githubusercontent.com/JetBrains/JetBrainsMono/master/fonts/ttf/JetBrainsMono-Bold.ttf" }
    readonly property string fontFamily: jbMono.status === FontLoader.Ready ? jbMono.name : "Consolas, monospace"

    // ─── Theme (Discord 2024/2025 dark palette, match main.qml) ───
    QtObject {
        id: theme

        readonly property color bgBase:       "#2a2139"
        readonly property color bgPrimary:    "#2a2139"
        readonly property color bgSecondary:  "#211a30"
        readonly property color bgTertiary:   "#191425"
        readonly property color bgHover:      "#342b44"
        readonly property color bgActive:     "#3e3450"
        readonly property color bgElevated:   "#211a30"
        readonly property color bgFloating:   "#110e1a"
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

        readonly property color border:        "#3a3048"
        readonly property color borderSubtle:  "#ffffff0a"
        readonly property color separator:     "#3a3048"

        readonly property color inputBg:       "#302840"
        readonly property color channelIcon:  "#80848e"
        readonly property color headerPrimary: "#f2f3f5"

        readonly property color online:  "#23a55a"
        readonly property color idle:    "#f0b132"
        readonly property color dnd:     "#f23f43"
        readonly property color offline: "#80848e"

        readonly property int guildBarWidth:   72
        readonly property int channelBarWidth: 240
        readonly property int headerHeight:    48
        readonly property int userPanelHeight: 52
        readonly property int messageInputH:  44
        readonly property int radiusSmall: 4
        readonly property int radiusMed:   8
        readonly property int radiusLarge: 12

        readonly property int animFast:   100
        readonly property int animNormal: 150
        readonly property int animSlow:   250
    }

    // ─── State ───
    property bool isLoggedIn: true
    property string currentUserId: "test_user_001"
    property string currentUserName: testData ? testData.test_user_name : "TestUser"
    property string currentStatus: "online"
    property string currentGuildId: "guild_001"
    property string currentGuildName: "Test Server"
    property string currentChannelId: "ch_001"
    property string currentChannelName: "general"

    // ─── Test Data Models ───
    ListModel { id: guildModel }
    ListModel { id: channelModel }
    ListModel { id: messageModel }
    ListModel { id: memberModel }
    ListModel { id: voiceParticipantModel }

    property bool isVoiceChannel: currentChannelId === "ch_003"
    property int _speakingIndex: 0

    Timer {
        id: voiceSpeakingTimer
        interval: 2000
        running: isVoiceChannel && voiceParticipantModel.count > 0
        repeat: true
        onTriggered: {
            for (var i = 0; i < voiceParticipantModel.count; i++)
                voiceParticipantModel.setProperty(i, "speaking", i === _speakingIndex)
            _speakingIndex = (_speakingIndex + 1) % Math.max(1, voiceParticipantModel.count)
        }
    }

    Component.onCompleted: {
        if (testData) {
            var guilds = JSON.parse(testData.guilds_json);
            for (var i = 0; i < guilds.length; i++) guildModel.append(guilds[i]);

            var channels = JSON.parse(testData.channels_json);
            for (var j = 0; j < channels.length; j++) {
                var ch = channels[j];
                channelModel.append({
                    channelId: ch.channelId || "",
                    guildId: ch.guildId || "",
                    name: ch.name || "",
                    channelType: ch.channelType || 0,
                    position: ch.position || 0,
                    parentId: ch.parentId || "",
                    hasUnread: ch.hasUnread || false,
                    mentionCount: ch.mentionCount || 0,
                    isHidden: ch.isHidden || false
                });
            }

            var messages = JSON.parse(testData.messages_json);
            for (var k = 0; k < messages.length; k++) messageModel.append(messages[k]);

            if (testData.members_json && testData.members_json.length > 2) {
                try {
                    var memberData = JSON.parse(testData.members_json);
                    if (memberData.members) {
                        for (var mi = 0; mi < memberData.members.length; mi++) {
                            var m = memberData.members[mi];
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
                            });
                        }
                    }
                } catch (e) { console.log("Members parse error:", e); }
            }

            if (testData.voice_participants_json && testData.voice_participants_json.length > 2) {
                try {
                    var vpData = JSON.parse(testData.voice_participants_json);
                    if (vpData.participants) {
                        for (var vpi = 0; vpi < vpData.participants.length; vpi++) {
                            var p = vpData.participants[vpi];
                            voiceParticipantModel.append({
                                userId: p.userId || "",
                                username: p.username || "Unknown",
                                avatarUrl: p.avatarUrl || "",
                                selfMute: !!p.selfMute,
                                selfDeaf: !!p.selfDeaf,
                                serverMute: !!p.serverMute,
                                serverDeaf: !!p.serverDeaf,
                                speaking: !!p.speaking,
                                selfVideo: !!p.selfVideo,
                                selfStream: !!p.selfStream,
                                suppress: !!p.suppress
                            });
                        }
                    }
                } catch (e) { console.log("Voice participants parse error:", e); }
            }

            console.log("Test data loaded: " + guilds.length + " guilds, " + channels.length + " channels, " + messages.length + " messages, " + memberModel.count + " members");
        } else {
            console.log("Warning: testData not available, using fallback data");
            guildModel.append({ guildId: "g1", name: "Fallback Server", iconUrl: "", hasUnread: true, mentionCount: 1 });
            channelModel.append({ channelId: "c1", guildId: "", name: "general", channelType: 0, position: 0, parentId: "", hasUnread: false, mentionCount: 0, isHidden: false });
            messageModel.append({ messageId: "m1", authorName: "System", content: "Test mode initialized", timestamp: "Now", isDeleted: false });
        }
    }

    // ─── Main Layout ───
    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Test mode banner
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 30
            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop { position: 0.0; color: testBannerColor }
                GradientStop { position: 0.5; color: testBannerDark }
                GradientStop { position: 1.0; color: testBannerColor }
            }

            RowLayout {
                anchors.centerIn: parent
                spacing: 10

                Twemoji { emoji: "\u{1F9EA}"; size: 16 }
                Text {
                    text: "UI TEST MODE — No Discord Connection"
                    color: "#ffffff"
                    font.family: fontFamily
                    font.pixelSize: 12
                    font.bold: true
                }
                Twemoji { emoji: "\u{1F9EA}"; size: 16 }
            }
        }

        // Main content
        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            // ══════════ Guild Sidebar ══════════
            Rectangle {
                Layout.preferredWidth: theme.guildBarWidth
                Layout.fillHeight: true
                color: theme.bgBase

                ColumnLayout {
                    anchors.fill: parent
                    anchors.topMargin: 12
                    spacing: 2

                    // Home button
                    Item {
                        Layout.preferredWidth: theme.guildBarWidth
                        Layout.preferredHeight: 52

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

                        Rectangle {
                            width: 44; height: 44
                            anchors.centerIn: parent
                            radius: currentGuildId === "" ? 14 : 22
                            color: currentGuildId === "" ? theme.accent : homeMa.containsMouse ? theme.accentLight : theme.bgSecondary
                            Behavior on radius { NumberAnimation { duration: theme.animNormal; easing.type: Easing.OutCubic } }
                            Behavior on color { ColorAnimation { duration: theme.animNormal } }

                            Text {
                                anchors.centerIn: parent
                                text: "⌂"
                                color: currentGuildId === "" || homeMa.containsMouse ? "#ffffff" : theme.textSecondary
                                font.pixelSize: 18
                                Behavior on color { ColorAnimation { duration: theme.animFast } }
                            }

                            MouseArea {
                                id: homeMa
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: { currentGuildId = ""; currentGuildName = "Direct Messages" }
                            }
                        }
                    }

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

                            Rectangle {
                                width: 4
                                height: currentGuildId === model.guildId ? 36 :
                                        guildDelegMa.containsMouse ? 20 : 8
                                radius: 2
                                color: theme.textNormal
                                anchors.left: parent.left
                                anchors.verticalCenter: parent.verticalCenter
                                visible: currentGuildId === model.guildId || guildDelegMa.containsMouse
                                Behavior on height { NumberAnimation { duration: theme.animNormal; easing.type: Easing.OutCubic } }
                            }

                            GuildIcon {
                                id: guildIcon
                                anchors.centerIn: parent
                                text: model.name ? model.name.charAt(0).toUpperCase() : "?"
                                iconUrl: model.iconUrl || ""
                                isActive: currentGuildId === model.guildId
                                iconSize: 48
                                fontSize: 14
                                onClicked: { currentGuildId = model.guildId; currentGuildName = model.name }
                            }

                            Rectangle {
                                visible: (model.mentionCount || 0) > 0
                                width: Math.max(16, mText.width + 6)
                                height: 16
                                radius: 8
                                color: theme.danger
                                anchors.right: guildIcon.right
                                anchors.bottom: guildIcon.bottom
                                anchors.margins: -2

                                Text {
                                    id: mText
                                    anchors.centerIn: parent
                                    text: model.mentionCount > 99 ? "99+" : model.mentionCount
                                    color: "#ffffff"
                                    font.family: fontFamily
                                    font.pixelSize: 10
                                    font.bold: true
                                }
                            }

                            MouseArea {
                                id: guildDelegMa
                                anchors.fill: parent
                                hoverEnabled: true
                                acceptedButtons: Qt.NoButton
                            }
                        }
                    }

                    Rectangle {
                        Layout.preferredHeight: 2
                        Layout.preferredWidth: 32
                        Layout.alignment: Qt.AlignHCenter
                        Layout.topMargin: 4
                        Layout.bottomMargin: 4
                        radius: 1
                        color: theme.separator
                    }

                    // User avatar
                    DAvatar {
                        Layout.preferredWidth: 42
                        Layout.preferredHeight: 42
                        Layout.alignment: Qt.AlignHCenter
                        Layout.bottomMargin: 10
                        size: 42
                        imageUrl: ""
                        fallbackText: currentUserName || "?"
                    }
                }
            }

            Rectangle { Layout.preferredWidth: 1; Layout.fillHeight: true; color: "#00000040" }

            // ══════════ Channel Sidebar ══════════
            Rectangle {
                Layout.preferredWidth: theme.channelBarWidth
                Layout.fillHeight: true
                color: theme.bgPrimary

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0

                    // Guild header
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: theme.headerHeight
                        color: theme.bgPrimary

                        Text {
                            anchors.left: parent.left
                            anchors.leftMargin: 16
                            anchors.verticalCenter: parent.verticalCenter
                            text: currentGuildName
                            color: theme.textNormal
                            font.family: fontFamily
                            font.pixelSize: 14
                            font.bold: true
                        }

                        Rectangle {
                            anchors.bottom: parent.bottom
                            width: parent.width
                            height: 1
                            color: theme.separator

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

                    // Channel list
                    ListView {
                        id: channelListView
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        Layout.topMargin: 8
                        model: channelModel
                        clip: true
                        spacing: 1
                        boundsBehavior: Flickable.StopAtBounds

                        ScrollBar.vertical: ScrollBar {
                            policy: ScrollBar.AsNeeded
                            contentItem: Rectangle {
                                implicitWidth: 4; radius: 2
                                color: theme.textFaint
                                opacity: parent.active ? 0.8 : 0.0
                                Behavior on opacity { NumberAnimation { duration: theme.animSlow } }
                            }
                            background: Item {}
                        }

                        delegate: Item {
                            width: channelListView.width - 16
                            x: 8
                            height: model.channelType === 4 ? 28 : 32
                            property bool isCategory: model.channelType === 4
                            property bool hasParent: (model.parentId && model.parentId !== "")
                            property bool isHiddenChannel: model.isHidden || false

                            Rectangle {
                                visible: isCategory
                                anchors.fill: parent
                                color: "transparent"
                                Text {
                                    anchors.left: parent.left
                                    anchors.leftMargin: 8
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: (model.name || "category").toUpperCase()
                                    color: theme.textFaint
                                    font.family: fontFamily
                                    font.pixelSize: 11
                                    font.weight: Font.Bold
                                    font.letterSpacing: 0.5
                                }
                            }
                            Rectangle {
                                visible: !isCategory
                                anchors.fill: parent
                                radius: theme.radiusSmall
                                color: currentChannelId === model.channelId ? theme.bgActive :
                                       chMa.containsMouse ? theme.bgHover : "transparent"
                                Behavior on color { ColorAnimation { duration: theme.animFast } }
                                RowLayout {
                                    anchors.fill: parent
                                    anchors.leftMargin: hasParent ? 24 : 8
                                    anchors.rightMargin: 8
                                    spacing: 6
                                    Item {
                                        Layout.preferredWidth: 20
                                        Layout.preferredHeight: 20
                                        Twemoji {
                                            anchors.centerIn: parent
                                            emoji: isHiddenChannel ? "\u{1F512}" : (model.channelType === 2 ? "\u{1F50A}" : "#")
                                            size: 16
                                            visible: model.channelType === 2 || isHiddenChannel
                                            opacity: currentChannelId === model.channelId ? 0.8 : 0.5
                                        }
                                        Text {
                                            anchors.centerIn: parent
                                            text: "#"
                                            color: currentChannelId === model.channelId ? theme.textSecondary : theme.textFaint
                                            font.family: fontFamily
                                            font.pixelSize: 16
                                            font.bold: true
                                            visible: model.channelType !== 2 && !isHiddenChannel
                                        }
                                    }
                                    Text {
                                        Layout.fillWidth: true
                                        text: model.name
                                        color: currentChannelId === model.channelId ? theme.textNormal :
                                               chMa.containsMouse ? theme.textSecondary : (isHiddenChannel ? theme.textFaint : theme.textMuted)
                                        font.family: fontFamily
                                        font.pixelSize: 13
                                        elide: Text.ElideRight
                                        Behavior on color { ColorAnimation { duration: theme.animFast } }
                                    }
                                    Rectangle {
                                        visible: (model.hasUnread || false) && currentChannelId !== model.channelId
                                        width: 6; height: 6; radius: 3
                                        color: theme.textNormal
                                    }
                                }
                                MouseArea {
                                    id: chMa
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: isHiddenChannel ? Qt.ArrowCursor : Qt.PointingHandCursor
                                    onClicked: {
                                        if (!isHiddenChannel) {
                                            currentChannelId = model.channelId
                                            currentChannelName = model.name
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // User panel
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 52
                        color: theme.bgSecondary

                        Rectangle { anchors.top: parent.top; width: parent.width; height: 1; color: theme.separator }

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 10
                            anchors.rightMargin: 10
                            spacing: 10

                            Rectangle {
                                width: 32; height: 32; radius: 16
                                color: theme.bgTertiary

                                Text {
                                    anchors.centerIn: parent
                                    text: currentUserName.charAt(0).toUpperCase()
                                    color: theme.textNormal
                                    font.family: fontFamily
                                    font.pixelSize: 13
                                    font.bold: true
                                }

                                Rectangle {
                                    width: 10; height: 10; radius: 5
                                    anchors.right: parent.right
                                    anchors.bottom: parent.bottom
                                    anchors.margins: -1
                                    color: theme.online
                                    border.width: 2
                                    border.color: theme.bgSecondary
                                }
                            }

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 0
                                Text {
                                    text: currentUserName
                                    color: theme.textNormal
                                    font.family: fontFamily
                                    font.pixelSize: 12
                                    font.bold: true
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }
                                Text {
                                    text: "Online"
                                    color: theme.textMuted
                                    font.family: fontFamily
                                    font.pixelSize: 10
                                }
                            }

                            Text {
                                text: "⚙"
                                color: theme.textMuted
                                font.pixelSize: 16
                            }
                        }
                    }
                }
            }

            Rectangle { Layout.preferredWidth: 1; Layout.fillHeight: true; color: "#00000040" }

            // ══════════ Main Content ══════════
            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                color: theme.bgBase

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0

                    // Channel header with test controls
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: theme.headerHeight
                        color: theme.bgBase

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 16
                            anchors.rightMargin: 16
                            spacing: 8

                            Text {
                                text: "#"
                                color: theme.textFaint
                                font.family: fontFamily
                                font.pixelSize: 20
                                font.bold: true
                            }
                            Text {
                                text: currentChannelName
                                color: theme.textNormal
                                font.family: fontFamily
                                font.pixelSize: 15
                                font.bold: true
                            }
                            Item { Layout.fillWidth: true }

                            // Test controls
                            Row {
                                spacing: 6

                                TestControlButton {
                                    text: "+ Message"
                                    onClicked: {
                                        messageModel.insert(0, {
                                            messageId: "msg_new_" + Date.now(),
                                            authorName: "NewUser",
                                            content: "This is a simulated new message!",
                                            timestamp: "Just now",
                                            isDeleted: false
                                        })
                                    }
                                }
                                TestControlButton {
                                    text: "🗑 Delete"
                                    onClicked: {
                                        if (messageModel.count > 0) {
                                            messageModel.setProperty(0, "isDeleted", true);
                                            messageModel.setProperty(0, "content", "[DELETED] " + messageModel.get(0).content);
                                        }
                                    }
                                }
                                TestControlButton {
                                    text: "✕ Clear"
                                    onClicked: messageModel.clear()
                                }
                            }
                        }

                        Rectangle {
                            anchors.bottom: parent.bottom
                            width: parent.width
                            height: 1
                            color: theme.separator

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

                    // Voice channel view (mock participants + speaking cycle)
                    Item {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        visible: isVoiceChannel
                        ColumnLayout {
                            anchors.fill: parent
                            spacing: 8
                            Text {
                                text: "\u{1F50A} Voice Channel — Mock participants (speaking cycles every 2s)"
                                color: theme.textMuted
                                font.pixelSize: 12
                                Layout.alignment: Qt.AlignHCenter
                            }
                            GridView {
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                cellWidth: 140
                                cellHeight: 160
                                clip: true
                                model: voiceParticipantModel
                                delegate: Rectangle {
                                    width: 134
                                    height: 154
                                    radius: theme.radiusMed
                                    color: model.speaking ? theme.bgTertiary : theme.bgSecondary
                                    border.width: model.speaking ? 2 : 0
                                    border.color: theme.positive
                                    ColumnLayout {
                                        anchors.fill: parent
                                        anchors.margins: 8
                                        spacing: 4
                                        Item { Layout.preferredWidth: 48; Layout.preferredHeight: 48; Layout.alignment: Qt.AlignHCenter
                                            Rectangle {
                                                anchors.fill: parent
                                                radius: width / 2
                                                color: theme.bgTertiary
                                            }
                                            Rectangle {
                                                anchors.fill: parent
                                                radius: width / 2
                                                border.width: model.speaking ? 3 : 0
                                                border.color: theme.positive
                                                color: "transparent"
                                            }
                                        }
                                        Text {
                                            text: model.username || "Unknown"
                                            Layout.fillWidth: true
                                            horizontalAlignment: Text.AlignHCenter
                                            color: theme.textNormal
                                            font.pixelSize: 12
                                            elide: Text.ElideRight
                                        }
                                        RowLayout {
                                            Layout.alignment: Qt.AlignHCenter
                                            spacing: 4
                                            Text { text: model.selfMute || model.serverMute ? "\u{1F507}" : ""; font.pixelSize: 10; visible: model.selfMute || model.serverMute }
                                            Text { text: model.speaking ? "\u{1F3A4}" : ""; font.pixelSize: 10; color: theme.positive; visible: model.speaking }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Messages
                    ListView {
                        id: messageList
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        visible: !isVoiceChannel
                        model: messageModel
                        clip: true
                        verticalLayoutDirection: ListView.BottomToTop
                        spacing: 0
                        boundsBehavior: Flickable.StopAtBounds

                        ScrollBar.vertical: ScrollBar {
                            policy: ScrollBar.AsNeeded
                            contentItem: Rectangle {
                                implicitWidth: 6; radius: 3
                                color: theme.textFaint
                                opacity: parent.active ? 0.6 : 0.0
                                Behavior on opacity { NumberAnimation { duration: theme.animSlow } }
                            }
                            background: Item {}
                        }

                        delegate: Rectangle {
                            width: messageList.width
                            height: msgRow.implicitHeight + 16
                            color: msgMa.containsMouse ? theme.bgModifier : "transparent"
                            Behavior on color { ColorAnimation { duration: theme.animFast } }

                            MouseArea {
                                id: msgMa
                                anchors.fill: parent
                                hoverEnabled: true
                                acceptedButtons: Qt.NoButton
                            }

                            RowLayout {
                                id: msgRow
                                anchors.left: parent.left
                                anchors.right: parent.right
                                anchors.leftMargin: 16
                                anchors.rightMargin: 16
                                anchors.verticalCenter: parent.verticalCenter
                                spacing: 16

                                DAvatar {
                                    Layout.preferredWidth: 40
                                    Layout.preferredHeight: 40
                                    Layout.alignment: Qt.AlignTop
                                    size: 40
                                    imageUrl: model.authorAvatarUrl || ""
                                    fallbackText: model.authorName || "?"
                                }

                                ColumnLayout {
                                    Layout.fillWidth: true
                                    spacing: 3

                                    RowLayout {
                                        spacing: 8
                                        Text {
                                            text: model.authorName || "Unknown"
                                            color: (model.authorRoleColor && model.authorRoleColor.length > 0) ? model.authorRoleColor : theme.accent
                                            font.family: fontFamily
                                            font.pixelSize: 16
                                            font.weight: Font.Medium
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
                                        Text {
                                            text: model.timestamp || ""
                                            color: theme.textFaint
                                            font.family: fontFamily
                                            font.pixelSize: 12
                                        }
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
                                        text: model.content || ""
                                        color: (model.isDeleted && (root.deletedMessageStyle === "strikethrough" || root.deletedMessageStyle === "deleted")) ? theme.danger : theme.textNormal
                                        font.family: fontFamily
                                        font.pixelSize: 16
                                        font.strikeout: (model.isDeleted || false) && (root.deletedMessageStyle === "strikethrough" || root.deletedMessageStyle === "deleted")
                                        wrapMode: Text.WordWrap
                                        lineHeight: 1.375
                                        opacity: (model.isDeleted && root.deletedMessageStyle === "faded") ? 0.5 : 1.0
                                    }
                                }
                            }
                        }
                    }

                    // Message input
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: theme.messageInputH
                        Layout.leftMargin: 16
                        Layout.rightMargin: 16
                        Layout.bottomMargin: 16
                        Layout.topMargin: 4
                        radius: theme.radiusMed
                        color: theme.bgSecondary
                        border.width: messageInput.activeFocus ? 1 : 0
                        border.color: theme.accent

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 12
                            anchors.rightMargin: 12
                            spacing: 10

                            Twemoji {
                                emoji: "\u{1F600}"
                                size: 22
                                Layout.alignment: Qt.AlignVCenter
                            }

                            TextField {
                                id: messageInput
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                placeholderText: "Message #" + currentChannelName + " (test mode)"
                                placeholderTextColor: theme.textFaint
                                color: theme.textNormal
                                font.family: fontFamily
                                font.pixelSize: 14
                                background: Item {}
                                verticalAlignment: TextInput.AlignVCenter

                                Keys.onReturnPressed: {
                                    if (text.trim() !== "") {
                                        messageModel.insert(0, {
                                            messageId: "msg_user_" + Date.now(),
                                            authorName: currentUserName,
                                            content: text,
                                            timestamp: "Just now",
                                            isDeleted: false
                                        });
                                        text = "";
                                    }
                                }
                            }

                            Twemoji {
                                emoji: "\u{1F4CE}"
                                size: 20
                                Layout.alignment: Qt.AlignVCenter
                                opacity: 0.6
                            }
                        }
                    }
                }
            }

            // ══════════ Member List (right sidebar) ══════════
            Rectangle {
                Layout.preferredWidth: 208
                Layout.minimumWidth: 180
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

                        delegate: Rectangle {
                            width: memberListView.width - 8
                            x: 4
                            height: 42
                            radius: theme.radiusSmall
                            color: memberMa.containsMouse ? theme.bgHover : "transparent"

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: 8
                                anchors.rightMargin: 8
                                spacing: 8

                                DAvatar {
                                    Layout.preferredWidth: 32
                                    Layout.preferredHeight: 32
                                    size: 32
                                    imageUrl: model.avatarUrl || ""
                                    fallbackText: model.displayName || model.username || "?"
                                }

                                ColumnLayout {
                                    Layout.fillWidth: true
                                    spacing: 2
                                    RowLayout {
                                        spacing: 4
                                        Text {
                                            text: model.displayName ? model.displayName : (model.username || "Unknown")
                                            color: theme.textNormal
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
                                acceptedButtons: Qt.NoButton
                            }
                        }
                    }
                }
            }
        }
    }

    // ══════════ Test Control Button Component ══════════
    component TestControlButton: Rectangle {
        property string text: ""
        signal clicked()

        width: btnLabel.implicitWidth + 16
        height: 26
        radius: theme.radiusSmall
        color: btnMa.containsMouse ? testBannerDark : testBannerColor
        opacity: btnMa.containsMouse ? 1.0 : 0.85
        Behavior on color { ColorAnimation { duration: theme.animFast } }
        Behavior on opacity { NumberAnimation { duration: theme.animFast } }

        Text {
            id: btnLabel
            anchors.centerIn: parent
            text: parent.text
            color: "#ffffff"
            font.family: fontFamily
            font.pixelSize: 10
            font.bold: true
        }
        MouseArea {
            id: btnMa
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: parent.clicked()
        }
    }
}
