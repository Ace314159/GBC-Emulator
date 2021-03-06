use super::MemoryHandler;
use super::IO;
use super::screen::Screen;
use super::PPU;

pub struct GbPPU {
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
    oam_dma_page: u8,
    
    // Rendering
    // BG
    clock_num: u16,
    hblank_clock: u16,
    prev_stat_signal: bool,
    lcd_was_off: bool,
    y_coord_inc: u8,
    // Sprites
    in_oam_dma: bool,
    oam_dma_clock: u16,
    disable_oam: bool,
    visible_sprites: [u8; 4 * 20], // Stores x, tile num, and attributes
    visible_sprite_count: usize,
    current_sprite_i: usize,

    // STAT
    coincidence_int: bool,
    oam_int: bool,
    vblank_int: bool,
    hblank_int: bool,

    vram: [u8; 0x2000],
    oam: [u8; 0xA0],
    screen: Screen,
    // pub _rendering_map: bool,
}

impl MemoryHandler for GbPPU {
    fn read(&self, addr: u16) -> u8 {
        macro_rules! shift { ($bit:ident, $num:expr) => { ((self.$bit as u8) << $num) } }

        match addr {
            0x8000 ..= 0x9FFF => if self.mode != 3 { self.vram[addr as usize - 0x8000] } else { 0xFF },
            0xFE00 ..= 0xFE9F => if self.mode < 2 && self.oam_dma_clock < 2 { self.oam[addr as usize - 0xFE00] } else { 0xFF },
            0xFF40 => shift!(lcd_enable, 7) | shift!(window_map_select, 6) | shift!(window_enable, 5) |
                      shift!(bg_window_tiles_select, 4) | shift!(bg_map_select, 3) | shift!(obj_size, 2) |
                      shift!(obj_enable, 1) | shift!(bg_priority, 0),
            0xFF41 => 0x80 | shift!(enable_coincidence_int, 6) | shift!(enable_oam_int, 5) | shift!(enable_vblank_int, 4) |
                      shift!(enable_hblank_int, 3) | shift!(coincidence_flag, 2) | self.mode,
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.y_coord,
            0xFF45 => self.y_coord_comp,
            0xFF46 => self.oam_dma_page,
            0xFF47 => if self.mode != 3 {
                self.bg_palette.iter().rev().fold(0, |acc, x| (acc << 2) | *x as u8 )
            } else { 0xFF },
            0xFF48 => if self.mode != 3 {
                self.obj_palettes[0].iter().rev().fold(0, |acc, x| (acc << 2) | *x as u8 )
            } else { 0xFF },
            0xFF49 => if self.mode != 3 {
                self.obj_palettes[1].iter().rev().fold(0, |acc, x| (acc << 2) | *x as u8 )
            } else { 0xFF },
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _ => panic!("Unexpected Address for PPU!"),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000 ..= 0x9FFF => if self.mode != 3 { self.vram[addr as usize - 0x8000] = value },
            0xFE00 ..= 0xFE9F => if self.mode < 2 && !self.disable_oam { self.oam[addr as usize - 0xFE00] = value },
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
                self.lcd_was_off = !old_lcd_enable && self.lcd_enable; // TODO: Add full support later
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

impl PPU for GbPPU {
    fn emulate_clock(&mut self) -> u8 {
        let mut interrupt = 0;

        if self.lcd_enable {
            interrupt = self.render_clock();
        } else {
            self.y_coord = 0;
            self.clock_num = 0;
            self.mode = 0;
        }

        let stat_signal = self.coincidence_int || self.oam_int || self.vblank_int || self.hblank_int;
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

    fn set_screen_size(&mut self, width: i32, height: i32) {
        self.screen.set_screen_size(width, height);
    }

    fn read_vram_bank(&self) -> u8 { 0xFF }
    fn read_cgb_palettes(&self, _addr: u16) -> u8 { 0xFF }
    fn read_hdma(&self) -> u8 { 0xFF }

    fn write_vram_bank(&mut self, _value: u8) {}
    fn write_cgb_palettes(&mut self, _addr: u16, _value: u8) {}
    fn write_hdma(&mut self, _addr: u16, _value: u8, _double_speed: bool) {}

    fn set_double_speed(&mut self, _double_speed: bool) {}
    
    fn in_oam_dma(&self) -> bool {
        self.in_oam_dma
    }

    fn oam_dma(&mut self) -> (bool, u16, u16) {
        if !self.in_oam_dma { return (false, 0, 0) }
        let should_write = self.oam_dma_clock < 160;
        let (oam_addr, cpu_addr) = if should_write {
            self.disable_oam = true;
            let cpu_addr = (self.oam_dma_page as u16) << 8 | (self.oam_dma_clock);
            (self.oam_dma_clock, cpu_addr)
        } else { (0, 0) };

        self.oam_dma_clock += 1;
        if self.oam_dma_clock == 160 + 2 {
            self.in_oam_dma = false;
            self.disable_oam = false;
            self.oam_dma_clock = 0;
        }
        (should_write, oam_addr, cpu_addr)
    }

    fn oam_write(&mut self, addr: u16, value: u8) {
        self.oam[addr as usize] = value;
    }

    fn in_hdma(&self) -> bool { false }
    fn hdma(&mut self, _double_speed: bool) -> (bool, u16, u16) { (false, 0, 0) }
    fn in_gdma(&self) -> bool { false }
    fn gdma(&mut self, _double_speed: bool) -> (bool, u16, u16) { (false, 0, 0) }
    fn _rendering_map(&mut self, _rendering_map: bool) {}
}

impl GbPPU {
    pub fn new(sdl_ctx: &sdl2::Sdl) -> Self {
        GbPPU {
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
            // Non-CGB
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
            disable_oam: false,
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
            // _rendering_map: false,
        }
    }

    // const SHADES: [[u8; 3]; 4] = [[175, 203, 70], [121, 170, 109], [34, 111, 95], [8, 41, 85]];
    // const SHADES: [[u8; 3]; 4] = [[0xA4, 0xC5, 0x05], [0x88, 0xA9, 0x05], [0x1D, 0x55, 0x1D], [0x05, 0x25, 0x05]];
    const SHADES: [[u8; 3]; 4] = [[0xFF, 0xFF, 0xFF], [0xAA, 0xAA, 0xAA], [0x55, 0x55, 0x55], [0x00, 0x00, 0x00]];


    fn render_clock(&mut self) -> u8 {
        let mut interrupt = 0;
        if self.y_coord < 144 && self.y_coord_inc != 0 {
            if self.lcd_was_off { // Special timing for first scanline when LCD is just turned on
                self.hblank_clock = 255 + self.scroll_x as u16 % 8;
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
                    if self.clock_num == 2 && self.y_coord == 2 {
                        if self.enable_oam_int { self.oam_int = true; }
                    } else if self.clock_num == 4 {
                        self.mode = 2;
                        if self.enable_oam_int { self.oam_int = true; }
                    }
                } else if self.clock_num >= 83 { // Pixel Transfer + HBlank
                    if self.clock_num == 84 {
                        self.mode = 3;
                        self.render_line();
                    } else if self.clock_num == self.hblank_clock {
                        self.mode = 0;
                        if self.enable_hblank_int { self.hblank_int = true; }
                    }
                }
            }
        } else { // VBlank
            if self.y_coord == 144 && self.clock_num == 2 {
                self.mode = 1;
                // if self._rendering_map { self._render_map(); }
                self.screen.render();
                interrupt = IO::VBLANK_INT;
            }
            if self.y_coord == 153 && self.clock_num == 6 {
                self.y_coord = 0;
                self.y_coord_inc = 0;
            }
            if self.y_coord > 144 || self.clock_num >= 4 {
                if self.enable_vblank_int { self.vblank_int = true; }
            }
            if self.y_coord == 144 && self.clock_num == 2 {
                if self.enable_oam_int { self.vblank_int = true; }
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
        // if self._rendering_map { return }
        fn bg_window_tiles_select_1(tile_num: u8) -> usize { (0x900u16.wrapping_add(tile_num as i8 as u16) << 4) as usize }
        fn bg_window_tiles_select_0(tile_num: u8) -> usize { 0x8000 + ((tile_num as usize) << 4) }
        self.hblank_clock = 254 + self.scroll_x as u16 % 8;
        let bg_map_offset: u16 = if self.bg_map_select { 0x9C00 } else { 0x9800 };
        let window_map_offset: u16 = if self.window_map_select { 0x9C00 } else { 0x9800 };
        let get_bg_window_tile = if self.bg_window_tiles_select {
            bg_window_tiles_select_0
        } else { bg_window_tiles_select_1 };
        let offsetted_y = self.y_coord.wrapping_add(self.scroll_y) as u16;
        self.current_sprite_i = 0;
        for x in 0u8..160u8 {
            let pixel_index = 3 * ((Screen::HEIGHT - 1 - self.y_coord as u32) * Screen::WIDTH + x as u32) as usize;

            let is_window_pixel = self.window_enable && self.y_coord >= self.window_y && x + 7 >= self.window_x;

            let (offsetted_x, offsetted_y, map_addr) = if is_window_pixel { // Window
                let offsetted_x = x + 7 - self.window_x;
                let offsetted_y = (self.y_coord - self.window_y) as u16;
                let map_x = offsetted_x / 8u8 % 32;
                (offsetted_x, offsetted_y, window_map_offset + (offsetted_y / 8 * 32) + map_x as u16)
            } else { // BG
                let offsetted_x = x.wrapping_add(self.scroll_x);
                let map_x = offsetted_x / 8u8 % 32;
                (offsetted_x, offsetted_y, bg_map_offset + (offsetted_y / 8 * 32) + map_x as u16)
            };
            // BG or Window
            let tile_num = self.vram[map_addr as usize - 0x8000];
            let tile_addr = get_bg_window_tile(tile_num);
            let tile_addr = tile_addr + 2 * (offsetted_y as usize % 8);
            let tile_lows: u8 = self.vram[tile_addr - 0x8000];
            let tile_highs: u8 = self.vram[tile_addr + 1 - 0x8000];
            let tile_x: u8 = offsetted_x % 8;
            let shift = 7 - tile_x;
            let high = (tile_highs >> shift) & 0x1;
            let low = (tile_lows >> shift) & 0x1;
            let bg_color = (high << 1 | low) as usize;

            let mut final_color = self.bg_palette[bg_color];
            if !is_window_pixel {
                // Sprite
                let mut i = self.current_sprite_i;
                while i < self.visible_sprite_count {
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
                                tile_addr + 2 * (sprite_y - self.y_coord - 1) as usize
                            } else {
                                tile_addr + 2 * (15 - (sprite_y - self.y_coord - 1)) as usize
                            };
                            
                            let tile_lows: u8 = self.vram[tile_addr - 0x8000];
                            let tile_highs: u8 = self.vram[tile_addr + 1 - 0x8000];
                            let tile_x = if flip_x {
                                7 - (sprite_x - x - 1)
                            } else {
                                sprite_x - x - 1
                            };
                            let high = (tile_highs >> tile_x) & 0x1;
                            let low = (tile_lows >> tile_x) & 0x1;
        
                            let obj_priority = attrs & 0x80 != 0;
                            let palette_num = (attrs & 0x10 != 0) as usize;
                            let obj_color = (high << 1 | low) as usize;
                            if obj_color != 0 {
                                let obj_color = self.obj_palettes[palette_num][obj_color];
                                if obj_priority {
                                    if bg_color == 0 {
                                        final_color = obj_color;
                                    }
                                } else { final_color = obj_color }
                                break;
                            }
                        }
                        if x + 1 == sprite_x { self.current_sprite_i += 1 }
                        i += 1;
                    } else { break }
                }
            }

            for i in 0..3 {
                self.screen.pixels[pixel_index + i] = GbPPU::SHADES[final_color][i];
            }
        }
    }

    fn _render_map(&mut self) {
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
                    self.screen.pixels[pixel_index + i] = GbPPU::SHADES[self.bg_palette[bg_color]][i];
                }
            }
        }
    }
}
