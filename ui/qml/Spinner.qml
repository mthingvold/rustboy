import QtQuick
import Pocketbox

// 14px ring spinner (0.8s per turn).
Rectangle {
    width: 14
    height: 14
    radius: 7
    color: "transparent"
    border.width: 2
    border.color: Theme.borderDashed
    Rectangle {
        width: 14
        height: 7
        color: "transparent"
        clip: true
        Rectangle {
            width: 14
            height: 14
            radius: 7
            color: "transparent"
            border.width: 2
            border.color: Theme.accent
        }
    }
    RotationAnimator on rotation {
        from: 0
        to: 360
        duration: 800
        loops: Animation.Infinite
        running: visible
    }
}
