//! The emulator-core interface the UI is written against.
//!
//! [`EmulatorCore`] lists the **planned** core functions: none of them exist in
//! `src/` yet, and the UI must not add them there (see ui/PLANNED_WORK.md). Each method's
//! documentation names the core function it is meant to map to.
//!
//! Until those functions exist, [`load`] returns a [`twin::TwinCore`], a digital twin
//! that simulates the planned behaviour with a generated test picture so the windowing can be
//! built and exercised. Hooking up the real core means writing one more `EmulatorCore`
//! implementation over `rustboy::lr35902::LR35902` and returning it from [`load`].

pub mod twin;

/// LCD width in pixels.
pub const LCD_WIDTH: usize = 160;
/// LCD height in pixels.
pub const LCD_HEIGHT: usize = 144;
/// Frames per second of real hardware (4194304 Hz / 70224 cycles per frame).
pub const FRAME_RATE: f64 = 59.7275;

/// A Game Boy joypad button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}

/// Operations the UI needs from an emulator core. Implementations run on the emulator thread
/// and never touch the UI.
pub trait EmulatorCore: Send {
    /// Advance emulation by exactly one video frame (until the PPU enters VBlank).
    ///
    /// Planned core function: `LR35902::run_frame() -> FrameInfo`
    /// (ui/PLANNED_WORK.md). An `Err` means the core hit an unrecoverable fault; the UI
    /// stops emulation and reports the message.
    fn run_frame(&mut self) -> Result<(), String>;

    /// The last completed frame as shade indices (0 = lightest … 3 = darkest), row-major,
    /// [`LCD_WIDTH`] × [`LCD_HEIGHT`]. The UI applies the user's LCD palette.
    ///
    /// Planned core function: `PPU::framebuffer() -> &[u8; 160 * 144]`, exposed through
    /// `MMU`/`LR35902` (ui/PLANNED_WORK.md).
    fn framebuffer(&self) -> &[u8];

    /// Press or release a joypad button.
    ///
    /// Planned core function: `Joypad::set_button(Button, bool)`, replacing SDL event
    /// polling (ui/PLANNED_WORK.md).
    fn set_button(&mut self, button: Button, pressed: bool);

    /// Soft reset: return to the post-boot state with the same cartridge.
    ///
    /// Planned core function: none. Reset rebuilds the CPU from the ROM bytes with
    /// `LR35902::new(Cartridge::new(rom))`, which requires the SDL-free constructors of
    /// ui/PLANNED_WORK.md.
    fn reset(&mut self);
}

/// Create a core for `rom` (the raw ROM image).
///
/// Planned core function: `LR35902::new(Cartridge::new(rom))` without SDL initialisation
/// (ui/PLANNED_WORK.md), with `Cartridge::new` returning an error in place of panicking
/// on bad headers (ui/PLANNED_WORK.md). Returns the digital twin until then.
pub fn load(rom: Vec<u8>) -> Result<Box<dyn EmulatorCore>, String> {
    Ok(Box::new(twin::TwinCore::new(rom)?))
}
