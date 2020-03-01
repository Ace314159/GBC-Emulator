use super::super::MMU;
use super::CPU;

use super::Flag;


impl CPU {
    pub fn exec(&mut self, mmu: &mut MMU) {
        let opcode = self.read_next_byte(mmu);

        // Register Macros
        macro_rules! get_reg16 { ($high:ident, $low:ident) => { 
            (self.regs.$high as u16) << 8 | (self.regs.$low as u16)
        } }
        macro_rules! set_reg16 { ($high:ident, $low:ident, $value:expr) => { 
            { self.regs.$high = ($value >> 8) as u8; self.regs.$low = ($value & 0xFF) as u8; }
        } }

        // Memory Macros
        macro_rules! mem_reg8 { ($addr:expr, $reg:ident) => {
            { let addr: u16 = $addr; self.write_byte(mmu, addr, self.regs.$reg); }
        }}
        macro_rules! mem_reg16 { ($addr:expr, $reg:ident) => {
            { let addr: u16 = $addr; self.write_word(mmu, addr, self.regs.$reg); }
        }}

        // Addressing Mode Macros
        macro_rules! read_ind { ($addr:expr) => {
            { let addr: u16 = $addr; self.read_byte(mmu, addr) }
        }}

        // ALU Macros
        macro_rules! a_op { ($op:tt, $operand:expr) => {
            self.regs.A = self.regs.A $op $operand;
        }}
        macro_rules! flags { ($Z:expr, $N:expr, $H:expr, $C:expr) => {
            self.regs.changeFlag($Z, Flag::Z);
            self.regs.changeFlag($N, Flag::N);
            self.regs.changeFlag($H, Flag::H);
            self.regs.changeFlag($C, Flag::C);
        }}
        
        // ALU Operation Macros
        macro_rules! CP { ($operand:expr) => { 
            { let old_a = self.regs.A; self.ADD(!$operand); self.regs.A = old_a; }
        }}
        macro_rules! INC_DEC { ($reg:ident, $op:ident) => { {
            let result = self.regs.$reg.$op(1);
        
            self.regs.changeFlag(result == 0, Flag::Z);
            self.regs.clearFlag(Flag::N);
            self.regs.changeFlag(((self.regs.$reg & 0xF).$op(1)) > 0xF, Flag::H);

            self.regs.$reg = result
        } }}
        macro_rules! INC_DEC16 { ($high:ident, $low:ident, $op: ident) => {
            { let value = get_reg16!($high, $low).$op(1); set_reg16!($high, $low, value); }
        }}

        // Rotate Macros
        macro_rules! rlc { ($reg:ident) => { {
            self.regs.changeFlag(self.regs.$reg >> 7 == 1, Flag::C);
            self.regs.$reg = (self.regs.$reg << 1) | (self.regs.$reg >> 7);

            self.regs.changeFlag(self.regs.$reg == 0, Flag::Z);
            self.regs.clearFlags(Flag::N as u8 | Flag::H as u8);
        } }}
        macro_rules! rl { ($reg:ident) => { {
            let old_c = self.regs.getFlag(Flag::C) as u8;
            self.regs.changeFlag(self.regs.$reg >> 7 == 1, Flag::C);
            self.regs.$reg = (self.regs.$reg << 1) | old_c as u8;
            
            self.regs.changeFlag(self.regs.$reg == 0, Flag::Z);
            self.regs.clearFlags(Flag::N as u8 | Flag::H as u8);
        } }}
        macro_rules! rrc { ($reg:ident) => { {
            self.regs.changeFlag(self.regs.$reg & 0x1 == 1, Flag::C);
            self.regs.$reg = (self.regs.$reg >> 1) | ((self.regs.$reg & 0x1) << 7);

            self.regs.changeFlag(self.regs.$reg == 0, Flag::Z);
            self.regs.clearFlags(Flag::N as u8 | Flag::H as u8);
        } }}
        macro_rules! rr { ($reg:ident) => { {
            let old_c = self.regs.getFlag(Flag::C) as u8;
            self.regs.changeFlag(self.regs.$reg & 0x1 == 1, Flag::C);
            self.regs.$reg = (self.regs.$reg >> 1) | ((old_c as u8) << 7);
            
            self.regs.changeFlag(self.regs.$reg == 0, Flag::Z);
            self.regs.clearFlags(Flag::N as u8 | Flag::H as u8);
        } }}

        // Jump Macros
        macro_rules! conditional { ($flag:ident, $pass:expr, $fail:expr) => {
            if self.regs.getFlag(Flag::$flag) { self.regs.PC = $pass } else { self.regs.PC = $fail }
        }}
        macro_rules! n_conditional { ($flag:ident, $pass:expr, $fail:expr) => {
            if !self.regs.getFlag(Flag::$flag) { self.regs.PC = $pass } else { self.regs.PC = $fail }
        }}

        match opcode {
            // 8 Bit Loads
            // LD nn,n
            0x06 => self.regs.B = self.read_next_byte(mmu),
            0x0E => self.regs.C = self.read_next_byte(mmu),
            0x16 => self.regs.D = self.read_next_byte(mmu),
            0x1E => self.regs.E = self.read_next_byte(mmu),
            0x26 => self.regs.H = self.read_next_byte(mmu),
            0x2E => self.regs.L = self.read_next_byte(mmu),

            // LD r1, r2
            // LD A, n
            0x7F => self.regs.A = self.regs.A,
            0x78 => self.regs.A = self.regs.B,
            0x79 => self.regs.A = self.regs.C,
            0x7A => self.regs.A = self.regs.D,
            0x7B => self.regs.A = self.regs.E,
            0x7C => self.regs.A = self.regs.H,
            0x7D => self.regs.A = self.regs.L,
            0x0A => self.regs.A = read_ind!(get_reg16!(B, C)),
            0x1A => self.regs.A = read_ind!(get_reg16!(D, E)),
            0x2A => {self.regs.A = read_ind!(get_reg16!(H, L)); INC_DEC16!(H, L, wrapping_add)},
            0x3A => {self.regs.A = read_ind!(get_reg16!(H, L)); INC_DEC16!(H, L, wrapping_sub)},
            0x7E => self.regs.A = read_ind!(get_reg16!(H, L)),
            0xFA => self.regs.A = read_ind!(self.read_next_word(mmu)),
            0x3E => self.regs.A = self.read_next_byte(mmu),
            
            0x40 => self.regs.B = self.regs.B,
            0x41 => self.regs.B = self.regs.C,
            0x42 => self.regs.B = self.regs.D,
            0x43 => self.regs.B = self.regs.E,
            0x44 => self.regs.B = self.regs.H,
            0x45 => self.regs.B = self.regs.L,
            0x46 => self.regs.B = read_ind!(get_reg16!(H, L)),
            0x48 => self.regs.C = self.regs.B,
            0x49 => self.regs.C = self.regs.C,
            0x4A => self.regs.C = self.regs.D,
            0x4B => self.regs.C = self.regs.E,
            0x4C => self.regs.C = self.regs.H,
            0x4D => self.regs.C = self.regs.L,
            0x4E => self.regs.C = read_ind!(get_reg16!(H, L)),
            0x50 => self.regs.D = self.regs.B,
            0x51 => self.regs.D = self.regs.C,
            0x52 => self.regs.D = self.regs.D,
            0x53 => self.regs.D = self.regs.E,
            0x54 => self.regs.D = self.regs.H,
            0x55 => self.regs.D = self.regs.L,
            0x56 => self.regs.D = read_ind!(get_reg16!(H, L)),
            0x58 => self.regs.E = self.regs.B,
            0x59 => self.regs.E = self.regs.C,
            0x5A => self.regs.E = self.regs.D,
            0x5B => self.regs.E = self.regs.E,
            0x5C => self.regs.E = self.regs.H,
            0x5D => self.regs.E = self.regs.L,
            0x5E => self.regs.E = read_ind!(get_reg16!(H, L)),
            0x60 => self.regs.H = self.regs.B,
            0x61 => self.regs.H = self.regs.C,
            0x62 => self.regs.H = self.regs.D,
            0x63 => self.regs.H = self.regs.E,
            0x64 => self.regs.H = self.regs.H,
            0x65 => self.regs.H = self.regs.L,
            0x66 => self.regs.H = read_ind!(get_reg16!(H, L)),
            0x68 => self.regs.L = self.regs.B,
            0x69 => self.regs.L = self.regs.C,
            0x6A => self.regs.L = self.regs.D,
            0x6B => self.regs.L = self.regs.E,
            0x6C => self.regs.L = self.regs.H,
            0x6D => self.regs.L = self.regs.L,
            0x6E => self.regs.L = read_ind!(get_reg16!(H, L)),
            0x70 => self.write_byte(mmu, get_reg16!(H, L), self.regs.B),
            0x71 => self.write_byte(mmu, get_reg16!(H, L), self.regs.C),
            0x72 => self.write_byte(mmu, get_reg16!(H, L), self.regs.D),
            0x73 => self.write_byte(mmu, get_reg16!(H, L), self.regs.E),
            0x74 => self.write_byte(mmu, get_reg16!(H, L), self.regs.H),
            0x75 => self.write_byte(mmu, get_reg16!(H, L), self.regs.L),
            0x36 => { let value = self.read_next_byte(mmu); self.write_byte(mmu, get_reg16!(H, L), value); },
            // LD n, A
            0x47 => self.regs.B = self.regs.A,
            0x4F => self.regs.C = self.regs.A,
            0x57 => self.regs.D = self.regs.A,
            0x5F => self.regs.E = self.regs.A,
            0x67 => self.regs.H = self.regs.A,
            0x6F => self.regs.L = self.regs.A,
            0x02 => self.write_byte(mmu, get_reg16!(B, C), self.regs.A),
            0x12 => self.write_byte(mmu, get_reg16!(D, E), self.regs.A),
            0x22 => {self.write_byte(mmu, get_reg16!(H, L), self.regs.A); INC_DEC16!(H, L, wrapping_add);},
            0x32 => {self.write_byte(mmu, get_reg16!(H, L), self.regs.A); INC_DEC16!(H, L, wrapping_sub);},
            0x77 => self.write_byte(mmu, get_reg16!(H, L), self.regs.A),
            0xEA => mem_reg8!(self.read_next_word(mmu), A),

            // "Zero" Page at Page 0xFF
            0xE0 => mem_reg8!(0xFF00 | (self.read_next_byte(mmu) as u16), A),
            0xE2 => self.write_byte(mmu, 0xFF00 | (self.regs.C as u16), self.regs.A),
            0xF0 => self.regs.A = read_ind!(0xFF00 | (self.read_next_byte(mmu) as u16)),
            0xF2 => self.regs.A = read_ind!(0xFF00 | (self.regs.C as u16)),

            // 16 Bit Loads
            // LD n, nn
            0x01 => set_reg16!(B, C, self.read_next_word(mmu)),
            0x11 => set_reg16!(D, E, self.read_next_word(mmu)),
            0x21 => set_reg16!(H, L, self.read_next_word(mmu)),
            0x31 => self.regs.SP = self.read_next_word(mmu),

            // Stack
            0x08 => mem_reg16!(self.read_next_word(mmu), SP),
            0xF8 => set_reg16!(H, L, self.regs.SP.wrapping_add(self.read_next_byte(mmu) as u16)),
            0xF9 => self.regs.SP = get_reg16!(H, L),
            // POP nn
            0xC1 => set_reg16!(B, C, self.stack_pop16(mmu)),
            0xD1 => set_reg16!(D, E, self.stack_pop16(mmu)),
            0xE1 => set_reg16!(H, L, self.stack_pop16(mmu)),
            0xF1 => set_reg16!(A, F, self.stack_pop16(mmu)),
            // PUSH nn
            0xC5 => self.stack_push16(mmu, get_reg16!(B, C)),
            0xD5 => self.stack_push16(mmu, get_reg16!(D, E)),
            0xE5 => self.stack_push16(mmu, get_reg16!(H, L)),
            0xF5 => self.stack_push16(mmu, get_reg16!(A, F)),

            // 8 Bit ALU
            // ADD A, n
            0x87 => self.ADD(self.regs.A),
            0x80 => self.ADD(self.regs.B),
            0x81 => self.ADD(self.regs.C),
            0x82 => self.ADD(self.regs.D),
            0x83 => self.ADD(self.regs.E),
            0x84 => self.ADD(self.regs.H),
            0x85 => self.ADD(self.regs.L),
            0x86 => self.ADD(self.read_byte(mmu, get_reg16!(H, L))),
            0xC6 => { let operand = self.read_next_byte(mmu); self.ADD(operand); },
            // ADC A, n
            0x8F => self.ADC(self.regs.A),
            0x88 => self.ADC(self.regs.B),
            0x89 => self.ADC(self.regs.C),
            0x8A => self.ADC(self.regs.D),
            0x8B => self.ADC(self.regs.E),
            0x8C => self.ADC(self.regs.H),
            0x8D => self.ADC(self.regs.L),
            0x8E => self.ADC(self.read_byte(mmu, get_reg16!(H, L))),
            0xCE => { let operand = self.read_next_byte(mmu); self.ADC(operand); },
            // SUB A, n
            0x97 => self.ADD(!self.regs.A),
            0x90 => self.ADD(!self.regs.B),
            0x91 => self.ADD(!self.regs.C),
            0x92 => self.ADD(!self.regs.D),
            0x93 => self.ADD(!self.regs.E),
            0x94 => self.ADD(!self.regs.H),
            0x95 => self.ADD(!self.regs.L),
            0x96 => self.ADD(!self.read_byte(mmu, get_reg16!(H, L))),
            0xD6 => { let operand = self.read_next_byte(mmu); self.ADD(!operand); },
            // SBC A, n
            0x9F => self.ADC(!self.regs.A),
            0x98 => self.ADC(!self.regs.B),
            0x99 => self.ADC(!self.regs.C),
            0x9A => self.ADC(!self.regs.D),
            0x9B => self.ADC(!self.regs.E),
            0x9C => self.ADC(!self.regs.H),
            0x9D => self.ADC(!self.regs.L),
            0x9E => self.ADC(!self.read_byte(mmu, get_reg16!(H, L))),
            // AND A, n
            0xA7 => { a_op!(&, self.regs.A); flags!(self.regs.A == 0, false, true, false); }
            0xA0 => { a_op!(&, self.regs.B); flags!(self.regs.A == 0, false, true, false); }
            0xA1 => { a_op!(&, self.regs.C); flags!(self.regs.A == 0, false, true, false); }
            0xA2 => { a_op!(&, self.regs.D); flags!(self.regs.A == 0, false, true, false); }
            0xA3 => { a_op!(&, self.regs.E); flags!(self.regs.A == 0, false, true, false); }
            0xA4 => { a_op!(&, self.regs.H); flags!(self.regs.A == 0, false, true, false); }
            0xA5 => { a_op!(&, self.regs.L); flags!(self.regs.A == 0, false, true, false); }
            0xA6 => { a_op!(&, read_ind!(get_reg16!(H, L))); flags!(self.regs.A == 0, false, true, false); }
            0xE6 => { a_op!(&, self.read_next_byte(mmu)); flags!(self.regs.A == 0, false, true, false); }
            // OR A, n
            0xB7 => { a_op!(|, self.regs.A); flags!(self.regs.A == 0, false, false, false); }
            0xB0 => { a_op!(|, self.regs.B); flags!(self.regs.A == 0, false, false, false); }
            0xB1 => { a_op!(|, self.regs.C); flags!(self.regs.A == 0, false, false, false); }
            0xB2 => { a_op!(|, self.regs.D); flags!(self.regs.A == 0, false, false, false); }
            0xB3 => { a_op!(|, self.regs.E); flags!(self.regs.A == 0, false, false, false); }
            0xB4 => { a_op!(|, self.regs.H); flags!(self.regs.A == 0, false, false, false); }
            0xB5 => { a_op!(|, self.regs.L); flags!(self.regs.A == 0, false, false, false); }
            0xB6 => { a_op!(|, read_ind!(get_reg16!(H, L))); flags!(self.regs.A == 0, false, false, false); }
            0xF6 => { a_op!(|, self.read_next_byte(mmu)); flags!(self.regs.A == 0, false, false, false); }
            // XOR A, n
            0xAF => { a_op!(^, self.regs.A); flags!(self.regs.A == 0, false, false, false); }
            0xA8 => { a_op!(^, self.regs.B); flags!(self.regs.A == 0, false, false, false); }
            0xA9 => { a_op!(^, self.regs.C); flags!(self.regs.A == 0, false, false, false); }
            0xAA => { a_op!(^, self.regs.D); flags!(self.regs.A == 0, false, false, false); }
            0xAB => { a_op!(^, self.regs.E); flags!(self.regs.A == 0, false, false, false); }
            0xAC => { a_op!(^, self.regs.H); flags!(self.regs.A == 0, false, false, false); }
            0xAD => { a_op!(^, self.regs.L); flags!(self.regs.A == 0, false, false, false); }
            0xAE => { a_op!(^, read_ind!(get_reg16!(H, L))); flags!(self.regs.A == 0, false, false, false); }
            0xEE => { a_op!(^, self.read_next_byte(mmu)); flags!(self.regs.A == 0, false, false, false); }
            // CP A, n
            0xBF => CP!(self.regs.A),
            0xB8 => CP!(self.regs.B),
            0xB9 => CP!(self.regs.C),
            0xBA => CP!(self.regs.D),
            0xBB => CP!(self.regs.E),
            0xBC => CP!(self.regs.H),
            0xBD => CP!(self.regs.L),
            0xBE => CP!(self.read_byte(mmu, get_reg16!(H, L))),
            0xFE => { let operand = self.read_next_byte(mmu); CP!(!operand); },
            // INC A, n
            0x3C => INC_DEC!(A, wrapping_add),
            0x04 => INC_DEC!(B, wrapping_add),
            0x0C => INC_DEC!(C, wrapping_add),
            0x14 => INC_DEC!(D, wrapping_add),
            0x1C => INC_DEC!(E, wrapping_add),
            0x24 => INC_DEC!(H, wrapping_add),
            0x2C => INC_DEC!(L, wrapping_add),
            0x34 => INC_DEC16!(H, L, wrapping_add),
            // DEC A, n
            0x3D => INC_DEC!(A, wrapping_sub),
            0x05 => INC_DEC!(B, wrapping_sub),
            0x0D => INC_DEC!(C, wrapping_sub),
            0x15 => INC_DEC!(D, wrapping_sub),
            0x1D => INC_DEC!(E, wrapping_sub),
            0x25 => INC_DEC!(H, wrapping_sub),
            0x2D => INC_DEC!(L, wrapping_sub),
            0x35 => INC_DEC16!(H, L, wrapping_sub),

            // 16 Bit ALU
            // ADD HL, n
            0x09 => self.ADD16(get_reg16!(B, C)),
            0x19 => self.ADD16(get_reg16!(D, E)),
            0x29 => self.ADD16(get_reg16!(H, L)),
            0x39 => self.ADD16(self.regs.SP),
            // ADD SP, n
            0xE8 => self.ADD_SP(mmu),
            // INC nn
            0x03 => INC_DEC16!(B, C, wrapping_add),
            0x13 => INC_DEC16!(D, E, wrapping_add),
            0x23 => INC_DEC16!(H, L, wrapping_add),
            0x33 => self.regs.SP = self.regs.SP.wrapping_add(1),
            // DEC nn
            0x0B => INC_DEC16!(B, C, wrapping_sub),
            0x1B => INC_DEC16!(D, E, wrapping_sub),
            0x2B => INC_DEC16!(H, L, wrapping_sub),
            0x3B => self.regs.SP = self.regs.SP.wrapping_sub(1),

            // Misc
            0xCB => self.CB(),
            0x27 => self.DAA(),
            0x2F => self.CPL(),
            0x3F => self.CCF(),
            0x37 => self.SCF(),
            0x00 => {}, // NOP
            0x76 => { println!("HALT"); }, // HALT
            0x10 => { println!("STOP"); }, // stack_pop16
            0xF3 => { println!("DI"); }, // DI
            0xFB => { println!("EI"); }, // EI
            
            // Rotates
            0x07 => rlc!(A),
            0x17 => rl!(A),
            0x0F => rrc!(A),
            0x1F => rr!(A),
            
            // Jumps
            0xC3 => self.regs.PC = self.read_next_word(mmu),
            0xC2 => n_conditional!(Z, self.read_next_word(mmu), self.regs.PC.wrapping_add(2)),
            0xCA => conditional!(Z, self.read_next_word(mmu), self.regs.PC.wrapping_add(2)),
            0xD2 => n_conditional!(C, self.read_next_word(mmu), self.regs.PC.wrapping_add(2)),
            0xDA => conditional!(C, self.read_next_word(mmu), self.regs.PC.wrapping_add(2)),
            0xE9 => self.regs.PC = get_reg16!(H, L),
            0x18 => self.regs.PC = self.regs.PC.wrapping_add(self.read_next_byte(mmu) as u16),
            0x20 => n_conditional!(Z, self.regs.PC.wrapping_add(self.read_next_byte(mmu) as u16), self.regs.PC.wrapping_add(1)),
            0x28 => conditional!(Z, self.regs.PC.wrapping_add(self.read_next_byte(mmu) as u16), self.regs.PC.wrapping_add(1)),
            0x30 => n_conditional!(C, self.regs.PC.wrapping_add(self.read_next_byte(mmu) as u16), self.regs.PC.wrapping_add(1)),
            0x38 => conditional!(C, self.regs.PC.wrapping_add(self.read_next_byte(mmu) as u16), self.regs.PC.wrapping_add(1)),

            // Calls
            0xCD => self.regs.PC = self.CALL(mmu),
            0xC4 => n_conditional!(Z, self.CALL(mmu), self.regs.PC.wrapping_add(2)),
            0xCC => conditional!(Z, self.CALL(mmu), self.regs.PC.wrapping_add(2)),
            0xD4 => n_conditional!(C, self.CALL(mmu), self.regs.PC.wrapping_add(2)),
            0xDC => conditional!(C, self.CALL(mmu), self.regs.PC.wrapping_add(2)),

            // Restarts
            0xC7 => self.RST(mmu, 0x00),
            0xCF => self.RST(mmu, 0x08),
            0xD7 => self.RST(mmu, 0x10),
            0xDF => self.RST(mmu, 0x18),
            0xE7 => self.RST(mmu, 0x20),
            0xEF => self.RST(mmu, 0x28),
            0xF7 => self.RST(mmu, 0x30),
            0xFF => self.RST(mmu, 0x38),
            
            // Returns
            0xC9 => self.regs.PC = self.RET(mmu),
            0xC0 => n_conditional!(Z, self.RET(mmu), self.regs.PC),
            0xC8 => conditional!(Z, self.RET(mmu), self.regs.PC),
            0xD0 => n_conditional!(C, self.RET(mmu), self.regs.PC),
            0xD8 => conditional!(C, self.RET(mmu), self.regs.PC),

            _ => println!("Unoffical Opcode {}", opcode),
        };
    }

    fn CB(&self) {

    }

    // Util
    // Memory
    fn read_byte(&self, mmu: &MMU, addr: u16) -> u8 {
        // 1 Machine Cycle
        return mmu.read(addr);
    }

    fn read_word(&self, mmu: &MMU, addr: u16) -> u16 {
        return self.read_byte(mmu, addr) as u16 | (self.read_byte(mmu, addr.wrapping_add(1)) as u16) << 8;
    }

    fn write_byte(&self, mmu: &mut MMU, addr: u16, value: u8) {
        // 1 Machine Cycle
        mmu.write(addr, value);
    }

    fn write_word(&self, mmu: &mut MMU, addr: u16, value: u16) {
        let bytes = value.to_be_bytes();
        self.write_byte(mmu, addr, bytes[1]);
        self.write_byte(mmu, addr.wrapping_add(1), bytes[0]);
    }

    fn read_next_byte(&mut self, mmu: &MMU) -> u8 {
        let value = self.read_byte(mmu, self.regs.PC);
        self.regs.PC = self.regs.PC.wrapping_add(1);
        return value;
    }

    fn read_next_word(&mut self, mmu: &MMU) -> u16 {
        return self.read_next_byte(mmu) as u16 | (self.read_next_byte(mmu) as u16) << 8;
    }

    // Stack
    fn stack_push8(&mut self, mmu: &mut MMU, value: u8) {
        self.regs.SP = self.regs.SP.wrapping_sub(1);
        self.write_byte(mmu, self.regs.SP, value);
    }

    fn stack_push16(&mut self, mmu: &mut MMU, value: u16) {
        let bytes= value.to_be_bytes();
        self.stack_push8(mmu, bytes[0]);
        self.stack_push8(mmu, bytes[1]);
    }

    fn stack_pop8(&mut self, mmu: &MMU) -> u8 {
        let value = self.read_byte(mmu, self.regs.SP);
        self.regs.SP = self.regs.SP.wrapping_add(1);
        return value;
    }

    fn stack_pop16(&mut self, mmu: &MMU) -> u16 {
        return self.stack_pop8(mmu) as u16 | (self.stack_pop8(mmu) as u16) << 8;
    }

    // Operations
    #[inline]
    fn ADD(&mut self, operand: u8) {
        let result: u16 = (self.regs.A as u16).wrapping_add(operand as u16);
        
        self.regs.changeFlag(result == 0, Flag::Z);
        self.regs.clearFlag(Flag::N);
        self.regs.changeFlag(((self.regs.A & 0xF).wrapping_add(operand & 0xF)) > 0xF, Flag::H);
        self.regs.changeFlag(result > 0xFF, Flag::C);

        self.regs.A = (result & 0xFF) as u8;
    }

    #[inline]
    fn ADC(&mut self, operand: u8) {
        let C: u8 = self.regs.F >> 4;
        let result: u16 = (self.regs.A as u16).wrapping_add(operand as u16).wrapping_add(C as u16);
        
        self.regs.changeFlag(result == 0, Flag::Z);
        self.regs.clearFlag(Flag::N);
        self.regs.changeFlag(((self.regs.A & 0xF).wrapping_add(operand & 0xF)
                                        .wrapping_add(C)) > 0xF,Flag::H);
        self.regs.changeFlag(result > 0xFF, Flag::C);

        self.regs.A = (result & 0xFF) as u8;
    }

    #[inline]
    fn ADD16(&mut self, operand: u16) {
        let HL = get_reg16!(self.regs, H, L);
        let result: u32 = (HL as u32).wrapping_add(operand as u32);

        // Z Flag Not Affected
        self.regs.clearFlag(Flag::N);
        self.regs.changeFlag((HL & 0xFFF).wrapping_add(operand & 0xFFF) > 0xFFF, Flag::H);
        self.regs.changeFlag(result > 0xFFFF, Flag::C);

        set_reg16!(self.regs, H, L)((result & 0xFFFF) as u16);
    }

    #[inline]
    fn ADD_SP(&mut self, mmu: &MMU) {
        let operand: u16 = self.read_next_byte(mmu) as u16;
        let result: u32 = (self.regs.SP as u32).wrapping_add(operand as u32);

        self.regs.clearFlags(Flag::Z as u8 | Flag::N as u8);
        self.regs.changeFlag((self.regs.SP & 0xFFF).wrapping_add(operand & 0xFFF) > 0xFFF, Flag::H);
        self.regs.changeFlag(result > 0xFFFF, Flag::C);

        self.regs.SP = (result & 0xFFFF) as u16;
    }

    // From https://forums.nesdev.com/viewtopic.php?t=15944
    #[inline]
    fn DAA(&mut self) {
        let N = self.regs.getFlag(Flag::N);
        let C = self.regs.getFlag(Flag::C);
        let H = self.regs.getFlag(Flag::H);
        if N {
            if C || self.regs.A > 0x99 { self.regs.A = self.regs.A.wrapping_add(0x60); self.regs.setFlag(Flag::C) }
            if H || self.regs.A & 0x0F > 0x09 { self.regs.A = self.regs.A.wrapping_add(0x06); }
        } else {
            if C { self.regs.A = self.regs.A.wrapping_sub(0x60); }
            if H { self.regs.A = self.regs.A.wrapping_sub(0x06); }
        }

        self.regs.changeFlag(self.regs.A == 0, Flag::Z);
        self.regs.clearFlag(Flag::H);
    }

    #[inline]
    fn CPL(&mut self) {
        self.regs.A = !self.regs.A;
        self.regs.setFlags(Flag::N as u8 | Flag::H as u8);
    }

    #[inline]
    fn CCF(&mut self) {
        self.regs.changeFlag(!self.regs.getFlag(Flag::C), Flag::C);
        self.regs.clearFlags(Flag::N as u8 | Flag::H as u8);
    }

    #[inline]
    fn SCF(&mut self) {
        self.regs.setFlag(Flag::C);
        self.regs.clearFlags(Flag::N as u8 | Flag::H as u8)
    }

    #[inline]
    fn CALL(&mut self, mmu: &mut MMU) -> u16 {
        self.stack_push16(mmu, self.regs.PC.wrapping_add(1));
        self.read_next_word(mmu)
    }

    #[inline]
    fn RST(&mut self, mmu: &mut MMU, addr: u16) {
        self.stack_push16(mmu, self.regs.PC.wrapping_add(1));
        self.regs.PC = addr;
    }

    #[inline]
    fn RET(&mut self, mmu: &mut MMU) -> u16 {
        self.stack_pop16(mmu)
    }
}
