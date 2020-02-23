mod memory;
mod cpu;

use memory::Memory;
use cpu::CPU;

use std::fs;
use std::rc::Rc;

pub struct GBC {
    mem: Rc<Memory>,
    cpu: CPU,
}

impl GBC {
    pub fn new(rom_file: &String) -> Self {
        let mut rom = fs::read(rom_file).unwrap();
        let boot_rom = fs::read("DMG_ROM.bin").unwrap();
        unsafe {
            std::ptr::copy(boot_rom.as_ptr(), rom.as_mut_ptr(), boot_rom.len());
        }

        let mem = Rc::new(Memory::new(rom));
        GBC {
            mem: Rc::clone(&mem),
            cpu: CPU::new(Rc::clone(&mem)),
        }
    }

    pub fn emulate(&self) {
        self.cpu.emulateInstr();
    }
}
