import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Pocketbox

ModalDialog {
    id: dialog
    width: 400
    topPadding: 32
    leftPadding: 28
    rightPadding: 28
    onAccepted: close()

    contentItem: ColumnLayout {
        spacing: 10

        // Logo: cartridge outline with label
        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            implicitWidth: 64; implicitHeight: 76
            color: "transparent"
            border.width: 5
            border.color: Theme.accent
            radius: 6
            Rectangle {
                anchors.horizontalCenter: parent.horizontalCenter
                y: 13
                width: 34; height: 24
                color: Theme.accent
            }
        }
        Text {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: 8
            text: "Pocketbox"
            font.family: Theme.fontPixel
            font.pixelSize: 24
            color: Theme.accent
        }
        Text {
            Layout.alignment: Qt.AlignHCenter
            text: "Version " + Qt.application.version
            font.family: Theme.fontUi
            font.pixelSize: 13
            color: Theme.text2
        }
        Text {
            Layout.fillWidth: true
            Layout.topMargin: 6
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.Wrap
            lineHeight: 1.2
            // [LICENSE] is a placeholder in the design spec; the repository has no license yet
            text: "A Game Boy and Game Boy Color emulator.\nLicense: [LICENSE]"
            font.family: Theme.fontUi
            font.pixelSize: 13
            color: Theme.textMuted
        }
        PbButton {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: 12
            kind: "primary"
            text: "Close"
            onClicked: dialog.close()
        }
    }
}
