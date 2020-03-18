mod header;
mod mbc;
mod ppu;
mod ram;
mod serial;
mod timer;

use header::Header;
use mbc::MemoryBankController;
use ppu::PPU;
use ram::RAM;
use serial::Serial;
use timer::Timer;

pub trait MemoryHandler {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

pub struct IO {
    mbc: Box<dyn MemoryBankController>,
    pub ppu: PPU,
    wram: RAM,
    serial: Serial,
    timer: Timer,
    pub int_enable: u8,
    hram: RAM,
    pub int_flags: u8,
    unusable: Unusable,
}

impl IO {
    pub fn new(rom: Vec<u8>) -> Self {
        let header = Header::new(&rom);
        IO {
            mbc: mbc::get_mbc(header.get_cartridge_type(), rom),
            ppu: PPU::new(),
            wram: RAM::new(0xC000, 0xDFFF),
            serial: Serial::new(),
            timer: Timer::new(),
            int_enable: 0,
            hram: RAM::new(0xFF80, 0xFFFE),
            int_flags: 0,
            unusable: Unusable {},
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000 ..= 0x7FFF => self.mbc.read(addr),
            0x8000 ..= 0x9FFF => self.ppu.read(addr),
            0xA000 ..= 0xBFFF => self.mbc.read(addr),
            0xC000 ..= 0xDFFF => self.wram.read(addr),
            0xFF01 ..= 0xFF02 => self.serial.read(addr),
            0xFF04 ..= 0xFF07 => self.timer.read(addr),
            0xFF0F => self.int_flags,
            0xFF40 ..= 0xFF4A => self.ppu.read(addr),
            0xFF80 ..= 0xFFFE => self.hram.read(addr),
            0xFFFF => self.int_enable,
            _ => self.unusable.read(addr),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000 ..= 0x7FFF => self.mbc.write(addr, value),
            0x8000 ..= 0x9FFF => self.ppu.write(addr, value),
            0xA000 ..= 0xBFFF => self.mbc.write(addr, value),
            0xC000 ..= 0xDFFF => self.wram.write(addr, value),
            0xFF01 ..= 0xFF02 => self.serial.write(addr, value),
            0xFF04 ..= 0xFF07 => self.timer.write(addr, value),
            0xFF0F => self.int_flags = value,
            0xFF40 ..= 0xFF4A => self.ppu.write(addr, value),
            0xFF80 ..= 0xFFFE => self.hram.write(addr, value),
            0xFFFF => self.int_enable = value,
            _ => self.unusable.write(addr, value),
        };
    }

    pub fn emulate_machine_cycle(&mut self) {
        self.int_flags |= self.timer.emulate();
        self.int_flags |= self.ppu.emulate_clock();
        self.int_flags |= self.ppu.emulate_clock();
        self.int_flags |= self.ppu.emulate_clock();
        self.int_flags |= self.ppu.emulate_clock();
    }

    pub fn swap_boot_rom(&mut self, boot_rom: &mut Vec<u8>) {
        let boot_rom_len = boot_rom.len();
        assert_eq!(boot_rom_len, 0x100);
        unsafe {
            let x = boot_rom[..boot_rom_len].as_mut_ptr() as *mut [u8; 0x100];
            std::ptr::swap(x, self.mbc.get_boot_rom_ptr());
        }
    }

    pub const VBLANK_INT: u8 = 1;
    pub const STAT_INT: u8 = 1 << 1;
    pub const TIMER_INT: u8 = 1 << 2;
    pub const SERIAL_INT: u8 = 1 << 3;
    pub const JOYPAD_INT: u8 = 1 << 4;
}

struct Unusable;
impl MemoryHandler for Unusable {
    fn read(&self, addr: u16) -> u8 { 0 }
    fn write(&mut self, addr: u16, value: u8) { }
}
