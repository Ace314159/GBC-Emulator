use super::MemoryBankController;
use super::MemoryHandler;
use super::Header;

pub struct MBC2 {
    rom_mask: usize,

    rom: Vec<u8>,
    rom_bank: usize,
    ram_enable: bool,
    ram: [u8; 0x200],

    _has_battery: bool,
}

impl MBC2 {
    pub fn new(header: Header, rom: Vec<u8>, has_battery: bool) -> Self {
        let rom_size = header.get_rom_size();
        assert_eq!(rom_size, rom.len());
        MBC2 {
            rom_mask: rom_size / 0x4000 - 1,

            rom,
            rom_bank: 1,
            ram_enable: false,
            ram: [0; 0x200],

            _has_battery: has_battery,
        }
    }
}

impl MemoryHandler for MBC2 {
    fn read(&self, addr: u16) -> u8 {
        if addr >= 0xA200 { return 0xFF }
        match addr & 0xC000 {
            0x0000 => self.rom[addr as usize],
            0x4000 => self.rom[self.rom_bank * 0x4000 + (addr - 0x4000) as usize],
            0x8000 => if self.ram_enable {
                self.ram[addr as usize - 0xA000]
            } else { 0xFF },
            _ => panic!("Shouldn't be here!"),
        }
    }
    
    fn write(&mut self, addr: u16, value: u8) {
        if addr < 0x4000 {
            if addr & 0x0100 == 0 {
                self.ram_enable = value & 0x0F == 0x0A;
            } else {
                self.rom_bank = (value as usize & 0x0F) & self.rom_mask;
                if self.rom_bank == 0 {
                    self.rom_bank = 1;
                }
            }
        } else if addr >= 0xA000 && addr < 0xA200 {
            if self.ram_enable {
                self.ram[addr as usize - 0xA000] = value & 0x0F;
            }
        }
    }
}

impl MemoryBankController for MBC2 {
    fn get_boot_rom_ptr(&mut self) -> *mut [u8; 0x100] {
        self.rom[..0x100].as_mut_ptr() as *mut [u8; 0x100]
    }
}
