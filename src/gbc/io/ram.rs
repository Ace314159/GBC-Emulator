use super::MemoryHandler;

pub struct HRAM {
    mem: [u8; 0xFFFE - 0xFF80 + 1],
}

impl HRAM {
    pub fn new() -> Self {
        HRAM {
            mem: [0; 0xFFFE - 0xFF80 + 1],
        }
    }
}

impl MemoryHandler for HRAM {
    fn read(&self, addr: u16) -> u8 {
        self.mem[addr as usize - 0xFF80]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.mem[addr as usize - 0xFF80] = value;
    }
}

pub struct WRAM {
    mem: Vec<u8>,
    bank: usize,
    num_banks: usize,
}

impl MemoryHandler for WRAM {
    fn read(&self, addr: u16) -> u8{
        if addr < 0xD000 {
            self.mem[addr as usize - 0xC000]
        } else {
            self.mem[self.bank * 0x1000 + addr as usize - 0xD000]
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if addr < 0xD000 {
            self.mem[addr as usize - 0xC000] = value;
        } else {
            self.mem[self.bank * 0x1000 + addr as usize - 0xD000] = value;
        }
    }
}

impl WRAM {
    pub fn new(num_banks: usize) -> Self {
        WRAM {
            mem: vec![0; num_banks * 0x1000],
            bank: 1,
            num_banks,
        }
    }

    pub fn write_bank(&mut self, bank: u8) {
        if self.num_banks == 2 { return }
        self.bank = if bank == 0 { 1 } else { bank as usize };
    }

    pub fn read_bank(&self) -> u8 {
        if self.num_banks == 2 { 0xFF } else { self.bank as u8 }
    }
}

