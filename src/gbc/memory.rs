mod mbc;
mod header;

use mbc::MemoryBankController;
use header::Header;

pub struct Memory {
    header: Header,
    mbc: Box<dyn MemoryBankController>,
}

impl Memory {
    pub fn new(rom: Vec<u8>) -> Self {
        let header = Header::new(&rom);
        Memory {
            mbc: mbc::get_mbc(header.get_cartridge_type(), rom),
            header,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.mbc.read(addr)
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.mbc.write(addr, value);
    }
}
