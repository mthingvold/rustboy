//! `Emulator`: QML-facing controller for the emulator thread.

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
        #[qproperty(bool, loaded)]
        #[qproperty(bool, powered)]
        #[qproperty(bool, paused)]
        #[qproperty(f64, speed)]
        #[qproperty(f64, fps)]
        #[qproperty(f64, frame)]
        #[qproperty(QString, rom_title)]
        #[qproperty(QString, rom_file)]
        #[qproperty(QString, rom_path)]
        type Emulator = super::EmulatorRust;

        /// Load and start a ROM from a local path or `file://` URL.
        #[qinvokable]
        fn load_rom(self: Pin<&mut Emulator>, path: &QString);

        /// Pause if running, resume if paused.
        #[qinvokable]
        fn toggle_pause(self: Pin<&mut Emulator>);

        /// Pause (`true`) or resume (`false`).
        #[qinvokable]
        fn request_pause(self: Pin<&mut Emulator>, paused: bool);

        /// Soft reset to frame 0 and resume.
        #[qinvokable]
        fn reset(self: Pin<&mut Emulator>);

        /// Power off, or power on from frame 0.
        #[qinvokable]
        fn toggle_power(self: Pin<&mut Emulator>);

        /// Run exactly one frame while paused.
        #[qinvokable]
        fn advance_frame(self: Pin<&mut Emulator>);

        /// Set the speed multiplier (0.25, 0.5, 1, 2 or 4).
        #[qinvokable]
        fn choose_speed(self: Pin<&mut Emulator>, speed: f64);

        /// Forward a Qt key press/release to the joypad. Returns whether the key is mapped.
        #[qinvokable]
        fn key_event(self: Pin<&mut Emulator>, key: i32, pressed: bool) -> bool;

        /// Show a short confirmation message.
        #[qsignal]
        fn toast(self: Pin<&mut Emulator>, message: QString);

        /// Show an error dialog.
        #[qsignal]
        fn error_occurred(self: Pin<&mut Emulator>, title: QString, message: QString);

        /// A new frame is available for the LCD.
        #[qsignal]
        fn frame_ready(self: Pin<&mut Emulator>);
    }

    impl cxx_qt::Threading for Emulator {}
    impl cxx_qt::Initialize for Emulator {}
}

use core::pin::Pin;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

use crate::core::Button;
use crate::emu_thread::{Command, EmuThread, Event, Status, FRAME_SEQ};

/// Rust state behind `Emulator`.
pub struct EmulatorRust {
    loaded: bool,
    powered: bool,
    paused: bool,
    speed: f64,
    fps: f64,
    frame: f64,
    rom_title: QString,
    rom_file: QString,
    rom_path: QString,
    thread: Option<EmuThread>,
    last_frame_seq: u64,
}

impl Default for EmulatorRust {
    fn default() -> Self {
        EmulatorRust {
            loaded: false,
            powered: false,
            paused: false,
            speed: 1.0,
            fps: 0.0,
            frame: 0.0,
            rom_title: QString::default(),
            rom_file: QString::default(),
            rom_path: QString::default(),
            thread: None,
            last_frame_seq: 0,
        }
    }
}

impl cxx_qt::Initialize for qobject::Emulator {
    fn initialize(mut self: Pin<&mut Self>) {
        let qt_thread = self.qt_thread();
        // Only the newest status matters; queue at most one status update at a time
        let latest: Arc<Mutex<Option<Status>>> = Arc::new(Mutex::new(None));

        let thread = EmuThread::spawn(move |event| match event {
            Event::Status(status) => {
                let first = latest.lock().unwrap().replace(status).is_none();
                if first {
                    let latest = latest.clone();
                    let _ = qt_thread.queue(move |obj| {
                        let status = latest.lock().unwrap().take();
                        if let Some(status) = status {
                            obj.apply_status(status);
                        }
                    });
                }
            }
            Event::Toast(message) => {
                let _ = qt_thread.queue(move |obj| obj.toast(QString::from(&message)));
            }
            Event::Error { title, message } => {
                let _ = qt_thread.queue(move |obj| {
                    obj.error_occurred(QString::from(&title), QString::from(&message))
                });
            }
        });
        self.as_mut().rust_mut().thread = Some(thread);
    }
}

/// Map a Qt key code to a joypad button.
fn button_for_key(key: i32) -> Option<Button> {
    // Values from Qt::Key
    match key {
        0x0100_0013 => Some(Button::Up),
        0x0100_0015 => Some(Button::Down),
        0x0100_0012 => Some(Button::Left),
        0x0100_0014 => Some(Button::Right),
        0x5A => Some(Button::A),                          // Z
        0x58 => Some(Button::B),                          // X
        0x0100_0004 | 0x0100_0005 => Some(Button::Start), // Return, Enter
        0x0100_0020 => Some(Button::Select),              // Shift
        _ => None,
    }
}

/// Turn a QML file URL (`file:///a%20b`) or plain path into a local path.
pub fn local_path(input: &str) -> String {
    let rest = match input.strip_prefix("file://") {
        Some(rest) => rest,
        None => return input.to_string(),
    };
    let bytes = rest.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Some(v) = std::str::from_utf8(&bytes[i + 1..i + 3])
                .ok()
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
            {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

impl qobject::Emulator {
    fn send(&self, command: Command) {
        if let Some(thread) = &self.thread {
            thread.send(command);
        }
    }

    fn apply_status(mut self: Pin<&mut Self>, s: Status) {
        self.as_mut().set_loaded(s.loaded);
        self.as_mut().set_powered(s.powered);
        self.as_mut().set_paused(s.paused);
        self.as_mut().set_speed(s.speed);
        self.as_mut().set_fps(s.fps);
        self.as_mut().set_frame(s.frame as f64);
        self.as_mut().set_rom_title(QString::from(&s.rom_title));
        self.as_mut().set_rom_file(QString::from(&s.rom_file));
        self.as_mut().set_rom_path(QString::from(&s.rom_path));

        let seq = FRAME_SEQ.load(Ordering::Acquire);
        if seq != self.last_frame_seq {
            self.as_mut().rust_mut().last_frame_seq = seq;
            self.frame_ready();
        }
    }

    fn load_rom(self: Pin<&mut Self>, path: &QString) {
        let path = local_path(&path.to_string());
        self.send(Command::Load(path.into()));
    }

    fn toggle_pause(self: Pin<&mut Self>) {
        self.send(Command::TogglePause);
    }

    fn request_pause(self: Pin<&mut Self>, paused: bool) {
        self.send(Command::SetPaused(paused));
    }

    fn reset(self: Pin<&mut Self>) {
        self.send(Command::Reset);
    }

    fn toggle_power(self: Pin<&mut Self>) {
        self.send(Command::TogglePower);
    }

    fn advance_frame(self: Pin<&mut Self>) {
        self.send(Command::AdvanceFrame);
    }

    fn choose_speed(self: Pin<&mut Self>, speed: f64) {
        self.send(Command::SetSpeed(speed));
    }

    fn key_event(self: Pin<&mut Self>, key: i32, pressed: bool) -> bool {
        match button_for_key(key) {
            Some(button) => {
                self.send(Command::Button(button, pressed));
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_file_urls() {
        assert_eq!(local_path("file:///home/a%20b/x.gb"), "/home/a b/x.gb");
        assert_eq!(local_path("/plain/path.gb"), "/plain/path.gb");
    }
}
