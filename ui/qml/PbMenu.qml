import QtQuick
import QtQuick.Controls.Basic
import Pocketbox

// Dropdown menu styled per spec §4: 280px, bg-raised, 6px padding.
Menu {
    id: menu
    width: 280
    padding: 6
    topMargin: 4
    delegate: PbMenuItem {}
    background: Rectangle {
        implicitWidth: 280
        color: Theme.bgRaised
        border.color: Theme.borderStrong
        radius: Theme.radiusLg
    }
}
