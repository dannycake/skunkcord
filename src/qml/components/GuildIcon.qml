// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

import QtQuick 2.15
import QtQuick.Layouts 1.15

Rectangle {
    id: root
    property string text: ""
    property string iconUrl: ""
    property bool isActive: false
    property int fontSize: 14
    property int iconSize: 48
    signal clicked()

    width: iconSize
    height: iconSize
    Layout.alignment: Qt.AlignHCenter
    radius: isActive ? (iconSize / 3) : (iconSize / 2)
    color: iconUrl && guildIconImg.status === Image.Ready
           ? "transparent"
           : isActive ? (typeof theme !== "undefined" ? theme.accent : "#5865f2")
           : guildIconMa.containsMouse ? (typeof theme !== "undefined" ? theme.accentLight : "#7289da") : (typeof theme !== "undefined" ? theme.bgSecondary : "#111318")
    clip: true

    Behavior on radius { NumberAnimation { duration: 150; easing.type: Easing.OutCubic } }
    Behavior on color { ColorAnimation { duration: 150 } }

    Rectangle {
        anchors.fill: parent
        anchors.margins: -3
        radius: parent.radius + 3
        color: "transparent"
        border.width: guildIconMa.containsMouse && !isActive ? 2 : 0
        border.color: typeof theme !== "undefined" ? theme.accentGlow : "#305865f2"
        Behavior on border.width { NumberAnimation { duration: 100 } }
    }

    Image {
        id: guildIconImg
        anchors.fill: parent
        source: iconUrl || ""
        sourceSize: Qt.size(iconSize * 2, iconSize * 2)
        fillMode: Image.PreserveAspectCrop
        visible: iconUrl && status === Image.Ready
        smooth: true
        cache: true
    }

    Text {
        anchors.centerIn: parent
        text: root.text
        color: isActive || guildIconMa.containsMouse ? "#ffffff" : (typeof theme !== "undefined" ? theme.textSecondary : "#949ba4")
        font.pixelSize: root.fontSize
        font.bold: true
        visible: !iconUrl || guildIconImg.status !== Image.Ready
        Behavior on color { ColorAnimation { duration: 100 } }
    }

    MouseArea {
        id: guildIconMa
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
