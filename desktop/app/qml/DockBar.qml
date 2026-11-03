// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick
import QtQuick.Window
import app.nectarlink

// The dock: a strip at the left or right screen edge (Preferences.dockEdge)
// that windows don't cover. Its buttons open the phone's notifications in a
// panel that slides out of the strip, or a page of the main window.
DockWindow {
    id: dock

    // Opens a page of the main window ("home", "messages", "calls"...).
    signal pageRequested(string page)

    title: qsTr("NectarLink dock")
    color: "transparent"
    edge: Preferences.dockEdge
    thickness: 60
    expandedWidth: 400
    visible: true

    // What the panel shows: "" (closed) or "notifications".
    property string panel: ""
    expanded: panel.length > 0
    readonly property bool left: edge === "left"

    function togglePanel(name) {
        dock.panel = dock.panel === name ? "" : name
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.surfaceContainer
        border.color: Theme.outlineVariant
        border.width: 1

        // The strip of buttons, at the screen edge.
        Column {
            id: strip
            width: dock.thickness
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            anchors.topMargin: 12
            x: dock.left ? 0 : parent.width - width
            spacing: 6

            DockButton {
                iconPath: Icons.home
                label: qsTr("Open NectarLink")
                onClicked: dock.pageRequested("home")
            }
            Rectangle { width: 28; height: 1; color: Theme.outlineVariant; anchors.horizontalCenter: parent.horizontalCenter }
            DockButton {
                iconPath: Icons.bell
                label: qsTr("Notifications")
                badge: NotificationList.count
                selected: dock.panel === "notifications"
                onClicked: dock.togglePanel("notifications")
            }
            DockButton {
                iconPath: Icons.messages
                label: qsTr("Messages")
                onClicked: dock.pageRequested("messages")
            }
            DockButton {
                iconPath: Icons.call
                label: qsTr("Calls")
                onClicked: dock.pageRequested("calls")
            }
            DockButton {
                iconPath: Icons.photo
                label: qsTr("Photos")
                onClicked: dock.pageRequested("photos")
            }
            DockButton {
                iconPath: Icons.apps
                label: qsTr("Apps")
                onClicked: dock.pageRequested("apps")
            }
            DockButton {
                iconPath: Icons.shelf
                label: qsTr("Files and clipboard (Shelf)")
                onClicked: AppController.shelfOpen = !AppController.shelfOpen
            }
        }

        Column {
            width: dock.thickness
            anchors.bottom: parent.bottom
            anchors.bottomMargin: 12
            x: strip.x
            spacing: 6
            DockButton {
                iconPath: Icons.window
                label: qsTr("Use as a window")
                onClicked: {
                    Preferences.dock = false
                    dock.pageRequested("home")
                }
            }
            DockButton {
                iconPath: dock.left ? Icons.chevronRight : Icons.back
                label: dock.left ? qsTr("Move to the right edge") : qsTr("Move to the left edge")
                onClicked: Preferences.dockEdge = dock.left ? "right" : "left"
            }
            DockButton {
                iconPath: Icons.settings
                label: qsTr("Settings")
                onClicked: dock.pageRequested("settings")
            }
        }

        // The panel that slides out of the strip.
        Item {
            id: panelArea
            visible: dock.expanded
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            anchors.margins: 12
            x: dock.left ? dock.thickness + 4 : 12
            width: parent.width - dock.thickness - 16

            Row {
                id: panelHeader
                width: parent.width
                height: 36
                Txt {
                    width: parent.width - closePanel.width
                    anchors.verticalCenter: parent.verticalCenter
                    role: "title"
                    text: qsTr("Notifications")
                    elide: Text.ElideRight
                }
                IconButton {
                    id: closePanel
                    iconPath: Icons.close
                    label: qsTr("Close")
                    onClicked: dock.panel = ""
                }
            }

            Txt {
                visible: NotificationList.count === 0
                anchors.top: panelHeader.bottom
                anchors.topMargin: 24
                width: parent.width
                horizontalAlignment: Text.AlignHCenter
                role: "body"
                muted: true
                wrapMode: Text.WordWrap
                text: qsTr("No notifications from your phone.")
            }

            ListView {
                id: list
                anchors.top: panelHeader.bottom
                anchors.topMargin: 8
                anchors.bottom: parent.bottom
                width: parent.width
                clip: true
                spacing: 6
                model: NotificationList
                delegate: Rectangle {
                    id: card
                    required property string deviceId
                    required property string key
                    required property string appName
                    required property string title
                    required property string text
                    required property string iconUrl
                    width: ListView.view.width
                    height: cardColumn.implicitHeight + 20
                    radius: Theme.radiusMd
                    color: Theme.surfaceContainerHigh

                    Image {
                        id: appIcon
                        x: 10
                        y: 10
                        width: 24
                        height: 24
                        source: card.iconUrl
                        sourceSize: Qt.size(48, 48)
                        fillMode: Image.PreserveAspectFit
                    }
                    Column {
                        id: cardColumn
                        x: 44
                        y: 10
                        width: card.width - 44 - dismiss.width - 8
                        spacing: 2
                        Txt { width: parent.width; role: "caption"; muted: true; text: card.appName; elide: Text.ElideRight }
                        Txt { width: parent.width; role: "label"; text: card.title; elide: Text.ElideRight; visible: text.length > 0 }
                        Txt {
                            width: parent.width
                            role: "body"
                            text: card.text
                            wrapMode: Text.WordWrap
                            maximumLineCount: 3
                            elide: Text.ElideRight
                            visible: text.length > 0
                        }
                    }
                    IconButton {
                        id: dismiss
                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.margins: 4
                        iconPath: Icons.close
                        label: qsTr("Dismiss")
                        onClicked: NotificationList.dismiss(card.deviceId, card.key)
                    }
                }
            }
        }
    }

    component DockButton: Item {
        id: button
        property string iconPath
        property string label
        property int badge: 0
        property bool selected: false
        signal clicked

        width: dock.thickness
        height: 44

        Rectangle {
            anchors.centerIn: parent
            width: 44
            height: 40
            radius: Theme.radiusMd
            color: button.selected ? Theme.primaryContainer
                 : mouse.containsMouse ? Theme.surfaceContainerHighest : "transparent"
        }
        Icon {
            anchors.centerIn: parent
            path: button.iconPath
            width: 22
            height: 22
            color: button.selected ? Theme.primaryContainerContent : Theme.surfaceContentVariant
        }
        Rectangle {
            visible: button.badge > 0
            x: parent.width / 2 + 6
            y: 2
            height: 18
            width: Math.max(18, badgeText.implicitWidth + 8)
            radius: 9
            color: Theme.primary
            Txt {
                id: badgeText
                anchors.centerIn: parent
                role: "caption"
                color: Theme.primaryContent
                text: button.badge > 99 ? "99+" : button.badge
            }
        }
        MouseArea {
            id: mouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: button.clicked()
        }
        Accessible.role: Accessible.Button
        Accessible.name: button.label
        Accessible.onPressAction: button.clicked()
    }
}
