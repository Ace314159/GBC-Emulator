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
        let mem = Rc::new(Memory::new(fs::read(rom_file).unwrap()));
        GBC {
            mem: Rc::clone(&mem),
            cpu: CPU::new(Rc::clone(&mem)),
        }
    }

    pub fn emulate(&self) {
        self.cpu.emulateInstr();
    }
}
