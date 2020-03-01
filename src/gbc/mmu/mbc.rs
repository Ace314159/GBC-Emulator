mod none;
mod mbc1;

pub trait MemoryBankController {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

pub fn get_mbc(cartridge_type: u8, rom: Vec<u8>) -> Box<dyn MemoryBankController> {
    match cartridge_type {
        0 => Box::new(none::None::new(rom)),
        1 => Box::new(mbc1::MBC1::new(rom)),
        _ => panic!("Unsupported MBC {}", cartridge_type),
    }
}
