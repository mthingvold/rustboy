import QtQuick
import Pocketbox

// Bottom-centre confirmation pill; a new message replaces the current one.
Rectangle {
    id: toast
    property string message: ""

    function show(text) {
        message = text
        visible = true
        hideTimer.restart()
    }

    visible: false
    height: 38
    width: row.implicitWidth + 32
    radius: height / 2
    color: Theme.text
    Accessible.role: Accessible.AlertMessage
    Accessible.name: message

    Row {
        id: row
        anchors.centerIn: parent
        spacing: 8
        Icon {
            name: "check"
            size: 15
            color: "#4D6B12"
            anchors.verticalCenter: parent.verticalCenter
        }
        Text {
            text: toast.message
            font.family: Theme.fontUi
            font.pixelSize: 13
            font.weight: Font.Medium
            color: Theme.textOnAccent
            anchors.verticalCenter: parent.verticalCenter
        }
    }

    Timer {
        id: hideTimer
        interval: 2200
        onTriggered: toast.visible = false
    }
}
