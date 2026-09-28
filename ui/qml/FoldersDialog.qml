import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Dialogs
import QtQuick.Layouts
import Pocketbox

// "Folders to scan": manage folders and options, then scan.
ModalDialog {
    id: dialog
    required property var library
    width: 560
    canAccept: folders.length > 0 && !library.scanning

    readonly property var folders: JSON.parse(library.foldersJson)
    // True while a scan started from this dialog is running; only that scan closes it
    property bool scanRequested: false

    onAccepted: { scanRequested = true; library.scan() }
    onOpened: scanRequested = false
    onClosed: if (library.scanning) library.cancelScan()

    FolderDialog {
        id: picker
        title: "Add folder"
        onAccepted: dialog.library.addFolder(selectedFolder.toString())
    }

    contentItem: ColumnLayout {
        spacing: 18

        RowLayout {
            spacing: 14
            Rectangle {
                Layout.alignment: Qt.AlignTop
                implicitWidth: 44; implicitHeight: 44
                radius: Theme.radiusLg
                color: Theme.bgHover
                Icon { anchors.centerIn: parent; name: "folder-search"; size: 22; color: Theme.accent }
            }
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4
                Text {
                    text: "Folders to scan"
                    font.family: Theme.fontUi
                    font.pixelSize: 18
                    font.weight: Font.DemiBold
                    color: Theme.text
                }
                Text {
                    Layout.fillWidth: true
                    wrapMode: Text.Wrap
                    text: "Pocketbox looks in these folders for .gb, .gbc and .zip files."
                    font.family: Theme.fontUi
                    font.pixelSize: 13
                    color: Theme.textMuted
                }
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 6

            Repeater {
                model: dialog.folders
                delegate: Rectangle {
                    required property string modelData
                    Layout.fillWidth: true
                    implicitHeight: 44
                    radius: Theme.radiusMd
                    color: rowHover.hovered ? "#2E2C28" : Theme.bgBar
                    HoverHandler { id: rowHover }
                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 12
                        anchors.rightMargin: 6
                        spacing: 10
                        Icon { name: "folder"; size: 16; color: Theme.textMuted }
                        Text {
                            Layout.fillWidth: true
                            text: modelData
                            elide: Text.ElideMiddle
                            font.family: Theme.fontMono
                            font.pixelSize: 13
                            color: Theme.text
                        }
                        IconButton {
                            implicitWidth: 32; implicitHeight: 32
                            radius: Theme.radiusSm
                            iconName: "trash-2"
                            iconSize: 15
                            iconColor: Theme.textMuted
                            tip: "Remove " + modelData
                            onClicked: dialog.library.removeFolder(modelData)
                        }
                    }
                }
            }

            // Empty: dashed outline
            Rectangle {
                visible: dialog.folders.length === 0
                Layout.fillWidth: true
                implicitHeight: 62
                radius: Theme.radiusMd
                color: "transparent"
                border.color: Theme.borderDashed
                Text {
                    anchors.centerIn: parent
                    text: "No folders added yet."
                    font.family: Theme.fontUi
                    font.pixelSize: 13
                    color: Theme.textMuted
                }
            }

            PbButton {
                Layout.topMargin: 4
                implicitHeight: 36
                iconName: "plus"
                text: "Add folder…"
                onClicked: picker.open()
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 10
            Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: Theme.border; Layout.bottomMargin: 4 }
            PbCheckBox {
                text: "Include subfolders"
                checked: dialog.library.includeSubfolders
                onToggled: { dialog.library.includeSubfolders = checked; dialog.library.saveSettings() }
            }
            PbCheckBox {
                text: "Rescan when Pocketbox starts"
                checked: dialog.library.rescanOnStart
                onToggled: { dialog.library.rescanOnStart = checked; dialog.library.saveSettings() }
            }
        }

        RowLayout {
            spacing: 10
            RowLayout {
                visible: dialog.library.scanning
                spacing: 10
                Spinner {}
                Text {
                    text: dialog.library.scannedFiles > 0 ? "Scanning… " + dialog.library.scannedFiles + " files" : "Scanning…"
                    font.family: Theme.fontUi
                    font.pixelSize: 13
                    color: Theme.text2
                }
            }
            Item { Layout.fillWidth: true }
            PbButton {
                text: "Cancel"
                onClicked: dialog.close()
            }
            PbButton {
                kind: "primary"
                enabled: dialog.canAccept
                text: dialog.folders.length === 0 ? "Scan"
                    : dialog.folders.length === 1 ? "Scan 1 folder"
                    : "Scan " + dialog.folders.length + " folders"
                onClicked: dialog.accepted()
            }
        }
    }
}
