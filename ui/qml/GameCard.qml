import QtQuick
import QtQuick.Controls.Basic
import Pocketbox

// One library tile. The whole card is a button that launches the game.
AbstractButton {
    id: card
    property var game: ({})

    hoverEnabled: true
    focusPolicy: Qt.StrongFocus
    Accessible.name: "Play " + (game.title || "")
    ToolTip.visible: hovered && titleText.truncated
    ToolTip.text: game.title || ""
    ToolTip.delay: 600

    background: Item {}
    contentItem: Column {
        spacing: 10

        Item {
            width: parent.width
            height: 150

            Rectangle {
                id: cover
                width: parent.width
                height: 150
                radius: Theme.radiusLg
                color: card.game.c1 || Theme.bgBar
                y: card.hovered && !Theme.reduceMotion ? -3 : 0
                Behavior on y { NumberAnimation { duration: 150; easing.type: Easing.OutQuad } }
                border.width: card.visualFocus ? 2 : 0
                border.color: Theme.accent

                Text {
                    anchors.fill: parent
                    anchors.margins: 14
                    text: card.game.title || ""
                    wrapMode: Text.Wrap
                    elide: Text.ElideRight
                    maximumLineCount: 4
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                    lineHeight: 1.2
                    font.family: Theme.fontPixel
                    font.pixelSize: 17
                    color: card.game.c2 || Theme.text
                }

                Rectangle {
                    x: 10
                    anchors.bottom: parent.bottom
                    anchors.bottomMargin: 10
                    width: badge.implicitWidth + 12
                    height: badge.implicitHeight + 2
                    radius: 4
                    color: card.game.c2 || Theme.text
                    Text {
                        id: badge
                        anchors.centerIn: parent
                        text: card.game.sys || ""
                        font.family: Theme.fontMono
                        font.pixelSize: 10
                        font.weight: Font.Medium
                        color: card.game.c1 || Theme.bgBar
                    }
                }
            }
        }

        Column {
            width: parent.width
            spacing: 2
            Text {
                id: titleText
                width: parent.width
                text: card.game.title || ""
                elide: Text.ElideRight
                font.family: Theme.fontUi
                font.pixelSize: 14
                font.weight: Font.DemiBold
                color: Theme.text
            }
            Text {
                width: parent.width
                text: card.game.meta || ""
                elide: Text.ElideRight
                font.family: Theme.fontUi
                font.pixelSize: 12
                color: Theme.textMuted
            }
        }
    }
}
