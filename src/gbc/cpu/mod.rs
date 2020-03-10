#[macro_use]
mod registers;
mod instructions;

use super::MMU;
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
            timer: Timer::new(),
        }
    }

    pub fn emulate(&mut self, mmu: &mut MMU) {
        self.handle_interrupts(mmu);
        if !self.is_halted || true {
            self.emulate_instr(mmu);
        }
    }

    const IF_ADDR: u16 = 0xFF0F;
    const IE_ADDR: u16 = 0xFFFF;
    const NUM_INTERRUPTS: usize = 5;
    const INTERRUPT_VECTORS: [u16; CPU::NUM_INTERRUPTS] = [0x0040, 0x0048, 0x0050, 0x0058, 0x0060];

    fn handle_interrupts(&mut self, mmu: &mut MMU) {
        if !self.is_halted && !self.IME { return }


        let interrupts = mmu.read(CPU::IF_ADDR) & mmu.read(CPU::IE_ADDR);

        for i in 0..5 {
            if interrupts & (1 << i) != 0 {
                let vector = if self.IME { CPU::INTERRUPT_VECTORS[i] } else { self.regs.PC };
                self.handle_interrupt(mmu, vector);
            }
        }
    }

    pub fn emulate_boot_rom(&mut self, mmu: &mut MMU) {
        while self.regs.PC < 0x100 {
            self.emulate_instr(mmu);
        }
    }
}
