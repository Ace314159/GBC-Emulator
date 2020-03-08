use super::MemoryBankController;

pub struct MBC1 {
    rom: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,
    is_rom_banking: bool,
    external_RAM: [u8; 0x2000],
}

impl MBC1 {
    pub fn new(rom: Vec<u8>) -> Self {
        MBC1 {
            rom,
            rom_bank: 0,
            ram_bank: 0,
            ram_enable: false,
            is_rom_banking: true,
            external_RAM: [0; 0x2000],
        }
    }
}

impl MemoryBankController for MBC1 {
    fn read(&self, addr: u16) -> u8 {
        match addr & 0xC000 {
            0x0000 => self.rom[addr as usize],
            0x4000 => self.rom[self.rom_bank * 0x4000 + addr as usize],
            0x8000 => self.external_RAM[addr as usize - 0xA000],
            _ => panic!("Shouldn't be here!"),
        }
    }
    
    fn write(&mut self, addr: u16, value: u8) {
        let bank = if value == 0 { 1usize } else { 0usize };
        match addr & 0xE000 {
            0x0000 => self.ram_enable = value & 0x0A != 0,
            0x2000 => self.rom_bank = self.rom_bank & !0x1F | (bank as usize) & 0x1F,
            0x4000 => if self.is_rom_banking { self.rom_bank = self.rom_bank & !0x60 | (value as usize) << 5; }
                      else { self.ram_bank = value as usize & 0x03; },
            0x6000 => self.is_rom_banking = value == 0,
            0xA000 => self.external_RAM[addr as usize - 0xA000] = value,
            _ => panic!("Shouldn't be here!"),
        }
        assert_eq!(self.is_rom_banking, true); // TODO: Add support for RAM Banking
    }

    fn get_boot_rom_ptr(&mut self) -> *mut [u8; 0x100] {
        self.rom[..0x100].as_mut_ptr() as *mut [u8; 0x100]
    }
}
