use std::str;

pub struct Header {
    _title: String,
    _supports_cgb: bool,
    _supports_sgb: bool,
    cartridge_type: u8,
    rom_size: u8,
    ram_size: u8,
}

impl Header {
    pub fn new(rom: &Vec<u8>) -> Self {
        let title = str::from_utf8(&rom[0x134..0x13F]).unwrap().to_string();
        let supports_cgb = rom[0x143] & 80 != 0;
        let supports_sgb = rom[0x146] == 0x03;
        let cartridge_type = rom[0x147];
        let rom_size = rom[0x148];
        let ram_size = rom[0x149];

        let header_checksum = rom[0x14D] as u8;
        let mut x = 0u8;
        for i in 0x134..0x14D {
            x = x.wrapping_add(!rom[i]);
        }
        assert_eq!(header_checksum, x);

        Header {
            _title: title,
            _supports_cgb: supports_cgb,
            _supports_sgb: supports_sgb,
            cartridge_type,
            rom_size,
            ram_size,
        }
    }

    pub fn get_cartridge_type(&self) -> u8 {
        self.cartridge_type
    }

    pub fn get_ram_size(&self) -> usize {
        [0, 0x0800, 0x2000, 0x8000, 0x20000, 0x10000][self.ram_size as usize]
    }

    pub fn get_rom_size(&self) -> usize {
        0x8000 << self.rom_size
    }
}
