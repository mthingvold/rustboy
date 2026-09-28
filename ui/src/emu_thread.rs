//! The emulator thread: owns the [`EmulatorCore`], runs it at the selected speed and reports
//! state changes through a callback. Nothing here depends on Qt.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::core::{self, Button, EmulatorCore, FRAME_RATE, LCD_HEIGHT, LCD_WIDTH};

/// The most recently completed frame (shade indices), shared with the LCD item.
pub static FRAME: Mutex<[u8; LCD_WIDTH * LCD_HEIGHT]> = Mutex::new([0; LCD_WIDTH * LCD_HEIGHT]);

/// Incremented every time [`FRAME`] is updated.
pub static FRAME_SEQ: AtomicU64 = AtomicU64::new(0);

/// Requests from the UI.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// Load and start the ROM at this path.
    Load(PathBuf),
    /// Pause if running, resume if paused.
    TogglePause,
    /// Pause (`true`) or resume (`false`).
    SetPaused(bool),
    /// Soft reset to frame 0 and resume.
    Reset,
    /// Power off if on, power on (from frame 0) if off.
    TogglePower,
    /// While paused, run exactly one frame.
    AdvanceFrame,
    /// Set the speed multiplier (0.25, 0.5, 1, 2 or 4).
    SetSpeed(f64),
    /// Press or release a joypad button.
    Button(Button, bool),
    /// Stop the thread.
    Shutdown,
}

/// Everything the UI shows about the running game.
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    /// A ROM has been loaded (it stays loaded while powered off).
    pub loaded: bool,
    /// The console is on.
    pub powered: bool,
    /// Emulation is paused.
    pub paused: bool,
    /// Speed multiplier.
    pub speed: f64,
    /// Frames run since power-on or reset.
    pub frame: u64,
    /// Measured frames per second (0 when not running).
    pub fps: f64,
    /// Path of the loaded ROM.
    pub rom_path: String,
    /// File name of the loaded ROM.
    pub rom_file: String,
    /// Display title of the loaded ROM.
    pub rom_title: String,
}

impl Default for Status {
    fn default() -> Self {
        Status {
            loaded: false,
            powered: false,
            paused: false,
            speed: 1.0,
            frame: 0,
            fps: 0.0,
            rom_path: String::new(),
            rom_file: String::new(),
            rom_title: String::new(),
        }
    }
}

/// Notifications to the UI.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// State changed or a frame completed.
    Status(Status),
    /// Show a short confirmation toast.
    Toast(String),
    /// Show an error dialog.
    Error { title: String, message: String },
}

/// Format a speed for labels and toasts, e.g. `0.25×`, `2×`.
pub fn speed_label(speed: f64) -> String {
    format!("{}×", speed)
}

/// Handle to the running emulator thread. Dropping it stops the thread.
pub struct EmuThread {
    tx: Sender<Command>,
    handle: Option<JoinHandle<()>>,
}

impl EmuThread {
    /// Start the thread. `notify` is called on the emulator thread for every [`Event`].
    pub fn spawn(notify: impl Fn(Event) + Send + 'static) -> EmuThread {
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::Builder::new()
            .name("emulator".into())
            .spawn(move || Worker::new(Box::new(notify)).run(rx))
            .expect("failed to start the emulator thread");
        EmuThread {
            tx,
            handle: Some(handle),
        }
    }

    /// Queue a command.
    pub fn send(&self, command: Command) {
        let _ = self.tx.send(command);
    }
}

impl Drop for EmuThread {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

struct Worker {
    notify: Box<dyn Fn(Event) + Send>,
    status: Status,
    core: Option<Box<dyn EmulatorCore>>,
    rom: Vec<u8>,
    next_frame: Instant,
    fps_since: Instant,
    fps_frames: u32,
}

impl Worker {
    fn new(notify: Box<dyn Fn(Event) + Send>) -> Worker {
        Worker {
            notify,
            status: Status::default(),
            core: None,
            rom: Vec::new(),
            next_frame: Instant::now(),
            fps_since: Instant::now(),
            fps_frames: 0,
        }
    }

    fn running(&self) -> bool {
        self.core.is_some() && self.status.powered && !self.status.paused
    }

    fn run(mut self, rx: Receiver<Command>) {
        loop {
            let command = if self.running() {
                let now = Instant::now();
                if self.next_frame > now {
                    match rx.recv_timeout(self.next_frame - now) {
                        Ok(c) => Some(c),
                        Err(RecvTimeoutError::Timeout) => None,
                        Err(RecvTimeoutError::Disconnected) => return,
                    }
                } else {
                    match rx.try_recv() {
                        Ok(c) => Some(c),
                        Err(TryRecvError::Empty) => None,
                        Err(TryRecvError::Disconnected) => return,
                    }
                }
            } else {
                match rx.recv() {
                    Ok(c) => Some(c),
                    Err(_) => return,
                }
            };

            match command {
                Some(Command::Shutdown) => return,
                Some(command) => self.handle(command),
                None => self.tick(),
            }
        }
    }

    fn emit_status(&self) {
        (self.notify)(Event::Status(self.status.clone()));
    }

    fn toast(&self, message: impl Into<String>) {
        (self.notify)(Event::Toast(message.into()));
    }

    fn error(&self, title: &str, message: String) {
        log::error!(target: "Core", "{}: {}", title, message);
        (self.notify)(Event::Error {
            title: title.to_string(),
            message,
        });
    }

    fn publish_frame(&self) {
        if let Some(core) = &self.core {
            let mut frame = FRAME.lock().unwrap_or_else(|e| e.into_inner());
            frame.copy_from_slice(core.framebuffer());
            FRAME_SEQ.fetch_add(1, Ordering::Release);
        }
    }

    /// Restart pacing and the fps measurement (after resuming, loading or changing speed).
    fn restart_clock(&mut self) {
        self.next_frame = Instant::now();
        self.fps_since = Instant::now();
        self.fps_frames = 0;
        self.status.fps = 0.0;
    }

    fn power_on(&mut self) -> bool {
        self.core = None;
        match core::load(self.rom.clone()) {
            Ok(core) => {
                self.core = Some(core);
                self.status.powered = true;
                self.status.paused = false;
                self.status.frame = 0;
                self.restart_clock();
                self.publish_frame();
                true
            }
            Err(message) => {
                self.status.powered = false;
                self.error("Couldn't start the game", message);
                false
            }
        }
    }

    fn handle(&mut self, command: Command) {
        match command {
            Command::Load(path) => self.load(&path),
            Command::TogglePause => {
                let paused = !self.status.paused;
                self.handle(Command::SetPaused(paused));
                return;
            }
            Command::SetPaused(paused) => {
                if !self.status.powered || self.status.paused == paused {
                    return;
                }
                self.status.paused = paused;
                self.restart_clock();
            }
            Command::Reset => {
                if !self.status.loaded {
                    return;
                }
                match self.core.as_mut() {
                    Some(core) => {
                        core.reset();
                        self.status.powered = true;
                        self.status.paused = false;
                        self.status.frame = 0;
                        self.restart_clock();
                        self.publish_frame();
                    }
                    None => {
                        if !self.power_on() {
                            self.emit_status();
                            return;
                        }
                    }
                }
                log::info!(target: "Core", "Reset");
                self.toast("Reset");
            }
            Command::TogglePower => {
                if !self.status.loaded {
                    return;
                }
                if self.status.powered {
                    self.core = None;
                    self.status.powered = false;
                    self.status.paused = false;
                    self.status.fps = 0.0;
                    log::info!(target: "Core", "Powered off");
                    self.toast("Powered off");
                } else if self.power_on() {
                    log::info!(target: "Core", "Powered on");
                    self.toast("Powered on");
                }
            }
            Command::AdvanceFrame => {
                if !(self.status.powered && self.status.paused) {
                    return;
                }
                self.run_one_frame();
            }
            Command::SetSpeed(speed) => {
                if self.status.speed == speed {
                    return;
                }
                self.status.speed = speed;
                self.restart_clock();
                log::info!(target: "Speed", "Emulation speed set to {}", speed_label(speed));
                self.toast(format!("Speed {}", speed_label(speed)));
            }
            Command::Button(button, pressed) => {
                if let Some(core) = self.core.as_mut() {
                    core.set_button(button, pressed);
                }
                return;
            }
            Command::Shutdown => return,
        }
        self.emit_status();
    }

    fn load(&mut self, path: &Path) {
        let rom = match crate::rom::read_rom(path) {
            Ok(rom) => rom,
            Err(message) => {
                self.error("Couldn't load ROM", message);
                return;
            }
        };

        let file = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let title = path
            .file_stem()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();

        // Keep whatever was running before if the new ROM is unusable
        let core = match core::load(rom.clone()) {
            Ok(core) => core,
            Err(message) => {
                self.error("Couldn't start the game", message);
                return;
            }
        };

        self.rom = rom;
        self.core = Some(core);
        self.status.powered = true;
        self.status.paused = false;
        self.status.frame = 0;
        self.restart_clock();
        self.publish_frame();
        self.status.loaded = true;
        self.status.rom_path = path.to_string_lossy().to_string();
        self.status.rom_file = file.clone();
        self.status.rom_title = title;
        log::info!(
            target: "Cart",
            "Loaded {} — {}",
            file,
            crate::library::format_size(self.rom.len() as u64)
        );
        self.toast(format!("Loaded {}", file));
    }

    fn run_one_frame(&mut self) -> bool {
        let result = match self.core.as_mut() {
            Some(core) => core.run_frame(),
            None => return false,
        };
        match result {
            Ok(()) => {
                self.status.frame += 1;
                self.publish_frame();
                true
            }
            Err(message) => {
                self.core = None;
                self.status.powered = false;
                self.status.paused = false;
                self.status.fps = 0.0;
                self.error("Emulation stopped", message);
                false
            }
        }
    }

    fn tick(&mut self) {
        if !self.run_one_frame() {
            self.emit_status();
            return;
        }

        let now = Instant::now();
        self.next_frame += Duration::from_secs_f64(1.0 / (FRAME_RATE * self.status.speed));
        // If emulation can't keep up, don't try to catch up with a burst of frames
        if self.next_frame + Duration::from_millis(250) < now {
            self.next_frame = now;
        }

        self.fps_frames += 1;
        let elapsed = now - self.fps_since;
        if elapsed >= Duration::from_millis(500) {
            self.status.fps = self.fps_frames as f64 / elapsed.as_secs_f64();
            self.fps_since = now;
            self.fps_frames = 0;
        }

        self.emit_status();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex as StdMutex};

    fn wait_for(events: &Arc<StdMutex<Vec<Event>>>, pred: impl Fn(&Status) -> bool) -> Status {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(Event::Status(s)) = events
                .lock()
                .unwrap()
                .iter()
                .rev()
                .find(|e| matches!(e, Event::Status(_)))
            {
                if pred(s) {
                    return s.clone();
                }
            }
            assert!(Instant::now() < deadline, "timed out waiting for status");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn load_pause_step_power_and_errors() {
        let dir = std::env::temp_dir().join(format!("pocketbox-emu-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let rom_path = dir.join("Demo.gb");
        std::fs::write(&rom_path, vec![0u8; 0x8000]).unwrap();

        let events = Arc::new(StdMutex::new(Vec::new()));
        let sink = events.clone();
        let emu = EmuThread::spawn(move |e| sink.lock().unwrap().push(e));

        emu.send(Command::Load(rom_path.clone()));
        let s = wait_for(&events, |s| s.loaded && s.frame > 3);
        assert_eq!(s.rom_title, "Demo");
        assert!(events
            .lock()
            .unwrap()
            .contains(&Event::Toast("Loaded Demo.gb".into())));

        emu.send(Command::SetPaused(true));
        let paused = wait_for(&events, |s| s.paused);
        emu.send(Command::AdvanceFrame);
        wait_for(&events, |s| s.paused && s.frame == paused.frame + 1);

        emu.send(Command::TogglePower);
        let off = wait_for(&events, |s| !s.powered);
        assert!(off.loaded && !off.paused);
        emu.send(Command::TogglePower);
        wait_for(&events, |s| s.powered && s.frame > 0);

        emu.send(Command::Load(dir.join("missing.gb")));
        let deadline = Instant::now() + Duration::from_secs(5);
        while !events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, Event::Error { .. }))
        {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }

        drop(emu);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn speed_labels() {
        assert_eq!(speed_label(0.25), "0.25×");
        assert_eq!(speed_label(2.0), "2×");
    }
}
