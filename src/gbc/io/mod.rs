extern crate sdl2;

mod header;
mod mbc;
mod apu;
mod ppu;
mod ram;
mod serial;
mod joypad;
mod timer;

use sdl2::event::Event;

use header::Header;
use mbc::MemoryBankController;
use apu::APU;
use ppu::PPU;
use ppu::CgbPPU;
use ppu::GbPPU;
use ram::WRAM;
use serial::Serial;
use joypad::Joypad;
use timer::Timer;
use ram::HRAM;

pub trait MemoryHandler {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

pub struct IO {
    // IO Devices
    mbc: Box<dyn MemoryBankController>,
    apu: APU,
    ppu: Box<dyn PPU>,
    wram: WRAM,
    serial: Serial,
    joypad: Joypad,
    timer: Timer,
    pub int_enable: u8,
    hram: HRAM,
    pub int_flags: u8,
    unusable: Unusable,

    in_cgb: bool,
    double_speed: bool,
    prepare_speed_switch: bool,
    in_gdma: bool,

    // Other
    pub sdl_ctx: sdl2::Sdl,
    pub c: u128,
    pub should_close: bool,
}

impl IO {
    pub fn new(rom: Vec<u8>) -> Self {
        let header = Header::new(&rom);
        let in_cgb = header.in_cgb();
        let sdl_ctx = sdl2::init().unwrap();

        IO {
            mbc: mbc::get_mbc(header, rom),
            ppu: if in_cgb { Box::new(CgbPPU::new(&sdl_ctx)) } else { Box::new(GbPPU::new(&sdl_ctx)) },
            apu: APU::new(&sdl_ctx),
            wram: WRAM::new(if in_cgb { 8 } else { 2 }),
            serial: Serial::new(),
            joypad: Joypad::new(),
            timer: Timer::new(),
            int_enable: 0,
            hram: HRAM::new(),
            int_flags: 0,
            unusable: Unusable {},

            in_cgb,
            double_speed: false,
            prepare_speed_switch: false,
            in_gdma: false,

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
            0xFF10 ..= 0xFF26 => self.apu.read(addr),
            0xFF40 ..= 0xFF4B => self.ppu.read(addr),
            0xFF4D => if self.in_cgb { 0x7E | (self.double_speed as u8) << 7 | self.prepare_speed_switch as u8 } else { 0xFF },
            0xFF4F => self.ppu.read_vram_bank(),
            0xFF68 ..= 0xFF6B => self.ppu.read_cgb_palettes(addr),
            0xFF70 => self.wram.read_bank(),
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
            0xFF0F => self.int_flags = value | 0xE0,
            0xFF10 ..= 0xFF26 => self.apu.write(addr, value),
            0xFF40 ..= 0xFF4B => self.ppu.write(addr, value),
            0xFF4D => self.prepare_speed_switch = value & 0x1 != 0,
            0xFF4F => self.ppu.write_vram_bank(value),
            0xFF51 ..= 0xFF55 => self.ppu.write_hdma(addr, value, self.double_speed),
            0xFF68 ..= 0xFF6B => self.ppu.write_cgb_palettes(addr, value),
            0xFF70 => self.wram.write_bank(value),
            0xFF80 ..= 0xFFFE => self.hram.write(addr, value),
            0xFFFF => self.int_enable = value,
            _ => self.unusable.write(addr, value),
        };
    }

    pub fn emulate_machine_cycle(&mut self) {
        self.c += 4;
        self.int_flags |= self.timer.emulate();
        self.oam_dma();
        self.gdma();
        self.int_flags |= self.ppu.emulate_clock();
        self.int_flags |= self.ppu.emulate_clock();
        self.int_flags |= self.ppu.emulate_clock();
        self.int_flags |= self.ppu.emulate_clock();
        self.apu.emulate_clock();
        self.mbc.emulate_clock();

        if self.c % 10000 == 0 {
            let mut keyboard_events: Vec<Event> = Vec::new();
            for event in self.sdl_ctx.event_pump().unwrap().poll_iter() {
                match event {
                    Event::Quit {..} => {
                        self.should_close = true;
                    },
                    // Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::LCtrl), .. } => { self.ppu._rendering_map = true },
                    // Event::KeyUp { keycode: Some(sdl2::keyboard::Keycode::LCtrl), .. } => { self.ppu._rendering_map = false },
                    Event::KeyDown {..} => { keyboard_events.push(event) },
                    Event::KeyUp {..} => { keyboard_events.push(event) },
                    _ => {},
                }
            }
            self.int_flags |= self.joypad.update_inputs(&keyboard_events);
        }
    }
    
    pub fn stop(&mut self) {
        if self.in_cgb && self.prepare_speed_switch {
            self.prepare_speed_switch = false;
            self.double_speed = !self.double_speed;
            self.ppu.set_double_speed(self.double_speed);
            self.apu.set_double_speed(self.double_speed);
            for _ in 0..(128 * 1024 - 75) {
                self.emulate_machine_cycle();
            }
        }
    }

    pub fn swap_boot_rom(&mut self, boot_rom: &mut Vec<u8>) {
        let boot_rom_len = boot_rom.len();
        assert_eq!(boot_rom_len, 0x900);

        let rom = self.mbc.get_boot_rom_ptr();
        let boot_rom_a = boot_rom[..0x100].as_mut_ptr() as *mut [u8; 0x100];
        let boot_rom_b = boot_rom[0x200..0x900].as_mut_ptr() as *mut [u8; 0x700];
        unsafe {
            let rom_a = (*rom)[..0x100].as_mut_ptr() as *mut [u8; 0x100];
            let rom_b = (*rom)[0x200..0x900].as_mut_ptr() as *mut [u8; 0x700];
            std::ptr::swap(rom_a, boot_rom_a);
            std::ptr::swap(rom_b, boot_rom_b);
        }
    }

    fn oam_dma(&mut self) {
        if !self.ppu.in_oam_dma() { return }
        let (should_write, oam_addr, cpu_addr)  = self.ppu.oam_dma();
        if should_write {
            self.ppu.oam_write(oam_addr, self.read(cpu_addr));
        }
    }

    fn gdma(&mut self) {
        if self.in_gdma { return }
        self.in_gdma = self.ppu.in_gdma();
        while self.in_gdma {
            let (should_write, cpu_addr, vram_addr) = self.ppu.gdma(self.double_speed);
            if should_write {
                self.write(vram_addr, self.read(cpu_addr));
                if self.double_speed {
                    self.write(vram_addr + 1, self.read(cpu_addr))
                }
            }
            self.emulate_machine_cycle();
            self.in_gdma = self.ppu.in_gdma();
        }
    }

    pub const VBLANK_INT: u8 = 1;
    pub const STAT_INT: u8 = 1 << 1;
    pub const TIMER_INT: u8 = 1 << 2;
    pub const _SERIAL_INT: u8 = 1 << 3;
    pub const JOYPAD_INT: u8 = 1 << 4;

    const GB_CLOCK_SPEED: u32 = 4194304 / 4;
    const GBC_CLOCK_SPEED: u32 = 8388608 / 4;
}

struct Unusable;
impl MemoryHandler for Unusable {
    fn read(&self, _addr: u16) -> u8 { 0xFF }
    fn write(&mut self, _addr: u16, _value: u8) { }
}
