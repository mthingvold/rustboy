import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Pocketbox

// Library: sidebar with filters and scan folders, header with search, grid of games.
Rectangle {
    id: view
    required property var library
    signal openFolders()
    signal openRom()
    signal launch(var game)

    color: Theme.bgApp

    property string filter: "all"
    readonly property var games: JSON.parse(library.gamesJson)
    readonly property var folders: JSON.parse(library.foldersJson)
    readonly property bool hasLibrary: library.hasScanned && folders.length > 0

    readonly property var filters: [
        { key: "all", label: "All games" },
        { key: "recent", label: "Recently played" },
        { key: "gb", label: "Game Boy" },
        { key: "gbc", label: "Game Boy Color" }
    ]

    function matches(game, key) {
        if (key === "recent") return game.playedAt > 0
        if (key === "gb") return game.sys === "GB"
        if (key === "gbc") return game.sys === "GBC"
        return true
    }

    function countFor(key) {
        if (!hasLibrary) return 0
        return games.filter(g => matches(g, key)).length
    }

    readonly property var shown: {
        if (!hasLibrary) return []
        const q = search.text.trim().toLowerCase()
        let list = games.filter(g => matches(g, filter) && (!q || g.title.toLowerCase().indexOf(q) !== -1))
        if (filter === "recent") list.sort((a, b) => b.playedAt - a.playedAt)
        return list
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // Sidebar
        Rectangle {
            Layout.preferredWidth: 232
            Layout.fillHeight: true
            color: Theme.bgSidebar
            Rectangle { anchors.right: parent.right; width: 1; height: parent.height; color: Theme.borderSoft }

            ColumnLayout {
                anchors.fill: parent
                anchors.topMargin: 18
                anchors.bottomMargin: 18
                anchors.leftMargin: 12
                anchors.rightMargin: 12
                spacing: 2

                Text {
                    text: "BROWSE"
                    leftPadding: 10
                    bottomPadding: 8
                    font.family: Theme.fontUi
                    font.pixelSize: 11
                    font.letterSpacing: 0.66
                    color: Theme.textFaint
                }

                Repeater {
                    model: view.filters
                    delegate: AbstractButton {
                        required property var modelData
                        readonly property bool selected: view.filter === modelData.key
                        Layout.fillWidth: true
                        implicitHeight: 34
                        hoverEnabled: true
                        focusPolicy: Qt.StrongFocus
                        Accessible.name: modelData.label
                        onClicked: view.filter = modelData.key
                        background: Rectangle {
                            radius: 7
                            color: selected || parent.hovered ? Theme.bgHover : "transparent"
                            border.width: parent.visualFocus ? 2 : 0
                            border.color: Theme.accent
                        }
                        contentItem: RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 10
                            anchors.rightMargin: 10
                            Text {
                                Layout.fillWidth: true
                                text: modelData.label
                                font.family: Theme.fontUi
                                font.pixelSize: 13
                                color: selected ? Theme.text : Theme.text2
                            }
                            Text {
                                text: view.countFor(modelData.key)
                                font.family: Theme.fontMono
                                font.pixelSize: 11
                                color: Theme.textFaint
                            }
                        }
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    Layout.topMargin: 22
                    Text {
                        Layout.fillWidth: true
                        leftPadding: 10
                        text: "SCAN FOLDERS"
                        font.family: Theme.fontUi
                        font.pixelSize: 11
                        font.letterSpacing: 0.66
                        color: Theme.textFaint
                    }
                    IconButton {
                        implicitWidth: 28
                        implicitHeight: 28
                        radius: Theme.radiusSm
                        iconName: "plus"
                        iconSize: 15
                        iconColor: Theme.textMuted
                        tip: "Set folders to scan"
                        onClicked: view.openFolders()
                    }
                }

                Repeater {
                    model: view.folders
                    delegate: Item {
                        required property string modelData
                        Layout.fillWidth: true
                        implicitHeight: 30
                        HoverHandler { id: folderHover }
                        ToolTip.visible: folderHover.hovered
                        ToolTip.text: modelData
                        ToolTip.delay: 600
                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 10
                            anchors.rightMargin: 10
                            spacing: 8
                            Icon { name: "folder"; size: 14; color: Theme.textFaint }
                            Text {
                                Layout.fillWidth: true
                                text: modelData
                                elide: Text.ElideMiddle
                                font.family: Theme.fontUi
                                font.pixelSize: 12
                                color: Theme.text2
                            }
                        }
                    }
                }

                Text {
                    visible: view.folders.length === 0
                    leftPadding: 10
                    topPadding: 4
                    text: "No folders yet"
                    font.family: Theme.fontUi
                    font.pixelSize: 12
                    color: Theme.textFaint
                }

                Item { Layout.fillHeight: true }

                PbButton {
                    Layout.fillWidth: true
                    implicitHeight: 36
                    iconName: "folder-search"
                    text: "Manage folders…"
                    onClicked: view.openFolders()
                }
            }
        }

        // Main area
        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.topMargin: 28
            Layout.bottomMargin: 28
            Layout.leftMargin: 32
            Layout.rightMargin: 32
            spacing: 24

            RowLayout {
                Layout.fillWidth: true
                spacing: 14

                Text {
                    text: view.filters.find(f => f.key === view.filter).label
                    font.family: Theme.fontUi
                    font.pixelSize: 24
                    font.weight: Font.DemiBold
                    color: Theme.text
                }
                Rectangle {
                    implicitWidth: countText.implicitWidth + 16
                    implicitHeight: countText.implicitHeight + 4
                    radius: height / 2
                    color: Theme.bgPill
                    Text {
                        id: countText
                        anchors.centerIn: parent
                        text: view.shown.length
                        font.family: Theme.fontMono
                        font.pixelSize: 12
                        color: Theme.text2
                    }
                }
                Item { Layout.fillWidth: true }

                TextField {
                    id: search
                    Layout.preferredWidth: 280
                    Layout.preferredHeight: 38
                    leftPadding: 35
                    placeholderText: "Search games"
                    placeholderTextColor: Theme.textFaint
                    color: Theme.text
                    font.family: Theme.fontUi
                    font.pixelSize: 13
                    Accessible.name: "Search games"
                    background: Rectangle {
                        radius: Theme.radiusMd
                        color: Theme.bgBar
                        border.color: search.activeFocus ? Theme.accent : Theme.border
                        border.width: search.activeFocus ? 2 : 1
                        Icon {
                            x: 12
                            anchors.verticalCenter: parent.verticalCenter
                            name: "search"
                            size: 15
                            color: Theme.textMuted
                        }
                    }
                    Keys.onEscapePressed: { text = ""; focus = false }
                }

                Item {
                    implicitWidth: 44
                    implicitHeight: 44
                    IconButton {
                        anchors.fill: parent
                        radius: Theme.radiusLg
                        iconName: "folder-search"
                        iconSize: 20
                        iconColor: Theme.accent
                        tip: "Choose folders to scan for ROMs"
                        onClicked: view.openFolders()
                        background: Rectangle {
                            radius: Theme.radiusLg
                            color: parent.hovered ? Theme.bgHover : Theme.bgBar
                            border.color: parent.visualFocus ? Theme.accent : Theme.borderStrong
                            border.width: parent.visualFocus ? 2 : 1
                        }
                    }
                    // "Needs setup" ping while no folders are set
                    Rectangle {
                        visible: view.folders.length === 0
                        x: parent.width - 7; y: -3
                        width: 10; height: 10; radius: 5
                        color: Theme.warn
                        SequentialAnimation on scale {
                            running: parent.visible && !Theme.reduceMotion
                            loops: Animation.Infinite
                            NumberAnimation { from: 1; to: 2.4; duration: 1400; easing.type: Easing.OutQuad }
                        }
                        SequentialAnimation on opacity {
                            running: parent.visible && !Theme.reduceMotion
                            loops: Animation.Infinite
                            NumberAnimation { from: 0.9; to: 0; duration: 1400; easing.type: Easing.OutQuad }
                        }
                    }
                    Rectangle {
                        visible: view.folders.length === 0
                        x: parent.width - 7; y: -3
                        width: 10; height: 10; radius: 5
                        color: Theme.warn
                    }
                }
            }

            // Empty state
            ColumnLayout {
                visible: !view.hasLibrary
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.bottomMargin: 60
                spacing: 14

                Item { Layout.fillHeight: true }
                AbstractButton {
                    Layout.alignment: Qt.AlignHCenter
                    implicitWidth: 112
                    implicitHeight: 112
                    focusPolicy: Qt.StrongFocus
                    Accessible.name: "Choose folders to scan"
                    onClicked: view.openFolders()
                    background: Rectangle {
                        radius: 28
                        color: Theme.bgBar
                        border.width: 2
                        border.color: parent.visualFocus ? Theme.accent : Theme.borderDashed
                    }
                    contentItem: Item {
                        Icon { anchors.centerIn: parent; name: "folder-search"; size: 48; stroke: 1.4; color: Theme.accent }
                    }
                }
                Text {
                    Layout.alignment: Qt.AlignHCenter
                    Layout.topMargin: 8
                    text: "Where are your games?"
                    font.family: Theme.fontUi
                    font.pixelSize: 20
                    font.weight: Font.DemiBold
                    color: Theme.text
                }
                Text {
                    Layout.alignment: Qt.AlignHCenter
                    Layout.maximumWidth: 380
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.Wrap
                    lineHeight: 1.2
                    text: "Choose the folders that hold your ROMs. Pocketbox scans them for .gb and .gbc files and builds your library."
                    font.family: Theme.fontUi
                    font.pixelSize: 14
                    color: Theme.textMuted
                }
                PbButton {
                    Layout.alignment: Qt.AlignHCenter
                    Layout.topMargin: 6
                    implicitHeight: 42
                    kind: "primary"
                    fontSize: 14
                    text: "Choose folders to scan"
                    onClicked: view.openFolders()
                }
                PbButton {
                    Layout.alignment: Qt.AlignHCenter
                    implicitHeight: 36
                    kind: "link"
                    text: "Or open a single ROM…"
                    onClicked: view.openRom()
                }
                Item { Layout.fillHeight: true }
            }

            // No search results
            Text {
                visible: view.hasLibrary && view.shown.length === 0
                Layout.fillWidth: true
                Layout.topMargin: 60
                horizontalAlignment: Text.AlignHCenter
                text: search.text.length > 0 ? "No games match “" + search.text + "”." : "No games here yet."
                font.family: Theme.fontUi
                font.pixelSize: 14
                color: Theme.textMuted
            }
            Item { visible: view.hasLibrary && view.shown.length === 0; Layout.fillHeight: true }

            // Grid: auto-fill columns of at least 170px, 20px column gap, 24px row gap
            GridView {
                id: grid
                visible: view.hasLibrary && view.shown.length > 0
                Layout.fillWidth: true
                Layout.fillHeight: true
                // Each cell carries its 20px gap on the right; let the last gap overhang
                Layout.rightMargin: -20
                clip: true
                model: view.shown
                readonly property int columns: Math.max(1, Math.floor(width / (170 + 20)))
                cellWidth: Math.floor(width / columns)
                cellHeight: 150 + 10 + 40 + 24
                keyNavigationEnabled: true
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ScrollBar {}
                delegate: Item {
                    required property var modelData
                    width: grid.cellWidth
                    height: grid.cellHeight
                    GameCard {
                        width: parent.width - 20
                        y: 4
                        game: modelData
                        onClicked: view.launch(modelData)
                    }
                }
            }
        }
    }
}
