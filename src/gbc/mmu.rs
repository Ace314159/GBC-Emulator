mod mbc;
mod header;

use mbc::MemoryBankController;
use header::Header;

pub struct MMU {
    mbc: Box<dyn MemoryBankController>,
    ram: [u8; 0x10000 - 0x8000],
}

impl MMU {
    pub fn new(rom: Vec<u8>) -> Self {
        let header = Header::new(&rom);
        MMU {
            mbc: mbc::get_mbc(header.get_cartridge_type(), rom),
            ram: [0; 0x10000 - 0x8000],
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        if addr < 0x8000 { self.mbc.read(addr) }
        else { self.ram[addr as usize - 0x8000] }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if addr < 0x8000 { self.mbc.write(addr, value); }
        else { self.ram[addr as usize - 0x8000] = value; }
    }
}
