use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

const QML_FILES: &[&str] = &[
    "qml/Main.qml",
    "qml/Icon.qml",
    "qml/IconButton.qml",
    "qml/PbButton.qml",
    "qml/PbCheckBox.qml",
    "qml/PbMenu.qml",
    "qml/PbMenuItem.qml",
    "qml/StatusBar.qml",
    "qml/Toast.qml",
    "qml/Spinner.qml",
    "qml/LibraryView.qml",
    "qml/GameCard.qml",
    "qml/GameView.qml",
    "qml/ModalDialog.qml",
    "qml/FoldersDialog.qml",
    "qml/AboutDialog.qml",
    "qml/ExitDialog.qml",
    "qml/MessageDialog.qml",
    "qml/LogViewer.qml",
];

fn main() {
    let qml_files = std::iter::once(QmlFile::from("qml/Theme.qml").singleton(true))
        .chain(QML_FILES.iter().map(|f| QmlFile::from(*f)));

    let builder = CxxQtBuilder::new_qml_module(QmlModule::new("Pocketbox").qml_files(qml_files));
    // Safety: only adds a warning flag. GCC 16 warns about Qt's own headers
    // (-Wsfinae-incomplete), which would otherwise bury real build output.
    let builder = unsafe {
        builder.cc_builder(|cc| {
            cc.flag_if_supported("-Wno-sfinae-incomplete");
        })
    };

    builder
        .qt_module("Quick")
        .files([
            "src/bridge/emulator.rs",
            "src/bridge/lcd.rs",
            "src/bridge/library.rs",
            "src/bridge/log.rs",
        ])
        .qrc_resources([
            "fonts/IBMPlexSans-Variable.ttf",
            "fonts/IBMPlexMono-Regular.ttf",
            "fonts/IBMPlexMono-Medium.ttf",
            "fonts/Silkscreen-Regular.ttf",
        ])
        .build();
}
