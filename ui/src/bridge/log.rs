//! `LogBridge`: gives the Log Viewer access to the in-memory log.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        type LogBridge = super::LogBridgeRust;

        /// JSON array of messages with a sequence number above `after`.
        #[qinvokable]
        fn since(self: &LogBridge, after: f64) -> QString;

        /// Forget all messages.
        #[qinvokable]
        fn clear(self: &LogBridge);

        /// Save all messages to a `.log` file (path or `file://` URL). Returns the file name on
        /// success, or an empty string.
        #[qinvokable]
        fn export_to(self: &LogBridge, target: &QString) -> QString;

        /// Log a message from QML.
        #[qinvokable]
        fn write(self: &LogBridge, level: &QString, subsystem: &QString, message: &QString);
    }
}

use cxx_qt_lib::QString;

use super::emulator::local_path;
use crate::logging;

/// Rust state behind `LogBridge` (none: the log is global).
#[derive(Default)]
pub struct LogBridgeRust;

impl qobject::LogBridge {
    fn since(&self, after: f64) -> QString {
        let entries = logging::since(after.max(0.0) as u64);
        QString::from(&serde_json::to_string(&entries).unwrap_or_else(|_| "[]".into()))
    }

    fn clear(&self) {
        logging::clear();
    }

    fn export_to(&self, target: &QString) -> QString {
        let path = std::path::PathBuf::from(local_path(&target.to_string()));
        match logging::export(&path) {
            Ok(()) => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                log::info!(target: "Core", "Exported log to {}", path.display());
                QString::from(&name)
            }
            Err(e) => {
                log::error!(target: "Core", "Couldn't export log to {}: {}", path.display(), e);
                QString::default()
            }
        }
    }

    fn write(&self, level: &QString, subsystem: &QString, message: &QString) {
        let level = match level.to_string().as_str() {
            "ERROR" => log::Level::Error,
            "WARN" => log::Level::Warn,
            "DEBUG" => log::Level::Debug,
            _ => log::Level::Info,
        };
        logging::push(level, &subsystem.to_string(), &message.to_string());
    }
}
