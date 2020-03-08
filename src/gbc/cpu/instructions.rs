use super::super::MMU;
use super::CPU;

use super::Flag;


impl CPU {
    pub fn exec(&mut self, mmu: &mut MMU) {
        // Register Macros
        macro_rules! get_reg16 { ($high:ident, $low:ident) => { 
            (self.regs.$high as u16) << 8 | (self.regs.$low as u16)
        } }
        macro_rules! set_reg16 { ($high:ident, $low:ident, $value:expr) => {
            { let value = $value; self.regs.$high = (value >> 8) as u8; self.regs.$low = value as u8; }
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
            self.regs.change_flag($Z, Flag::Z);
            self.regs.change_flag($N, Flag::N);
            self.regs.change_flag($H, Flag::H);
            self.regs.change_flag($C, Flag::C);
        }}
        
        // ALU Operation Macros
        macro_rules! INC { ($reg:ident) => { {
            let result = self.regs.$reg.wrapping_add(1);
        
            self.regs.change_flag(result & 0xFF == 0, Flag::Z);
            self.regs.clear_flag(Flag::N);
            self.regs.change_flag(((self.regs.$reg ^ 1 ^ result as u8) & 0x10) != 0, Flag::H);

            self.regs.$reg = result;
        } }}
        macro_rules! DEC { ($reg:ident) => { {
            let result = self.regs.$reg.wrapping_sub(1);
        
            self.regs.change_flag(result & 0xFF == 0, Flag::Z);
            self.regs.set_flag(Flag::N);
            self.regs.change_flag(self.regs.$reg & 0x0F < 1, Flag::H);

            self.regs.$reg = result;
        } }}
        macro_rules! INC_DEC16 { ($high:ident, $low:ident, $op: ident) => {
            { let value = get_reg16!($high, $low).$op(1); set_reg16!($high, $low, value); }
        }}

        // Jump Macros
        macro_rules! conditional { ($flag:ident, $pass:expr, $fail:expr) => {
            if self.regs.get_flag(Flag::$flag) { self.regs.PC = $pass } else { self.regs.PC = $fail }
        }}
        macro_rules! n_conditional { ($flag:ident, $pass:expr, $fail:expr) => {
            if !self.regs.get_flag(Flag::$flag) { self.regs.PC = $pass } else { self.regs.PC = $fail }
        }}

        let opcode = self.read_next_byte(mmu);

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
            0xF8 => set_reg16!(H, L, self.ADD_SP(mmu)),
            0xF9 => self.regs.SP = get_reg16!(H, L),
            // POP nn
            0xC1 => set_reg16!(B, C, self.stack_pop16(mmu)),
            0xD1 => set_reg16!(D, E, self.stack_pop16(mmu)),
            0xE1 => set_reg16!(H, L, self.stack_pop16(mmu)),
            0xF1 => set_reg16!(A, F, self.stack_pop16(mmu) & 0xFFF0),
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
            0x97 => self.SUB(self.regs.A),
            0x90 => self.SUB(self.regs.B),
            0x91 => self.SUB(self.regs.C),
            0x92 => self.SUB(self.regs.D),
            0x93 => self.SUB(self.regs.E),
            0x94 => self.SUB(self.regs.H),
            0x95 => self.SUB(self.regs.L),
            0x96 => self.SUB(self.read_byte(mmu, get_reg16!(H, L))),
            0xD6 => { let operand = self.read_next_byte(mmu); self.SUB(operand); },
            // SBC A, n
            0x9F => self.SBC(self.regs.A),
            0x98 => self.SBC(self.regs.B),
            0x99 => self.SBC(self.regs.C),
            0x9A => self.SBC(self.regs.D),
            0x9B => self.SBC(self.regs.E),
            0x9C => self.SBC(self.regs.H),
            0x9D => self.SBC(self.regs.L),
            0x9E => self.SBC(self.read_byte(mmu, get_reg16!(H, L))),
            0xDE => { let operand = self.read_next_byte(mmu); self.SBC(operand); },
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
            0xBF => self.CP(self.regs.A),
            0xB8 => self.CP(self.regs.B),
            0xB9 => self.CP(self.regs.C),
            0xBA => self.CP(self.regs.D),
            0xBB => self.CP(self.regs.E),
            0xBC => self.CP(self.regs.H),
            0xBD => self.CP(self.regs.L),
            0xBE => self.CP(self.read_byte(mmu, get_reg16!(H, L))),
            0xFE => { let operand = self.read_next_byte(mmu); self.CP(operand); },
            // INC A, n
            0x3C => INC!(A),
            0x04 => INC!(B),
            0x0C => INC!(C),
            0x14 => INC!(D),
            0x1C => INC!(E),
            0x24 => INC!(H),
            0x2C => INC!(L),
            0x34 => INC_DEC16!(H, L, wrapping_add),
            // DEC A, n
            0x3D => DEC!(A),
            0x05 => DEC!(B),
            0x0D => DEC!(C),
            0x15 => DEC!(D),
            0x1D => DEC!(E),
            0x25 => DEC!(H),
            0x2D => DEC!(L),
            0x35 => INC_DEC16!(H, L, wrapping_sub),

            // 16 Bit ALU
            // ADD HL, n
            0x09 => self.ADD16(get_reg16!(B, C)),
            0x19 => self.ADD16(get_reg16!(D, E)),
            0x29 => self.ADD16(get_reg16!(H, L)),
            0x39 => self.ADD16(self.regs.SP),
            // ADD SP, n
            0xE8 => self.regs.SP = self.ADD_SP(mmu),
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
            0xCB => self.CB(mmu),
            0x27 => self.DAA(),
            0x2F => self.CPL(),
            0x3F => self.CCF(),
            0x37 => self.SCF(),
            0x00 => {}, // NOP
            0x76 => { println!("HALT"); }, // HALT
            0x10 => { println!("STOP"); }, // STOP
            0xF3 => { println!("DI"); }, // DI
            0xFB => { println!("EI"); }, // EI
            
            // Rotates
            0x07 => self.regs.A = self.RLC(self.regs.A),
            0x17 => self.regs.A = self.RL(self.regs.A),
            0x0F => self.regs.A = self.RRC(self.regs.A),
            0x1F => self.regs.A = self.RR(self.regs.A),
            
            // Jumps
            0xC3 => self.regs.PC = self.read_next_word(mmu),
            0xC2 => n_conditional!(Z, self.read_next_word(mmu), self.regs.PC.wrapping_add(2)),
            0xCA => conditional!(Z, self.read_next_word(mmu), self.regs.PC.wrapping_add(2)),
            0xD2 => n_conditional!(C, self.read_next_word(mmu), self.regs.PC.wrapping_add(2)),
            0xDA => conditional!(C, self.read_next_word(mmu), self.regs.PC.wrapping_add(2)),
            0xE9 => self.regs.PC = get_reg16!(H, L),
            0x18 => self.regs.PC = self.relative(mmu),
            0x20 => n_conditional!(Z, self.relative(mmu), self.regs.PC.wrapping_add(1)),
            0x28 => conditional!(Z, self.relative(mmu), self.regs.PC.wrapping_add(1)),
            0x30 => n_conditional!(C, self.relative(mmu), self.regs.PC.wrapping_add(1)),
            0x38 => conditional!(C, self.relative(mmu), self.regs.PC.wrapping_add(1)),

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
            0xD9 => { self.regs.PC = self.RET(mmu); /* TODO: Enable Interrupts */ },

            _ => panic!("Unoffical Opcode {:X}", opcode),
        };
    }

    fn CB(&mut self, mmu: &mut MMU) {
        // Register Macros
        macro_rules! get_reg16 { ($high:ident, $low:ident) => { 
            (self.regs.$high as u16) << 8 | (self.regs.$low as u16)
        } }
        macro_rules! set_reg16 { ($high:ident, $low:ident, $value:expr) => {
            { let value = $value; self.regs.$high = (value >> 8) as u8; self.regs.$low = value as u8; }
        } }

        // Addressing Mode Macros
        macro_rules! read_ind { ($addr:expr) => {
            { let addr: u16 = $addr; self.read_byte(mmu, addr) }
        }}
        macro_rules! write_ind { ($addr:expr, $value:expr) => {
            { let addr: u16 = $addr; let value = $value; self.write_byte(mmu, addr, value); }
        }}


        let opcode = self.read_next_byte(mmu);

        match opcode {
            // Misc
            0x37 => self.regs.A = self.SWAP(self.regs.A),
            0x30 => self.regs.B = self.SWAP(self.regs.B),
            0x31 => self.regs.C = self.SWAP(self.regs.C),
            0x32 => self.regs.D = self.SWAP(self.regs.D),
            0x33 => self.regs.E = self.SWAP(self.regs.E),
            0x34 => self.regs.H = self.SWAP(self.regs.H),
            0x35 => self.regs.L = self.SWAP(self.regs.L),
            0x36 => write_ind!(get_reg16!(H, L), self.SWAP(read_ind!(get_reg16!(H, L)))),

            // Rotates
            0x07 => self.regs.A = self.RLC(self.regs.A),
            0x00 => self.regs.B = self.RLC(self.regs.B),
            0x01 => self.regs.C = self.RLC(self.regs.C),
            0x02 => self.regs.D = self.RLC(self.regs.D),
            0x03 => self.regs.E = self.RLC(self.regs.E),
            0x04 => self.regs.H = self.RLC(self.regs.H),
            0x05 => self.regs.L = self.RLC(self.regs.L),
            0x06 => write_ind!(get_reg16!(H, L), self.RLC(read_ind!(get_reg16!(H, L)))),
            0x17 => self.regs.A = self.RL(self.regs.A),
            0x10 => self.regs.B = self.RL(self.regs.B),
            0x11 => self.regs.C = self.RL(self.regs.C),
            0x12 => self.regs.D = self.RL(self.regs.D),
            0x13 => self.regs.E = self.RL(self.regs.E),
            0x14 => self.regs.H = self.RL(self.regs.H),
            0x15 => self.regs.L = self.RL(self.regs.L),
            0x16 => write_ind!(get_reg16!(H, L), self.RL(read_ind!(get_reg16!(H, L)))),
            0x0F => self.regs.A = self.RRC(self.regs.A),
            0x08 => self.regs.B = self.RRC(self.regs.B),
            0x09 => self.regs.C = self.RRC(self.regs.C),
            0x0A => self.regs.D = self.RRC(self.regs.D),
            0x0B => self.regs.E = self.RRC(self.regs.E),
            0x0C => self.regs.H = self.RRC(self.regs.H),
            0x0D => self.regs.L = self.RRC(self.regs.L),
            0x0E => write_ind!(get_reg16!(H, L), self.RRC(read_ind!(get_reg16!(H, L)))),
            0x1F => self.regs.A = self.RR(self.regs.A),
            0x18 => self.regs.B = self.RR(self.regs.B),
            0x19 => self.regs.C = self.RR(self.regs.C),
            0x1A => self.regs.D = self.RR(self.regs.D),
            0x1B => self.regs.E = self.RR(self.regs.E),
            0x1C => self.regs.H = self.RR(self.regs.H),
            0x1D => self.regs.L = self.RR(self.regs.L),
            0x1E => write_ind!(get_reg16!(H, L), self.RR(read_ind!(get_reg16!(H, L)))),
            0x27 => self.regs.A = self.SLA(self.regs.A),
            0x20 => self.regs.B = self.SLA(self.regs.B),
            0x21 => self.regs.C = self.SLA(self.regs.C),
            0x22 => self.regs.D = self.SLA(self.regs.D),
            0x23 => self.regs.E = self.SLA(self.regs.E),
            0x24 => self.regs.H = self.SLA(self.regs.H),
            0x25 => self.regs.L = self.SLA(self.regs.L),
            0x26 => write_ind!(get_reg16!(H, L), self.SLA(read_ind!(get_reg16!(H, L)))),
            0x2F => self.regs.A = self.SRA(self.regs.A),
            0x28 => self.regs.B = self.SRA(self.regs.B),
            0x29 => self.regs.C = self.SRA(self.regs.C),
            0x2A => self.regs.D = self.SRA(self.regs.D),
            0x2B => self.regs.E = self.SRA(self.regs.E),
            0x2C => self.regs.H = self.SRA(self.regs.H),
            0x2D => self.regs.L = self.SRA(self.regs.L),
            0x2E => write_ind!(get_reg16!(H, L), self.SRA(read_ind!(get_reg16!(H, L)))),
            0x3F => self.regs.A = self.SRL(self.regs.A),
            0x38 => self.regs.B = self.SRL(self.regs.B),
            0x39 => self.regs.C = self.SRL(self.regs.C),
            0x3A => self.regs.D = self.SRL(self.regs.D),
            0x3B => self.regs.E = self.SRL(self.regs.E),
            0x3C => self.regs.H = self.SRL(self.regs.H),
            0x3D => self.regs.L = self.SRL(self.regs.L),
            0x3E => write_ind!(get_reg16!(H, L), self.SRL(read_ind!(get_reg16!(H, L)))),
            
            // Bit Operations
            0x47 => self.BIT(self.regs.A, 0),
            0x40 => self.BIT(self.regs.B, 0),
            0x41 => self.BIT(self.regs.C, 0),
            0x42 => self.BIT(self.regs.D, 0),
            0x43 => self.BIT(self.regs.E, 0),
            0x44 => self.BIT(self.regs.H, 0),
            0x45 => self.BIT(self.regs.L, 0),
            0x46 => self.BIT(read_ind!(get_reg16!(H, L)), 0),
            0x4F => self.BIT(self.regs.A, 1),
            0x48 => self.BIT(self.regs.B, 1),
            0x49 => self.BIT(self.regs.C, 1),
            0x4A => self.BIT(self.regs.D, 1),
            0x4B => self.BIT(self.regs.E, 1),
            0x4C => self.BIT(self.regs.H, 1),
            0x4D => self.BIT(self.regs.L, 1),
            0x4E => self.BIT(read_ind!(get_reg16!(H, L)), 1),
            0x57 => self.BIT(self.regs.A, 2),
            0x50 => self.BIT(self.regs.B, 2),
            0x51 => self.BIT(self.regs.C, 2),
            0x52 => self.BIT(self.regs.D, 2),
            0x53 => self.BIT(self.regs.E, 2),
            0x54 => self.BIT(self.regs.H, 2),
            0x55 => self.BIT(self.regs.L, 2),
            0x56 => self.BIT(read_ind!(get_reg16!(H, L)), 2),
            0x5F => self.BIT(self.regs.A, 3),
            0x58 => self.BIT(self.regs.B, 3),
            0x59 => self.BIT(self.regs.C, 3),
            0x5A => self.BIT(self.regs.D, 3),
            0x5B => self.BIT(self.regs.E, 3),
            0x5C => self.BIT(self.regs.H, 3),
            0x5D => self.BIT(self.regs.L, 3),
            0x5E => self.BIT(read_ind!(get_reg16!(H, L)), 3),
            0x67 => self.BIT(self.regs.A, 4),
            0x60 => self.BIT(self.regs.B, 4),
            0x61 => self.BIT(self.regs.C, 4),
            0x62 => self.BIT(self.regs.D, 4),
            0x63 => self.BIT(self.regs.E, 4),
            0x64 => self.BIT(self.regs.H, 4),
            0x65 => self.BIT(self.regs.L, 4),
            0x66 => self.BIT(read_ind!(get_reg16!(H, L)), 4),
            0x6F => self.BIT(self.regs.A, 5),
            0x68 => self.BIT(self.regs.B, 5),
            0x69 => self.BIT(self.regs.C, 5),
            0x6A => self.BIT(self.regs.D, 5),
            0x6B => self.BIT(self.regs.E, 5),
            0x6C => self.BIT(self.regs.H, 5),
            0x6D => self.BIT(self.regs.L, 5),
            0x6E => self.BIT(read_ind!(get_reg16!(H, L)), 5),
            0x77 => self.BIT(self.regs.A, 6),
            0x70 => self.BIT(self.regs.B, 6),
            0x71 => self.BIT(self.regs.C, 6),
            0x72 => self.BIT(self.regs.D, 6),
            0x73 => self.BIT(self.regs.E, 6),
            0x74 => self.BIT(self.regs.H, 6),
            0x75 => self.BIT(self.regs.L, 6),
            0x76 => self.BIT(read_ind!(get_reg16!(H, L)), 6),
            0x7F => self.BIT(self.regs.A, 7),
            0x78 => self.BIT(self.regs.B, 7),
            0x79 => self.BIT(self.regs.C, 7),
            0x7A => self.BIT(self.regs.D, 7),
            0x7B => self.BIT(self.regs.E, 7),
            0x7C => self.BIT(self.regs.H, 7),
            0x7D => self.BIT(self.regs.L, 7),
            0x7E => self.BIT(read_ind!(get_reg16!(H, L)), 7),
            0x87 => self.regs.A = self.RES(self.regs.A, 0),
            0x80 => self.regs.B = self.RES(self.regs.B, 0),
            0x81 => self.regs.C = self.RES(self.regs.C, 0),
            0x82 => self.regs.D = self.RES(self.regs.D, 0),
            0x83 => self.regs.E = self.RES(self.regs.E, 0),
            0x84 => self.regs.H = self.RES(self.regs.H, 0),
            0x85 => self.regs.L = self.RES(self.regs.L, 0),
            0x86 => write_ind!(get_reg16!(H, L), self.RES(read_ind!(get_reg16!(H, L)), 0)),
            0x8F => self.regs.A = self.RES(self.regs.A, 1),
            0x88 => self.regs.B = self.RES(self.regs.B, 1),
            0x89 => self.regs.C = self.RES(self.regs.C, 1),
            0x8A => self.regs.D = self.RES(self.regs.D, 1),
            0x8B => self.regs.E = self.RES(self.regs.E, 1),
            0x8C => self.regs.H = self.RES(self.regs.H, 1),
            0x8D => self.regs.L = self.RES(self.regs.L, 1),
            0x8E => write_ind!(get_reg16!(H, L), self.RES(read_ind!(get_reg16!(H, L)), 1)),
            0x97 => self.regs.A = self.RES(self.regs.A, 2),
            0x90 => self.regs.B = self.RES(self.regs.B, 2),
            0x91 => self.regs.C = self.RES(self.regs.C, 2),
            0x92 => self.regs.D = self.RES(self.regs.D, 2),
            0x93 => self.regs.E = self.RES(self.regs.E, 2),
            0x94 => self.regs.H = self.RES(self.regs.H, 2),
            0x95 => self.regs.L = self.RES(self.regs.L, 2),
            0x96 => write_ind!(get_reg16!(H, L), self.RES(read_ind!(get_reg16!(H, L)), 2)),
            0x9F => self.regs.A = self.RES(self.regs.A, 3),
            0x98 => self.regs.B = self.RES(self.regs.B, 3),
            0x99 => self.regs.C = self.RES(self.regs.C, 3),
            0x9A => self.regs.D = self.RES(self.regs.D, 3),
            0x9B => self.regs.E = self.RES(self.regs.E, 3),
            0x9C => self.regs.H = self.RES(self.regs.H, 3),
            0x9D => self.regs.L = self.RES(self.regs.L, 3),
            0x9E => write_ind!(get_reg16!(H, L), self.RES(read_ind!(get_reg16!(H, L)), 3)),
            0xA7 => self.regs.A = self.RES(self.regs.A, 4),
            0xA0 => self.regs.B = self.RES(self.regs.B, 4),
            0xA1 => self.regs.C = self.RES(self.regs.C, 4),
            0xA2 => self.regs.D = self.RES(self.regs.D, 4),
            0xA3 => self.regs.E = self.RES(self.regs.E, 4),
            0xA4 => self.regs.H = self.RES(self.regs.H, 4),
            0xA5 => self.regs.L = self.RES(self.regs.L, 4),
            0xA6 => write_ind!(get_reg16!(H, L), self.RES(read_ind!(get_reg16!(H, L)), 4)),
            0xAF => self.regs.A = self.RES(self.regs.A, 5),
            0xA8 => self.regs.B = self.RES(self.regs.B, 5),
            0xA9 => self.regs.C = self.RES(self.regs.C, 5),
            0xAA => self.regs.D = self.RES(self.regs.D, 5),
            0xAB => self.regs.E = self.RES(self.regs.E, 5),
            0xAC => self.regs.H = self.RES(self.regs.H, 5),
            0xAD => self.regs.L = self.RES(self.regs.L, 5),
            0xAE => write_ind!(get_reg16!(H, L), self.RES(read_ind!(get_reg16!(H, L)), 5)),
            0xB7 => self.regs.A = self.RES(self.regs.A, 6),
            0xB0 => self.regs.B = self.RES(self.regs.B, 6),
            0xB1 => self.regs.C = self.RES(self.regs.C, 6),
            0xB2 => self.regs.D = self.RES(self.regs.D, 6),
            0xB3 => self.regs.E = self.RES(self.regs.E, 6),
            0xB4 => self.regs.H = self.RES(self.regs.H, 6),
            0xB5 => self.regs.L = self.RES(self.regs.L, 6),
            0xB6 => write_ind!(get_reg16!(H, L), self.RES(read_ind!(get_reg16!(H, L)), 6)),
            0xBF => self.regs.A = self.RES(self.regs.A, 7),
            0xB8 => self.regs.B = self.RES(self.regs.B, 7),
            0xB9 => self.regs.C = self.RES(self.regs.C, 7),
            0xBA => self.regs.D = self.RES(self.regs.D, 7),
            0xBB => self.regs.E = self.RES(self.regs.E, 7),
            0xBC => self.regs.H = self.RES(self.regs.H, 7),
            0xBD => self.regs.L = self.RES(self.regs.L, 7),
            0xBE => write_ind!(get_reg16!(H, L), self.RES(read_ind!(get_reg16!(H, L)), 7)),
            0xC7 => self.regs.A = self.SET(self.regs.A, 0),
            0xC0 => self.regs.B = self.SET(self.regs.B, 0),
            0xC1 => self.regs.C = self.SET(self.regs.C, 0),
            0xC2 => self.regs.D = self.SET(self.regs.D, 0),
            0xC3 => self.regs.E = self.SET(self.regs.E, 0),
            0xC4 => self.regs.H = self.SET(self.regs.H, 0),
            0xC5 => self.regs.L = self.SET(self.regs.L, 0),
            0xC6 => write_ind!(get_reg16!(H, L), self.SET(read_ind!(get_reg16!(H, L)), 0)),
            0xCF => self.regs.A = self.SET(self.regs.A, 1),
            0xC8 => self.regs.B = self.SET(self.regs.B, 1),
            0xC9 => self.regs.C = self.SET(self.regs.C, 1),
            0xCA => self.regs.D = self.SET(self.regs.D, 1),
            0xCB => self.regs.E = self.SET(self.regs.E, 1),
            0xCC => self.regs.H = self.SET(self.regs.H, 1),
            0xCD => self.regs.L = self.SET(self.regs.L, 1),
            0xCE => write_ind!(get_reg16!(H, L), self.SET(read_ind!(get_reg16!(H, L)), 1)),
            0xD7 => self.regs.A = self.SET(self.regs.A, 2),
            0xD0 => self.regs.B = self.SET(self.regs.B, 2),
            0xD1 => self.regs.C = self.SET(self.regs.C, 2),
            0xD2 => self.regs.D = self.SET(self.regs.D, 2),
            0xD3 => self.regs.E = self.SET(self.regs.E, 2),
            0xD4 => self.regs.H = self.SET(self.regs.H, 2),
            0xD5 => self.regs.L = self.SET(self.regs.L, 2),
            0xD6 => write_ind!(get_reg16!(H, L), self.SET(read_ind!(get_reg16!(H, L)), 2)),
            0xDF => self.regs.A = self.SET(self.regs.A, 3),
            0xD8 => self.regs.B = self.SET(self.regs.B, 3),
            0xD9 => self.regs.C = self.SET(self.regs.C, 3),
            0xDA => self.regs.D = self.SET(self.regs.D, 3),
            0xDB => self.regs.E = self.SET(self.regs.E, 3),
            0xDC => self.regs.H = self.SET(self.regs.H, 3),
            0xDD => self.regs.L = self.SET(self.regs.L, 3),
            0xDE => write_ind!(get_reg16!(H, L), self.SET(read_ind!(get_reg16!(H, L)), 3)),
            0xE7 => self.regs.A = self.SET(self.regs.A, 4),
            0xE0 => self.regs.B = self.SET(self.regs.B, 4),
            0xE1 => self.regs.C = self.SET(self.regs.C, 4),
            0xE2 => self.regs.D = self.SET(self.regs.D, 4),
            0xE3 => self.regs.E = self.SET(self.regs.E, 4),
            0xE4 => self.regs.H = self.SET(self.regs.H, 4),
            0xE5 => self.regs.L = self.SET(self.regs.L, 4),
            0xE6 => write_ind!(get_reg16!(H, L), self.SET(read_ind!(get_reg16!(H, L)), 4)),
            0xEF => self.regs.A = self.SET(self.regs.A, 5),
            0xE8 => self.regs.B = self.SET(self.regs.B, 5),
            0xE9 => self.regs.C = self.SET(self.regs.C, 5),
            0xEA => self.regs.D = self.SET(self.regs.D, 5),
            0xEB => self.regs.E = self.SET(self.regs.E, 5),
            0xEC => self.regs.H = self.SET(self.regs.H, 5),
            0xED => self.regs.L = self.SET(self.regs.L, 5),
            0xEE => write_ind!(get_reg16!(H, L), self.SET(read_ind!(get_reg16!(H, L)), 5)),
            0xF7 => self.regs.A = self.SET(self.regs.A, 6),
            0xF0 => self.regs.B = self.SET(self.regs.B, 6),
            0xF1 => self.regs.C = self.SET(self.regs.C, 6),
            0xF2 => self.regs.D = self.SET(self.regs.D, 6),
            0xF3 => self.regs.E = self.SET(self.regs.E, 6),
            0xF4 => self.regs.H = self.SET(self.regs.H, 6),
            0xF5 => self.regs.L = self.SET(self.regs.L, 6),
            0xF6 => write_ind!(get_reg16!(H, L), self.SET(read_ind!(get_reg16!(H, L)), 6)),
            0xFF => self.regs.A = self.SET(self.regs.A, 7),
            0xF8 => self.regs.B = self.SET(self.regs.B, 7),
            0xF9 => self.regs.C = self.SET(self.regs.C, 7),
            0xFA => self.regs.D = self.SET(self.regs.D, 7),
            0xFB => self.regs.E = self.SET(self.regs.E, 7),
            0xFC => self.regs.H = self.SET(self.regs.H, 7),
            0xFD => self.regs.L = self.SET(self.regs.L, 7),
            0xFE => write_ind!(get_reg16!(H, L), self.SET(read_ind!(get_reg16!(H, L)), 7)),
        }
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
        let result: u16 = self.regs.A as u16 + operand as u16;
        
        self.regs.change_flag(result & 0xFF == 0, Flag::Z);
        self.regs.clear_flag(Flag::N);
        self.regs.change_flag(((self.regs.A ^ operand ^ result as u8) & 0x10) != 0, Flag::H);
        self.regs.change_flag(result > 0xFF, Flag::C);

        self.regs.A = result as u8;
    }

    #[inline]
    fn ADC(&mut self, operand: u8) {
        let C: u8 = self.regs.get_flag(Flag::C) as u8;
        let result: u16 = self.regs.A as u16 + operand as u16 + C as u16;
        
        self.regs.change_flag(result & 0xFF == 0, Flag::Z);
        self.regs.clear_flag(Flag::N);
        self.regs.change_flag((self.regs.A ^ operand ^ result as u8) & 0x10 != 0,Flag::H);
        self.regs.change_flag(result > 0xFF, Flag::C);

        self.regs.A = result as u8;
    }

    #[inline]
    fn SUB(&mut self, operand: u8) {
        let result = self.regs.A.wrapping_sub(operand);
        
        self.regs.change_flag(result == 0, Flag::Z);
        self.regs.set_flag(Flag::N);
        self.regs.change_flag(self.regs.A & 0x0F < operand & 0x0F, Flag::H);
        self.regs.change_flag(self.regs.A < operand, Flag::C);

        self.regs.A = result;
    }

    #[inline]
    fn SBC(&mut self, operand: u8) {
        let C: u8 = self.regs.get_flag(Flag::C) as u8;
        let result = self.regs.A.wrapping_sub(operand).wrapping_sub(C);
        
        self.regs.change_flag(result == 0, Flag::Z);
        self.regs.set_flag(Flag::N);
        self.regs.change_flag(self.regs.A & 0x0F < (operand & 0x0F) + C,Flag::H);
        self.regs.change_flag((self.regs.A as u16) < (operand as u16) + (C as u16), Flag::C);

        self.regs.A = result;
    }

    #[inline]
    fn CP(&mut self, operand: u8) {
        let old_A = self.regs.A;
        self.SUB(operand);
        self.regs.A = old_A;
    }

    #[inline]
    fn ADD16(&mut self, operand: u16) {
        let HL = get_reg16!(self.regs, H, L);
        let result: u32 = (HL as u32).wrapping_add(operand as u32);

        // Z Flag Not Affected
        self.regs.clear_flag(Flag::N);
        self.regs.change_flag((HL ^ operand ^ result as u16) & 0x1000 != 0, Flag::H);
        self.regs.change_flag(result > 0xFFFF, Flag::C);

        set_reg16!(self.regs, H, L)(result as u16);
    }

    #[inline]
    fn ADD_SP(&mut self, mmu: &MMU) -> u16 {
        let operand = self.read_next_byte(mmu) as i8 as u16;
        let result: u32 = (self.regs.SP as u32).wrapping_add(operand as u32);

        self.regs.clear_flags(Flag::Z as u8 | Flag::N as u8);
        self.regs.change_flag((self.regs.SP ^ operand ^ (result & 0xFFFF) as u16) & 0x10 != 0, Flag::H);
        self.regs.change_flag(((self.regs.SP ^ operand ^ result as u16) & 0x100) != 0, Flag::C);

        result as u16
    }

    // From https://forums.nesdev.com/viewtopic.php?t=15944
    #[inline]
    fn DAA(&mut self) {
        let N = self.regs.get_flag(Flag::N);
        let C = self.regs.get_flag(Flag::C);
        let H = self.regs.get_flag(Flag::H);
        if !N {
            if C || self.regs.A > 0x99 { self.regs.A = self.regs.A.wrapping_add(0x60); self.regs.set_flag(Flag::C) }
            if H || self.regs.A & 0x0F > 0x09 { self.regs.A = self.regs.A.wrapping_add(0x06); }
        } else {
            if C { self.regs.A = self.regs.A.wrapping_sub(0x60); }
            if H { self.regs.A = self.regs.A.wrapping_sub(0x06); }
        }

        self.regs.change_flag(self.regs.A == 0, Flag::Z);
        self.regs.clear_flag(Flag::H);
    }

    #[inline]
    fn CPL(&mut self) {
        self.regs.A = !self.regs.A;
        self.regs.set_flags(Flag::N as u8 | Flag::H as u8);
    }

    #[inline]
    fn CCF(&mut self) {
        self.regs.change_flag(!self.regs.get_flag(Flag::C), Flag::C);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);
    }

    #[inline]
    fn SCF(&mut self) {
        self.regs.set_flag(Flag::C);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8)
    }

    #[inline]
    fn SWAP(&self, value: u8) -> u8 {
        return (value << 4) | (value & 0xF);
    }

    #[inline]
    fn RLC(&mut self, value: u8) -> u8 {
        self.regs.change_flag(value & 0x80 != 0, Flag::C);
        let return_val = value.rotate_left(1);

        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);
        return_val
    }

    #[inline]
    fn RL(&mut self, value: u8) -> u8 {
        let old_c = self.regs.get_flag(Flag::C) as u8;
        self.regs.change_flag(value & 0x80 != 0, Flag::C);
        let return_val = (value << 1) | old_c as u8;
        
        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);
        return_val
    }

    #[inline]
    fn RRC(&mut self, value: u8) -> u8 {
        self.regs.change_flag(value & 0x1 == 1, Flag::C);
        let return_val = value.rotate_right(1);

        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);
        return_val
    }

    #[inline]
    fn RR(&mut self, value: u8) -> u8 {
        let old_c = self.regs.get_flag(Flag::C) as u8;
        self.regs.change_flag(value & 0x1 == 1, Flag::C);
        let return_val = (value >> 1) | ((old_c as u8) << 7);
        
        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);
        return_val
    }

    #[inline]
    fn SLA(&mut self, value: u8) -> u8 {
        self.regs.change_flag(value & 0x80 != 0, Flag::C);
        let return_val = value << 1;

        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);
        return_val
    }

    #[inline]
    fn SRA(&mut self, value: u8) -> u8 {
        self.regs.change_flag(value & 0x1 == 1, Flag::C);
        let return_val = (value >> 1) | (value & 0x80);

        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);
        return_val
    }

    #[inline]
    fn SRL(&mut self, value: u8) -> u8 {
        self.regs.change_flag(value & 0x1 == 1, Flag::C);
        let return_val = value >> 1;

        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);
        return_val
    }

    #[inline]
    fn BIT(&mut self, value: u8, bit: u8) {
        self.regs.change_flag(value & (1 << bit) == 0, Flag::Z);

        self.regs.clear_flag(Flag::N);
        self.regs.set_flag(Flag::H);
    }

    #[inline]
    fn SET(&self, value: u8, bit: u8) -> u8 {
        value | (1 << bit)
    }

    #[inline]
    fn RES(&self, value: u8, bit: u8) -> u8 {
        value & !(1 << bit)
    }

    #[inline]
    fn relative(&mut self, mmu: &MMU) -> u16 {
        let val = self.read_next_byte(mmu) as i8;
        self.regs.PC.wrapping_add(val as u16)
    }

    #[inline]
    fn CALL(&mut self, mmu: &mut MMU) -> u16 {
        self.stack_push16(mmu, self.regs.PC.wrapping_add(2));
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
