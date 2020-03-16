mod screen;

use super::MemoryHandler;
use super::IO;
use screen::Screen;

pub struct PPU {
    // Registers
    // Control
    lcd_enable: bool,
    window_map_select: bool,
    window_enable: bool,
    bg_window_tiles_select: bool,
    bg_map_select: bool,
    obj_size: bool,
    obj_enable: bool,
    bg_priority: bool,
    // Status
    coincidence_int: bool,
    oam_int: bool,
    vblank_int: bool,
    hblank_int: bool,
    coincidence_flag: bool,
    mode: u8,
    // Position and Scrolling
    scroll_y: u8,
    scroll_x: u8,
    y_coord: u8,
    y_coord_comp: u8,
    window_y: u8,
    window_x: u8,
    // Palettes
    bg_palette: u8,
    obj_palette1: u8,
    obj_palette2: u8,
    
    // Rendering
    clock_num: u8,
    hblank_clock: u8,
    prev_stat_signal: bool,

    vram: [u8; 0x2000],
    screen: Screen,
}

impl MemoryHandler for PPU {
    fn read(&self, addr: u16) -> u8 {
        macro_rules! shift { ($bit:ident, $num:expr) => { ((self.$bit as u8) << $num) } }

        match addr {
            0x8000 ..= 0x9FFF => if self.mode != 3 { self.vram[addr as usize - 0x8000] } else { 0xFF },
            0xFF40 => shift!(lcd_enable, 7) | shift!(window_map_select, 6) | shift!(window_enable, 5) |
                      shift!(bg_window_tiles_select, 4) | shift!(bg_map_select, 3) | shift!(obj_size, 2) |
                      shift!(obj_enable, 1) | shift!(bg_priority, 0),
            0xFF41 => shift!(coincidence_int, 6) | shift!(oam_int, 5) | shift!(vblank_int, 4) | shift!(hblank_int, 3) |
                      shift!(coincidence_flag, 2) | self.mode,
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.y_coord,
            0xFF45 => self.y_coord_comp,
            0xFF47 => self.bg_palette,
            0xFF48 => self.obj_palette1,
            0xFF49 => self.obj_palette2,
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _ => panic!("Unexpected Address for PPU!"),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000 ..= 0x9FFF => if true || self.mode != 3 { self.vram[addr as usize - 0x8000] = value },
            0xFF40 => {
                self.lcd_enable = value & (1 << 7) != 0;
                self.window_map_select = value & (1 << 6) != 0;
                self.window_enable = value & (1 << 5) != 0;
                self.bg_window_tiles_select = value & (1 << 4) != 0;
                self.bg_map_select = value & (1 << 3) != 0;
                self.obj_size = value & (1 << 2) != 0;
                self.obj_enable = value & (1 << 1) != 0;
                self.bg_priority = value & (1 << 0) != 0;
            },
            0xFF41 => {
                self.coincidence_int = value & (1 << 6) != 0;
                self.oam_int = value & (1 << 5) != 0;
                self.vblank_int = value & (1 << 4) != 0;
                self.hblank_int = value & (1 << 3) != 0;
            },
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF44 => {},
            0xFF45 => self.y_coord_comp = value,
            0xFF47 => self.bg_palette = value,
            0xFF48 => self.obj_palette1 = value,
            0xFF49 => self.obj_palette2 = value,
            0xFF4A => self.window_y = value,
            0xFF4B => self.window_x = value,
            _ => panic!("Unexpected Address for PPU!"),
        }
    }
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            // Registers
            // Control
            lcd_enable: false,
            window_map_select: false,
            window_enable: false,
            bg_window_tiles_select: true,
            bg_map_select: false,
            obj_size: false,
            obj_enable: false,
            bg_priority: false,
            // Status
            coincidence_int: false,
            oam_int: false,
            vblank_int: false,
            hblank_int: false,
            coincidence_flag: false,
            mode: 0,
            // Position and Scrolling
            scroll_y: 0,
            scroll_x: 0,
            y_coord: 0,
            y_coord_comp: 0,
            window_y: 0,
            window_x: 0,
            // Palettes
            bg_palette: 0,
            obj_palette1: 0,
            obj_palette2: 0,

            clock_num: 0,
            hblank_clock: 0,
            prev_stat_signal: false,

            vram: [0; 0x2000],
            screen: Screen::new(),
        }
    }

    pub fn emulate(&mut self) -> u8 {
        let mut interrupt = 0;
        if self.y_coord < 144 {
            if self.clock_num == 80 { // OAM Search
                self.mode = 2;
                self.hblank_clock = 248 + self.scroll_x % 8;

                let map_offset: u16 = if self.bg_map_select { 0x9C00 } else { 0x9800 };
                let y = self.y_coord.wrapping_add(self.scroll_y);
                for map_num in 0..20 {
                    let map_addr = map_offset + (y as u16 / 8 * 32) + map_num;
                    let tile_num = self.vram[map_addr as usize - 0x8000];
                    let tile_addr = if self.bg_window_tiles_select {
                        0x8000 + ((tile_num as usize) << 4)
                    } else { 0x9000 + ((tile_num as i8 as usize) << 4) };
                    let tile_addr = tile_addr + 2 * (y as usize % 8);

                    let highs = self.vram[tile_addr - 0x8000];
                    let lows = self.vram[tile_addr + 1 - 0x8000];
                    for bit in 0..8 {
                        let pixel_index = 3 * ((Screen::HEIGHT - 1 - self.y_coord as u32) * Screen::WIDTH + 
                                                map_num as u32 * 8 + bit) as usize;
                        let high = (highs >> (7 - bit)) & 0x1;
                        let low = (lows >> (7 - bit)) & 0x1;
                        match (high << 1) | low {
                            0 => {
                                self.screen.pixels[pixel_index] = 0;
                                self.screen.pixels[pixel_index + 1] = 0;
                                self.screen.pixels[pixel_index + 2] = 0;
                            },
                            1 => {
                                self.screen.pixels[pixel_index] = 255;
                                self.screen.pixels[pixel_index + 1] = 0;
                                self.screen.pixels[pixel_index + 2] = 0;
                            },
                            2 => {
                                self.screen.pixels[pixel_index] = 0;
                                self.screen.pixels[pixel_index + 1] = 255;
                                self.screen.pixels[pixel_index + 2] = 0;
                            },
                            3 => {
                                self.screen.pixels[pixel_index] = 255;
                                self.screen.pixels[pixel_index + 1] = 255;
                                self.screen.pixels[pixel_index + 2] = 255;
                            },
                            _ => panic!("Unexpted Tile!"),
                        }
                    }
                }
            } else if self.clock_num == self.hblank_clock {
                self.mode = 3;
            } else if self.clock_num > self.hblank_clock {
                self.mode = 0;
            }
        } else { // VBlank
            if self.y_coord == 144 && self.clock_num == 1 {
                self.screen.render();
                interrupt = IO::VBLANK_INT;
            }
            self.mode = 1;
        }

        let stat_signal = self.y_coord == self.y_coord_comp && self.coincidence_int ||
                                self.mode == 0 && self.hblank_int ||
                                self.mode == 2 && self.oam_int ||
                                self.mode == 1 && (self.vblank_int | self.oam_int);
        if !self.prev_stat_signal && stat_signal {
            interrupt |= IO::STAT_INT;
        }
        self.prev_stat_signal = stat_signal;

        if self.clock_num == 114 {
            self.clock_num = 1;
            self.y_coord = (self.y_coord + 1) % 154;
        } else { self.clock_num += 1; }
        interrupt
    }

    pub fn should_close(&self) -> bool {
        self.screen.should_close()
    }
}
