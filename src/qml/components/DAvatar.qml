// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

import QtQuick
import Qt5Compat.GraphicalEffects
import "../Discord.js" as D

Item {
    id: root
    property int size: 32
    property string imageUrl: ""
    property string fallbackText: "?"
    property bool showStatus: false
    property color statusColor: "#23a55a"
    property color fallbackColor: D.avatarColor(fallbackText)

    width: size
    height: size

    Rectangle {
        id: fallbackRect
        anchors.fill: parent
        radius: size / 2
        color: fallbackColor
        visible: !imageUrl || avatarImg.status !== Image.Ready
        clip: true

        Text {
            anchors.centerIn: parent
            text: (fallbackText || "?").charAt(0).toUpperCase()
            color: "#ffffff"
            font.pixelSize: Math.max(10, size * 0.45)
            font.bold: true
        }
    }

    // Circular mask so avatar image is always round (Rectangle+clip can render square on some setups)
    Item {
        id: avatarImgWrapper
        width: size
        height: size
        visible: false
        Image {
            id: avatarImg
            anchors.fill: parent
            source: imageUrl || ""
            fillMode: Image.PreserveAspectCrop
            smooth: true
            asynchronous: true
        }
    }
    Item {
        id: avatarMask
        width: size
        height: size
        visible: false
        Rectangle {
            anchors.fill: parent
            radius: size / 2
            color: "white"
        }
    }
    OpacityMask {
        anchors.fill: parent
        source: avatarImgWrapper
        maskSource: avatarMask
        visible: imageUrl && avatarImg.status === Image.Ready
    }

    Rectangle {
        visible: showStatus
        width: size * 0.3
        height: width
        radius: width / 2
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: -1
        color: statusColor
        border.width: 2
        border.color: parent.parent && parent.parent.color !== undefined ? parent.parent.color : "#313338"
    }
}
