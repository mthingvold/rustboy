//! Library target exposing the emulator core to frontends (see `ui/`).
//!
//! This file only re-exports the existing modules; the `rustboy` binary in
//! `main.rs` keeps declaring them itself.

#![allow(dead_code)]

pub mod cartridge;
pub mod enums;
pub mod joypad;
pub mod lr35902;
pub mod mmu;
pub mod ppu;
pub mod serial;
pub mod sound;
pub mod timer;
pub mod traits;

mod testing;
