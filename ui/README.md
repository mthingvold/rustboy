# Pocketbox UI (skeleton)

Qt 6 / QML desktop shell for rustboy, bridged to Rust with
[cxx-qt](https://github.com/KDAB/cxx-qt). It implements the windowing from
`ui-design/DESIGN_SPEC.md`; it does not run Game Boy games yet.

The UI never calls into `src/`. It talks to the `EmulatorCore` trait in
`src/core/mod.rs`, whose methods are the *planned* core functions listed in
`PLANNED_WORK.md`. Until the core provides them, `TwinCore` (a digital twin)
stands in and draws a test scene: the D-pad moves the block, Z jumps, and the
cells at the top-left count frames in binary.

## Build and run

Needs Qt 6 (Quick, QuickControls2, Dialogs) and a C++17 compiler.

```sh
cargo run -p pocketbox-ui                 # from the repository root
cargo run -p pocketbox-ui -- --load path/to/game.gb
cargo test -p pocketbox-ui
```

Settings and the library cache live in `~/.config/pocketbox/`
(override with `POCKETBOX_CONFIG_DIR`). For screenshots,
`--show folders|about|exit|log|paused` opens that surface at start.

Keys: arrows = D-pad, Z = A, X = B, Enter = Start, Shift = Select,
Space = pause, N = advance frame, Ctrl+1…5 = speed.

## Layout

| Path | What |
|---|---|
| `qml/Theme.qml` | Design tokens (colours, fonts, spacing) |
| `qml/Main.qml` | Window, menu bar, actions and shortcuts, status bar, dialogs |
| `qml/LibraryView.qml`, `GameCard.qml` | Library sidebar, search and grid |
| `qml/GameView.qml` | Integer-scaled LCD, overlays, control strip |
| `qml/LogViewer.qml` | Separate Log Viewer window |
| `qml/*Dialog.qml` | Folders, About, Exit, error dialogs |
| `src/bridge/` | QObjects exposed to QML (`Emulator`, `LcdItem`, `Library`, `LogBridge`) |
| `src/core/` | Planned core API (`EmulatorCore`) and the digital twin |
| `src/emu_thread.rs` | Emulator thread: commands, frame pacing, status |
| `src/library.rs`, `rom.rs`, `settings.rs` | Folder scanning, ROM headers, persistence |
| `src/logging.rs` | `log` backend feeding the Log Viewer |
| `fonts/` | IBM Plex Sans/Mono and Silkscreen (SIL OFL, licences included) |

## Not built yet

Save states, rewind and the test runner are phases 5–6 and depend on core
functions listed in `PLANNED_WORK.md`; their menu items and buttons are
present but disabled. Reduce-motion detection isn't wired up
(`Theme.reduceMotion`).
