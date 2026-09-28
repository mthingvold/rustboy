import QtQuick
import QtQuick.Layouts
import Pocketbox

// 28px bar: status dot + state, ROM or library summary, run stats, Log button.
Rectangle {
    id: bar
    property color dotColor: Theme.textDisabled
    property string stateText: "Idle"
    property string detailText: ""
    property string metaText: ""
    signal openLog()

    implicitHeight: 28
    color: Theme.bgBar
    Rectangle { anchors.top: parent.top; width: parent.width; height: 1; color: Theme.border }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 14
        anchors.rightMargin: 14
        spacing: 16

        Row {
            spacing: 6
            Rectangle {
                width: 8; height: 8; radius: 4
                color: bar.dotColor
                anchors.verticalCenter: parent.verticalCenter
            }
            Text {
                text: bar.stateText
                font.family: Theme.fontUi
                font.pixelSize: 12
                color: Theme.textMuted
            }
        }
        Text {
            Layout.fillWidth: true
            text: bar.detailText
            elide: Text.ElideRight
            font.family: Theme.fontUi
            font.pixelSize: 12
            color: Theme.textMuted
        }
        Text {
            text: bar.metaText
            font.family: Theme.fontMono
            font.pixelSize: 12
            color: Theme.textMuted
        }
        PbButton {
            implicitHeight: 22
            radius: 5
            fontSize: 11
            iconName: "terminal"
            text: "Log"
            onClicked: bar.openLog()
        }
    }
}
