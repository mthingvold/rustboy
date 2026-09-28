import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Pocketbox

// Game view: integer-scaled LCD in a bezel, state overlays, and the control strip.
Rectangle {
    id: view
    required property var emulator
    required property int paletteIndex
    // Actions from Main.qml so menu, shortcuts and buttons share enabled state
    required property var actions

    color: Theme.bgGame

    readonly property var lcdColors: Theme.lcdPalettes[paletteIndex]
    readonly property bool poweredOn: emulator.loaded && emulator.powered

    // Largest integer scale that fits beside the bezel padding and control strip
    readonly property int scale: Math.max(1, Math.floor(Math.min(
        (width - 40 - 32) / 160,
        (height - 40 - 14 - strip.height - 32) / 144)))

    function refresh() { lcd.refresh() }

    // Joypad input; Space, N, Ctrl+… are handled as window shortcuts first
    focus: true
    Keys.onPressed: event => {
        if (!event.isAutoRepeat && emulator.keyEvent(event.key, true)) event.accepted = true
    }
    Keys.onReleased: event => {
        if (!event.isAutoRepeat && emulator.keyEvent(event.key, false)) event.accepted = true
    }
    TapHandler { onTapped: view.forceActiveFocus() }

    ColumnLayout {
        anchors.centerIn: parent
        spacing: 14

        // Bezel
        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            implicitWidth: 160 * view.scale + 40
            implicitHeight: 144 * view.scale + 40
            radius: 16
            color: Theme.bezel
            border.color: Theme.bgRaised

            LcdItem {
                id: lcd
                x: 20
                y: 20
                width: 160 * view.scale
                height: 144 * view.scale
                lcdPalette: view.paletteIndex
                onLcdPaletteChanged: refresh()
                Accessible.role: Accessible.Graphic
                Accessible.name: "Game screen"

                // Rewinding tag (shown once rewind exists; see PLANNED_WORK.md)
                Rectangle {
                    visible: false
                    x: 16; y: 16
                    width: rewindRow.implicitWidth + 24
                    height: 30
                    color: view.lcdColors[3]
                    Row {
                        id: rewindRow
                        anchors.centerIn: parent
                        spacing: 8
                        Icon { name: "rewind"; size: 16; color: view.lcdColors[0]; anchors.verticalCenter: parent.verticalCenter }
                        Text { text: "REWIND"; font.family: Theme.fontPixel; font.pixelSize: 14; color: view.lcdColors[0] }
                    }
                }

                // Paused
                Rectangle {
                    anchors.fill: parent
                    visible: view.poweredOn && view.emulator.paused
                    color: Theme.pausedOverlay
                    Column {
                        anchors.centerIn: parent
                        spacing: 10
                        Text {
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: "PAUSED"
                            font.family: Theme.fontPixel
                            font.pixelSize: Math.max(20, 8.5 * view.scale)
                            color: Theme.text
                        }
                        Text {
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: "Frame " + view.emulator.frame + " · press N or use Advance Frame to step"
                            font.family: Theme.fontMono
                            font.pixelSize: 13
                            color: "#D6D2C6"
                        }
                    }
                }

                // Powered off
                Rectangle {
                    anchors.fill: parent
                    visible: view.emulator.loaded && !view.emulator.powered
                    color: Theme.lcdOff
                    Column {
                        anchors.centerIn: parent
                        spacing: 14
                        Icon { anchors.horizontalCenter: parent.horizontalCenter; name: "power"; size: 36; color: Theme.textMuted }
                        Text {
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: "Powered off"
                            font.family: Theme.fontUi
                            font.pixelSize: 15
                            color: Theme.textMuted
                        }
                        PbButton {
                            anchors.horizontalCenter: parent.horizontalCenter
                            kind: "primary"
                            text: "Power on"
                            onClicked: view.emulator.togglePower()
                        }
                    }
                }
            }
        }

        // Control strip
        Rectangle {
            id: strip
            Layout.alignment: Qt.AlignHCenter
            implicitWidth: stripRow.implicitWidth + 12
            implicitHeight: 52
            radius: 12
            color: Theme.bgBar
            border.color: Theme.border

            RowLayout {
                id: stripRow
                anchors.centerIn: parent
                spacing: 6

                IconButton {
                    iconName: view.emulator.paused ? "play" : "pause"
                    tip: (view.emulator.paused ? "Resume" : "Pause") + " (Space)"
                    enabled: view.actions.pause.enabled
                    onClicked: view.actions.pause.trigger()
                }
                IconButton {
                    iconName: "step-forward"
                    tip: "Advance frame (N) — pause first"
                    enabled: view.actions.advance.enabled
                    onClicked: view.actions.advance.trigger()
                }
                IconButton {
                    iconName: "rewind"
                    tip: "Rewind (Backspace) — needs save-state support in the core"
                    enabled: view.actions.rewind.enabled
                    onClicked: view.actions.rewind.trigger()
                }
                Rectangle { implicitWidth: 1; implicitHeight: 24; Layout.leftMargin: 4; Layout.rightMargin: 4; color: Theme.border }
                IconButton {
                    iconName: "rotate-ccw"
                    tip: "Reset (Ctrl+R)"
                    enabled: view.actions.reset.enabled
                    onClicked: view.actions.reset.trigger()
                }
                IconButton {
                    iconName: "power"
                    iconColor: Theme.danger
                    tip: view.emulator.powered ? "Power off" : "Power on"
                    enabled: view.actions.power.enabled
                    onClicked: view.actions.power.trigger()
                }
                Rectangle { implicitWidth: 1; implicitHeight: 24; Layout.leftMargin: 4; Layout.rightMargin: 4; color: Theme.border }

                // Speed segmented control
                Rectangle {
                    implicitWidth: speedRow.implicitWidth + 6
                    implicitHeight: 40
                    radius: Theme.radiusMd
                    color: Theme.bgApp
                    Accessible.role: Accessible.Grouping
                    Accessible.name: "Emulation speed"
                    Row {
                        id: speedRow
                        anchors.centerIn: parent
                        spacing: 2
                        Repeater {
                            model: [0.25, 0.5, 1, 2, 4]
                            delegate: AbstractButton {
                                required property real modelData
                                readonly property bool selected: view.emulator.speed === modelData
                                implicitWidth: Math.max(46, label.implicitWidth + 16)
                                implicitHeight: 34
                                enabled: view.emulator.loaded
                                opacity: enabled ? 1 : 0.4
                                hoverEnabled: true
                                focusPolicy: Qt.StrongFocus
                                checkable: false
                                Accessible.name: "Speed " + modelData + "×"
                                Accessible.checked: selected
                                onClicked: view.emulator.chooseSpeed(modelData)
                                background: Rectangle {
                                    radius: Theme.radiusSm
                                    color: selected ? Theme.bgActive : (parent.hovered ? Theme.bgHover : "transparent")
                                    border.width: parent.visualFocus ? 2 : 0
                                    border.color: Theme.accent
                                }
                                contentItem: Text {
                                    id: label
                                    text: modelData + "×"
                                    horizontalAlignment: Text.AlignHCenter
                                    verticalAlignment: Text.AlignVCenter
                                    font.family: Theme.fontMono
                                    font.pixelSize: 12
                                    color: selected ? Theme.accent : Theme.textMuted
                                }
                            }
                        }
                    }
                }

                Rectangle { implicitWidth: 1; implicitHeight: 24; Layout.leftMargin: 4; Layout.rightMargin: 4; color: Theme.border }
                PbButton {
                    kind: "ghost"
                    implicitHeight: 40
                    iconName: "save"
                    text: "Save"
                    enabled: view.actions.quickSave.enabled
                    ToolTip.visible: hovered
                    ToolTip.text: "Quick save (F5) — needs save-state support in the core"
                    onClicked: view.actions.quickSave.trigger()
                }
                PbButton {
                    kind: "ghost"
                    implicitHeight: 40
                    iconName: "download"
                    text: "Load"
                    enabled: view.actions.quickLoad.enabled
                    ToolTip.visible: hovered
                    ToolTip.text: "Quick load (F8)"
                    onClicked: view.actions.quickLoad.trigger()
                }
            }
        }
    }
}
