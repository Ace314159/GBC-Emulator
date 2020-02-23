mod memory;
mod cpu;

use memory::Memory;
use cpu::CPU;

use std::fs;

pub struct GBC {
    mem: Memory,
    cpu: CPU,
}

impl GBC {
    pub fn new(rom_file: &String) -> Self {
        let mut rom = fs::read(rom_file).unwrap();
        let boot_rom = fs::read("DMG_ROM.bin").unwrap();
        unsafe {
            std::ptr::copy(boot_rom.as_ptr(), rom.as_mut_ptr(), boot_rom.len());
        }
        GBC {
            mem: Memory::new(rom),
            cpu: CPU::new(),
        }
    }

    pub fn emulate(&mut self) {
        self.cpu.emulateInstr(&self.mem);
    }
}
