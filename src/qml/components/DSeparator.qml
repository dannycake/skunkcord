// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

import QtQuick 2.15

Rectangle {
    id: root
    property bool vertical: false
    property color separatorColor: "#0affffff"
    property int thickness: 1

    width: vertical ? thickness : parent ? parent.width : 0
    height: vertical ? (parent ? parent.height : 0) : thickness
    color: separatorColor
}
