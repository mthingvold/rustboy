import QtQuick
import QtQuick.Controls.Basic
import Pocketbox

// Square icon-only button with hover background, tooltip and accessible name.
AbstractButton {
    id: control
    property string iconName: ""
    property real iconSize: 18
    property color iconColor: Theme.text
    property string tip: ""
    property bool active: false
    property int radius: Theme.radiusMd
    property color activeColor: Theme.accent

    implicitWidth: 40
    implicitHeight: 40
    opacity: enabled ? 1 : 0.4
    focusPolicy: Qt.StrongFocus
    hoverEnabled: true
    Accessible.name: tip
    ToolTip.visible: hovered && tip.length > 0
    ToolTip.text: tip
    ToolTip.delay: 600

    background: Rectangle {
        radius: control.radius
        color: control.active ? control.activeColor : (control.hovered && control.enabled ? Theme.bgHover : "transparent")
        border.width: control.visualFocus ? 2 : 0
        border.color: Theme.accent
    }
    contentItem: Item {
        Icon {
            anchors.centerIn: parent
            name: control.iconName
            size: control.iconSize
            color: control.active ? Theme.textOnAccent : control.iconColor
        }
    }
}
