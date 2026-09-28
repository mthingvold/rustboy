# Planned work: emulator functions the UI needs

The UI does not modify `src/`. Every emulator capability it needs is listed here
and declared as a planned function on the `EmulatorCore` trait in
`ui/src/core/mod.rs`. Until the core provides them, the UI runs against
`TwinCore` (`ui/src/core/twin.rs`), a digital twin that draws a test scene.

To hook up the real core: implement `EmulatorCore` over
`rustboy::lr35902::LR35902` and return it from `core::load`.

| Planned function | Used for | Notes on current core |
|---|---|---|
| `LR35902::new(Cartridge)` without SDL | Load ROM, power on, reset | `MMU::new` initialises SDL and opens a window; `Joypad` owns the SDL event pump and exits the process on window close. |
| `Cartridge::new` returning `Result` | Error dialog on bad ROMs | Panics on non-UTF-8 titles; `mbc::from` panics on MBC2/MBC3. |
| `LR35902::run_frame()` | Frame pacing, speed, Advance Frame, fps / frame counter | `run()` loops forever and calls `step()` twice per iteration. |
| `PPU` framebuffer of shade indices (160×144, 0–3) | LCD view, palettes | `Screen::draw` is commented out; `background_pixels` panics on non-zero colours. |
| `Joypad::set_button(Button, bool)` | Keyboard input | Input only comes from SDL events; Start/Select/A/B never update their bits. |
| `log` crate macros in place of `println!` | Log Viewer | UI installs a `log` backend already. |
| Later (phase 5): `snapshot()` / `restore()` incl. MBC state | Save states, rewind | Nothing is serialisable yet. |
| Later (phase 6): serial output capture | Test runner (Blargg) | Serial never completes transfers. |
