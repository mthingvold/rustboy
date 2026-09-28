//! Pocketbox: the Qt/QML desktop frontend for rustboy.
//!
//! The UI runs against the planned emulator API in [`core`]; until the rustboy core provides
//! it, a digital twin stands in (see `PLANNED_WORK.md` in this directory).

mod bridge;
mod core;
mod emu_thread;
mod library;
mod logging;
mod rom;
mod settings;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

fn main() {
    logging::init();
    log::info!(target: "Core", "Pocketbox {} started", env!("CARGO_PKG_VERSION"));

    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    if let Some(mut app) = app.as_mut() {
        app.as_mut()
            .set_application_name(&QString::from("Pocketbox"));
        app.as_mut()
            .set_organization_name(&QString::from("Pocketbox"));
        app.as_mut()
            .set_application_version(&QString::from(env!("CARGO_PKG_VERSION")));
    }

    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/Pocketbox/qml/Main.qml"));
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
