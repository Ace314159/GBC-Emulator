#[macro_use]

mod registers;
mod instructions;

use super::IO;
use registers::Registers;
use registers::Flag;

pub struct CPU {
    regs: Registers,
    IME: bool,
    is_halted: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            regs: Registers::new(),
            IME: false,
            is_halted: false,
        }
    }

    pub fn emulate(&mut self, io: &mut IO) {
        self.handle_interrupts(io);
        if !self.is_halted {
            self.emulate_instr(io);
        } else {
            io.emulate_machine_cycle();
        }
    }

    pub const INTERRUPT_VECTORS: [u16; 5] = [0x0040, 0x0048, 0x0050, 0x0058, 0x0060];

    fn handle_interrupts(&mut self, io: &mut IO) {
        if !self.is_halted && !self.IME { return }

        let interrupts = io.IF & io.IE;

        for i in 0..CPU::INTERRUPT_VECTORS.len() {
            let mask = 1 << i;
            if interrupts & mask != 0 {
                self.is_halted = false;
                if self.IME {
                    self.handle_interrupt(io, CPU::INTERRUPT_VECTORS[i]);
                    io.IF &= !mask;
                }
            }
        }
    }

    pub fn emulate_boot_rom(&mut self, io: &mut IO) {
        while self.regs.PC < 0x100 {
            self.emulate_instr(io);
        }
    }
}
