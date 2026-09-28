import QtQuick
import QtQuick.Controls.Basic
import Pocketbox

// Text button. kind: "primary" (accent fill), "secondary" (outlined), "danger" (danger fill),
// "link" (underlined text) or "ghost" (plain text, hover background).
AbstractButton {
    id: control
    property string kind: "secondary"
    property string iconName: ""
    property int fontSize: 13
    property int radius: Theme.radiusMd

    implicitHeight: 38
    implicitWidth: row.implicitWidth + (kind === "primary" ? 36 : 32)
    opacity: enabled ? 1 : 0.4
    focusPolicy: Qt.StrongFocus
    hoverEnabled: true
    Accessible.name: text

    readonly property color fg: kind === "primary" ? Theme.textOnAccent
                               : kind === "danger" ? "#FFFFFF"
                               : kind === "link" ? Theme.text2 : Theme.text

    background: Rectangle {
        radius: control.radius
        color: control.kind === "primary" ? Theme.accent
             : control.kind === "danger" ? Theme.dangerFill
             : (control.hovered && control.enabled && control.kind !== "link") ? Theme.bgHover : "transparent"
        border.width: control.kind === "secondary" ? 1 : 0
        border.color: Theme.borderStrong
        Rectangle {
            anchors.fill: parent
            anchors.margins: -4
            radius: parent.radius + 4
            color: "transparent"
            border.width: control.visualFocus ? 2 : 0
            border.color: Theme.accent
        }
    }
    contentItem: Item {
        Row {
            id: row
            anchors.centerIn: parent
            spacing: 8
            Icon {
                visible: control.iconName.length > 0
                name: control.iconName
                size: 15
                color: control.fg
                anchors.verticalCenter: parent.verticalCenter
            }
            Text {
                text: control.text
                color: control.fg
                font.family: Theme.fontUi
                font.pixelSize: control.fontSize
                font.weight: (control.kind === "primary" || control.kind === "danger") ? Font.DemiBold : Font.Normal
                font.underline: control.kind === "link"
                anchors.verticalCenter: parent.verticalCenter
            }
        }
    }
}
