#[macro_use]
mod registers;
mod instructions;

use super::MMU;
use registers::Registers;
use registers::Flag;

pub struct CPU {
    regs: Registers,
    IME: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            regs: Registers::new(),
            IME: false,
        }
    }

    pub fn emulate_instr(&mut self, mmu: &mut MMU) {
        self.exec(mmu);
    }

    pub fn emulate_boot_rom(&mut self, mmu: &mut MMU) {
        while self.regs.PC < 0x100 {
            self.exec(mmu);
        }
    }
}
