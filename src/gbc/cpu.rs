use super::Memory;


const FLAG_Z: u8 = 0x80;
const FLAG_N: u8 = 0x40;
const FLAG_H: u8 = 0x20;
const FLAG_C: u8 = 0x10;

pub struct CPU {
    A: u8,
    B: u8,
    C: u8,
    D: u8,
    E: u8,
    H: u8,
    L: u8,
    F: u8,
    SP: u16,
    PC: u16,

}

impl CPU {
    pub fn new(mem: Rc<Memory>) -> Self {
        CPU { 
            A: 0,
            B: 0,
            C: 0,
            D: 0,
            E: 0,
            H: 0,
            L: 0,
            F: 0,
            SP: 0,
            PC: 0,
            mem,
        }
    }

    pub fn emulateInstr(&mut self, mem: &Memory) {

    }

    fn setReg(&self, reg1: &mut u8, reg2: &mut u8, value: u16) {
        *reg1 = (value >> 8) as u8;
        *reg2 = (value & 0xFF) as u8;
    }
    
    fn getReg(&self, reg1: u8, reg2: u8) -> u16 {
        return ((reg1 as u16) << 8) | (reg2 as u16);
    }

    fn setFlags(&mut self, mask: u8) {
        self.F |= mask;
    }

    fn clearFlags(&mut self, mask: u8) {
        self.F &= !mask;
    }
}
