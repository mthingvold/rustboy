//! In-memory `log` backend feeding the Log Viewer window.
//!
//! Messages are kept in a bounded ring buffer. Each one gets an increasing sequence number so
//! the viewer can poll for "everything after N" without missing or repeating entries.

use std::collections::VecDeque;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use log::{Level, LevelFilter, Log, Metadata, Record};

/// How many messages are retained before the oldest are dropped.
pub const CAPACITY: usize = 10_000;

/// One log message as shown in the viewer.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Entry {
    /// Sequence number, unique and increasing for the lifetime of the process.
    pub seq: u64,
    /// Local wall-clock time, `HH:MM:SS.mmm`.
    pub t: String,
    /// `INFO`, `WARN`, `ERROR` or `DEBUG` (`TRACE` is folded into `DEBUG`).
    pub lvl: &'static str,
    /// Subsystem, taken from the record's target (e.g. `Library`, `Core`).
    pub sys: String,
    /// Message text.
    pub msg: String,
}

struct Buffer {
    entries: VecDeque<Entry>,
    next_seq: u64,
}

static BUFFER: Mutex<Buffer> = Mutex::new(Buffer {
    entries: VecDeque::new(),
    next_seq: 1,
});

struct Logger;

static LOGGER: Logger = Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        // Dependencies log under their module path; keep only our own subsystems
        let target = record.target();
        if target.contains("::") {
            return;
        }
        push(record.level(), target, &record.args().to_string());
    }

    fn flush(&self) {}
}

/// Install the logger as the global `log` backend. Safe to call more than once.
pub fn init() {
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(LevelFilter::Debug);
    }
}

fn level_name(level: Level) -> &'static str {
    match level {
        Level::Error => "ERROR",
        Level::Warn => "WARN",
        Level::Info => "INFO",
        Level::Debug | Level::Trace => "DEBUG",
    }
}

/// Append a message directly (used for messages that originate in QML).
pub fn push(level: Level, subsystem: &str, message: &str) {
    let t = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
    let mut buf = BUFFER.lock().unwrap_or_else(|e| e.into_inner());
    let seq = buf.next_seq;
    buf.next_seq += 1;
    if buf.entries.len() == CAPACITY {
        buf.entries.pop_front();
    }
    buf.entries.push_back(Entry {
        seq,
        t,
        lvl: level_name(level),
        sys: subsystem.to_string(),
        msg: message.to_string(),
    });
}

/// All retained messages with a sequence number greater than `after`.
pub fn since(after: u64) -> Vec<Entry> {
    let buf = BUFFER.lock().unwrap_or_else(|e| e.into_inner());
    buf.entries
        .iter()
        .filter(|e| e.seq > after)
        .cloned()
        .collect()
}

/// Drop every retained message. Sequence numbers keep increasing.
pub fn clear() {
    BUFFER
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .entries
        .clear();
}

/// Write every retained message to `path`, one per line.
pub fn export(path: &Path) -> std::io::Result<()> {
    let entries = since(0);
    let mut file = std::io::BufWriter::new(std::fs::File::create(path)?);
    for e in entries {
        writeln!(file, "{} {:<5} {:<8} {}", e.t, e.lvl, e.sys, e.msg)?;
    }
    file.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn since_returns_only_newer_entries_and_export_writes_them() {
        push(Level::Info, "TestA", "first");
        let first = since(0).last().unwrap().seq;
        push(Level::Trace, "TestA", "second");

        let newer = since(first);
        assert_eq!(newer.len(), 1);
        assert_eq!(newer[0].msg, "second");
        assert_eq!(newer[0].lvl, "DEBUG");

        let path = std::env::temp_dir().join(format!("pocketbox-log-{}.log", std::process::id()));
        export(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("INFO  TestA    first"));
        std::fs::remove_file(path).unwrap();
    }
}
