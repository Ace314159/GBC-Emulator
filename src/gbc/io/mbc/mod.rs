mod none;
mod mbc1;
mod mbc2;

use super::MemoryHandler;
use super::Header;

pub trait MemoryBankController: MemoryHandler {
    fn get_boot_rom_ptr(&mut self) -> *mut [u8; 0x100];
}

pub fn get_mbc(header: Header, rom: Vec<u8>) -> Box<dyn MemoryBankController> {
    match header.get_cartridge_type() {
        0x00 => Box::new(none::None::new(header, rom)),
        0x01 => Box::new(mbc1::MBC1::new(header, rom, false, false)),
        0x02 => Box::new(mbc1::MBC1::new(header, rom, true, false)),
        0x03 => Box::new(mbc1::MBC1::new(header, rom, true, true)),
        0x05 => Box::new(mbc2::MBC2::new(header, rom, false)),
        0x06 => Box::new(mbc2::MBC2::new(header, rom, true)),
        _ => panic!("Unsupported Cartridge Type {}", header.get_cartridge_type()),
    }
}
