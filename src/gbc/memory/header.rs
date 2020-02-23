use std::str;
use std::num::Wrapping;

pub struct Header {
    title: String,
    supports_cgb: bool,
    supports_sgb: bool,
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

        let header_checksum = Wrapping(rom[0x14D] as u8);
        let mut x = Wrapping(0u8);
        for i in 0x134..0x14D {
            x += Wrapping(!rom[i]);
        }
        assert_eq!(header_checksum, x);

        Header {
            title,
            supports_cgb,
            supports_sgb,
            cartridge_type,
            rom_size,
            ram_size,
        }
    }

    pub fn get_cartridge_type(&self) -> u8 {
        self.cartridge_type
    }
}
