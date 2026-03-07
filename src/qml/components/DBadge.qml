// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

import QtQuick 2.15

Rectangle {
    id: root
    property int count: 0
    property int fontSize: 10
    property color textColor: "#ffffff"
    property color bgColor: "#f23f43"

    visible: count > 0
    width: countText.implicitWidth + 8
    height: countText.implicitHeight + 4
    radius: height / 2
    color: bgColor

    Text {
        id: countText
        anchors.centerIn: parent
        text: count > 99 ? "99+" : count.toString()
        color: textColor
        font.pixelSize: fontSize
        font.bold: true
    }
}
