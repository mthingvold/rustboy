import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Dialogs
import QtQuick.Layouts
import Pocketbox

// Main window: menu bar, Library / Game views, status bar, toasts and dialogs.
ApplicationWindow {
    id: root
    title: "Pocketbox"
    width: 1280
    height: 800
    minimumWidth: 960
    minimumHeight: 720
    visible: true
    color: Theme.bgApp
    font.family: Theme.fontUi
    font.pixelSize: 14

    // "library" or "game"
    property string view: "library"
    property bool quitting: false

    readonly property bool inGame: view === "game" && emulator.loaded
    readonly property bool poweredOn: emulator.loaded && emulator.powered
    readonly property var folderList: JSON.parse(library.foldersJson)

    function launch(path) {
        emulator.loadRom(path)
        library.markPlayed(path)
        view = "game"
        gameView.forceActiveFocus()
    }

    function showLibrary() {
        if (poweredOn && !emulator.paused) emulator.requestPause(true)
        view = "library"
    }

    Emulator {
        id: emulator
        onToast: message => toast.show(message)
        onErrorOccurred: (title, message) => {
            if (!emulator.loaded) root.view = "library"
            messageDialog.show(title, message)
        }
        onFrameReady: gameView.refresh()
    }

    Library {
        id: library
        onScanFinished: (count, cancelled) => {
            if (cancelled) return
            if (foldersDialog.scanRequested) foldersDialog.close()
            toast.show("Found " + count + " ROM" + (count === 1 ? "" : "s"))
        }
    }

    // ---- Actions (menu items, shortcuts and control-strip buttons share these) ----

    Action {
        id: loadRomAction
        text: "Load ROM…"
        shortcut: "Ctrl+O"
        property string shortcutText: "Ctrl+O"
        onTriggered: romDialog.open()
    }
    // Save states need snapshot()/restore() in the core (PLANNED_WORK.md); disabled until then
    Action { id: quickSaveAction; text: "Quick Save"; enabled: false; property string shortcutText: "F5" }
    Action { id: quickLoadAction; text: "Quick Load"; enabled: false; property string shortcutText: "F8" }
    Action { id: saveStateAction; text: "Save State to File…"; enabled: false; property string shortcutText: "Ctrl+S" }
    Action { id: loadStateAction; text: "Load State from File…"; enabled: false; property string shortcutText: "Ctrl+L" }
    Action { id: aboutAction; text: "About Pocketbox"; onTriggered: aboutDialog.open() }
    Action {
        id: exitAction
        text: "Exit"
        shortcut: "Ctrl+Q"
        property string shortcutText: "Ctrl+Q"
        onTriggered: exitDialog.open()
    }

    Action {
        id: pauseAction
        text: emulator.paused ? "Resume" : "Pause"
        enabled: root.poweredOn
        shortcut: "Space"
        property string shortcutText: "Space"
        onTriggered: emulator.togglePause()
    }
    Action {
        id: resetAction
        text: "Reset"
        enabled: emulator.loaded
        shortcut: "Ctrl+R"
        property string shortcutText: "Ctrl+R"
        onTriggered: emulator.reset()
    }
    Action {
        id: powerAction
        text: emulator.powered ? "Power Off" : "Power On"
        enabled: emulator.loaded
        shortcut: "Ctrl+Shift+P"
        property string shortcutText: "Ctrl+⇧+P"
        onTriggered: emulator.togglePower()
    }
    Action {
        id: advanceAction
        text: "Advance Frame"
        enabled: root.poweredOn && emulator.paused
        shortcut: "N"
        property string shortcutText: "N"
        onTriggered: emulator.advanceFrame()
    }
    // Rewind needs a snapshot ring buffer (PLANNED_WORK.md)
    Action { id: rewindAction; text: "Rewind"; enabled: false; property string shortcutText: "Backspace" }

    Repeater {
        id: speedActions
        model: [0.25, 0.5, 1, 2, 4]
        delegate: Item {
            required property real modelData
            required property int index
            Shortcut {
                sequence: "Ctrl+" + (index + 1)
                enabled: emulator.loaded
                onActivated: emulator.chooseSpeed(modelData)
            }
        }
    }

    Action {
        id: logAction
        text: "Log Viewer"
        shortcut: "Ctrl+Shift+L"
        property string shortcutText: "Ctrl+⇧+L"
        onTriggered: logViewer.show()
    }
    // The headless test runner is phase 6
    Action { id: testsAction; text: "Run Emulator Tests"; enabled: false }

    // ---- Menu bar ----

    menuBar: MenuBar {
        id: menuBar
        implicitHeight: 36
        leftPadding: 8 + logo.width
        rightPadding: 8

        background: Rectangle {
            color: Theme.bgBar
            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Theme.border }

            Row {
                id: logo
                x: 14
                anchors.verticalCenter: parent.verticalCenter
                spacing: 8
                rightPadding: 12
                Rectangle {
                    width: 14; height: 16
                    color: "transparent"
                    border.width: 2
                    border.color: Theme.accent
                    radius: 2
                    anchors.verticalCenter: parent.verticalCenter
                    Rectangle { x: 4; y: 4; width: 6; height: 4; color: Theme.accent }
                }
                Text {
                    text: "Pocketbox"
                    font.family: Theme.fontPixel
                    font.pixelSize: 13
                    color: Theme.accent
                    anchors.verticalCenter: parent.verticalCenter
                }
            }

            PbButton {
                visible: root.view === "game"
                anchors.right: parent.right
                anchors.rightMargin: 8
                anchors.verticalCenter: parent.verticalCenter
                implicitHeight: 28
                radius: Theme.radiusSm
                kind: "ghost"
                iconName: "arrow-left"
                text: "Library"
                onClicked: root.showLibrary()
            }
        }

        delegate: MenuBarItem {
            id: menuBarItem
            implicitHeight: 28
            leftPadding: 10
            rightPadding: 10
            contentItem: Text {
                text: menuBarItem.text
                font.family: Theme.fontUi
                font.pixelSize: 13
                color: Theme.text
                verticalAlignment: Text.AlignVCenter
            }
            background: Rectangle {
                radius: Theme.radiusSm
                color: menuBarItem.highlighted || menuBarItem.menu.visible ? Theme.bgActive
                     : menuBarItem.hovered ? Theme.bgHover : "transparent"
            }
        }

        PbMenu {
            title: "File"
            PbMenuItem { action: loadRomAction }
            MenuSeparator { contentItem: Rectangle { implicitHeight: 1; color: Theme.border } topPadding: 5; bottomPadding: 5; leftPadding: 6; rightPadding: 6 }
            PbMenuItem { header: true; text: "Save states" }
            PbMenuItem { action: quickSaveAction }
            PbMenuItem { action: quickLoadAction }
            PbMenuItem { action: saveStateAction }
            PbMenuItem { action: loadStateAction }
            MenuSeparator { contentItem: Rectangle { implicitHeight: 1; color: Theme.border } topPadding: 5; bottomPadding: 5; leftPadding: 6; rightPadding: 6 }
            PbMenuItem { action: aboutAction }
            PbMenuItem { action: exitAction }
        }

        PbMenu {
            title: "Emulation"
            PbMenuItem { action: pauseAction }
            PbMenuItem { action: resetAction }
            PbMenuItem { action: powerAction }
            MenuSeparator { contentItem: Rectangle { implicitHeight: 1; color: Theme.border } topPadding: 5; bottomPadding: 5; leftPadding: 6; rightPadding: 6 }
            PbMenuItem { action: advanceAction }
            PbMenuItem { action: rewindAction }
            MenuSeparator { contentItem: Rectangle { implicitHeight: 1; color: Theme.border } topPadding: 5; bottomPadding: 5; leftPadding: 6; rightPadding: 6 }
            PbMenuItem { header: true; text: "Emulation speed" }
            PbMenuItem { text: "0.25×"; shortcutText: "Ctrl+1"; enabled: emulator.loaded; mark: emulator.speed === 0.25; onTriggered: emulator.chooseSpeed(0.25) }
            PbMenuItem { text: "0.5×"; shortcutText: "Ctrl+2"; enabled: emulator.loaded; mark: emulator.speed === 0.5; onTriggered: emulator.chooseSpeed(0.5) }
            PbMenuItem { text: "1×  (normal)"; shortcutText: "Ctrl+3"; enabled: emulator.loaded; mark: emulator.speed === 1; onTriggered: emulator.chooseSpeed(1) }
            PbMenuItem { text: "2×"; shortcutText: "Ctrl+4"; enabled: emulator.loaded; mark: emulator.speed === 2; onTriggered: emulator.chooseSpeed(2) }
            PbMenuItem { text: "4×"; shortcutText: "Ctrl+5"; enabled: emulator.loaded; mark: emulator.speed === 4; onTriggered: emulator.chooseSpeed(4) }
        }

        PbMenu {
            title: "Tools"
            PbMenuItem { action: logAction }
            PbMenuItem { action: testsAction }
        }
    }

    // ---- Views ----

    StackLayout {
        anchors.fill: parent
        currentIndex: root.view === "game" ? 1 : 0

        LibraryView {
            library: library
            onOpenFolders: foldersDialog.open()
            onOpenRom: romDialog.open()
            onLaunch: game => root.launch(game.path)
        }

        GameView {
            id: gameView
            emulator: emulator
            paletteIndex: library.lcdPalette
            actions: ({
                pause: pauseAction, advance: advanceAction, rewind: rewindAction,
                reset: resetAction, power: powerAction,
                quickSave: quickSaveAction, quickLoad: quickLoadAction
            })
        }
    }

    footer: StatusBar {
        dotColor: !root.inGame || !emulator.powered ? Theme.textDisabled
                : emulator.paused ? Theme.warn : Theme.accent
        stateText: !root.inGame ? "Idle"
                 : !emulator.powered ? "Powered off"
                 : emulator.paused ? "Paused" : "Running"
        detailText: root.inGame ? emulator.romTitle + " — " + emulator.romFile
                  : (library.hasScanned && root.folderList.length > 0)
                    ? library.gameCount + " game" + (library.gameCount === 1 ? "" : "s") + " in "
                      + root.folderList.length + " folder" + (root.folderList.length === 1 ? "" : "s")
                    : "No folders set"
        metaText: root.inGame
                  ? "Speed " + emulator.speed + "×  ·  "
                    + (root.poweredOn && !emulator.paused ? emulator.fps.toFixed(1) : "0.0")
                    + " fps  ·  frame " + emulator.frame
                  : ""
        onOpenLog: logViewer.show()
    }

    Toast {
        id: toast
        z: 100
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 48
    }

    // ---- Dialogs and windows ----

    FileDialog {
        id: romDialog
        title: "Load ROM"
        nameFilters: ["Game Boy ROMs (*.gb *.gbc *.zip)"]
        onAccepted: root.launch(selectedFile.toString())
    }

    FoldersDialog { id: foldersDialog; library: library }
    AboutDialog { id: aboutDialog }
    MessageDialog { id: messageDialog }
    ExitDialog {
        id: exitDialog
        gameRunning: root.poweredOn
        gameTitle: emulator.romTitle
        onQuitConfirmed: {
            root.quitting = true
            logViewer.close()
            Qt.quit()
        }
    }

    LogViewer { id: logViewer; visible: false }

    // Command line: --load <rom> opens a ROM at start. Development aid for screenshots:
    // --show folders|about|exit|log|paused opens that surface.
    function argValue(name) {
        const args = Qt.application.arguments
        const i = args.indexOf(name)
        return i >= 0 && i + 1 < args.length ? args[i + 1] : ""
    }

    Component.onCompleted: {
        // Settings must be loaded before anything (e.g. --load) records to them
        library.startup()
        const rom = argValue("--load")
        if (rom) launch(rom)
        const show = argValue("--show")
        if (show === "folders") foldersDialog.open()
        else if (show === "about") aboutDialog.open()
        else if (show === "exit") exitDialog.open()
        else if (show === "log") logViewer.show()
        else if (show === "paused") pauseLater.start()
    }

    Timer { id: pauseLater; interval: 700; onTriggered: emulator.requestPause(true) }
    onClosing: close => {
        if (!quitting) {
            close.accepted = false
            exitDialog.open()
        }
    }
}
