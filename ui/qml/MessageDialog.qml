import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Pocketbox

// Simple error / information dialog with an OK button.
ModalDialog {
    id: dialog
    property string title: ""
    property string message: ""

    function show(t, m) {
        title = t
        message = m
        open()
    }

    width: 440
    onAccepted: close()

    contentItem: ColumnLayout {
        spacing: 14
        Text {
            text: dialog.title
            font.family: Theme.fontUi
            font.pixelSize: 18
            font.weight: Font.DemiBold
            color: Theme.text
        }
        Text {
            Layout.fillWidth: true
            wrapMode: Text.Wrap
            lineHeight: 1.2
            text: dialog.message
            font.family: Theme.fontUi
            font.pixelSize: 13
            color: Theme.textMuted
        }
        RowLayout {
            Layout.topMargin: 4
            Item { Layout.fillWidth: true }
            PbButton { kind: "primary"; text: "OK"; onClicked: dialog.close() }
        }
    }
}
