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
    enable_coincidence_int: bool,
    enable_oam_int: bool,
    enable_vblank_int: bool,
    enable_hblank_int: bool,
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
    bg_palette: [usize; 4],
    obj_palettes: [[usize; 4]; 2],
    // OAM DMA
    pub oam_dma_page: u8,
    
    // Rendering
    // BG
    clock_num: u16,
    hblank_clock: u16,
    prev_stat_signal: bool,
    lcd_was_off: bool,
    y_coord_inc: u8,
    // Sprites
    pub in_oam_dma: bool,
    pub oam_dma_clock: u16,
    visible_sprites: [u8; 4 * 20], // Stores x, tile num, and attributes
    visible_sprite_count: usize,
    current_sprite_i: usize,

    // STAT
    coincidence_int: bool,
    oam_int: bool,
    vblank_int: bool,
    hblank_int: bool,

    vram: [u8; 0x2000],
    pub oam: [u8; 0xA0],
    screen: Screen,
}

impl MemoryHandler for PPU {
    fn read(&self, addr: u16) -> u8 {
        macro_rules! shift { ($bit:ident, $num:expr) => { ((self.$bit as u8) << $num) } }

        match addr {
            0x8000 ..= 0x9FFF => if self.mode != 3 { self.vram[addr as usize - 0x8000] } else { 0xFF },
            0xFE00 ..= 0xFE9F => if self.mode < 2 && self.oam_dma_clock < 2 { self.oam[addr as usize - 0xFE00] } else { 0xFF },
            0xFF40 => shift!(lcd_enable, 7) | shift!(window_map_select, 6) | shift!(window_enable, 5) |
                      shift!(bg_window_tiles_select, 4) | shift!(bg_map_select, 3) | shift!(obj_size, 2) |
                      shift!(obj_enable, 1) | shift!(bg_priority, 0),
            0xFF41 => shift!(enable_coincidence_int, 6) | shift!(enable_oam_int, 5) | shift!(enable_vblank_int, 4) |
                      shift!(enable_hblank_int, 3) | shift!(coincidence_flag, 2) | self.mode,
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.y_coord,
            0xFF45 => self.y_coord_comp,
            0xFF46 => self.oam_dma_page,
            0xFF47 => if self.mode != 3 {
                self.bg_palette.iter().rev().fold(0, |acc, x| (acc << 2) | *x as u8 )
            } else { 0xFF},
            0xFF48 => if self.mode != 3 {
                self.obj_palettes[0].iter().rev().fold(0, |acc, x| (acc << 2) | *x as u8 )
            } else { 0xFF},
            0xFF49 => if self.mode != 3 {
                self.obj_palettes[1].iter().rev().fold(0, |acc, x| (acc << 2) | *x as u8 )
            } else { 0xFF},
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _ => panic!("Unexpected Address for PPU!"),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000 ..= 0x9FFF => if self.mode != 3 { self.vram[addr as usize - 0x8000] = value },
            0xFE00 ..= 0xFE9F => if self.mode < 2 && self.oam_dma_clock < 2 { self.oam[addr as usize - 0xFE00] = value },
            0xFF40 => {
                let old_lcd_enable = self.lcd_enable;
                self.lcd_enable = value & (1 << 7) != 0;
                self.window_map_select = value & (1 << 6) != 0;
                self.window_enable = value & (1 << 5) != 0;
                self.bg_window_tiles_select = value & (1 << 4) != 0;
                self.bg_map_select = value & (1 << 3) != 0;
                self.obj_size = value & (1 << 2) != 0;
                self.obj_enable = value & (1 << 1) != 0;
                self.bg_priority = value & (1 << 0) != 0;
                self.lcd_was_off = !old_lcd_enable && self.lcd_enable;
                if self.lcd_was_off {
                    self.mode = 0;
                    self.clock_num = 7;
                }
            },
            0xFF41 => {
                self.enable_coincidence_int = value & (1 << 6) != 0;
                self.enable_oam_int = value & (1 << 5) != 0;
                self.enable_vblank_int = value & (1 << 4) != 0;
                self.enable_hblank_int = value & (1 << 3) != 0;
            },
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF44 => {},
            0xFF45 => {
                self.y_coord_comp = value;
                if self.lcd_enable {
                    if self.y_coord == self.y_coord_comp {
                        self.coincidence_flag = true;
                        self.coincidence_int = true;
                    } else {
                        self.coincidence_flag = false;
                        self.coincidence_int = false;
                    }
                }
            },
            0xFF46 => { self.oam_dma_page = value; self.in_oam_dma = true; self.oam_dma_clock = 0; },
            0xFF47 => if self.mode != 3 { for i in 0..4 { self.bg_palette[i] = (value as usize >> 2 * i) & 0x3; } },
            0xFF48 => if self.mode != 3 { for i in 0..4 { self.obj_palettes[0][i] = (value as usize >> 2 * i) & 0x3; } },
            0xFF49 => if self.mode != 3 { for i in 0..4 { self.obj_palettes[1][i] = (value as usize >> 2 * i) & 0x3; } },
            0xFF4A => self.window_y = value,
            0xFF4B => self.window_x = value,
            _ => panic!("Unexpected Address for PPU!"),
        }
    }
}

impl PPU {
    pub fn new(sdl_ctx: &sdl2::Sdl) -> Self {
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
            enable_coincidence_int: false,
            enable_oam_int: false,
            enable_vblank_int: false,
            enable_hblank_int: false,
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
            bg_palette: [0, 1, 2, 3],
            obj_palettes: [[0, 1, 2, 3], [0, 1, 2, 3]],
            // OAM DMA
            oam_dma_page: 0,

            // Rendering
            // BG
            clock_num: 0,
            hblank_clock: 0,
            prev_stat_signal: false,
            lcd_was_off: false,
            y_coord_inc: 1,
            // Sprites
            in_oam_dma: false,
            oam_dma_clock: 0,
            visible_sprites: [0; 4 * 20],
            visible_sprite_count: 0,
            current_sprite_i: 0,

            // STAT
            coincidence_int: false,
            oam_int: false,
            vblank_int: false,
            hblank_int: false,

            vram: [0; 0x2000],
            oam: [0; 0xA0],
            screen: Screen::new(sdl_ctx),
        }
    }

    const SHADES: [[u8; 3]; 4] = [[175, 203, 70], [121, 170, 109], [34, 111, 95], [8, 41, 85]];

    pub fn emulate_clock(&mut self) -> u8 {
        let mut interrupt = 0;

        if self.lcd_enable {
            interrupt = self.render_clock();
        } else {
            self.y_coord = 0;
            self.clock_num = 0;
            self.mode = 0;
        }

        let stat_signal = self.coincidence_int || self.oam_int; || self.vblank_int || self.hblank_int;
        self.coincidence_int = false;
        self.oam_int = false;
        self.vblank_int = false;
        self.hblank_int = false;
        if !self.prev_stat_signal && stat_signal {
            interrupt |= IO::STAT_INT;
        }
        self.prev_stat_signal = stat_signal;

        interrupt
    }

    fn render_clock(&mut self) -> u8 {
        let mut interrupt = 0;
        if self.y_coord < 144 && self.y_coord_inc != 0 {
            if self.lcd_was_off { // Special timing for first scanline when LCD is just turned on
                if self.clock_num == 84 {
                    self.mode = 3;
                }  else if self.clock_num == 256 {
                    self.mode = 0;
                }
            } else {
                if self.clock_num < 80 { // OAM Scan
                    if self.clock_num == 0 { self.oam_scan() };
                    if self.clock_num == 0 && self.y_coord != 0 {
                        self.coincidence_flag = false;
                    }
                    if self.clock_num == 4 {
                        self.mode = 2;
                    }
                } else if self.clock_num >= 83 { // Pixel Transfer + HBlank
                    if self.clock_num == 84 {
                        self.mode = 3;
                        self.render_line();
                    } else if self.clock_num == self.hblank_clock {
                        self.mode = 0;
                    }
                }
            }
        } else { // VBlank
            if self.y_coord == 144 && self.clock_num == 4 {
                self.screen.render();
                interrupt = IO::VBLANK_INT;
            }
            if self.y_coord == 153 && self.clock_num == 6 {
                self.y_coord = 0;
                self.y_coord_inc = 0;
            }
            if self.y_coord != 144 || self.clock_num >= 4 {
                self.mode = 1;
                if self.enable_vblank_int || self.enable_oam_int { self.vblank_int = true; }
            }
        }

        if self.y_coord != 0 && self.y_coord == self.y_coord_comp && self.enable_coincidence_int {
            self.coincidence_int = true;
        }

        self.clock_num += 1;
        if self.clock_num == 456 {
            self.clock_num = 0;
            self.y_coord += self.y_coord_inc;
            self.y_coord_inc = 1;
            self.lcd_was_off = false;
        }
        interrupt
    }

    fn oam_scan(&mut self) {
        self.visible_sprite_count = 0;
        if !self.obj_enable { return }
        let mut oam_addr = 0usize;
        let sprite_height = if self.obj_size { 16 } else { 8 };
        let mut visible_sprite_xs: Vec<[usize; 2]> = Vec::new();
        while self.visible_sprite_count < 10 && oam_addr < self.oam.len() {
            let y = self.oam[oam_addr];
            if self.y_coord + 16 >= y && self.y_coord + 16 - sprite_height < y {
                visible_sprite_xs.push([self.oam[oam_addr + 1] as usize, oam_addr]);
                self.visible_sprite_count += 1;
            }
            oam_addr += 4;
        }
        
        // Sort visible sprites for easy processing
        visible_sprite_xs.sort_by_key(|x| x[0]);
        for (i, oam_i) in visible_sprite_xs.iter().enumerate() {
            for j in 0..4 {
                self.visible_sprites[i * 4 + j] = self.oam[oam_i[1] + j];
            }
        }
    }

    fn render_line(&mut self) {
        self.hblank_clock = 247 + self.scroll_x as u16 % 8;
        let bg_map_offset: u16 = if self.bg_map_select { 0x9C00 } else { 0x9800 };
        let y = self.y_coord.wrapping_add(self.scroll_y) as u16;
        self.current_sprite_i = 0;
        for x in 0u8..160u8 {
            // BG
            let map_x = x.wrapping_add(self.scroll_x) / 8u8 % 32;
            let bg_map_addr = bg_map_offset + (y / 8 * 32) + map_x as u16;
            let tile_num = self.vram[bg_map_addr as usize - 0x8000];
            let tile_addr = if self.bg_window_tiles_select {
                0x8000 + ((tile_num as usize) << 4)
            } else { (0x900u16.wrapping_add(tile_num as i8 as u16) << 4) as usize};
            let tile_addr = tile_addr + 2 * (y as usize % 8);
            let tile_highs: u8 = self.vram[tile_addr - 0x8000];
            let tile_lows: u8 = self.vram[tile_addr + 1 - 0x8000];
            let tile_x: u8 = x.wrapping_add(self.scroll_x) % 8;
            let high = (tile_highs >> (7 - tile_x)) & 0x1;
            let low = (tile_lows >> (7 - tile_x)) & 0x1;
            let bg_color = (high << 1 | low) as usize;

            // Sprite
            let mut obj_shade = 0;
            let mut i = self.current_sprite_i;
            while i < self.visible_sprite_count && obj_shade == 0 {
                let sprite_x: u8 = self.visible_sprites[i * 4 + 1];
                if x + 8 >= sprite_x {
                    if x < sprite_x {
                        let sprite_y: u8 = self.visible_sprites[i * 4];
                        let tile_num: u8 = self.visible_sprites[i * 4 + 2];
                        let attrs: u8 = self.visible_sprites[i * 4 + 3];
                        let flip_y = attrs & 0x40 != 0;
                        let flip_x = attrs & 0x20 != 0;
                        let tile_addr = (0x8000 | (tile_num as u16) << 4) as usize;
    
                        let tile_addr = if flip_y {
                            tile_addr + 2 * (sprite_y - self.y_coord - 1 - 8) as usize
                        } else {
                            tile_addr + 2 * (15 - (sprite_y - self.y_coord - 1)) as usize
                        };
                        
                        let tile_highs: u8 = self.vram[tile_addr - 0x8000];
                        let tile_lows: u8 = self.vram[tile_addr + 1 - 0x8000];
                        let tile_x = if flip_x {
                            7 - (sprite_x - x - 1)
                        } else {
                            sprite_x - x - 1
                        };
                        let high = (tile_highs >> tile_x) & 0x1;
                        let low = (tile_lows >> tile_x) & 0x1;
    
                        let obj_priority = attrs & 0x80 != 0;
                        let palette_num = (attrs & 0x10 != 0) as usize;
                        let shade = self.obj_palettes[palette_num][(high << 1 | low) as usize];
                        if !obj_priority || bg_color == 0 { obj_shade = shade; }
                    }
                    if x + 1 == sprite_x { self.current_sprite_i += 1 }
                    i += 1;
                } else { break }
            }

            let pixel_index = 3 * ((Screen::HEIGHT - 1 - self.y_coord as u32) * Screen::WIDTH + x as u32) as usize;
            for i in 0..3 {
                self.screen.pixels[pixel_index + i] = if obj_shade != 0 {
                    PPU::SHADES[obj_shade][i]
                } else {
                    PPU::SHADES[self.bg_palette[bg_color]][i]
                };
            }
        }
    }

    fn render_map(&mut self) {
        let bg_map_offset: u16 = if self.bg_map_select { 0x9C00 } else { 0x9800 };
        for y in 0u16..32u16 * 8 {
            for x in 0u16..32u16 * 8 {
                let map_x = x / 8;
                let bg_map_addr = bg_map_offset + (y / 8 * 32) + map_x as u16;
                let tile_num = self.vram[bg_map_addr as usize - 0x8000];
                let tile_addr = if self.bg_window_tiles_select {
                    0x8000 + ((tile_num as usize) << 4)
                } else { (0x900u16.wrapping_add(tile_num as i8 as u16) << 4) as usize};
                let tile_addr = tile_addr + 2 * (y as usize % 8);
                let tile_highs: u8 = self.vram[tile_addr - 0x8000];
                let tile_lows: u8 = self.vram[tile_addr + 1 - 0x8000];
                let tile_x: u8 = x as u8 % 8;
                let high = (tile_highs >> (7 - tile_x)) & 0x1;
                let low = (tile_lows >> (7 - tile_x)) & 0x1;
                let bg_color = (high << 1 | low) as usize;

                let pixel_index = 3 * ((Screen::HEIGHT - 1 - y as u32) * Screen::WIDTH + x as u32) as usize;
                for i in 0..3 {
                    self.screen.pixels[pixel_index + i] = PPU::SHADES[self.bg_palette[bg_color]][i];
                }
            }
        }
    }
}
