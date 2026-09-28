import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Dialogs
import QtQuick.Layouts
import Pocketbox

// Separate "Log Viewer — Pocketbox" window. Polls the Rust log buffer while visible.
Window {
    id: win
    title: "Log Viewer — Pocketbox"
    width: 760
    height: 480
    minimumWidth: 480
    minimumHeight: 240
    color: Theme.bgBar

    property string level: "ALL"
    property var entries: []
    property real lastSeq: 0
    property bool cleared: false
    property string exportedName: ""
    readonly property var colors: ({ "INFO": Theme.info, "WARN": Theme.warn, "ERROR": Theme.danger, "DEBUG": Theme.textMuted })

    LogBridge { id: bridge }

    function show() {
        visible = true
        raise()
        requestActivate()
        poll()
    }

    function matches(e) {
        if (level !== "ALL" && e.lvl !== level) return false
        const q = search.text.trim().toLowerCase()
        return !q || (e.msg + " " + e.sys).toLowerCase().indexOf(q) !== -1
    }

    function rebuild() {
        rows.clear()
        for (const e of entries) if (matches(e)) rows.append(e)
        scrollToEnd()
    }

    function poll() {
        const fresh = JSON.parse(bridge.since(lastSeq))
        if (fresh.length === 0) return
        lastSeq = fresh[fresh.length - 1].seq
        entries = entries.concat(fresh)
        // Keep the view in step with the Rust buffer's capacity
        if (entries.length > 10000) entries = entries.slice(entries.length - 10000)
        for (const e of fresh) if (matches(e)) rows.append(e)
        while (rows.count > 10000) rows.remove(0)
        scrollToEnd()
    }

    function scrollToEnd() {
        if (autoScroll.checked) list.positionViewAtEnd()
    }

    ListModel { id: rows }

    Timer {
        interval: 250
        running: win.visible
        repeat: true
        onTriggered: win.poll()
    }

    FileDialog {
        id: exportDialog
        title: "Export log"
        fileMode: FileDialog.SaveFile
        nameFilters: ["Log files (*.log)"]
        defaultSuffix: "log"
        onAccepted: {
            const name = bridge.exportTo(selectedFile.toString())
            if (name.length > 0) {
                win.exportedName = name
                exportedTimer.restart()
            }
        }
    }
    Timer { id: exportedTimer; interval: 1800; onTriggered: win.exportedName = "" }

    Shortcut { sequence: "Esc"; onActivated: win.close() }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Toolbar
        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 10
            Layout.leftMargin: 14
            Layout.rightMargin: 14
            spacing: 8

            Rectangle {
                implicitWidth: levelRow.implicitWidth + 6
                implicitHeight: 32
                radius: Theme.radiusMd
                color: Theme.bgApp
                Row {
                    id: levelRow
                    anchors.centerIn: parent
                    spacing: 4
                    Repeater {
                        model: ["ALL", "INFO", "WARN", "ERROR", "DEBUG"]
                        delegate: AbstractButton {
                            required property string modelData
                            readonly property bool selected: win.level === modelData
                            implicitHeight: 26
                            implicitWidth: lvlText.implicitWidth + 20
                            hoverEnabled: true
                            focusPolicy: Qt.StrongFocus
                            Accessible.name: lvlText.text
                            onClicked: { win.level = modelData; win.rebuild() }
                            background: Rectangle {
                                radius: Theme.radiusSm
                                color: selected ? Theme.bgActive : (parent.hovered ? Theme.bgHover : "transparent")
                                border.width: parent.visualFocus ? 2 : 0
                                border.color: Theme.accent
                            }
                            contentItem: Text {
                                id: lvlText
                                text: modelData === "ALL" ? "All" : modelData.charAt(0) + modelData.slice(1).toLowerCase()
                                horizontalAlignment: Text.AlignHCenter
                                verticalAlignment: Text.AlignVCenter
                                font.family: Theme.fontUi
                                font.pixelSize: 12
                                font.weight: Font.Medium
                                color: selected ? Theme.text : Theme.textMuted
                            }
                        }
                    }
                }
            }

            TextField {
                id: search
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                leftPadding: 30
                placeholderText: "Filter messages or subsystem"
                placeholderTextColor: Theme.textFaint
                color: Theme.text
                font.family: Theme.fontUi
                font.pixelSize: 12
                Accessible.name: "Filter log messages"
                onTextChanged: win.rebuild()
                background: Rectangle {
                    radius: Theme.radiusMd
                    color: Theme.bgApp
                    border.color: search.activeFocus ? Theme.accent : Theme.border
                    border.width: search.activeFocus ? 2 : 1
                    Icon { x: 10; anchors.verticalCenter: parent.verticalCenter; name: "search"; size: 14; color: Theme.textMuted }
                }
            }

            PbCheckBox {
                id: autoScroll
                text: "Auto-scroll"
                checked: true
                font.pixelSize: 12
                onToggled: win.scrollToEnd()
            }
        }
        Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.borderSoft }

        // Messages
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: Theme.bgLog

            ListView {
                id: list
                anchors.fill: parent
                anchors.topMargin: 6
                anchors.bottomMargin: 6
                clip: true
                model: rows
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ScrollBar {}
                delegate: Item {
                    required property string t
                    required property string lvl
                    required property string sys
                    required property string msg
                    width: list.width
                    height: 22
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 14
                        anchors.rightMargin: 14
                        spacing: 12
                        Text { text: t; font.family: Theme.fontMono; font.pixelSize: 12; color: "#7D786E" }
                        Text {
                            Layout.preferredWidth: 44
                            text: lvl
                            font.family: Theme.fontMono
                            font.pixelSize: 12
                            font.weight: Font.Medium
                            color: win.colors[lvl]
                        }
                        Text {
                            Layout.preferredWidth: 56
                            text: sys
                            elide: Text.ElideRight
                            font.family: Theme.fontMono
                            font.pixelSize: 12
                            color: Theme.text2
                        }
                        Text {
                            Layout.fillWidth: true
                            text: msg
                            elide: Text.ElideRight
                            font.family: Theme.fontMono
                            font.pixelSize: 12
                            color: Theme.text
                        }
                    }
                }
            }

            Text {
                visible: rows.count === 0
                anchors.horizontalCenter: parent.horizontalCenter
                y: 40
                text: win.cleared && win.entries.length === 0 ? "Log cleared. New messages will appear here." : "No messages match this filter."
                font.family: Theme.fontUi
                font.pixelSize: 13
                color: Theme.textMuted
            }
        }

        // Footer
        Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.borderSoft }
        RowLayout {
            Layout.fillWidth: true
            Layout.preferredHeight: 40
            Layout.leftMargin: 14
            Layout.rightMargin: 14
            spacing: 8
            Text {
                Layout.fillWidth: true
                text: "Showing " + rows.count + " of " + win.entries.length + " messages"
                font.family: Theme.fontUi
                font.pixelSize: 12
                color: Theme.textMuted
            }
            PbButton {
                implicitHeight: 28
                radius: Theme.radiusSm
                fontSize: 12
                text: "Clear"
                onClicked: {
                    bridge.clear()
                    win.entries = []
                    win.cleared = true
                    rows.clear()
                }
            }
            PbButton {
                implicitHeight: 28
                radius: Theme.radiusSm
                fontSize: 12
                kind: "primary"
                text: win.exportedName.length > 0 ? "Saved " + win.exportedName : "Export…"
                onClicked: exportDialog.open()
            }
        }
    }
}
