mod none;
mod mbc1;

use super::MemoryHandler;

pub trait MemoryBankController: MemoryHandler {
    fn get_boot_rom_ptr(&mut self) -> *mut [u8; 0x100];
}

pub fn get_mbc(cartridge_type: u8, rom: Vec<u8>) -> Box<dyn MemoryBankController> {
    match cartridge_type {
        0 => Box::new(none::None::new(rom)),
        1 => Box::new(mbc1::MBC1::new(rom)),
        _ => panic!("Unsupported MBC {}", cartridge_type),
    }
}
