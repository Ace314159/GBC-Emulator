mod none;
mod mbc1;

use super::MemoryHandler;
use super::Header;

pub trait MemoryBankController: MemoryHandler {
    fn get_boot_rom_ptr(&mut self) -> *mut [u8; 0x100];
}

pub fn get_mbc(header: Header, rom: Vec<u8>) -> Box<dyn MemoryBankController> {
    match header.get_cartridge_type() {
        0 => Box::new(none::None::new(header, rom)),
        1 => Box::new(mbc1::MBC1::new(header, rom, false, false)),
        2 => Box::new(mbc1::MBC1::new(header, rom, true, false)),
        3 => Box::new(mbc1::MBC1::new(header, rom, true, true)),
        _ => panic!("Unsupported Cartridge Type {}", header.get_cartridge_type()),
    }
}
