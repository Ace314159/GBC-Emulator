mod none;
mod mbc1;
mod mbc2;
mod mbc3;

use super::MemoryHandler;
use super::Header;

pub trait MemoryBankController: MemoryHandler {
    fn get_boot_rom_ptr(&mut self) -> *mut [u8; 0x900];
    fn emulate_clock(&mut self);
}

pub fn get_mbc(header: Header, rom: Vec<u8>) -> Box<dyn MemoryBankController> {
    match header.get_cartridge_type() {
        0x00 => Box::new(none::None::new(header, rom, false, false)),
        0x01 => Box::new(mbc1::MBC1::new(header, rom, false, false)),
        0x02 => Box::new(mbc1::MBC1::new(header, rom, true, false)),
        0x03 => Box::new(mbc1::MBC1::new(header, rom, true, true)),
        0x05 => Box::new(mbc2::MBC2::new(header, rom, false)),
        0x06 => Box::new(mbc2::MBC2::new(header, rom, true)),
        0x08 => Box::new(none::None::new(header, rom, true, false)),
        0x09 => Box::new(none::None::new(header, rom, true, true)),
        // TODO: Implement MMM01
        0x0F => Box::new(mbc3::MBC3::new(header, rom, false, false, true)),
        0x10 => Box::new(mbc3::MBC3::new(header, rom, true, true, true)),
        0x11 => Box::new(mbc3::MBC3::new(header, rom, false, false, false)),
        0x12 => Box::new(mbc3::MBC3::new(header, rom, false, true, false)),
        0x13 => Box::new(mbc3::MBC3::new(header, rom, false, true, true)),
        _ => panic!("Unsupported Cartridge Type {:X}", header.get_cartridge_type()),
    }
}
