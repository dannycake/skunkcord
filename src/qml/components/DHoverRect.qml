// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

import QtQuick 2.15

Rectangle {
    id: root
    property bool hoverEnabled: true
    property int cursorShape: Qt.PointingHandCursor
    property color hoverColor: "#ffffff0d"
    property color normalColor: "transparent"
    property color activeColor: "transparent"
    property bool isActive: false
    signal clicked()

    color: isActive ? activeColor : (hoverMa.containsMouse ? hoverColor : normalColor)

    MouseArea {
        id: hoverMa
        anchors.fill: parent
        hoverEnabled: root.hoverEnabled
        cursorShape: root.cursorShape
        onClicked: root.clicked()
    }
}
