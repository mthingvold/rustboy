import QtQuick
import QtQuick.Controls.Basic
import Pocketbox

// Menu row: 20px check column, label, right-aligned mono shortcut.
// Set `header: true` for a non-interactive section header.
MenuItem {
    id: item
    property bool header: false
    // Shows the check mark without making the item checkable (for radio groups bound to state)
    property bool mark: checked
    property string shortcutText: action && action.shortcutText !== undefined ? action.shortcutText : ""

    implicitWidth: 268
    implicitHeight: header ? 26 : 32
    enabled: !header && (action ? action.enabled : true)
    hoverEnabled: true
    focusPolicy: header ? Qt.NoFocus : Qt.StrongFocus
    leftPadding: header ? 36 : 8
    rightPadding: 10

    indicator: Item {}
    arrow: Item {}

    background: Rectangle {
        radius: Theme.radiusSm
        color: !item.header && item.enabled && item.highlighted ? Theme.bgActive : "transparent"
    }

    contentItem: Item {
        implicitHeight: item.header ? 26 : 32

        Text {
            visible: item.header
            anchors.left: parent.left
            anchors.bottom: parent.bottom
            anchors.bottomMargin: 4
            text: item.text.toUpperCase()
            font.family: Theme.fontUi
            font.pixelSize: 11
            font.letterSpacing: 0.66
            color: Theme.textFaint
        }

        Row {
            visible: !item.header
            anchors.fill: parent
            spacing: 8
            Item {
                width: 20
                height: parent.height
                Icon {
                    anchors.centerIn: parent
                    visible: item.mark
                    name: "check"
                    size: 15
                    color: Theme.accent
                }
            }
            Text {
                width: parent.width - 28 - shortcut.width
                height: parent.height
                verticalAlignment: Text.AlignVCenter
                text: item.text
                elide: Text.ElideRight
                font.family: Theme.fontUi
                font.pixelSize: 13
                color: item.enabled ? Theme.text : Theme.textDisabled
            }
            Text {
                id: shortcut
                height: parent.height
                verticalAlignment: Text.AlignVCenter
                text: item.shortcutText
                font.family: Theme.fontMono
                font.pixelSize: 11
                color: item.enabled ? Theme.textFaint : Theme.textDisabled
            }
        }
    }
}
