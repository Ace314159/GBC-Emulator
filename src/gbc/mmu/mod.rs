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
        if addr < 0x8000 || addr >= 0xA000 && addr < 0xC000 { self.mbc.read(addr) }
        else { self.ram[addr as usize - 0x8000] }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if addr < 0x8000 || addr >= 0xA000 && addr < 0xC000 { self.mbc.write(addr, value); }
        else { self.ram[addr as usize - 0x8000] = value; }

        if addr == 0xFF02 && value == 0x81 {
            println!("{}", self.ram[0xFF01 - 0x8000] as char);
            self.ram[0xFF02 - 0x8000] &= !0x80;
        }
    }

    pub fn swap_boot_rom(&mut self, boot_rom: &mut Vec<u8>) {
        let boot_rom_len = boot_rom.len();
        assert_eq!(boot_rom_len, 0x100);
        unsafe {
            let x = boot_rom[..boot_rom_len].as_mut_ptr() as *mut [u8; 0x100];
            std::ptr::swap(x, self.mbc.get_boot_rom_ptr());
        }
    }
}
