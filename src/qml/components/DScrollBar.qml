// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

import QtQuick 2.15

Rectangle {
    id: root
    property real position: 0
    property real sizeRatio: 1
    property int barWidth: 8
    property int minHeight: 40
    property color barColor: "#202225"
    property color barColorHover: "#2e3035"

    width: barWidth
    height: Math.max(minHeight, parent ? parent.height * sizeRatio : minHeight)
    radius: barWidth / 2
    color: scrollBarMa.containsMouse ? barColorHover : barColor
    anchors.right: parent ? parent.right : undefined
    anchors.rightMargin: 2
    anchors.top: parent ? parent.top : undefined
    anchors.topMargin: parent ? parent.height * position : 0

    Behavior on color { ColorAnimation { duration: 100 } }

    MouseArea {
        id: scrollBarMa
        anchors.fill: parent
        hoverEnabled: true
    }
}
