mod display;
mod oam;
mod registers;

use sdl2::pixels::Color;
use sdl2::rect::Point;

use crate::enums::{Mode, Mode::*};
use crate::traits::{Byte, MemoryMap};

use oam::{OAMEntry, OAMFlags};
use registers::lcdc::LCDC;
use registers::lcds::LCDS;
use sdl2::VideoSubsystem;

const V_RAM_SIZE: usize = 0x2000;
const OAM_SIZE: usize = 0x0100;

#[allow(dead_code)]
const TILE_SIZE: usize = 128;

/// GameBoy Screen Height
pub const HEIGHT: u8 = 144;
pub const MAX_SCREEN_Y: u8 = HEIGHT - 1;

/// GameBoy Screen Width
pub const WIDTH: usize = 160;

const CYCLES_PER_LINE: u8 = 114;
const V_BLANK_LINES: u8 = 10;

const TOTAL_LINES: u8 = HEIGHT + V_BLANK_LINES;

const VRAM_OFFSET: u16 = 0x8000;

#[allow(dead_code)]
pub struct PPU {
    display: display::Screen,

    clock: u64, // Behaves as a counter of how many cycles / ticks have occurred, used to determine
    // an appropriate "mode" to switch to at a given point
    mode: Mode,             // PPU Mode
    vram: [u8; V_RAM_SIZE], // VRAM
    oam: [u8; OAM_SIZE],    // OAM / Sprite Attribute Table

    bg_ids: [u8; WIDTH], // Background colour ids (0..=3) of the current line, for OBJ-to-BG priority

    lcdc: LCDC, // 0xFF40 : LCDC Register : LCD C(ontrol) Register
    lcds: LCDS, // 0xFF41 : LCDS Register : LCD S(tatus) Register
    scy: u8,    // 0xFF42 : Scroll Y
    scx: u8,    // 0xFF43 : Scroll X
    ly: u8,     // 0xFF44 : LY  (LCD Y)
    lyc: u8,    // 0xFF45 : LYC (LY Compare)
    dma: u8,    // 0xFF46 : DMA (DMA Transfer and Start Address)
    bgp: u8,    // 0xFF47 : BGP Palette Data (Non-CGB)
    obp0: u8,   // 0xFF48 : Object Palette 0 (Non-CGB)
    obp1: u8,   // 0xFF49 : Object Palette 1 (Non-CGB)
    wy: u8,     // 0xFF4A : Window Y Position
    wx: u8,     // 0xFF4B : Window X Position

    pub stat_interrupt: bool, // A flag to represent a STAT interrupt request; this
    // corresponds to bit 1 in the 0xFF0F (Interrupt Flag) register
    pub vblank_interrupt: bool, // A flag to represent a VBlank interrupt request; this flag
                                // corresponds to bit 0 in the 0xFF0F (Interrupt Flag) register
}

#[allow(dead_code)]
#[allow(unused)]
impl PPU {
    pub fn new(video_context: VideoSubsystem) -> PPU {
        PPU {
            clock: 0,
            display: display::Screen::new(video_context),

            mode: Mode0,
            vram: [0; V_RAM_SIZE],
            oam: [0; OAM_SIZE],

            bg_ids: [0; WIDTH],

            lcdc: LCDC::new(),
            lcds: LCDS::new(),
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            dma: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,

            stat_interrupt: false,
            vblank_interrupt: false,
        }
    }

    pub fn oam_write(&mut self, index: usize, value: u8) {
        self.oam[index] = value;
    }

    pub fn dma_transfer(&mut self, value: u8) {
        self.dma = value;

        return;

        let source = ((value as u16) << 8) * 0x0100;

        for i in 0x00..=0x9F {
            let read = self.read(source + i as u16);
            self.write(0xFE00 + i as u16, read);
        }

        self.clock += 80;
    }

    /// 'Main' Execution - run the PPU for a given number of cycles
    pub fn run_for(&mut self, cycles: u64) {
        // Display is turned off
        if !self.lcdc.lcd_display_enable() {
            return;
        }

        let mut cycles_left = cycles;

        while cycles_left > 0 {
            let current = std::cmp::min(cycles_left, 80);

            // Any amount of "cycles given" should be cleanly divisible by 4, as the CPU will call
            //  this function, and should 'scale' the cycles by 4, to correspond with the difference
            //  between the CPU and PPU in clock speed
            assert_eq!(current % 4, 0);
            self.clock += current / 4;

            // One horizontal row is completed, as one horizontal row takes 114 cycles
            //  1 row = 20 cycles (OAM Search) + ~43 cycles (Pixel Transfer) + ~51 cycles (H-Blank)
            if self.clock >= CYCLES_PER_LINE as u64 {
                self.clock %= CYCLES_PER_LINE as u64;

                // Advance to next line (wrapping "around" back to top if necessary)
                self.ly += 1;
                self.ly %= HEIGHT + V_BLANK_LINES;

                // If toggled, check for lyc interrupt
                if self.lcds.lyc_interrupt() && self.ly == self.lyc {
                    // Enable the LCDS Coincidence bit
                    let value = self.lcds.read() | 0x04;
                    self.lcds.write(value);

                    self.stat_interrupt = true;
                }
            }

            // Compute the mode the PPU 'should' be in, based on whether ly is on the visible area,
            //  (and if so, whether in OAM / Transfer / HBlank), or in a VBlank area
            let target_mode = match self.ly {
                0..=MAX_SCREEN_Y => match self.clock {
                    0..=20 => Mode2,
                    21..=43 => Mode3,
                    44..=114 => Mode0,

                    _ => panic!("impossible clock value: {}", self.clock),
                },

                HEIGHT..=TOTAL_LINES => Mode1,

                _ => panic!("impossible ly value: {}", self.ly),
            };

            // Check before setting the mode, as interrupts can be generated by switching modes
            if self.mode != target_mode {
                self.enter_mode(target_mode);
            }

            cycles_left = match cycles_left.checked_sub(current) {
                Some(i) => i,
                None => 0,
            };
        }
    }

    fn enter_mode(&mut self, mode: Mode) {
        // Compute any byproducts (such as interrupts or line draws) from this mode switch
        match mode {
            // Entering HBlank Mode
            Mode0 => {
                // Entering HBlank mode indicates pixel transfer is complete, and so the row
                //  can be drawn
                self.draw_row();

                if self.lcds.mode_0_h_blank_interrupt() {
                    self.stat_interrupt = true;
                }
            }

            // Entering VBlank Mode
            Mode1 => {
                self.display.present();
                self.vblank_interrupt = true;
                if self.lcds.mode_1_v_blank_interrupt() {
                    self.stat_interrupt = true;
                }
            }

            // Entering OAM Search mode
            Mode2 => {
                if self.lcds.mode_2_oam_interrupt() {
                    self.stat_interrupt = true;
                }
            }

            // No interrupts / changes from Pixel Transfer mode
            Mode3 => (),
        };

        self.mode = mode;
    }

    fn draw_row(&mut self) {
        let h = self.lcdc.obj_size().1;

        // 1 - OAM Search
        //  The hardware takes the first 10 objects, in OAM order, that overlap the current line
        let mut visible: Vec<OAMEntry> = Vec::new();

        let line = self.ly as i16 + 16;

        for entry_number in 0..40 {
            let entry = self.oam_entry(entry_number);

            // Comparison method by which the GameBoy PPU uses to determine whether a given object
            //  should be considered 'visible' on the current line
            if line >= entry.y as i16 && line < entry.y as i16 + h as i16 {
                visible.push(entry);
                if visible.len() == 10 {
                    break;
                }
            };
        }

        // 2 - Pixel Transfer

        // 2.a - Background (a disabled background is drawn as a blank row, so that nothing from
        //  the previous frame is left behind)
        let background_pixels = self.background_pixels();
        self.display.draw(background_pixels);

        // 2.b - Objects
        let obj_pixels = self.object_pixels(visible);
        self.display.draw(obj_pixels);
    }

    /// Colours of the visible objects' pixels on the current line. `visible` must be in OAM order.
    /// At most one pixel is produced per screen column: that of the highest priority object that
    /// is opaque there, unless that object is behind the background.
    fn object_pixels(&mut self, visible: Vec<OAMEntry>) -> Vec<(Point, Color)> {
        let mut pixels = Vec::new();

        if !self.lcdc.obj_display_enable() {
            return pixels;
        }

        let height = self.lcdc.obj_size().1 as i32;
        let line = self.ly as i32 + 16;

        // DMG priority: lowest x wins, ties go to the lower OAM index. The sort is stable, so
        //  ties keep OAM order.
        // https://gbdev.io/pandocs/OAM.html#object-priority-and-conflicts
        let mut ordered: Vec<&OAMEntry> = visible.iter().collect();
        ordered.sort_by_key(|object| object.x);

        let mut claimed = [false; WIDTH];

        for object in ordered {
            let mut row = line - object.y as i32;
            if object.flags.flip_y {
                row = height - 1 - row;
            }

            // 8x16 objects ignore bit 0 of the tile number; rows 8-15 run into the next tile
            let tile = if height == 16 {
                object.tile_number & 0xFE
            } else {
                object.tile_number
            };
            let address = tile as usize * 16 + row as usize * 2;
            let lo = self.vram[address];
            let hi = self.vram[address + 1];

            let palette = if object.flags.palette {
                self.obp1
            } else {
                self.obp0
            };

            for x in 0..8i32 {
                let screen_x = object.x as i32 - 8 + x;
                if screen_x < 0 || screen_x >= WIDTH as i32 {
                    continue;
                }

                let bit = if object.flags.flip_x { x } else { 7 - x };
                let id = (((hi >> bit) & 1) << 1) | ((lo >> bit) & 1);

                // Colour id 0 is transparent for objects
                if id == 0 || claimed[screen_x as usize] {
                    continue;
                }
                claimed[screen_x as usize] = true;

                // Objects flagged as 'behind BG' only show through BG colour id 0
                if object.flags.priority && self.bg_ids[screen_x as usize] != 0 {
                    continue;
                }

                let shade = (palette >> (id * 2)) & 0b11;
                pixels.push((
                    Point::new(screen_x, self.ly as i32),
                    PPU::shade_color(shade),
                ));
            }
        }

        pixels
    }

    /// Maps a DMG shade (0 = lightest .. 3 = darkest) to a screen colour.
    fn shade_color(shade: u8) -> Color {
        match shade & 0b11 {
            0 => Color::RGB(255, 255, 255),
            1 => Color::RGB(170, 170, 170),
            2 => Color::RGB(85, 85, 85),
            _ => Color::RGB(0, 0, 0),
        }
    }

    fn background_pixels(&mut self) -> Vec<(Point, Color)> {
        let mut pixels: Vec<(Point, Color)> = Vec::new();

        if !self.lcdc.bg_display() {
            self.bg_ids = [0; WIDTH];
            return (0..WIDTH)
                .map(|i| (Point::new(i as i32, self.ly as i32), PPU::shade_color(0)))
                .collect();
        }

        let base_address = self.lcdc.bg_tile_map_display_select().0;
        let background_y = self.scy.wrapping_add(self.ly) as u16;
        let y_adjustment = ((background_y / 8) * 32);

        for i in 0..WIDTH {
            let x = (((self.scx as usize) + i) & 0xFF) as u16;
            let tile_base = base_address + y_adjustment + (x / 8);
            let tile_number = self.vram[(tile_base - VRAM_OFFSET) as usize];

            let row_offset = (background_y % 8) * 2;

            // LCDC Bit 4: 1 = idx from VRAM_OFFSET, 0= signed idx 0x9000
            // 0x9000 is the start of tile map 2
            let tile_addressed_data = if self.lcdc.bg_window_tile_data_select().0 == VRAM_OFFSET {
                VRAM_OFFSET + (tile_number as u16) * 16
            } else {
                (0x9000i32 + ((tile_number as i8) as i32) * 16) as u16
            };

            let address = tile_addressed_data + row_offset;

            let lo_byte = self.vram[(address - VRAM_OFFSET) as usize];
            let hi_byte = self.vram[(address + 1 - VRAM_OFFSET) as usize];

            let bit = 7 - (x % 8) as u8;
            let column = (((hi_byte >> bit) & 1) << 1) | ((lo_byte >> bit) & 1);

            self.bg_ids[i] = column;

            let shade = (self.bgp >> (column * 2)) & 0b11;
            pixels.push((
                Point::new(i as i32, self.ly as i32),
                PPU::shade_color(shade),
            ));
        }

        pixels
    }

    fn oam_entry(&mut self, entry_number: u8) -> OAMEntry {
        assert!(entry_number < 40, "asking for entry number beyond 40");

        // OAM entries are aligned on 4-byte boundaries beginning at 0xFE00
        // Read straight from OAM: the PPU's own access isn't subject to the CPU's mode restrictions
        let base = entry_number as usize * 4;
        let flags = self.oam[base + 3];

        OAMEntry {
            y: self.oam[base],
            x: self.oam[base + 1],
            tile_number: self.oam[base + 2],
            flags: OAMFlags {
                priority: (flags & 0x80) >> 7 != 0,
                flip_y: (flags & 0x40) >> 6 != 0,
                flip_x: (flags & 0x20) >> 5 != 0,
                palette: (flags & 0x10) >> 4 != 0,
            },
        }
    }

    // Helper

    fn addr_into_vram_space(address: u16) -> usize {
        (address - VRAM_OFFSET) as usize
    }

    fn addr_into_oam_space(address: u16) -> usize {
        address as usize - 0xFE00
    }

    /// Returns the bg palette color in 00 ..= 11 associated with given 00 ..= 11
    fn bg_palette(&mut self, color: u8) -> u8 {
        let ff47 = self.read(0xFF47);

        match color {
            0b00 => ((ff47 & 0x01) << 1) | ((ff47 & 0x02) >> 1), // Bits 0-1 -> 00
            0b01 => ((ff47 & 0x04) >> 1) | ((ff47 & 0x08) >> 3), // Bits 2-3 -> 01
            0b10 => ((ff47 & 0x10) >> 3) | ((ff47 & 0x20) >> 5), // Bits 4-5 -> 10
            0b11 => ((ff47 & 0x40) >> 5) | ((ff47 & 0x80) >> 7), // Bits 6-7 -> 11

            _ => panic!("unexpected color code: {}", color),
        }
    }

    /*
      Potentially unused code. May be useful for the
      graphical implementations we create later.
    */
    #[allow(dead_code)]
    pub fn get_tile_set_1(&self) -> Vec<u8> {
        self.vram[0..0x07ff].to_vec()
    }
    #[allow(dead_code)]
    pub fn get_tile_set_1_and_0(&self) -> Vec<u8> {
        self.vram[0x0800..0x0FFF].to_vec()
    }
    #[allow(dead_code)]
    pub fn get_tile_set_0(&self) -> Vec<u8> {
        self.vram[0x1000..0x17ff].to_vec()
    }
    #[allow(dead_code)]
    pub fn get_tile_map_0(&self) -> Vec<u8> {
        self.vram[0x1800..0x1bff].to_vec()
    }
    #[allow(dead_code)]
    pub fn get_tile_map_1(&self) -> Vec<u8> {
        self.vram[0x1c00..0x1FFF].to_vec()
    }
}

impl MemoryMap for PPU {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            // VRAM Space
            0x8000..=0x9FFF => match self.lcds.mode_flag() {
                // Mode 0 / 1 / 2 allow VRAM access
                Mode0 | Mode1 | Mode2 => self.vram[PPU::addr_into_vram_space(address)],

                // Cannot access VRAM / OAM in Mode 3
                Mode3 => 0xFF,
            },

            // OAM Space
            0xFE00..=0xFE9F => match self.lcds.mode_flag() {
                // Mode 0 / 1 allow OAM access
                Mode0 | Mode1 => self.oam[PPU::addr_into_oam_space(address)],

                // Cannot access OAM in Mode 2 / 3
                Mode2 | Mode3 => 0xFF,
            },

            0xFF40 => self.lcdc.read(),
            0xFF41 => self.lcds.read(),
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => self.dma,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            0xFF4D => 0x00, // CGB Mode Only - KEY1 - Prepare Speed Switch
            0xFF4F => 0x00, // CGB Mode Only - VBK - VRAM Bank

            // LCD VRAM DMA Transfers (CGB Mode Only)
            0xFF51 => 0x00, // HDMA1 - New DMA Source, High
            0xFF52 => 0x00, // HDMA2 - New DMA Source, Low
            0xFF53 => 0x00, // HDMA3 - New DMA Destination, High
            0xFF54 => 0x00, // HDMA4 - New DMA Destination, Low
            0xFF55 => 0x00, // HDMA5 - New DMA Length/Mode/Sort

            // LCD Color Palettes (CGB Mode Only)
            0xFF68 => 0x00, // BCPS/BGPI - Background Palette Index
            0xFF69 => 0x00, // BCPD/BGPD - Background Palette Data
            0xFF6A => 0x00, // OCPS/OBPI - Sprite Palette Index
            0xFF6B => 0x00, // OCPD/OBPD - Sprite Palette Data

            _ => panic!("unmapped address: {:#06X}", address),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        // println!("address: {:#06X}", address);

        //if address == 0xFF00 || address == 0xFF01 {
        //println!("address: {:#06X} {:010b}", address, value);
        //}

        // TODO - Add checks on VRAM / OAM against MODE to ensure access is possible
        match address {
            // VRAM Space
            0x8000..=0x9FFF => match self.lcds.mode_flag() {
                // Mode 0 / 1 / 2 allow VRAM access
                Mode0 | Mode1 | Mode2 => self.vram[PPU::addr_into_vram_space(address)] = value,

                // Cannot access VRAM / OAM in Mode 3
                Mode3 => (),
            },

            // OAM Space
            0xFE00..=0xFE9F => match self.lcds.mode_flag() {
                // Mode 0 / 1 allow OAM access
                Mode0 | Mode1 => self.oam[PPU::addr_into_oam_space(address)] = value,

                // Cannot access OAM in Mode 2 / 3
                Mode2 | Mode3 => (),
            },

            // I/O Registers
            0xFF40 => self.lcdc.write(value),
            0xFF41 => self.lcds.write(value),
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => (), // read-only
            0xFF45 => self.lyc = value,
            0xFF46 => self.dma_transfer(value),
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,

            0xFF4C => (), // unmapped
            0xFF4D => (), // CGB Mode Only - KEY1 - Prepare Speed Switch
            0xFF4E => (), // unmapped
            0xFF4F => (), // CGB Mode Only - VBK - VRAM Bank

            // LCD VRAM DMA Transfers (CGB Mode Only)
            0xFF51 => (), // HDMA1 - New DMA Source, High
            0xFF52 => (), // HDMA2 - New DMA Source, Low
            0xFF53 => (), // HDMA3 - New DMA Destination, High
            0xFF54 => (), // HDMA4 - New DMA Destination, Low
            0xFF55 => (), // HDMA5 - New DMA Length/Mode/Sort

            // LCD Color Palettes (CGB Mode Only)
            0xFF68 => (), // BCPS/BGPI - Background Palette Index
            0xFF69 => (), // BCPD/BGPD - Background Palette Data
            0xFF6A => (), // OCPS/OBPI - Sprite Palette Index
            0xFF6B => (), // OCPD/OBPD - Sprite Palette Data

            _ => panic!("unmapped address: {:#06X}", address),
        }
    }
}

#[cfg(test)]
mod test {

    #[allow(unused_imports)]
    use crate::testing::mooneye_all;

    /* TODO
    #[test]
    fn acceptance_oam_dma() {
        mooneye_all(&format!("{}/{}", MOONEYE, "acceptance/oam_dma"));
    }
     */

    #[test]
    fn acceptance_ppu() {
        mooneye_all("acceptance/ppu");
    }
}
