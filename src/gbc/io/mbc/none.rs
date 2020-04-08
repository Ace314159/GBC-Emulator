use super::MemoryBankController;
use super::MemoryHandler;
use super::Header;

pub struct None {
    rom: Vec<u8>,
    ram: [u8; 0x2000],

    has_ram: bool,
    _has_battery: bool,
}

impl None {
    pub fn new(_header: Header, rom: Vec<u8>, has_ram: bool, has_battery: bool) -> Self {
        None {
            rom,
            ram: [0; 0x2000],

            has_ram,
            _has_battery: has_battery,
        }
    }
}

impl MemoryHandler for None {
    fn read(&self, addr: u16) -> u8 {
        if addr < 0x8000 {
            self.rom[addr as usize]
        } else if self.has_ram {
            self.ram[addr as usize - 0xA000]
        } else { 0xFF }
    }
    
    fn write(&mut self, addr: u16, value: u8) {
        if addr >= 0xA000 && addr < 0xC000 && self.has_ram {
            self.ram[addr as usize - 0xA000] = value;
        }
    }
}

impl MemoryBankController for None {
    fn get_boot_rom_ptr(&mut self) -> *mut [u8; 0x100] {
        self.rom[..0x100].as_mut_ptr() as *mut [u8; 0x100]
    }
}
