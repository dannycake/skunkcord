// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

import QtQuick 2.15
import "../Discord.js" as D

Item {
    id: root
    property string emoji: ""
    property int size: 22

    width: size
    height: size

    Image {
        id: twImg
        anchors.fill: parent
        source: emoji !== "" ? D.twemojiUrl(emoji) : ""
        sourceSize.width: parent.width * 2
        sourceSize.height: parent.height * 2
        smooth: true
        asynchronous: true
        fillMode: Image.PreserveAspectFit
        visible: status === Image.Ready
    }
    Text {
        anchors.centerIn: parent
        text: emoji
        font.pixelSize: parent.size - 4
        visible: twImg.status !== Image.Ready
    }
}
