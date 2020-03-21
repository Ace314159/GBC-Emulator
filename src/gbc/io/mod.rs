extern crate sdl2;

mod header;
mod mbc;
mod ppu;
mod ram;
mod serial;
mod joypad;
mod timer;

use sdl2::event::Event;

use header::Header;
use mbc::MemoryBankController;
use ppu::PPU;
use ram::RAM;
use serial::Serial;
use joypad::Joypad;
use timer::Timer;

pub trait MemoryHandler {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

pub struct IO {
    // IO Devices
    mbc: Box<dyn MemoryBankController>,
    ppu: PPU,
    wram: RAM,
    serial: Serial,
    joypad: Joypad,
    timer: Timer,
    pub int_enable: u8,
    hram: RAM,
    pub int_flags: u8,
    unusable: Unusable,

    // Other
    pub sdl_ctx: sdl2::Sdl,
    pub c: u128,
    pub should_close: bool,
}

impl IO {
    pub fn new(rom: Vec<u8>) -> Self {
        let header = Header::new(&rom);
        let sdl_ctx = sdl2::init().unwrap();

        IO {
            mbc: mbc::get_mbc(header.get_cartridge_type(), rom),
            ppu: PPU::new(&sdl_ctx),
            wram: RAM::new(0xC000, 0xDFFF),
            serial: Serial::new(),
            joypad: Joypad::new(),
            timer: Timer::new(),
            int_enable: 0,
            hram: RAM::new(0xFF80, 0xFFFE),
            int_flags: 0,
            unusable: Unusable {},

            sdl_ctx,
            c: 8,
            should_close: false,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000 ..= 0x7FFF => self.mbc.read(addr),
            0x8000 ..= 0x9FFF => self.ppu.read(addr),
            0xA000 ..= 0xBFFF => self.mbc.read(addr),
            0xC000 ..= 0xDFFF => self.wram.read(addr),
            0xE000 ..= 0xFDFF => self.wram.read(addr & 0xDFFF),
            0xFE00 ..= 0xFE9F => self.ppu.read(addr),
            0xFF00 => self.joypad.read(addr),
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
            0xE000 ..= 0xFDFF => self.wram.write(addr & 0xDFFF, value),
            0xFE00 ..= 0xFE9F => self.ppu.write(addr, value),
            0xFF00 => self.joypad.write(addr, value),
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
        self.c += 4;
        self.int_flags |= self.timer.emulate();
        self.oam_dma();
        self.int_flags |= self.ppu.emulate_clock();
        self.int_flags |= self.ppu.emulate_clock();
        self.int_flags |= self.ppu.emulate_clock();
        self.int_flags |= self.ppu.emulate_clock();

        if self.c % 10000 == 0 {
            let mut keyboard_events: Vec<Event> = Vec::new();
            for event in self.sdl_ctx.event_pump().unwrap().poll_iter() {
                match event {
                    Event::Quit {..} => {
                        self.should_close = true;
                    },
                    Event::KeyDown {..} => { keyboard_events.push(event) },
                    Event::KeyUp {..} => { keyboard_events.push(event) },
                    _ => {},
                }
            }
            self.int_flags |= self.joypad.update_inputs(&keyboard_events);
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

    fn oam_dma(&mut self) {
        if !self.ppu.in_oam_dma { return }
        let cpu_addr = (self.ppu.oam_dma_page as u16) << 8 | self.ppu.oam_dma_clock;
        self.ppu.oam[self.ppu.oam_dma_clock as usize] = self.read(cpu_addr);

        self.ppu.oam_dma_clock += 1;
        if self.ppu.oam_dma_clock == 160 {
            self.ppu.in_oam_dma = false;
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
    fn read(&self, addr: u16) -> u8 { 0xFF }
    fn write(&mut self, addr: u16, value: u8) { }
}
