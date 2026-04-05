// Copyright (c) Skunk Ventures LLC
// SPDX-License-Identifier: MIT

import QtQuick 2.15

Row {
    id: root
    property int publicFlags: 0
    property bool isBot: false
    property int premiumType: 0
    property int badgeSize: 16
    spacing: 2

    // BOT pill — text label, not an icon
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

    // Nitro badge (premiumType 1=classic, 2=nitro, 3=basic)
    Image {
        visible: premiumType >= 1 && premiumType <= 3
        width: badgeSize
        height: badgeSize
        source: "../assets/badges/nitro.png"
        fillMode: Image.PreserveAspectFit
        smooth: true
    }

    // Staff (1<<0)
    Image {
        visible: (publicFlags & 1) !== 0
        width: badgeSize
        height: badgeSize
        source: "../assets/badges/staff.png"
        fillMode: Image.PreserveAspectFit
        smooth: true
    }

    // Partner (1<<1)
    Image {
        visible: (publicFlags & 2) !== 0
        width: badgeSize
        height: badgeSize
        source: "../assets/badges/partner.png"
        fillMode: Image.PreserveAspectFit
        smooth: true
    }

    // HypeSquad Events (1<<2)
    Image {
        visible: (publicFlags & 4) !== 0
        width: badgeSize
        height: badgeSize
        source: "../assets/badges/hypesquad_events.png"
        fillMode: Image.PreserveAspectFit
        smooth: true
    }

    // Bug Hunter L1 (1<<3)
    Image {
        visible: (publicFlags & 8) !== 0
        width: badgeSize
        height: badgeSize
        source: "../assets/badges/bug_hunter_l1.png"
        fillMode: Image.PreserveAspectFit
        smooth: true
    }

    // HypeSquad Bravery (1<<6)
    Image {
        visible: (publicFlags & 64) !== 0
        width: badgeSize
        height: badgeSize
        source: "../assets/badges/hypesquad_bravery.png"
        fillMode: Image.PreserveAspectFit
        smooth: true
    }

    // HypeSquad Brilliance (1<<7)
    Image {
        visible: (publicFlags & 128) !== 0
        width: badgeSize
        height: badgeSize
        source: "../assets/badges/hypesquad_brilliance.png"
        fillMode: Image.PreserveAspectFit
        smooth: true
    }

    // HypeSquad Balance (1<<8)
    Image {
        visible: (publicFlags & 256) !== 0
        width: badgeSize
        height: badgeSize
        source: "../assets/badges/hypesquad_balance.png"
        fillMode: Image.PreserveAspectFit
        smooth: true
    }

    // Early Supporter (1<<9)
    Image {
        visible: (publicFlags & 512) !== 0
        width: badgeSize
        height: badgeSize
        source: "../assets/badges/early_supporter.png"
        fillMode: Image.PreserveAspectFit
        smooth: true
    }

    // Bug Hunter L2 (1<<14)
    Image {
        visible: (publicFlags & 16384) !== 0
        width: badgeSize
        height: badgeSize
        source: "../assets/badges/bug_hunter_l2.png"
        fillMode: Image.PreserveAspectFit
        smooth: true
    }

    // Verified Bot Developer (1<<17)
    Image {
        visible: (publicFlags & 131072) !== 0
        width: badgeSize
        height: badgeSize
        source: "../assets/badges/verified_bot_dev.png"
        fillMode: Image.PreserveAspectFit
        smooth: true
    }

    // Certified Moderator (1<<18)
    Image {
        visible: (publicFlags & 262144) !== 0
        width: badgeSize
        height: badgeSize
        source: "../assets/badges/certified_moderator.png"
        fillMode: Image.PreserveAspectFit
        smooth: true
    }

    // Active Developer (1<<22)
    Image {
        visible: (publicFlags & 4194304) !== 0
        width: badgeSize
        height: badgeSize
        source: "../assets/badges/active_developer.png"
        fillMode: Image.PreserveAspectFit
        smooth: true
    }
}
