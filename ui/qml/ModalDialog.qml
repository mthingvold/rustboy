import QtQuick
import QtQuick.Controls.Basic
import Pocketbox

// Base for modal dialogs: centred card over the scrim. Esc cancels; Enter triggers accept().
Popup {
    id: dialog
    signal accepted()
    property bool canAccept: true

    anchors.centerIn: Overlay.overlay
    modal: true
    focus: true
    padding: 24
    closePolicy: Popup.CloseOnEscape

    Overlay.modal: Rectangle { color: Theme.scrim }

    background: Rectangle {
        color: Theme.bgRaised
        border.color: Theme.borderStrong
        radius: Theme.radiusXl
    }

    // Enter = primary action
    Shortcut {
        enabled: dialog.opened && dialog.canAccept
        sequences: ["Return", "Enter"]
        onActivated: dialog.accepted()
    }
}
