import QtQuick
import QtQuick.Shapes
import Pocketbox

// Stroke-only line icon (Lucide geometry, 24×24 viewBox) tinted with `color`.
Item {
    id: icon
    property string name
    property real size: 16
    property color color: Theme.text
    property real stroke: 1.75

    implicitWidth: size
    implicitHeight: size

    readonly property var paths: ({
        "play": "M6 3 L20 12 L6 21 Z",
        "pause": "M6 4h4v16h-4Z M14 4h4v16h-4Z",
        "step-forward": "M5 4 L15 12 L5 20 Z M19 5 L19 19",
        "rewind": "M11 19 L2 12 L11 5 Z M22 19 L13 12 L22 5 Z",
        "rotate-ccw": "M3 12a9 9 0 1 0 3-6.7L3 8 M3 3v5h5",
        "power": "M12 2v10 M18.4 6.6a9 9 0 1 1-12.77.04",
        "save": "M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z M17 21 L17 13 L7 13 L7 21 M7 3 L7 8 L15 8",
        "download": "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4 M7 10 L12 15 L17 10 M12 15 L12 3",
        "x": "M18 6 L6 18 M6 6 L18 18",
        "plus": "M12 5v14 M5 12h14",
        "trash-2": "M3 6h18 M8 6V4h8v2 M19 6l-1 14H6L5 6",
        "check": "M20 6 L9 17 L4 12",
        "search": "M18 11a7 7 0 1 1-14 0a7 7 0 1 1 14 0 M21 21l-4.3-4.3",
        "terminal": "M4 17 L10 11 L4 5 M12 19 L20 19",
        "arrow-left": "M19 12H5 M12 19l-7-7 7-7",
        "folder": "M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z",
        "folder-search": "M10.5 20H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H20a2 2 0 0 1 2 2v3 M20 17a3 3 0 1 1-6 0a3 3 0 1 1 6 0 M21 21l-1.9-1.9"
    })

    Shape {
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            scale: Qt.size(icon.size / 24, icon.size / 24)
            strokeColor: icon.color
            strokeWidth: icon.stroke * icon.size / 24
            fillColor: "transparent"
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            PathSvg { path: icon.paths[icon.name] || "" }
        }
    }
}
