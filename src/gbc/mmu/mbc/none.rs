use super::MemoryBankController;

pub struct None {
    rom: Vec<u8>,
}

impl None {
    pub fn new(rom: Vec<u8>) -> Self {
        None {
            rom,
        }
    }
}

impl MemoryBankController for None {
    fn read(&self, addr: u16) -> u8 {
        if addr < 0x8000 {
            self.rom[addr as usize]
        } else {
            0 // TODO: Add External RAM Support
        }
    }
    
    fn write(&mut self, addr: u16, value: u8) {
        if addr > 0x8000 {
            panic!("External RAM not supported!");
        }
    }
}
