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
        return self.rom[addr as usize];
    }
    
    fn write(&mut self, addr: u16, value: u8) {
        // Cannot write to ROM
    }
}
