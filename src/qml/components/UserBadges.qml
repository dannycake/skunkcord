// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

import QtQuick 2.15

Row {
    id: root
    property int publicFlags: 0
    property bool isBot: false
    property int premiumType: 0
    property int badgeSize: 16
    spacing: 2

    // BOT pill — blurple background, shown for bots
    Rectangle {
        visible: isBot
        width: botText.implicitWidth + 6
        height: badgeSize
        radius: 3
        color: "#5865f2"

        Text {
            id: botText
            anchors.centerIn: parent
            text: "BOT"
            color: "#ffffff"
            font.pixelSize: 10
            font.bold: true
        }
    }

    // Nitro badge — from premiumType (1=classic, 2=nitro, 3=basic)
    Rectangle {
        visible: premiumType >= 1 && premiumType <= 3
        width: badgeSize
        height: badgeSize
        radius: 2
        color: "#5865f2"

        Text {
            anchors.centerIn: parent
            text: "✦"
            color: "#ffffff"
            font.pixelSize: 10
            font.bold: true
        }
    }

    // Staff (1<<0)
    Rectangle {
        visible: (publicFlags & 1) !== 0
        width: badgeSize
        height: badgeSize
        radius: 2
        color: "#5865f2"
        Text { anchors.centerIn: parent; text: "⚙"; color: "#fff"; font.pixelSize: 10 }
    }
    // Partner (1<<1)
    Rectangle {
        visible: (publicFlags & 2) !== 0
        width: badgeSize
        height: badgeSize
        radius: 2
        color: "#5865f2"
        Text { anchors.centerIn: parent; text: "✓"; color: "#fff"; font.pixelSize: 10 }
    }
    // HypeSquad Events (1<<2)
    Rectangle {
        visible: (publicFlags & 4) !== 0
        width: badgeSize
        height: badgeSize
        radius: 2
        color: "#f0b132"
        Text { anchors.centerIn: parent; text: "★"; color: "#fff"; font.pixelSize: 10 }
    }
    // Bug Hunter L1 (1<<3)
    Rectangle {
        visible: (publicFlags & 8) !== 0
        width: badgeSize
        height: badgeSize
        radius: 2
        color: "#23a55a"
        Text { anchors.centerIn: parent; text: "🐛"; color: "#fff"; font.pixelSize: 8 }
    }
    // HypeSquad Bravery (1<<6)
    Rectangle {
        visible: (publicFlags & 64) !== 0
        width: badgeSize
        height: badgeSize
        radius: 2
        color: "#9b59b6"
        Text { anchors.centerIn: parent; text: "🛡"; color: "#fff"; font.pixelSize: 8 }
    }
    // HypeSquad Brilliance (1<<7)
    Rectangle {
        visible: (publicFlags & 128) !== 0
        width: badgeSize
        height: badgeSize
        radius: 2
        color: "#faa61a"
        Text { anchors.centerIn: parent; text: "💡"; color: "#fff"; font.pixelSize: 8 }
    }
    // HypeSquad Balance (1<<8)
    Rectangle {
        visible: (publicFlags & 256) !== 0
        width: badgeSize
        height: badgeSize
        radius: 2
        color: "#3ba55c"
        Text { anchors.centerIn: parent; text: "⚖"; color: "#fff"; font.pixelSize: 8 }
    }
    // Early Supporter (1<<9)
    Rectangle {
        visible: (publicFlags & 512) !== 0
        width: badgeSize
        height: badgeSize
        radius: 2
        color: "#9b59b6"
        Text { anchors.centerIn: parent; text: "💜"; color: "#fff"; font.pixelSize: 8 }
    }
    // Bug Hunter L2 (1<<14)
    Rectangle {
        visible: (publicFlags & 16384) !== 0
        width: badgeSize
        height: badgeSize
        radius: 2
        color: "#f0b132"
        Text { anchors.centerIn: parent; text: "🐛"; color: "#fff"; font.pixelSize: 8 }
    }
    // Verified Bot Developer (1<<17)
    Rectangle {
        visible: (publicFlags & 131072) !== 0
        width: badgeSize
        height: badgeSize
        radius: 2
        color: "#5865f2"
        Text { anchors.centerIn: parent; text: "⚙"; color: "#fff"; font.pixelSize: 10 }
    }
    // Certified Moderator (1<<18)
    Rectangle {
        visible: (publicFlags & 262144) !== 0
        width: badgeSize
        height: badgeSize
        radius: 2
        color: "#5865f2"
        Text { anchors.centerIn: parent; text: "🛡"; color: "#fff"; font.pixelSize: 8 }
    }
    // Active Developer (1<<22)
    Rectangle {
        visible: (publicFlags & 4194304) !== 0
        width: badgeSize
        height: badgeSize
        radius: 2
        color: "#23a55a"
        Text { anchors.centerIn: parent; text: "▸"; color: "#fff"; font.pixelSize: 10 }
    }
}
