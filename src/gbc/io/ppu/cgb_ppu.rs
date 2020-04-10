use super::MemoryHandler;
use super::IO;
use super::screen::Screen;
use super::PPU;

pub struct CgbPPU {
    // Registers
    // Control
    lcd_enable: bool,
    window_map_select: bool,
    window_enable: bool,
    bg_window_tiles_select: bool,
    bg_map_select: bool,
    obj_size: bool,
    obj_enable: bool,
    bg_window_priority: bool,
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
    // Non-CGB
    gb_bg_palette: u8,
    gb_obj_palette0: u8,
    gb_obj_palette1: u8,
    // CGB
    bg_palette_i: u8,
    bg_palette_inc: bool,
    bg_palettes: [u8; 0x40],
    bg_colors: [[[u8; 3]; 4]; 8],
    obj_palette_i: u8,
    obj_palette_inc: bool,
    obj_palettes: [u8; 0x40],
    obj_colors: [[[u8; 3]; 4]; 8],
    // OAM DMA
    oam_dma_page: u8,
    // HDMA
    hdma_src: u16,
    hdma_dest: u16,
    
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
    gdma_clock: u16,
    in_hdma: bool,
    hdma_clock: u16,
    visible_sprites: [u8; 4 * 20], // Stores x, tile num, and attributes
    visible_sprite_count: usize,
    current_sprite_i: usize,

    // STAT
    coincidence_int: bool,
    oam_int: bool,
    vblank_int: bool,
    hblank_int: bool,

    vram: [[u8; 0x2000]; 2],
    vram_bank: usize,
    pub oam: [u8; 0xA0],
    screen: Screen,
    pub _rendering_map: bool,
    _rendered_map: bool,
}

impl MemoryHandler for CgbPPU {
    fn read(&self, addr: u16) -> u8 {
        macro_rules! shift { ($bit:ident, $num:expr) => { ((self.$bit as u8) << $num) } }

        match addr {
            0x8000 ..= 0x9FFF => if self.mode != 3 { self.vram[self.vram_bank][addr as usize - 0x8000] } else { 0xFF },
            0xFE00 ..= 0xFE9F => if self.mode < 2 && self.oam_dma_clock < 2 { self.oam[addr as usize - 0xFE00] } else { 0xFF },
            0xFF40 => shift!(lcd_enable, 7) | shift!(window_map_select, 6) | shift!(window_enable, 5) |
                      shift!(bg_window_tiles_select, 4) | shift!(bg_map_select, 3) | shift!(obj_size, 2) |
                      shift!(obj_enable, 1) | shift!(bg_window_priority, 0),
            0xFF41 => 0x80 | shift!(enable_coincidence_int, 6) | shift!(enable_oam_int, 5) | shift!(enable_vblank_int, 4) |
                      shift!(enable_hblank_int, 3) | shift!(coincidence_flag, 2) | self.mode,
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.y_coord,
            0xFF45 => self.y_coord_comp,
            0xFF46 => self.oam_dma_page,
            0xFF47 => self.gb_bg_palette,
            0xFF48 => self.gb_obj_palette0,
            0xFF49 => self.gb_obj_palette1,
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _ => panic!("Unexpected Address for PPU!"),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000 ..= 0x9FFF => if self.mode != 3 { self.vram[self.vram_bank][addr as usize - 0x8000] = value },
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
                self.bg_window_priority = value & (1 << 0) != 0;
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
            0xFF47 => self.gb_bg_palette = value,
            0xFF48 => self.gb_obj_palette0 = value,
            0xFF49 => self.gb_obj_palette1 = value,
            0xFF4A => self.window_y = value,
            0xFF4B => self.window_x = value,
            _ => panic!("Unexpected Address for PPU!"),
        }
    }
}

impl PPU for CgbPPU {
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

    fn read_vram_bank(&self) -> u8 {
        (0xFE | self.vram_bank) as u8
    }

    fn read_cgb_palettes(&self, addr: u16) -> u8 {
        if self.mode == 3 { return 0xFF }
        match addr {
            0xFF68 => self.bg_palette_i | (self.bg_palette_inc as u8) << 7,
            0xFF69 => self.bg_palettes[self.bg_palette_i as usize],
            0xFF6A => self.obj_palette_i | (self.obj_palette_inc as u8) << 7,
            0xFF6B => self.obj_palettes[self.obj_palette_i as usize],
            _ => panic!("Unexpected Address for PPU!"),
        }
    }

    fn write_vram_bank(&mut self, value: u8) {
        self.vram_bank = value as usize & 0x1;
    }

    fn write_cgb_palettes(&mut self, addr: u16, value: u8) {
        if self.mode == 3 { return }
        match addr {
            0xFF68 => {
                self.bg_palette_i = value & 0x3F;
                self.bg_palette_inc = value & 0x80 != 0;
            },
            0xFF69 => {
                let palette_num = self.bg_palette_i as usize / 8;
                let color_num = self.bg_palette_i as usize % 8 / 2;
                let value = if self.bg_palette_i % 2 == 1 {
                    let green = (value & 0x3) << 3 | self.bg_palettes[self.bg_palette_i as usize - 1] >> 5 & 0x7;
                    let blue = (value >> 2) & 0x1F;
                    self.bg_colors[palette_num][color_num][1] = ((green as u16) * 255 / 31) as u8;
                    self.bg_colors[palette_num][color_num][2] = ((blue as u16) * 255 / 31) as u8;
                    value
                } else {
                    let red = value & 0x1F;
                    let green = (self.bg_palettes[self.bg_palette_i as usize + 1] & 0x3) << 3 | value >> 5 & 0x7;
                    self.bg_colors[palette_num][color_num][0] = ((red as u16) * 255 / 31) as u8;
                    self.bg_colors[palette_num][color_num][1] = ((green as u16) * 255 / 31) as u8;
                    value
                };
                self.bg_palettes[self.bg_palette_i as usize] = value;
                if self.bg_palette_inc { self.bg_palette_i = (self.bg_palette_i + 1) % 0x40 }
            },
            0xFF6A => {
                self.obj_palette_i = value & 0x3F;
                self.obj_palette_inc = value & 0x80 != 0;
            },
            0xFF6B => {
                let palette_num = self.obj_palette_i as usize / 8;
                let color_num = self.obj_palette_i as usize % 8 / 2;
                let value = if self.obj_palette_i % 2 == 1 {
                    let green = (value & 0x3) << 3 | self.obj_palettes[self.obj_palette_i as usize - 1] >> 5 & 0x7;
                    let blue = (value >> 2) & 0x1F;
                    self.obj_colors[palette_num][color_num][1] = ((green as u16) * 255 / 31) as u8;
                    self.obj_colors[palette_num][color_num][2] = ((blue as u16) * 255 / 31) as u8;
                    value
                } else {
                    let red = value & 0x1F;
                    let green = (self.obj_palettes[self.obj_palette_i as usize + 1] & 0x3) << 3 | value >> 5 & 0x7;
                    self.obj_colors[palette_num][color_num][0] = ((red as u16) * 255 / 31) as u8;
                    self.obj_colors[palette_num][color_num][1] = ((green as u16) * 255 / 31) as u8;
                    value
                };
                self.obj_palettes[self.obj_palette_i as usize] = value;
                if self.obj_palette_inc { self.obj_palette_i = (self.obj_palette_i + 1) % 0x40 }
            },
            _ => panic!("Unexpected Address for PPU!"),
        }
    }

    fn write_hdma(&mut self, addr: u16, value: u8, double_speed: bool) {
        match addr {
            0xFF51 => self.hdma_src = self.hdma_src & !0xFF00 | (value as u16) << 8,
            0xFF52 => self.hdma_src = self.hdma_src & !0x00FF | (value & 0x0F) as u16,
            0xFF53 => self.hdma_dest = self.hdma_dest & !0xFF00 | ((value & 0x1F) as u16) << 8,
            0xFF54 => self.hdma_dest = self.hdma_dest & !0x00FF | (value & 0xF0) as u16,
            0xFF55 => if value & 0x80 != 0 {
                self.in_hdma = true;
            } else {
                let num_dma_blocks = (value & 0x7F) + 1;
                self.hdma_src &= 0xFFF0;
                self.hdma_dest &= 0x1FF0;
                self.gdma_clock = 1 + num_dma_blocks as u16 * if double_speed { 16 } else { 8 };
            }
            _ => panic!("Unexpected Address for PPU!"),
        }
    }

    fn set_double_speed(&mut self, double_speed: bool) {
        self.screen.set_double_speed(double_speed);
    }
    
    fn in_oam_dma(&self) -> bool {
        self.in_oam_dma
    }

    fn oam_dma(&mut self) -> (bool, u16, u16) {
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

    fn in_gdma(&self) -> bool {
        self.gdma_clock != 0
    }

    fn gdma(&mut self, double_speed: bool) -> (bool, u16, u16) {
        self.gdma_clock -= 1;
        if self.gdma_clock != 0 {
            let inc = if double_speed { 1 } else { 2 };
            let cpu_addr = self.hdma_src;
            let vram_addr = 0x8000 | self.hdma_dest;
            self.hdma_src += inc;
            self.hdma_dest += inc;
            (true, cpu_addr, vram_addr)
        } else { (false, 0, 0) }
    }

    fn _rendering_map(&mut self, _rendering_map: bool) { self._rendering_map = _rendering_map }
}

impl CgbPPU {
    pub fn new(sdl_ctx: &sdl2::Sdl) -> Self {
        CgbPPU {
            // Registers
            // Control
            lcd_enable: false,
            window_map_select: false,
            window_enable: false,
            bg_window_tiles_select: true,
            bg_map_select: false,
            obj_size: false,
            obj_enable: false,
            bg_window_priority: false,
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
            gb_bg_palette: 0xFF,
            gb_obj_palette0: 0xFF,
            gb_obj_palette1: 0xFF,
            // CGB
            bg_palette_i: 0,
            bg_palette_inc: false,
            bg_palettes: [0; 0x40],
            bg_colors: [[[0; 3]; 4]; 8],
            obj_palette_i: 0,
            obj_palette_inc: false,
            obj_palettes: [0; 0x40],
            obj_colors: [[[0; 3]; 4]; 8],
            // OAM DMA
            oam_dma_page: 0,
            // HDMA
            hdma_src: 0,
            hdma_dest: 0,

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
            gdma_clock: 0,
            in_hdma: false,
            hdma_clock: 0,
            visible_sprites: [0; 4 * 20],
            visible_sprite_count: 0,
            current_sprite_i: 0,

            // STAT
            coincidence_int: false,
            oam_int: false,
            vblank_int: false,
            hblank_int: false,

            vram: [[0; 0x2000]; 2],
            vram_bank: 0,
            oam: [0; 0xA0],
            screen: Screen::new(sdl_ctx),
            _rendering_map: false,
            _rendered_map: false,
        }
    }

    fn render_clock(&mut self) -> u8 {
        let mut interrupt = 0;
        if self.y_coord < 144 && self.y_coord_inc != 0 {
            if self.lcd_was_off { // Special timing for first scanline when LCD is just turned on
                self.hblank_clock = 255 + self.scroll_x as u16 % 8;
                if self.clock_num == 84 {
                    self.mode = 3;
                }  else if self.clock_num == self.hblank_clock {
                    self.mode = 0;
                    if self.enable_hblank_int { self.hblank_int = true; }
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
            if self.y_coord == 144 && self.clock_num == 4 {
                self.mode = 1;
                if self._rendering_map { self._render_map(); }
                self.screen.render();
                interrupt = IO::VBLANK_INT;
            }
            if self.y_coord == 153 && self.clock_num == 8 {
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
        if self._rendering_map { return }
        else if self._rendered_map {for i in self.screen.pixels.iter_mut() { *i = 0 }; self._rendered_map = false; }
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

            let attrs = self.vram[1][map_addr as usize - 0x8000];
            let bg_palette_num = attrs as usize & 0x7;
            let vram_bank = (attrs & 0x08 != 0) as usize;
            let flip_x = attrs & 0x20 != 0;
            let flip_y = attrs & 0x40 != 0;
            let bg_priority = attrs & 0x80 != 0;

            // BG or Window
            let tile_num = self.vram[0][map_addr as usize - 0x8000];
            let tile_addr = get_bg_window_tile(tile_num);
            let tile_addr = if flip_y { tile_addr + 2 * (7 - offsetted_y as usize % 8)
            } else { tile_addr + 2 * (offsetted_y as usize % 8) };
            let tile_lows: u8 = self.vram[vram_bank][tile_addr - 0x8000];
            let tile_highs: u8 = self.vram[vram_bank][tile_addr + 1 - 0x8000];
            let tile_x: u8 = offsetted_x % 8;
            let shift = if flip_x { tile_x } else { 7 - tile_x };
            let high = (tile_highs >> shift) & 0x1;
            let low = (tile_lows >> shift) & 0x1;
            let bg_color = (high << 1 | low) as usize;

            let mut final_color = self.bg_colors[bg_palette_num][bg_color];
            if !is_window_pixel && !bg_priority {
                // Sprite
                let mut i = self.current_sprite_i;
                while i < self.visible_sprite_count {
                    let sprite_x: u8 = self.visible_sprites[i * 4 + 1];
                    if x + 8 >= sprite_x {
                        if x < sprite_x {
                            if x == sprite_x + 1 {
                                self.hblank_clock += 11 - std::cmp::min(5, (sprite_x + self.scroll_x) % 8) as u16;
                            }
                            let sprite_y: u8 = self.visible_sprites[i * 4];
                            let tile_num: u8 = self.visible_sprites[i * 4 + 2];
                            let attrs: u8 = self.visible_sprites[i * 4 + 3];
                            let flip_y = attrs & 0x40 != 0;
                            let flip_x = attrs & 0x20 != 0;
                            let tile_addr = (0x8000 | (tile_num as u16) << 4) as usize;
        
                            let vram_bank = (attrs & 0x08 != 0) as usize;
                            let tile_addr = if flip_y {
                                tile_addr + 2 * (sprite_y - self.y_coord - 1) as usize
                            } else {
                                tile_addr + 2 * (15 - (sprite_y - self.y_coord - 1)) as usize
                            };
                            
                            let tile_lows: u8 = self.vram[vram_bank][tile_addr - 0x8000];
                            let tile_highs: u8 = self.vram[vram_bank][tile_addr + 1 - 0x8000];
                            let tile_x = if flip_x {
                                7 - (sprite_x - x - 1)
                            } else {
                                sprite_x - x - 1
                            };
                            let high = (tile_highs >> tile_x) & 0x1;
                            let low = (tile_lows >> tile_x) & 0x1;
        
                            let obj_priority = attrs & 0x80 != 0;
                            let palette_num = attrs as usize & 0x7;
                            let obj_color = (high << 1 | low) as usize;
                            if obj_color != 0 {
                                let obj_color = self.obj_colors[palette_num][obj_color];
                                if obj_priority && self.bg_window_priority {
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
                self.screen.pixels[pixel_index + i] = final_color[i];
            }
        }
    }

    fn _render_map(&mut self) {
        let bg_map_offset: u16 = if self.bg_map_select { 0x9C00 } else { 0x9800 };
        for y in 0u16..32u16 * 8 {
            for x in 0u16..32u16 * 8 {
                let map_x = x / 8;
                let bg_map_addr = bg_map_offset + (y / 8 * 32) + map_x as u16;
                let tile_num = self.vram[0][bg_map_addr as usize - 0x8000];
                let tile_addr = if self.bg_window_tiles_select {
                    0x8000 + ((tile_num as usize) << 4)
                } else { (0x900u16.wrapping_add(tile_num as i8 as u16) << 4) as usize};
                let tile_addr = tile_addr + 2 * (y as usize % 8);
                let tile_highs: u8 = self.vram[0][tile_addr - 0x8000];
                let tile_lows: u8 = self.vram[0][tile_addr + 1 - 0x8000];
                let tile_x: u8 = x as u8 % 8;
                let high = (tile_highs >> (7 - tile_x)) & 0x1;
                let low = (tile_lows >> (7 - tile_x)) & 0x1;
                let bg_color = (high << 1 | low) as usize;

                let pixel_index = 3 * ((Screen::HEIGHT - 1 - y as u32) * Screen::WIDTH + x as u32) as usize;
                for i in 0..3 {
                    self.screen.pixels[pixel_index + i] = self.bg_colors[0][bg_color][i];
                }
            }
        }
        self._rendered_map = true;
    }

    fn _render_tiles(&mut self) {
        for y in 0u16..24u16 * 8 {
            for x in 0u16..32u16 * 8 {
                let tile_num = y / 8 * 16 + x / 8;
                let tile_addr = 0x8000 + ((tile_num as usize) << 4);
                let tile_addr = tile_addr + 2 * (y as usize % 8);
                let vram_bank = (x >= 16 * 8) as usize;
                let tile_highs: u8 = self.vram[vram_bank][tile_addr - 0x8000];
                let tile_lows: u8 = self.vram[vram_bank][tile_addr + 1 - 0x8000];
                let tile_x: u8 = x as u8 % 8;
                let high = (tile_highs >> (7 - tile_x)) & 0x1;
                let low = (tile_lows >> (7 - tile_x)) & 0x1;
                let bg_color = (high << 1 | low) as usize;

                let pixel_index = 3 * ((Screen::HEIGHT - 1 - y as u32) * Screen::WIDTH + x as u32) as usize;
                let shades = [0xFF, 0xAA, 0x55, 0x00];
                for i in 0..3 {
                    self.screen.pixels[pixel_index + i] = shades[bg_color];
                }
            }
        }
        self._rendered_map = true;
    }
}
