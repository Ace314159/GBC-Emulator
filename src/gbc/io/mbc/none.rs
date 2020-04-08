use super::MemoryBankController;
use super::MemoryHandler;
use super::Header;

pub struct None {
    rom: Vec<u8>,
}

impl None {
    pub fn new(_header: Header, rom: Vec<u8>) -> Self {
        None {
            rom,
        }
    }
}

impl MemoryHandler for None {
    fn read(&self, addr: u16) -> u8 {
        if addr < 0x8000 {
            self.rom[addr as usize]
        } else {
            0xFF // TODO: Add External RAM Support
        }
    }
    
    fn write(&mut self, addr: u16, _value: u8) {
        if addr > 0x8000 {
            panic!("External RAM not supported!");
        }
    }
}

impl MemoryBankController for None {
    fn get_boot_rom_ptr(&mut self) -> *mut [u8; 0x100] {
        self.rom[..0x100].as_mut_ptr() as *mut [u8; 0x100]
    }
}
