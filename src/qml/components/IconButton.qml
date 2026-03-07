// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

import QtQuick 2.15

Rectangle {
    id: root
    property string icon: ""
    property int fontSize: 16
    property bool usesTwemoji: false
    signal clicked()

    width: 26
    height: 26
    radius: 4
    color: iconBtnMa.containsMouse ? (typeof theme !== "undefined" ? theme.bgHover : "#1e2129") : "transparent"

    Text {
        anchors.centerIn: parent
        text: parent.icon
        font.pixelSize: parent.fontSize
        color: typeof theme !== "undefined" ? theme.textFaint : "#4e5058"
    }

    MouseArea {
        id: iconBtnMa
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
