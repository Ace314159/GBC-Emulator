use super::MemoryHandler;

pub struct PPU {
    // Registers
    // Control
    lcd_enable: bool,
    window_tiles_select: bool,
    window_enable: bool,
    bg_window_tiles_select: bool,
    bg_tiles_select: bool,
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
    
    vram: [u8; 0x2000]
}

impl MemoryHandler for PPU {
    fn read(&self, addr: u16) -> u8 {
        macro_rules! shift { ($bit:ident, $num:expr) => { ((self.$bit as u8) << $num) } }

        match addr {
            0x8000 ..= 0x9FFF => self.vram[addr as usize - 0x8000],
            0xFF40 => shift!(lcd_enable, 7) | shift!(window_tiles_select, 6) | shift!(window_enable, 5) |
                      shift!(bg_window_tiles_select, 4) | shift!(bg_tiles_select, 3) | shift!(obj_size, 2) |
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
            0x8000 ..= 0x9FFF => self.vram[addr as usize - 0x8000] = value,
            0xFF40 => {
                self.lcd_enable = value & (1 << 7) != 0;
                self.window_tiles_select = value & (1 << 6) != 0;
                self.window_enable = value & (1 << 5) != 0;
                self.bg_window_tiles_select = value & (1 << 3) != 0;
                self.bg_tiles_select = value & (1 << 3) != 0;
                self.obj_size = value & (1 << 2) != 0;
                self.obj_enable = value & (1 << 1) != 0;
                self.bg_priority = value & (1 << 0) != 0;
            },
            0xFF41 => {
                self.coincidence_int = value & (1 << 6) != 0;
                self.oam_int = value & (1 << 5) != 0;
                self.vblank_int = value & (1 << 4) != 0;
                self.hblank_int = value & (1 << 3) != 0;
                self.coincidence_flag = value & (1 << 2) != 0;
                self.mode = value & 0x3;
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
            window_tiles_select: false,
            window_enable: false,
            bg_window_tiles_select: false,
            bg_tiles_select: false,
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

            vram: [0; 0x2000],
        }
    }
}
