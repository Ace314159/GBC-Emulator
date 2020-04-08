#[macro_use]

mod registers;
mod instructions;

use super::IO;
use registers::Registers;
use registers::Flag;

pub struct CPU {
    regs: Registers,
    prev_ime: bool,
    ime: bool,
    is_halted: bool,
    p: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            regs: Registers::new(),
            prev_ime: false,
            ime: false,
            is_halted: false,
            p: false,
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
        if !self.is_halted && !self.prev_ime { self.prev_ime = self.ime; return }

        let mut interrupts = io.int_flags & io.int_enable;

        let cached_prev_ime = self.prev_ime; // For the Special Case
        for i in 0..CPU::INTERRUPT_VECTORS.len() {
            let mask = 1 << i;
            if interrupts & mask != 0 {
                self.is_halted = false;
                if cached_prev_ime {
                    self.handle_interrupt(io, CPU::INTERRUPT_VECTORS[i]);
                    interrupts = io.int_flags & io.int_enable;
                    if self.regs.sp == 0xFFFE && interrupts & mask == 0 {
                        // Special Case when IE is written during upper byte push
                        self.regs.pc = 0;
                    } else { io.int_flags &= !mask; }
                }
            }
        }
        self.prev_ime = self.ime;
    }

    pub fn emulate_boot_rom(&mut self, io: &mut IO) {
        while !io.should_close && self.regs.pc != 0x100 {
            self.emulate_instr(io);
        }
    }
}
