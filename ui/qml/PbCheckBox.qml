import QtQuick
import QtQuick.Controls.Basic
import Pocketbox

CheckBox {
    id: control
    spacing: 10
    font.family: Theme.fontUi
    font.pixelSize: 13
    opacity: enabled ? 1 : 0.4
    indicator: Rectangle {
        x: control.leftPadding
        y: (control.height - height) / 2
        implicitWidth: 16
        implicitHeight: 16
        radius: 4
        color: control.checked ? Theme.accent : "transparent"
        border.width: control.checked ? 0 : 1
        border.color: Theme.borderDashed
        Icon {
            anchors.centerIn: parent
            visible: control.checked
            name: "check"
            size: 13
            stroke: 2.4
            color: Theme.textOnAccent
        }
        Rectangle {
            anchors.fill: parent
            anchors.margins: -4
            radius: 7
            color: "transparent"
            border.width: control.visualFocus ? 2 : 0
            border.color: Theme.accent
        }
    }
    contentItem: Text {
        text: control.text
        font: control.font
        color: Theme.text
        leftPadding: control.indicator.width + control.spacing
        verticalAlignment: Text.AlignVCenter
    }
}
