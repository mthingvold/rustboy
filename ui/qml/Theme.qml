pragma Singleton
import QtQuick
import Pocketbox

// Design tokens from ui-design/DESIGN_SPEC.md §2.
QtObject {
    // Colour
    readonly property color bgApp: "#1C1B19"
    readonly property color bgSidebar: "#201F1C"
    readonly property color bgGame: "#141412"
    readonly property color bgLog: "#151513"
    readonly property color bgBar: "#242320"
    readonly property color bgRaised: "#2A2926"
    readonly property color bgHover: "#34322D"
    readonly property color bgActive: "#3A3833"
    readonly property color bgPill: "#2E2C28"
    readonly property color border: "#3A3833"
    readonly property color borderSoft: "#33312C"
    readonly property color borderStrong: "#45423C"
    readonly property color borderDashed: "#5A574F"
    readonly property color text: "#ECE8DF"
    readonly property color text2: "#C9C4B8"
    readonly property color textMuted: "#A8A397"
    readonly property color textFaint: "#8F8A7F"
    readonly property color textDisabled: "#6B675E"
    readonly property color accent: "#B5CC3A"
    readonly property color textOnAccent: "#1C1B19"
    readonly property color warn: "#E3B552"
    readonly property color danger: "#EF8A7C"
    readonly property color dangerFill: "#C4513F"
    readonly property color info: "#8FBCE6"
    readonly property color scrim: Qt.rgba(10 / 255, 10 / 255, 9 / 255, 0.62)
    readonly property color bezel: "#0C0C0B"
    readonly property color lcdOff: "#1A1F16"
    readonly property color pausedOverlay: Qt.rgba(15 / 255, 20 / 255, 12 / 255, 0.62)

    // LCD palettes, lightest to darkest (index matches Library.lcdPalette)
    readonly property var lcdPalettes: [
        ["#9BBC0F", "#8BAC0F", "#306230", "#0F380F"],
        ["#C6C6B8", "#9A9A8E", "#56564F", "#1F1F1C"]
    ]

    // Typography
    readonly property FontLoader plexSans: FontLoader { source: Qt.resolvedUrl("../fonts/IBMPlexSans-Variable.ttf") }
    readonly property FontLoader plexMono: FontLoader { source: Qt.resolvedUrl("../fonts/IBMPlexMono-Regular.ttf") }
    readonly property FontLoader plexMonoMedium: FontLoader { source: Qt.resolvedUrl("../fonts/IBMPlexMono-Medium.ttf") }
    readonly property FontLoader silkscreen: FontLoader { source: Qt.resolvedUrl("../fonts/Silkscreen-Regular.ttf") }

    readonly property string fontUi: plexSans.status === FontLoader.Ready ? plexSans.name : "sans-serif"
    readonly property string fontMono: plexMono.status === FontLoader.Ready ? plexMono.name : "monospace"
    readonly property string fontPixel: silkscreen.status === FontLoader.Ready ? silkscreen.name : "monospace"

    // Spacing and radius
    readonly property int space1: 4
    readonly property int space2: 8
    readonly property int space3: 12
    readonly property int space4: 16
    readonly property int space5: 24
    readonly property int space6: 32
    readonly property int radiusSm: 6
    readonly property int radiusMd: 8
    readonly property int radiusLg: 10
    readonly property int radiusXl: 14

    // Motion. Qt has no portable "reduce motion" query yet; when true, hover lift and ping stop.
    property bool reduceMotion: false
}
