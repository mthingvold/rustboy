//! Digital twin of the emulator core.
//!
//! [`TwinCore`] implements [`EmulatorCore`] without emulating a CPU. It checks that the ROM has
//! a cartridge header, then draws a deterministic test scene one frame at a time: parallax
//! hills and ground that scroll with the frame counter, a block moved by the D-pad (A jumps),
//! and the frame number as binary cells in the top-left corner so single-frame stepping is
//! visible.

use super::{Button, EmulatorCore, LCD_HEIGHT, LCD_WIDTH};

const PLAYER_SIZE: i32 = 8;
const GROUND_TOP: i32 = 120;
const START_X: i32 = 40;

/// Simulated core used until the real core exposes the planned API.
pub struct TwinCore {
    frame: u64,
    pixels: Vec<u8>,
    held: [bool; 8],
    x: i32,
    /// Height above the ground, in pixels.
    y: i32,
    vy: i32,
}

fn index(button: Button) -> usize {
    match button {
        Button::Up => 0,
        Button::Down => 1,
        Button::Left => 2,
        Button::Right => 3,
        Button::A => 4,
        Button::B => 5,
        Button::Start => 6,
        Button::Select => 7,
    }
}

impl TwinCore {
    /// Create a twin for `rom`. Fails like a real core would if `rom` has no cartridge header.
    pub fn new(rom: Vec<u8>) -> Result<TwinCore, String> {
        if crate::rom::parse_header(&rom).is_none() {
            return Err("This file is too small to be a Game Boy ROM.".to_string());
        }
        let mut twin = TwinCore {
            frame: 0,
            pixels: vec![0; LCD_WIDTH * LCD_HEIGHT],
            held: [false; 8],
            x: START_X,
            y: 0,
            vy: 0,
        };
        twin.draw();
        Ok(twin)
    }

    /// Frames run since power-on or the last reset.
    #[cfg(test)]
    pub fn frame(&self) -> u64 {
        self.frame
    }

    fn held(&self, button: Button) -> bool {
        self.held[index(button)]
    }

    fn update(&mut self) {
        if self.held(Button::Left) {
            self.x -= 1;
        }
        if self.held(Button::Right) {
            self.x += 1;
        }
        self.x = self.x.clamp(0, LCD_WIDTH as i32 - PLAYER_SIZE);

        if self.held(Button::A) && self.y == 0 {
            self.vy = 5;
        }
        if self.y > 0 || self.vy > 0 {
            self.y += self.vy;
            // Gravity: one pixel per frame², applied every other frame for a floatier arc
            if self.frame % 2 == 0 {
                self.vy -= 1;
            }
            if self.y <= 0 {
                self.y = 0;
                self.vy = 0;
            }
        }
    }

    fn set(&mut self, x: i32, y: i32, shade: u8) {
        if x >= 0 && y >= 0 && (x as usize) < LCD_WIDTH && (y as usize) < LCD_HEIGHT {
            self.pixels[y as usize * LCD_WIDTH + x as usize] = shade;
        }
    }

    fn draw(&mut self) {
        let f = self.frame as i32;

        for y in 0..LCD_HEIGHT as i32 {
            for x in 0..LCD_WIDTH as i32 {
                // Hills: a triangle wave scrolling at a quarter of the ground speed
                let hx = (x + f / 4).rem_euclid(64);
                let hill_top = 84 + (hx - 32).abs() / 2;
                let shade = if y >= GROUND_TOP {
                    // Ground with dashes scrolling one pixel per frame
                    if y == GROUND_TOP || ((x + f).rem_euclid(16) < 6 && y == GROUND_TOP + 6) {
                        3
                    } else {
                        2
                    }
                } else if y >= hill_top {
                    1
                } else {
                    0
                };
                self.set(x, y, shade);
            }
        }

        // Player block with an "eye" facing right
        let top = GROUND_TOP - PLAYER_SIZE - self.y;
        for dy in 0..PLAYER_SIZE {
            for dx in 0..PLAYER_SIZE {
                self.set(self.x + dx, top + dy, 3);
            }
        }
        self.set(self.x + 5, top + 2, 0);
        self.set(self.x + 6, top + 2, 0);

        // Frame counter: 16 binary cells, least significant bit on the right
        for bit in 0..16 {
            let on = (self.frame >> (15 - bit)) & 1 == 1;
            for dy in 0..3 {
                for dx in 0..3 {
                    self.set(4 + bit * 4 + dx, 4 + dy, if on { 3 } else { 1 });
                }
            }
        }
    }
}

impl EmulatorCore for TwinCore {
    fn run_frame(&mut self) -> Result<(), String> {
        self.frame += 1;
        self.update();
        self.draw();
        Ok(())
    }

    fn framebuffer(&self) -> &[u8] {
        &self.pixels
    }

    fn set_button(&mut self, button: Button, pressed: bool) {
        self.held[index(button)] = pressed;
    }

    fn reset(&mut self) {
        self.frame = 0;
        self.held = [false; 8];
        self.x = START_X;
        self.y = 0;
        self.vy = 0;
        self.draw();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rom() -> Vec<u8> {
        vec![0u8; 0x8000]
    }

    #[test]
    fn rejects_files_without_a_header() {
        assert!(TwinCore::new(vec![0u8; 16]).is_err());
    }

    #[test]
    fn frames_are_deterministic_and_change() {
        let mut a = TwinCore::new(rom()).unwrap();
        let mut b = TwinCore::new(rom()).unwrap();
        let first = a.framebuffer().to_vec();
        a.run_frame().unwrap();
        b.run_frame().unwrap();
        assert_eq!(a.framebuffer(), b.framebuffer());
        assert_ne!(a.framebuffer(), &first[..]);
        assert!(a.framebuffer().iter().all(|s| *s <= 3));
    }

    #[test]
    fn dpad_moves_player_and_reset_restores() {
        let mut twin = TwinCore::new(rom()).unwrap();
        twin.set_button(Button::Right, true);
        for _ in 0..10 {
            twin.run_frame().unwrap();
        }
        assert_eq!(twin.x, START_X + 10);

        twin.reset();
        assert_eq!(twin.frame(), 0);
        assert_eq!(twin.x, START_X);
    }

    #[test]
    fn a_jumps_and_lands() {
        let mut twin = TwinCore::new(rom()).unwrap();
        twin.set_button(Button::A, true);
        twin.run_frame().unwrap();
        twin.set_button(Button::A, false);
        assert!(twin.y > 0);
        for _ in 0..60 {
            twin.run_frame().unwrap();
        }
        assert_eq!(twin.y, 0);
    }
}
