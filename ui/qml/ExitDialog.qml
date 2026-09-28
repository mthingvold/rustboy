import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Pocketbox

// "Quit Pocketbox?" confirmation.
ModalDialog {
    id: dialog
    property bool gameRunning: false
    property string gameTitle: ""
    signal quitConfirmed()

    width: 440
    onAccepted: quitConfirmed()

    contentItem: ColumnLayout {
        spacing: 14

        Text {
            text: "Quit Pocketbox?"
            font.family: Theme.fontUi
            font.pixelSize: 18
            font.weight: Font.DemiBold
            color: Theme.text
        }
        Text {
            Layout.fillWidth: true
            wrapMode: Text.Wrap
            lineHeight: 1.2
            text: dialog.gameRunning
                  ? dialog.gameTitle + " is still running. Progress since your last save will be lost unless you quick save."
                  : "Your library and settings are saved automatically."
            font.family: Theme.fontUi
            font.pixelSize: 13
            color: Theme.textMuted
        }
        PbCheckBox {
            visible: dialog.gameRunning
            // Needs save states in the core; see PLANNED_WORK.md
            enabled: false
            checked: false
            text: "Quick save before quitting"
        }
        RowLayout {
            Layout.topMargin: 4
            spacing: 10
            Item { Layout.fillWidth: true }
            PbButton { text: "Cancel"; onClicked: dialog.close() }
            PbButton { kind: "danger"; text: "Quit"; onClicked: dialog.quitConfirmed() }
        }
    }
}
