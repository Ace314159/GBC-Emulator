#[macro_use]
mod registers;
mod instructions;

use super::MMU;
use registers::Registers;
use registers::Flag;

pub struct CPU {
    regs: Registers,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            regs: Registers::new(),
        }
    }

    pub fn emulate_instr(&mut self, mem: &mut MMU) {
        self.exec(mem);
    }
}
