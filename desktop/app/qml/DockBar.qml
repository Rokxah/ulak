// SPDX-License-Identifier: GPL-3.0-or-later
import QtQuick
import QtQuick.Window
import app.nectarlink

// The dock: a strip at the left or right screen edge (Preferences.dockEdge)
// that windows don't cover. Its buttons open the phone's notifications,
// conversations and recent calls in a panel that slides out of the strip,
// or a page of the main window. With Preferences.dockAutoHide it waits in a
// thin strip at the edge until the pointer comes.
DockWindow {
    id: dock

    // Opens a page of the main window ("home", "messages", "calls"...).
    signal pageRequested(string page)

    title: qsTr("Ulak dock")
    color: "transparent"
    edge: Preferences.dockEdge
    autoHide: Preferences.dockAutoHide
    // The strip is 56 wide, plus a gap to the screen edge.
    thickness: 64
    expandedWidth: 420
    // Out of the way while something runs full screen.
    // As tall as the buttons; taller while the panel is open.
    contentHeight: expanded
        ? Math.max(buttons.implicitHeight + 24, Math.min(Math.round(Screen.height * 0.75), 760))
        : buttons.implicitHeight + 24
    visible: !fullScreenApp

    // What the panel shows: "" (closed), "notifications", "messages" or "calls".
    property string panel: ""
    expanded: panel.length > 0
    readonly property bool left: edge === "left"
    // Hidden in the thin strip: only the strip is drawn.
    readonly property bool tucked: autoHide && !revealed && !expanded
    // Unused for a while: drawn at Preferences.dockIdleOpacity.
    property bool idle: false
    readonly property real restingOpacity: Math.max(0.1, Math.min(1, Preferences.dockIdleOpacity / 100))

    readonly property int deviceIndex: DeviceList.count > 0
        ? Math.min(Math.max(0, AppController.currentDevice), DeviceList.count - 1) : 0
    readonly property string deviceId: DeviceList.count > 0 ? DeviceList.deviceIdAt(deviceIndex) : ""

    readonly property var threads: { try { return JSON.parse(Messages.threads) } catch (e) { return [] } }
    readonly property var calls: { try { return JSON.parse(PhoneCall.callLog) } catch (e) { return [] } }
    readonly property int unreadThreads: threads.filter(t => t.unread).length
    readonly property int missedCalls: {
        let n = 0
        for (const c of calls) {
            if (c.direction !== "missed")
                break
            n++
        }
        return n
    }

    function togglePanel(name) {
        dock.panel = dock.panel === name ? "" : name
        if (dock.panel === "messages" && dock.deviceId.length > 0)
            Messages.open(dock.deviceId)
        else if (dock.panel === "calls" && dock.deviceId.length > 0)
            PhoneCall.open(dock.deviceId)
    }

    // Opens a conversation in the main window.
    function openThread(id) {
        dock.pageRequested("messages")
        pendingThread.thread = id
        pendingThread.restart()
    }
    Timer {
        id: pendingThread
        property string thread: ""
        interval: 400
        onTriggered: Messages.openThread(thread)
    }

    // Pointer at the edge: come out; gone for a moment: go back in.
    HoverHandler {
        id: hover
        onHoveredChanged: {
            if (hovered) {
                hideTimer.stop()
                idleTimer.stop()
                dock.idle = false
                dock.revealed = true
            } else {
                hideTimer.restart()
                idleTimer.restart()
            }
        }
    }
    Timer {
        id: hideTimer
        interval: 250
        onTriggered: if (!hover.hovered) dock.revealed = false
    }
    Timer {
        id: idleTimer
        interval: 2500
        running: true
        onTriggered: if (!hover.hovered && !dock.expanded) dock.idle = true
    }
    onExpandedChanged: {
        dock.idle = false
        if (!expanded)
            idleTimer.restart()
    }

    function shortTime(ms) {
        if (!ms)
            return ""
        const d = new Date(ms)
        const today = new Date()
        return d.toDateString() === today.toDateString()
            ? d.toLocaleTimeString(Qt.locale(), Locale.ShortFormat)
            : d.toLocaleDateString(Qt.locale(), Locale.ShortFormat)
    }

    // The thin strip while tucked away, at the screen edge.
    Rectangle {
        visible: dock.tucked
        x: dock.left ? 1 : parent.width - width - 1
        y: 12
        width: dock.peek - 2
        height: parent.height - 24
        radius: width / 2
        color: Theme.primary
        opacity: 0.75
    }

    Rectangle {
        id: pill
        anchors.fill: parent
        opacity: dock.tucked ? 0 : (dock.idle ? dock.restingOpacity : 1)
        Behavior on opacity { NumberAnimation { duration: 180 } }
        // Slides in from the edge.
        transform: Translate {
            x: dock.tucked ? (dock.left ? -dock.width : dock.width) : 0
            Behavior on x { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }
        }
        anchors.margins: 4
        anchors.leftMargin: dock.left ? 6 : 4
        anchors.rightMargin: dock.left ? 4 : 6
        radius: 18
        color: Theme.surfaceContainer
        border.color: Theme.outlineVariant
        border.width: 1

        // The buttons, in a column along the edge.
        Column {
            id: buttons
            width: 56
            anchors.top: parent.top
            anchors.topMargin: 8
            x: dock.left ? 0 : parent.width - width
            spacing: 4

            DockButton {
                iconPath: Icons.home
                label: qsTr("Open Ulak")
                onClicked: dock.pageRequested("home")
            }
            Separator {}
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
                badge: dock.unreadThreads
                selected: dock.panel === "messages"
                onClicked: dock.togglePanel("messages")
            }
            DockButton {
                iconPath: Icons.call
                label: qsTr("Calls")
                badge: dock.missedCalls
                selected: dock.panel === "calls"
                onClicked: dock.togglePanel("calls")
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
            Separator {}
            DockButton {
                iconPath: Icons.pin
                label: dock.autoHide ? qsTr("Keep the dock open") : qsTr("Hide the dock until the pointer comes")
                selected: !dock.autoHide
                onClicked: Preferences.dockAutoHide = !Preferences.dockAutoHide
            }
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
            x: dock.left ? buttons.width + 4 : 12
            width: parent.width - buttons.width - 16

            Row {
                id: panelHeader
                width: parent.width
                height: 36
                Txt {
                    width: parent.width - openPage.width - closePanel.width
                    anchors.verticalCenter: parent.verticalCenter
                    role: "title"
                    text: dock.panel === "messages" ? qsTr("Messages")
                        : dock.panel === "calls" ? qsTr("Recent calls")
                        : qsTr("Notifications")
                    elide: Text.ElideRight
                }
                IconButton {
                    id: openPage
                    visible: dock.panel !== "notifications"
                    iconPath: Icons.openExternal
                    label: qsTr("Open in the window")
                    onClicked: dock.pageRequested(dock.panel)
                }
                IconButton {
                    id: closePanel
                    iconPath: Icons.close
                    label: qsTr("Close")
                    onClicked: dock.panel = ""
                }
            }

            Txt {
                id: emptyText
                readonly property int count: dock.panel === "messages" ? dock.threads.length
                    : dock.panel === "calls" ? dock.calls.length : NotificationList.count
                visible: count === 0
                anchors.top: panelHeader.bottom
                anchors.topMargin: 24
                width: parent.width
                horizontalAlignment: Text.AlignHCenter
                role: "body"
                muted: true
                wrapMode: Text.WordWrap
                text: dock.deviceId.length === 0 ? qsTr("No phone paired yet.")
                    : dock.panel === "messages" ? qsTr("No conversations yet.")
                    : dock.panel === "calls" ? qsTr("No recent calls.")
                    : qsTr("No notifications from your phone.")
            }

            // Notifications
            ListView {
                visible: dock.panel === "notifications"
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

            // Conversations
            ListView {
                visible: dock.panel === "messages"
                anchors.top: panelHeader.bottom
                anchors.topMargin: 8
                anchors.bottom: parent.bottom
                width: parent.width
                clip: true
                spacing: 4
                model: dock.panel === "messages" ? dock.threads : []
                delegate: PanelRow {
                    required property var modelData
                    width: ListView.view.width
                    photo: modelData.photo || ""
                    initials: modelData.title || "?"
                    heading: modelData.title || ""
                    detail: (modelData.lastSender ? modelData.lastSender + ": " : "") + (modelData.snippet || "")
                    caption: (modelData.appName && modelData.appName !== "SMS" ? modelData.appName + " · " : "") + dock.shortTime(modelData.date)
                    bold: !!modelData.unread
                    onClicked: dock.openThread(modelData.id)
                }
            }

            // Recent calls
            ListView {
                visible: dock.panel === "calls"
                anchors.top: panelHeader.bottom
                anchors.topMargin: 8
                anchors.bottom: parent.bottom
                width: parent.width
                clip: true
                spacing: 4
                model: dock.panel === "calls" ? dock.calls : []
                delegate: PanelRow {
                    id: callRow
                    required property var modelData
                    width: ListView.view.width
                    photo: modelData.photo || ""
                    initials: modelData.name || modelData.number || "?"
                    heading: modelData.name || modelData.number || ""
                    detail: modelData.direction === "missed" ? qsTr("Missed")
                        : modelData.direction === "outgoing" ? qsTr("Outgoing")
                        : modelData.direction === "rejected" ? qsTr("Declined")
                        : qsTr("Incoming")
                    caption: dock.shortTime(modelData.date)
                    alert: modelData.direction === "missed"
                    actionIcon: PhoneCall.canDial && modelData.number ? Icons.call : ""
                    actionLabel: qsTr("Call %1").arg(heading)
                    onActionClicked: PhoneCall.dial(callRow.modelData.number)
                    onClicked: dock.pageRequested("calls")
                }
            }
        }
    }

    // A conversation or call in a panel.
    component PanelRow: Rectangle {
        id: row
        property string photo
        property string initials
        property string heading
        property string detail
        property string caption
        property bool bold: false
        property bool alert: false
        property string actionIcon: ""
        property string actionLabel: ""
        signal clicked
        signal actionClicked

        height: 56
        radius: Theme.radiusMd
        color: rowMouse.containsMouse ? Theme.surfaceContainerHighest : Theme.surfaceContainerHigh

        MouseArea {
            id: rowMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: row.clicked()
        }
        Item {
            id: avatar
            x: 10
            anchors.verticalCenter: parent.verticalCenter
            width: 36
            height: 36
            Avatar {
                anchors.fill: parent
                visible: row.photo.length === 0
                name: row.initials
            }
            RoundedImage {
                anchors.fill: parent
                visible: row.photo.length > 0
                source: row.photo
                radius: width / 2
            }
        }
        Column {
            anchors.left: avatar.right
            anchors.leftMargin: 10
            anchors.right: action.visible ? action.left : parent.right
            anchors.rightMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            spacing: 1
            Row {
                width: parent.width
                Txt {
                    width: parent.width - when.implicitWidth - 6
                    role: "label"
                    weight: row.bold ? 700 : 500
                    color: row.alert ? Theme.error : Theme.surfaceContent
                    text: row.heading
                    elide: Text.ElideRight
                }
                Txt { id: when; leftPadding: 6; role: "caption"; muted: true; text: row.caption }
            }
            Txt {
                width: parent.width
                role: "bodySmall"
                muted: !row.bold && !row.alert
                color: row.alert ? Theme.error : (muted ? Theme.surfaceContentVariant : Theme.surfaceContent)
                text: row.detail
                elide: Text.ElideRight
            }
        }
        IconButton {
            id: action
            visible: row.actionIcon.length > 0
            anchors.right: parent.right
            anchors.rightMargin: 6
            anchors.verticalCenter: parent.verticalCenter
            iconPath: row.actionIcon
            label: row.actionLabel
            onClicked: row.actionClicked()
        }
        Accessible.role: Accessible.Button
        Accessible.name: row.heading + ", " + row.detail
        Accessible.onPressAction: row.clicked()
    }

    component Separator: Rectangle {
        width: 28
        height: 1
        x: 14
        color: Theme.outlineVariant
    }

    component DockButton: Item {
        id: button
        property string iconPath
        property string label
        property int badge: 0
        property bool selected: false
        signal clicked

        width: 56
        height: 42

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
