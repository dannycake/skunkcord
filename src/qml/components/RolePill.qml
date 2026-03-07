// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

import QtQuick 2.15

Rectangle {
    id: root
    property string roleName: ""
    property string roleColor: ""
    property int fontSize: 10

    visible: roleName !== ""
    height: 16
    width: roleText.implicitWidth + 8
    radius: 3
    color: {
        if (!roleColor || roleColor.length < 4) return "transparent"
        var c = roleColor
        if (c.charAt(0) === "#" && c.length >= 7) {
            var r = parseInt(c.substring(1, 3), 16) / 255
            var g = parseInt(c.substring(3, 5), 16) / 255
            var b = parseInt(c.substring(5, 7), 16) / 255
            return Qt.rgba(r, g, b, 0.15)
        }
        return "transparent"
    }
    border.width: 1
    border.color: {
        if (!roleColor || roleColor.length < 4) return "transparent"
        var c = roleColor
        if (c.charAt(0) === "#" && c.length >= 7) {
            var r = parseInt(c.substring(1, 3), 16) / 255
            var g = parseInt(c.substring(3, 5), 16) / 255
            var b = parseInt(c.substring(5, 7), 16) / 255
            return Qt.rgba(r, g, b, 0.4)
        }
        return "transparent"
    }

    Text {
        id: roleText
        anchors.centerIn: parent
        text: roleName
        font.pixelSize: fontSize
        color: roleColor && roleColor.length >= 4 ? roleColor : (typeof theme !== "undefined" ? theme.textMuted : "#6d7178")
    }
}
