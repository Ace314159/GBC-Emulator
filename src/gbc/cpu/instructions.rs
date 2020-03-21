use super::IO;
use super::CPU;

use super::Flag;


impl CPU {
    pub fn emulate_instr(&mut self, io: &mut IO) {
        let opcode = self.read_next_byte(io);
        if self.p {
            println!("{:04X}:  {:02X}      A:{:02X} F:{:02X} B:{:02X} \
            C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} Cy:{} LY:{}",
            self.regs.pc - 1, opcode, self.regs.a, self.regs.f, self.regs.b, self.regs.c, self.regs.d,
            self.regs.e, self.regs.h, self.regs.l, self.regs.sp, io.c - 1, io.read(0xFF44));
        }
        self.decode_exec(io, opcode);
    }

    pub fn decode_exec(&mut self, io: &mut IO, opcode: u8) {
        // Register Macros
        macro_rules! get_reg16 { ($high:ident, $low:ident) => { 
            (self.regs.$high as u16) << 8 | (self.regs.$low as u16)
        } }
        macro_rules! set_reg16 { ($high:ident, $low:ident, $value:expr) => {
            { let value = $value; self.regs.$high = (value >> 8) as u8; self.regs.$low = value as u8; }
        } }

        // Memory Macros
        macro_rules! mem_reg8 { ($addr:expr, $reg:ident) => {
            { let addr: u16 = $addr; self.write_byte(io, addr, self.regs.$reg); }
        }}
        macro_rules! mem_reg16 { ($addr:expr, $reg:ident) => {
            { let addr: u16 = $addr; self.write_word(io, addr, self.regs.$reg); }
        }}

        // Addressing Mode Macros
        macro_rules! read_ind { ($addr:expr) => {
            { let addr: u16 = $addr; self.read_byte(io, addr) }
        }}
        macro_rules! write_ind { ($addr:expr, $value:expr) => {
            { let addr: u16 = $addr; let value = $value; self.write_byte(io, addr, value); }
        }}

        // ALU Macros
        macro_rules! a_op { ($op:tt, $operand:expr) => {
            self.regs.a = self.regs.a $op $operand;
        }}
        macro_rules! flags { ($Z:expr, $N:expr, $H:expr, $C:expr) => {
            self.regs.change_flag($Z, Flag::Z);
            self.regs.change_flag($N, Flag::N);
            self.regs.change_flag($H, Flag::H);
            self.regs.change_flag($C, Flag::C);
        }}
        
        // ALU Operation Macros
        macro_rules! INC_DEC16 { ($high:ident, $low:ident, $op: ident) => {
            set_reg16!($high, $low, get_reg16!($high, $low).$op(1))
        }}

        // Jump Macros
        macro_rules! conditional { ($flag:ident, $pass:block, $fail:expr) => {
            if self.regs.get_flag(Flag::$flag) { self.regs.pc = $pass; } else { $fail; }
        }}
        macro_rules! n_conditional { ($flag:ident, $pass:block, $fail:expr) => {
            if !self.regs.get_flag(Flag::$flag) { self.regs.pc = $pass; } else { $fail; }
        }}


        match opcode {
            // 8 Bit Loads
            // LD nn,n
            0x06 => self.regs.b = self.read_next_byte(io),
            0x0E => self.regs.c = self.read_next_byte(io),
            0x16 => self.regs.d = self.read_next_byte(io),
            0x1E => self.regs.e = self.read_next_byte(io),
            0x26 => self.regs.h = self.read_next_byte(io),
            0x2E => self.regs.l = self.read_next_byte(io),

            // LD r1, r2
            // LD A, n
            0x7F => self.regs.a = self.regs.a,
            0x78 => self.regs.a = self.regs.b,
            0x79 => self.regs.a = self.regs.c,
            0x7A => self.regs.a = self.regs.d,
            0x7B => self.regs.a = self.regs.e,
            0x7C => self.regs.a = self.regs.h,
            0x7D => self.regs.a = self.regs.l,
            0x0A => self.regs.a = read_ind!(get_reg16!(b, c)),
            0x1A => self.regs.a = read_ind!(get_reg16!(d, e)),
            0x2A => {self.regs.a = read_ind!(get_reg16!(h, l)); INC_DEC16!(h, l, wrapping_add)},
            0x3A => {self.regs.a = read_ind!(get_reg16!(h, l)); INC_DEC16!(h, l, wrapping_sub)},
            0x7E => self.regs.a = read_ind!(get_reg16!(h, l)),
            0xFA => self.regs.a = read_ind!(self.read_next_word(io)),
            0x3E => self.regs.a = self.read_next_byte(io),
            
            0x40 => self.regs.b = self.regs.b,
            0x41 => self.regs.b = self.regs.c,
            0x42 => self.regs.b = self.regs.d,
            0x43 => self.regs.b = self.regs.e,
            0x44 => self.regs.b = self.regs.h,
            0x45 => self.regs.b = self.regs.l,
            0x46 => self.regs.b = read_ind!(get_reg16!(h, l)),
            0x48 => self.regs.c = self.regs.b,
            0x49 => self.regs.c = self.regs.c,
            0x4A => self.regs.c = self.regs.d,
            0x4B => self.regs.c = self.regs.e,
            0x4C => self.regs.c = self.regs.h,
            0x4D => self.regs.c = self.regs.l,
            0x4E => self.regs.c = read_ind!(get_reg16!(h, l)),
            0x50 => self.regs.d = self.regs.b,
            0x51 => self.regs.d = self.regs.c,
            0x52 => self.regs.d = self.regs.d,
            0x53 => self.regs.d = self.regs.e,
            0x54 => self.regs.d = self.regs.h,
            0x55 => self.regs.d = self.regs.l,
            0x56 => self.regs.d = read_ind!(get_reg16!(h, l)),
            0x58 => self.regs.e = self.regs.b,
            0x59 => self.regs.e = self.regs.c,
            0x5A => self.regs.e = self.regs.d,
            0x5B => self.regs.e = self.regs.e,
            0x5C => self.regs.e = self.regs.h,
            0x5D => self.regs.e = self.regs.l,
            0x5E => self.regs.e = read_ind!(get_reg16!(h, l)),
            0x60 => self.regs.h = self.regs.b,
            0x61 => self.regs.h = self.regs.c,
            0x62 => self.regs.h = self.regs.d,
            0x63 => self.regs.h = self.regs.e,
            0x64 => self.regs.h = self.regs.h,
            0x65 => self.regs.h = self.regs.l,
            0x66 => self.regs.h = read_ind!(get_reg16!(h, l)),
            0x68 => self.regs.l = self.regs.b,
            0x69 => self.regs.l = self.regs.c,
            0x6A => self.regs.l = self.regs.d,
            0x6B => self.regs.l = self.regs.e,
            0x6C => self.regs.l = self.regs.h,
            0x6D => self.regs.l = self.regs.l,
            0x6E => self.regs.l = read_ind!(get_reg16!(h, l)),
            0x70 => self.write_byte(io, get_reg16!(h, l), self.regs.b),
            0x71 => self.write_byte(io, get_reg16!(h, l), self.regs.c),
            0x72 => self.write_byte(io, get_reg16!(h, l), self.regs.d),
            0x73 => self.write_byte(io, get_reg16!(h, l), self.regs.e),
            0x74 => self.write_byte(io, get_reg16!(h, l), self.regs.h),
            0x75 => self.write_byte(io, get_reg16!(h, l), self.regs.l),
            0x36 => { let value = self.read_next_byte(io); self.write_byte(io, get_reg16!(h, l), value); },
            // LD n, A
            0x47 => self.regs.b = self.regs.a,
            0x4F => self.regs.c = self.regs.a,
            0x57 => self.regs.d = self.regs.a,
            0x5F => self.regs.e = self.regs.a,
            0x67 => self.regs.h = self.regs.a,
            0x6F => self.regs.l = self.regs.a,
            0x02 => self.write_byte(io, get_reg16!(b, c), self.regs.a),
            0x12 => self.write_byte(io, get_reg16!(d, e), self.regs.a),
            0x22 => {self.write_byte(io, get_reg16!(h, l), self.regs.a); INC_DEC16!(h, l, wrapping_add);},
            0x32 => {self.write_byte(io, get_reg16!(h, l), self.regs.a); INC_DEC16!(h, l, wrapping_sub);},
            0x77 => self.write_byte(io, get_reg16!(h, l), self.regs.a),
            0xEA => mem_reg8!(self.read_next_word(io), a),

            // "Zero" Page at Page 0xFF
            0xE0 => mem_reg8!(0xFF00 | (self.read_next_byte(io) as u16), a),
            0xE2 => self.write_byte(io, 0xFF00 | (self.regs.c as u16), self.regs.a),
            0xF0 => self.regs.a = read_ind!(0xFF00 | (self.read_next_byte(io) as u16)),
            0xF2 => self.regs.a = read_ind!(0xFF00 | (self.regs.c as u16)),

            // 16 Bit Loads
            // LD n, nn
            0x01 => set_reg16!(b, c, self.read_next_word(io)),
            0x11 => set_reg16!(d, e, self.read_next_word(io)),
            0x21 => set_reg16!(h, l, self.read_next_word(io)),
            0x31 => self.regs.sp = self.read_next_word(io),

            // Stack
            0x08 => mem_reg16!(self.read_next_word(io), sp),
            0xF8 => { set_reg16!(h, l, self.add_sp(io)); self.internal_cycle(io); },
            0xF9 => { self.regs.sp = get_reg16!(h, l); self.internal_cycle(io); },
            // POP nn
            0xC1 => set_reg16!(b, c, self.stack_pop16(io)),
            0xD1 => set_reg16!(d, e, self.stack_pop16(io)),
            0xE1 => set_reg16!(h, l, self.stack_pop16(io)),
            0xF1 => set_reg16!(a, f, self.stack_pop16(io) & 0xFFF0),
            // PUSH nn
            0xC5 => { self.internal_cycle(io); self.stack_push16(io, get_reg16!(b, c)); },
            0xD5 => { self.internal_cycle(io); self.stack_push16(io, get_reg16!(d, e)); },
            0xE5 => { self.internal_cycle(io); self.stack_push16(io, get_reg16!(h, l)); },
            0xF5 => { self.internal_cycle(io); self.stack_push16(io, get_reg16!(a, f)); },

            // 8 Bit ALU
            // ADD A, n
            0x87 => self.add(self.regs.a),
            0x80 => self.add(self.regs.b),
            0x81 => self.add(self.regs.c),
            0x82 => self.add(self.regs.d),
            0x83 => self.add(self.regs.e),
            0x84 => self.add(self.regs.h),
            0x85 => self.add(self.regs.l),
            0x86 => self.add(self.read_byte(io, get_reg16!(h, l))),
            0xC6 => { let operand = self.read_next_byte(io); self.add(operand); },
            // ADC A, n
            0x8F => self.adc(self.regs.a),
            0x88 => self.adc(self.regs.b),
            0x89 => self.adc(self.regs.c),
            0x8A => self.adc(self.regs.d),
            0x8B => self.adc(self.regs.e),
            0x8C => self.adc(self.regs.h),
            0x8D => self.adc(self.regs.l),
            0x8E => self.adc(self.read_byte(io, get_reg16!(h, l))),
            0xCE => { let operand = self.read_next_byte(io); self.adc(operand); },
            // SUB A, n
            0x97 => self.sub(self.regs.a),
            0x90 => self.sub(self.regs.b),
            0x91 => self.sub(self.regs.c),
            0x92 => self.sub(self.regs.d),
            0x93 => self.sub(self.regs.e),
            0x94 => self.sub(self.regs.h),
            0x95 => self.sub(self.regs.l),
            0x96 => self.sub(self.read_byte(io, get_reg16!(h, l))),
            0xD6 => { let operand = self.read_next_byte(io); self.sub(operand); },
            // SBC A, n
            0x9F => self.sbc(self.regs.a),
            0x98 => self.sbc(self.regs.b),
            0x99 => self.sbc(self.regs.c),
            0x9A => self.sbc(self.regs.d),
            0x9B => self.sbc(self.regs.e),
            0x9C => self.sbc(self.regs.h),
            0x9D => self.sbc(self.regs.l),
            0x9E => self.sbc(self.read_byte(io, get_reg16!(h, l))),
            0xDE => { let operand = self.read_next_byte(io); self.sbc(operand); },
            // AND A, n
            0xA7 => { a_op!(&, self.regs.a); flags!(self.regs.a == 0, false, true, false); }
            0xA0 => { a_op!(&, self.regs.b); flags!(self.regs.a == 0, false, true, false); }
            0xA1 => { a_op!(&, self.regs.c); flags!(self.regs.a == 0, false, true, false); }
            0xA2 => { a_op!(&, self.regs.d); flags!(self.regs.a == 0, false, true, false); }
            0xA3 => { a_op!(&, self.regs.e); flags!(self.regs.a == 0, false, true, false); }
            0xA4 => { a_op!(&, self.regs.h); flags!(self.regs.a == 0, false, true, false); }
            0xA5 => { a_op!(&, self.regs.l); flags!(self.regs.a == 0, false, true, false); }
            0xA6 => { a_op!(&, read_ind!(get_reg16!(h, l))); flags!(self.regs.a == 0, false, true, false); }
            0xE6 => { a_op!(&, self.read_next_byte(io)); flags!(self.regs.a == 0, false, true, false); }
            // OR A, n
            0xB7 => { a_op!(|, self.regs.a); flags!(self.regs.a == 0, false, false, false); }
            0xB0 => { a_op!(|, self.regs.b); flags!(self.regs.a == 0, false, false, false); }
            0xB1 => { a_op!(|, self.regs.c); flags!(self.regs.a == 0, false, false, false); }
            0xB2 => { a_op!(|, self.regs.d); flags!(self.regs.a == 0, false, false, false); }
            0xB3 => { a_op!(|, self.regs.e); flags!(self.regs.a == 0, false, false, false); }
            0xB4 => { a_op!(|, self.regs.h); flags!(self.regs.a == 0, false, false, false); }
            0xB5 => { a_op!(|, self.regs.l); flags!(self.regs.a == 0, false, false, false); }
            0xB6 => { a_op!(|, read_ind!(get_reg16!(h, l))); flags!(self.regs.a == 0, false, false, false); }
            0xF6 => { a_op!(|, self.read_next_byte(io)); flags!(self.regs.a == 0, false, false, false); }
            // XOR A, n
            0xAF => { a_op!(^, self.regs.a); flags!(self.regs.a == 0, false, false, false); }
            0xA8 => { a_op!(^, self.regs.b); flags!(self.regs.a == 0, false, false, false); }
            0xA9 => { a_op!(^, self.regs.c); flags!(self.regs.a == 0, false, false, false); }
            0xAA => { a_op!(^, self.regs.d); flags!(self.regs.a == 0, false, false, false); }
            0xAB => { a_op!(^, self.regs.e); flags!(self.regs.a == 0, false, false, false); }
            0xAC => { a_op!(^, self.regs.h); flags!(self.regs.a == 0, false, false, false); }
            0xAD => { a_op!(^, self.regs.l); flags!(self.regs.a == 0, false, false, false); }
            0xAE => { a_op!(^, read_ind!(get_reg16!(h, l))); flags!(self.regs.a == 0, false, false, false); }
            0xEE => { a_op!(^, self.read_next_byte(io)); flags!(self.regs.a == 0, false, false, false); }
            // CP A, n
            0xBF => self.cp(self.regs.a),
            0xB8 => self.cp(self.regs.b),
            0xB9 => self.cp(self.regs.c),
            0xBA => self.cp(self.regs.d),
            0xBB => self.cp(self.regs.e),
            0xBC => self.cp(self.regs.h),
            0xBD => self.cp(self.regs.l),
            0xBE => self.cp(self.read_byte(io, get_reg16!(h, l))),
            0xFE => { let operand = self.read_next_byte(io); self.cp(operand); },
            // INC n
            0x3C => self.regs.a = self.inc(self.regs.a),
            0x04 => self.regs.b = self.inc(self.regs.b),
            0x0C => self.regs.c = self.inc(self.regs.c),
            0x14 => self.regs.d = self.inc(self.regs.d),
            0x1C => self.regs.e = self.inc(self.regs.e),
            0x24 => self.regs.h = self.inc(self.regs.h),
            0x2C => self.regs.l = self.inc(self.regs.l),
            0x34 => write_ind!(get_reg16!(h, l), self.inc(read_ind!(get_reg16!(h, l)))),
            // DEC n
            0x3D => self.regs.a = self.dec(self.regs.a),
            0x05 => self.regs.b = self.dec(self.regs.b),
            0x0D => self.regs.c = self.dec(self.regs.c),
            0x15 => self.regs.d = self.dec(self.regs.d),
            0x1D => self.regs.e = self.dec(self.regs.e),
            0x25 => self.regs.h = self.dec(self.regs.h),
            0x2D => self.regs.l = self.dec(self.regs.l),
            0x35 => write_ind!(get_reg16!(h, l), self.dec(read_ind!(get_reg16!(h, l)))),

            // 16 Bit ALU
            // ADD HL, n
            0x09 => { self.add16(get_reg16!(b, c)); self.internal_cycle(io); },
            0x19 => { self.add16(get_reg16!(d, e)); self.internal_cycle(io); },
            0x29 => { self.add16(get_reg16!(h, l)); self.internal_cycle(io); },
            0x39 => { self.add16(self.regs.sp); self.internal_cycle(io); },
            // ADD SP, n
            0xE8 => { self.regs.sp = self.add_sp(io); self.internal_cycle(io); self.internal_cycle(io); },
            // INC nn
            0x03 => { INC_DEC16!(b, c, wrapping_add); self.internal_cycle(io); },
            0x13 => { INC_DEC16!(d, e, wrapping_add); self.internal_cycle(io); },
            0x23 => { INC_DEC16!(h, l, wrapping_add); self.internal_cycle(io); },
            0x33 => { self.regs.sp = self.regs.sp.wrapping_add(1); self.internal_cycle(io); },
            // DEC nn
            0x0B => { INC_DEC16!(b, c, wrapping_sub); self.internal_cycle(io); },
            0x1B => { INC_DEC16!(d, e, wrapping_sub); self.internal_cycle(io); },
            0x2B => { INC_DEC16!(h, l, wrapping_sub); self.internal_cycle(io); },
            0x3B => { self.regs.sp = self.regs.sp.wrapping_sub(1); self.internal_cycle(io); },

            // Misc
            0xCB => self.prefix(io),
            0x27 => self.daa(),
            0x2F => self.cpl(),
            0x3F => self.ccf(),
            0x37 => self.scf(),
            0x00 => {}, // NOP
            0x76 => self.halt(io),
            0x10 => { /*println!("STOP Called")*/ }, // STOP
            0xF3 => { self.prev_ime = false; self.ime = false; },
            0xFB => { self.ime = true; /* Interrupt not handled until next instruction */ },
            
            // Rotates
            0x07 => { self.regs.a = self.rlc(self.regs.a); self.regs.clear_flag(Flag::Z); },
            0x17 => { self.regs.a = self.rl(self.regs.a); self.regs.clear_flag(Flag::Z); },
            0x0F => { self.regs.a = self.rrc(self.regs.a); self.regs.clear_flag(Flag::Z); },
            0x1F => { self.regs.a = self.rr(self.regs.a); self.regs.clear_flag(Flag::Z); },
            
            // Jumps
            // JP nn
            0xC3 => { self.regs.pc = self.read_next_word(io); self.internal_cycle(io); },
            //JP cc, nn
            0xC2 => n_conditional!(Z, { let a = self.read_next_word(io); self.internal_cycle(io); a}, self.read_next_word(io)),
            0xCA => conditional!(Z, { let a = self.read_next_word(io); self.internal_cycle(io); a}, self.read_next_word(io)),
            0xD2 => n_conditional!(C, { let a = self.read_next_word(io); self.internal_cycle(io); a}, self.read_next_word(io)),
            0xDA => conditional!(C, { let a = self.read_next_word(io); self.internal_cycle(io); a}, self.read_next_word(io)),
            // JP (HL)
            0xE9 => self.regs.pc = get_reg16!(h, l),
            // JR n
            0x18 => { self.regs.pc = self.relative(io); self.internal_cycle(io); },
            // JR cc, n
            0x20 => n_conditional!(Z, { let a = self.relative(io); self.internal_cycle(io); a }, self.read_next_byte(io)),
            0x28 => conditional!(Z, { let a = self.relative(io); self.internal_cycle(io); a }, self.read_next_byte(io)),
            0x30 => n_conditional!(C, { let a = self.relative(io); self.internal_cycle(io); a }, self.read_next_byte(io)),
            0x38 => conditional!(C, { let a = self.relative(io); self.internal_cycle(io); a }, self.read_next_byte(io)),

            // Calls
            0xCD => self.regs.pc = self.call(io),
            0xC4 => n_conditional!(Z, { self.call(io) }, self.read_next_word(io)),
            0xCC => conditional!(Z, { self.call(io) }, self.read_next_word(io)),
            0xD4 => n_conditional!(C, { self.call(io) }, self.read_next_word(io)),
            0xDC => conditional!(C, { self.call(io) }, self.read_next_word(io)),

            // Restarts
            0xC7 => self.rst(io, 0x00),
            0xCF => self.rst(io, 0x08),
            0xD7 => self.rst(io, 0x10),
            0xDF => self.rst(io, 0x18),
            0xE7 => self.rst(io, 0x20),
            0xEF => self.rst(io, 0x28),
            0xF7 => self.rst(io, 0x30),
            0xFF => self.rst(io, 0x38),
            
            // Returns
            0xC9 => self.regs.pc = self.ret(io),
            0xC0 => { self.internal_cycle(io); n_conditional!(Z, { self.ret(io) }, self.regs.pc); },
            0xC8 => { self.internal_cycle(io); conditional!(Z, { self.ret(io) }, self.regs.pc); },
            0xD0 => { self.internal_cycle(io); n_conditional!(C, { self.ret(io) }, self.regs.pc); },
            0xD8 => { self.internal_cycle(io); conditional!(C, { self.ret(io) }, self.regs.pc); },
            0xD9 => { self.regs.pc = self.ret(io); self.prev_ime = true; self.ime = true; },

            _ => panic!("Unoffical Opcode {:X}", opcode),
        };
    }

    fn prefix(&mut self, io: &mut IO) {
        // Register Macros
        macro_rules! get_reg16 { ($high:ident, $low:ident) => { 
            (self.regs.$high as u16) << 8 | (self.regs.$low as u16)
        } }

        // Addressing Mode Macros
        macro_rules! read_ind { ($addr:expr) => {
            { let addr: u16 = $addr; self.read_byte(io, addr) }
        }}
        macro_rules! write_ind { ($addr:expr, $value:expr) => {
            { let addr: u16 = $addr; let value = $value; self.write_byte(io, addr, value); }
        }}


        let opcode = self.read_next_byte(io);

        match opcode {
            // Misc
            0x37 => self.regs.a = self.swap(self.regs.a),
            0x30 => self.regs.b = self.swap(self.regs.b),
            0x31 => self.regs.c = self.swap(self.regs.c),
            0x32 => self.regs.d = self.swap(self.regs.d),
            0x33 => self.regs.e = self.swap(self.regs.e),
            0x34 => self.regs.h = self.swap(self.regs.h),
            0x35 => self.regs.l = self.swap(self.regs.l),
            0x36 => write_ind!(get_reg16!(h, l), self.swap(read_ind!(get_reg16!(h, l)))),

            // Rotates
            0x07 => self.regs.a = self.rlc(self.regs.a),
            0x00 => self.regs.b = self.rlc(self.regs.b),
            0x01 => self.regs.c = self.rlc(self.regs.c),
            0x02 => self.regs.d = self.rlc(self.regs.d),
            0x03 => self.regs.e = self.rlc(self.regs.e),
            0x04 => self.regs.h = self.rlc(self.regs.h),
            0x05 => self.regs.l = self.rlc(self.regs.l),
            0x06 => write_ind!(get_reg16!(h, l), self.rlc(read_ind!(get_reg16!(h, l)))),
            0x17 => self.regs.a = self.rl(self.regs.a),
            0x10 => self.regs.b = self.rl(self.regs.b),
            0x11 => self.regs.c = self.rl(self.regs.c),
            0x12 => self.regs.d = self.rl(self.regs.d),
            0x13 => self.regs.e = self.rl(self.regs.e),
            0x14 => self.regs.h = self.rl(self.regs.h),
            0x15 => self.regs.l = self.rl(self.regs.l),
            0x16 => write_ind!(get_reg16!(h, l), self.rl(read_ind!(get_reg16!(h, l)))),
            0x0F => self.regs.a = self.rrc(self.regs.a),
            0x08 => self.regs.b = self.rrc(self.regs.b),
            0x09 => self.regs.c = self.rrc(self.regs.c),
            0x0A => self.regs.d = self.rrc(self.regs.d),
            0x0B => self.regs.e = self.rrc(self.regs.e),
            0x0C => self.regs.h = self.rrc(self.regs.h),
            0x0D => self.regs.l = self.rrc(self.regs.l),
            0x0E => write_ind!(get_reg16!(h, l), self.rrc(read_ind!(get_reg16!(h, l)))),
            0x1F => self.regs.a = self.rr(self.regs.a),
            0x18 => self.regs.b = self.rr(self.regs.b),
            0x19 => self.regs.c = self.rr(self.regs.c),
            0x1A => self.regs.d = self.rr(self.regs.d),
            0x1B => self.regs.e = self.rr(self.regs.e),
            0x1C => self.regs.h = self.rr(self.regs.h),
            0x1D => self.regs.l = self.rr(self.regs.l),
            0x1E => write_ind!(get_reg16!(h, l), self.rr(read_ind!(get_reg16!(h, l)))),
            0x27 => self.regs.a = self.sla(self.regs.a),
            0x20 => self.regs.b = self.sla(self.regs.b),
            0x21 => self.regs.c = self.sla(self.regs.c),
            0x22 => self.regs.d = self.sla(self.regs.d),
            0x23 => self.regs.e = self.sla(self.regs.e),
            0x24 => self.regs.h = self.sla(self.regs.h),
            0x25 => self.regs.l = self.sla(self.regs.l),
            0x26 => write_ind!(get_reg16!(h, l), self.sla(read_ind!(get_reg16!(h, l)))),
            0x2F => self.regs.a = self.sra(self.regs.a),
            0x28 => self.regs.b = self.sra(self.regs.b),
            0x29 => self.regs.c = self.sra(self.regs.c),
            0x2A => self.regs.d = self.sra(self.regs.d),
            0x2B => self.regs.e = self.sra(self.regs.e),
            0x2C => self.regs.h = self.sra(self.regs.h),
            0x2D => self.regs.l = self.sra(self.regs.l),
            0x2E => write_ind!(get_reg16!(h, l), self.sra(read_ind!(get_reg16!(h, l)))),
            0x3F => self.regs.a = self.srl(self.regs.a),
            0x38 => self.regs.b = self.srl(self.regs.b),
            0x39 => self.regs.c = self.srl(self.regs.c),
            0x3A => self.regs.d = self.srl(self.regs.d),
            0x3B => self.regs.e = self.srl(self.regs.e),
            0x3C => self.regs.h = self.srl(self.regs.h),
            0x3D => self.regs.l = self.srl(self.regs.l),
            0x3E => write_ind!(get_reg16!(h, l), self.srl(read_ind!(get_reg16!(h, l)))),
            
            // Bit Operations
            0x47 => self.bit(self.regs.a, 0),
            0x40 => self.bit(self.regs.b, 0),
            0x41 => self.bit(self.regs.c, 0),
            0x42 => self.bit(self.regs.d, 0),
            0x43 => self.bit(self.regs.e, 0),
            0x44 => self.bit(self.regs.h, 0),
            0x45 => self.bit(self.regs.l, 0),
            0x46 => self.bit(read_ind!(get_reg16!(h, l)), 0),
            0x4F => self.bit(self.regs.a, 1),
            0x48 => self.bit(self.regs.b, 1),
            0x49 => self.bit(self.regs.c, 1),
            0x4A => self.bit(self.regs.d, 1),
            0x4B => self.bit(self.regs.e, 1),
            0x4C => self.bit(self.regs.h, 1),
            0x4D => self.bit(self.regs.l, 1),
            0x4E => self.bit(read_ind!(get_reg16!(h, l)), 1),
            0x57 => self.bit(self.regs.a, 2),
            0x50 => self.bit(self.regs.b, 2),
            0x51 => self.bit(self.regs.c, 2),
            0x52 => self.bit(self.regs.d, 2),
            0x53 => self.bit(self.regs.e, 2),
            0x54 => self.bit(self.regs.h, 2),
            0x55 => self.bit(self.regs.l, 2),
            0x56 => self.bit(read_ind!(get_reg16!(h, l)), 2),
            0x5F => self.bit(self.regs.a, 3),
            0x58 => self.bit(self.regs.b, 3),
            0x59 => self.bit(self.regs.c, 3),
            0x5A => self.bit(self.regs.d, 3),
            0x5B => self.bit(self.regs.e, 3),
            0x5C => self.bit(self.regs.h, 3),
            0x5D => self.bit(self.regs.l, 3),
            0x5E => self.bit(read_ind!(get_reg16!(h, l)), 3),
            0x67 => self.bit(self.regs.a, 4),
            0x60 => self.bit(self.regs.b, 4),
            0x61 => self.bit(self.regs.c, 4),
            0x62 => self.bit(self.regs.d, 4),
            0x63 => self.bit(self.regs.e, 4),
            0x64 => self.bit(self.regs.h, 4),
            0x65 => self.bit(self.regs.l, 4),
            0x66 => self.bit(read_ind!(get_reg16!(h, l)), 4),
            0x6F => self.bit(self.regs.a, 5),
            0x68 => self.bit(self.regs.b, 5),
            0x69 => self.bit(self.regs.c, 5),
            0x6A => self.bit(self.regs.d, 5),
            0x6B => self.bit(self.regs.e, 5),
            0x6C => self.bit(self.regs.h, 5),
            0x6D => self.bit(self.regs.l, 5),
            0x6E => self.bit(read_ind!(get_reg16!(h, l)), 5),
            0x77 => self.bit(self.regs.a, 6),
            0x70 => self.bit(self.regs.b, 6),
            0x71 => self.bit(self.regs.c, 6),
            0x72 => self.bit(self.regs.d, 6),
            0x73 => self.bit(self.regs.e, 6),
            0x74 => self.bit(self.regs.h, 6),
            0x75 => self.bit(self.regs.l, 6),
            0x76 => self.bit(read_ind!(get_reg16!(h, l)), 6),
            0x7F => self.bit(self.regs.a, 7),
            0x78 => self.bit(self.regs.b, 7),
            0x79 => self.bit(self.regs.c, 7),
            0x7A => self.bit(self.regs.d, 7),
            0x7B => self.bit(self.regs.e, 7),
            0x7C => self.bit(self.regs.h, 7),
            0x7D => self.bit(self.regs.l, 7),
            0x7E => self.bit(read_ind!(get_reg16!(h, l)), 7),
            0x87 => self.regs.a = self.res(self.regs.a, 0),
            0x80 => self.regs.b = self.res(self.regs.b, 0),
            0x81 => self.regs.c = self.res(self.regs.c, 0),
            0x82 => self.regs.d = self.res(self.regs.d, 0),
            0x83 => self.regs.e = self.res(self.regs.e, 0),
            0x84 => self.regs.h = self.res(self.regs.h, 0),
            0x85 => self.regs.l = self.res(self.regs.l, 0),
            0x86 => write_ind!(get_reg16!(h, l), self.res(read_ind!(get_reg16!(h, l)), 0)),
            0x8F => self.regs.a = self.res(self.regs.a, 1),
            0x88 => self.regs.b = self.res(self.regs.b, 1),
            0x89 => self.regs.c = self.res(self.regs.c, 1),
            0x8A => self.regs.d = self.res(self.regs.d, 1),
            0x8B => self.regs.e = self.res(self.regs.e, 1),
            0x8C => self.regs.h = self.res(self.regs.h, 1),
            0x8D => self.regs.l = self.res(self.regs.l, 1),
            0x8E => write_ind!(get_reg16!(h, l), self.res(read_ind!(get_reg16!(h, l)), 1)),
            0x97 => self.regs.a = self.res(self.regs.a, 2),
            0x90 => self.regs.b = self.res(self.regs.b, 2),
            0x91 => self.regs.c = self.res(self.regs.c, 2),
            0x92 => self.regs.d = self.res(self.regs.d, 2),
            0x93 => self.regs.e = self.res(self.regs.e, 2),
            0x94 => self.regs.h = self.res(self.regs.h, 2),
            0x95 => self.regs.l = self.res(self.regs.l, 2),
            0x96 => write_ind!(get_reg16!(h, l), self.res(read_ind!(get_reg16!(h, l)), 2)),
            0x9F => self.regs.a = self.res(self.regs.a, 3),
            0x98 => self.regs.b = self.res(self.regs.b, 3),
            0x99 => self.regs.c = self.res(self.regs.c, 3),
            0x9A => self.regs.d = self.res(self.regs.d, 3),
            0x9B => self.regs.e = self.res(self.regs.e, 3),
            0x9C => self.regs.h = self.res(self.regs.h, 3),
            0x9D => self.regs.l = self.res(self.regs.l, 3),
            0x9E => write_ind!(get_reg16!(h, l), self.res(read_ind!(get_reg16!(h, l)), 3)),
            0xA7 => self.regs.a = self.res(self.regs.a, 4),
            0xA0 => self.regs.b = self.res(self.regs.b, 4),
            0xA1 => self.regs.c = self.res(self.regs.c, 4),
            0xA2 => self.regs.d = self.res(self.regs.d, 4),
            0xA3 => self.regs.e = self.res(self.regs.e, 4),
            0xA4 => self.regs.h = self.res(self.regs.h, 4),
            0xA5 => self.regs.l = self.res(self.regs.l, 4),
            0xA6 => write_ind!(get_reg16!(h, l), self.res(read_ind!(get_reg16!(h, l)), 4)),
            0xAF => self.regs.a = self.res(self.regs.a, 5),
            0xA8 => self.regs.b = self.res(self.regs.b, 5),
            0xA9 => self.regs.c = self.res(self.regs.c, 5),
            0xAA => self.regs.d = self.res(self.regs.d, 5),
            0xAB => self.regs.e = self.res(self.regs.e, 5),
            0xAC => self.regs.h = self.res(self.regs.h, 5),
            0xAD => self.regs.l = self.res(self.regs.l, 5),
            0xAE => write_ind!(get_reg16!(h, l), self.res(read_ind!(get_reg16!(h, l)), 5)),
            0xB7 => self.regs.a = self.res(self.regs.a, 6),
            0xB0 => self.regs.b = self.res(self.regs.b, 6),
            0xB1 => self.regs.c = self.res(self.regs.c, 6),
            0xB2 => self.regs.d = self.res(self.regs.d, 6),
            0xB3 => self.regs.e = self.res(self.regs.e, 6),
            0xB4 => self.regs.h = self.res(self.regs.h, 6),
            0xB5 => self.regs.l = self.res(self.regs.l, 6),
            0xB6 => write_ind!(get_reg16!(h, l), self.res(read_ind!(get_reg16!(h, l)), 6)),
            0xBF => self.regs.a = self.res(self.regs.a, 7),
            0xB8 => self.regs.b = self.res(self.regs.b, 7),
            0xB9 => self.regs.c = self.res(self.regs.c, 7),
            0xBA => self.regs.d = self.res(self.regs.d, 7),
            0xBB => self.regs.e = self.res(self.regs.e, 7),
            0xBC => self.regs.h = self.res(self.regs.h, 7),
            0xBD => self.regs.l = self.res(self.regs.l, 7),
            0xBE => write_ind!(get_reg16!(h, l), self.res(read_ind!(get_reg16!(h, l)), 7)),
            0xC7 => self.regs.a = self.set(self.regs.a, 0),
            0xC0 => self.regs.b = self.set(self.regs.b, 0),
            0xC1 => self.regs.c = self.set(self.regs.c, 0),
            0xC2 => self.regs.d = self.set(self.regs.d, 0),
            0xC3 => self.regs.e = self.set(self.regs.e, 0),
            0xC4 => self.regs.h = self.set(self.regs.h, 0),
            0xC5 => self.regs.l = self.set(self.regs.l, 0),
            0xC6 => write_ind!(get_reg16!(h, l), self.set(read_ind!(get_reg16!(h, l)), 0)),
            0xCF => self.regs.a = self.set(self.regs.a, 1),
            0xC8 => self.regs.b = self.set(self.regs.b, 1),
            0xC9 => self.regs.c = self.set(self.regs.c, 1),
            0xCA => self.regs.d = self.set(self.regs.d, 1),
            0xCB => self.regs.e = self.set(self.regs.e, 1),
            0xCC => self.regs.h = self.set(self.regs.h, 1),
            0xCD => self.regs.l = self.set(self.regs.l, 1),
            0xCE => write_ind!(get_reg16!(h, l), self.set(read_ind!(get_reg16!(h, l)), 1)),
            0xD7 => self.regs.a = self.set(self.regs.a, 2),
            0xD0 => self.regs.b = self.set(self.regs.b, 2),
            0xD1 => self.regs.c = self.set(self.regs.c, 2),
            0xD2 => self.regs.d = self.set(self.regs.d, 2),
            0xD3 => self.regs.e = self.set(self.regs.e, 2),
            0xD4 => self.regs.h = self.set(self.regs.h, 2),
            0xD5 => self.regs.l = self.set(self.regs.l, 2),
            0xD6 => write_ind!(get_reg16!(h, l), self.set(read_ind!(get_reg16!(h, l)), 2)),
            0xDF => self.regs.a = self.set(self.regs.a, 3),
            0xD8 => self.regs.b = self.set(self.regs.b, 3),
            0xD9 => self.regs.c = self.set(self.regs.c, 3),
            0xDA => self.regs.d = self.set(self.regs.d, 3),
            0xDB => self.regs.e = self.set(self.regs.e, 3),
            0xDC => self.regs.h = self.set(self.regs.h, 3),
            0xDD => self.regs.l = self.set(self.regs.l, 3),
            0xDE => write_ind!(get_reg16!(h, l), self.set(read_ind!(get_reg16!(h, l)), 3)),
            0xE7 => self.regs.a = self.set(self.regs.a, 4),
            0xE0 => self.regs.b = self.set(self.regs.b, 4),
            0xE1 => self.regs.c = self.set(self.regs.c, 4),
            0xE2 => self.regs.d = self.set(self.regs.d, 4),
            0xE3 => self.regs.e = self.set(self.regs.e, 4),
            0xE4 => self.regs.h = self.set(self.regs.h, 4),
            0xE5 => self.regs.l = self.set(self.regs.l, 4),
            0xE6 => write_ind!(get_reg16!(h, l), self.set(read_ind!(get_reg16!(h, l)), 4)),
            0xEF => self.regs.a = self.set(self.regs.a, 5),
            0xE8 => self.regs.b = self.set(self.regs.b, 5),
            0xE9 => self.regs.c = self.set(self.regs.c, 5),
            0xEA => self.regs.d = self.set(self.regs.d, 5),
            0xEB => self.regs.e = self.set(self.regs.e, 5),
            0xEC => self.regs.h = self.set(self.regs.h, 5),
            0xED => self.regs.l = self.set(self.regs.l, 5),
            0xEE => write_ind!(get_reg16!(h, l), self.set(read_ind!(get_reg16!(h, l)), 5)),
            0xF7 => self.regs.a = self.set(self.regs.a, 6),
            0xF0 => self.regs.b = self.set(self.regs.b, 6),
            0xF1 => self.regs.c = self.set(self.regs.c, 6),
            0xF2 => self.regs.d = self.set(self.regs.d, 6),
            0xF3 => self.regs.e = self.set(self.regs.e, 6),
            0xF4 => self.regs.h = self.set(self.regs.h, 6),
            0xF5 => self.regs.l = self.set(self.regs.l, 6),
            0xF6 => write_ind!(get_reg16!(h, l), self.set(read_ind!(get_reg16!(h, l)), 6)),
            0xFF => self.regs.a = self.set(self.regs.a, 7),
            0xF8 => self.regs.b = self.set(self.regs.b, 7),
            0xF9 => self.regs.c = self.set(self.regs.c, 7),
            0xFA => self.regs.d = self.set(self.regs.d, 7),
            0xFB => self.regs.e = self.set(self.regs.e, 7),
            0xFC => self.regs.h = self.set(self.regs.h, 7),
            0xFD => self.regs.l = self.set(self.regs.l, 7),
            0xFE => write_ind!(get_reg16!(h, l), self.set(read_ind!(get_reg16!(h, l)), 7)),
        }
    }

    // Util
    // Memory
    fn internal_cycle(&self, io: &mut IO) {
        io.emulate_machine_cycle();
    }

    fn read_byte(&self, io: &mut IO, addr: u16) -> u8 {
        io.emulate_machine_cycle();
        io.read(addr)
    }

    fn write_byte(&self, io: &mut IO, addr: u16, value: u8) {
        io.emulate_machine_cycle();
        io.write(addr, value);
    }

    fn write_word(&self, io: &mut IO, addr: u16, value: u16) {
        let bytes = value.to_be_bytes();
        self.write_byte(io, addr, bytes[1]);
        self.write_byte(io, addr.wrapping_add(1), bytes[0]);
    }

    fn read_next_byte(&mut self, io: &mut IO) -> u8 {
        let value = self.read_byte(io, self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        value
    }

    fn read_next_word(&mut self, io: &mut IO) -> u16 {
        self.read_next_byte(io) as u16 | (self.read_next_byte(io) as u16) << 8
    }

    // Stack
    fn stack_push8(&mut self, io: &mut IO, value: u8) {
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        self.write_byte(io, self.regs.sp, value);
    }

    fn stack_push16(&mut self, io: &mut IO, value: u16) {
        let bytes= value.to_be_bytes();
        self.stack_push8(io, bytes[0]);
        self.stack_push8(io, bytes[1]);
    }

    fn stack_pop8(&mut self, io: &mut IO) -> u8 {
        let value = self.read_byte(io, self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(1);
        value
    }

    fn stack_pop16(&mut self, io: &mut IO) -> u16 {
        self.stack_pop8(io) as u16 | (self.stack_pop8(io) as u16) << 8
    }

    // Interrupts
    pub fn handle_interrupt(&mut self, io: &mut IO, vector: u16) {
        // ISR - Interrupt Service Routine
        self.prev_ime = false;
        self.ime = false;
        self.internal_cycle(io);
        self.internal_cycle(io);
        self.stack_push16(io, self.regs.pc);
        self.internal_cycle(io);
        self.regs.pc = vector;
    }

    // Operations
    #[inline]
    fn add(&mut self, operand: u8) {
        let result: u16 = self.regs.a as u16 + operand as u16;
        
        self.regs.change_flag(result & 0xFF == 0, Flag::Z);
        self.regs.clear_flag(Flag::N);
        self.regs.change_flag(((self.regs.a ^ operand ^ result as u8) & 0x10) != 0, Flag::H);
        self.regs.change_flag(result > 0xFF, Flag::C);

        self.regs.a = result as u8;
    }

    #[inline]
    fn adc(&mut self, operand: u8) {
        let c = self.regs.get_flag(Flag::C) as u8;
        let result: u16 = self.regs.a as u16 + operand as u16 + c as u16;
        
        self.regs.change_flag(result & 0xFF == 0, Flag::Z);
        self.regs.clear_flag(Flag::N);
        self.regs.change_flag((self.regs.a ^ operand ^ result as u8) & 0x10 != 0,Flag::H);
        self.regs.change_flag(result > 0xFF, Flag::C);

        self.regs.a = result as u8;
    }

    #[inline]
    fn sub(&mut self, operand: u8) {
        let result = self.regs.a.wrapping_sub(operand);
        
        self.regs.change_flag(result == 0, Flag::Z);
        self.regs.set_flag(Flag::N);
        self.regs.change_flag(self.regs.a & 0x0F < operand & 0x0F, Flag::H);
        self.regs.change_flag(self.regs.a < operand, Flag::C);

        self.regs.a = result;
    }

    #[inline]
    fn sbc(&mut self, operand: u8) {
        let c: u8 = self.regs.get_flag(Flag::C) as u8;
        let result = self.regs.a.wrapping_sub(operand).wrapping_sub(c);
        
        self.regs.change_flag(result == 0, Flag::Z);
        self.regs.set_flag(Flag::N);
        self.regs.change_flag(self.regs.a & 0x0F < (operand & 0x0F) + c,Flag::H);
        self.regs.change_flag((self.regs.a as u16) < (operand as u16) + (c as u16), Flag::C);

        self.regs.a = result;
    }

    #[inline]
    fn cp(&mut self, operand: u8) {
        let old_a = self.regs.a;
        self.sub(operand);
        self.regs.a = old_a;
    }

    #[inline]
    fn inc(&mut self, operand: u8) -> u8{
        let result = operand.wrapping_add(1);
    
        self.regs.change_flag(result & 0xFF == 0, Flag::Z);
        self.regs.clear_flag(Flag::N);
        self.regs.change_flag(((operand ^ 1 ^ result as u8) & 0x10) != 0, Flag::H);

        result
    }

    #[inline]
    fn dec(&mut self, operand: u8) -> u8 {
        let result = operand.wrapping_sub(1);
    
        self.regs.change_flag(result & 0xFF == 0, Flag::Z);
        self.regs.set_flag(Flag::N);
        self.regs.change_flag(operand & 0x0F < 1, Flag::H);

        result
    }

    #[inline]
    fn add16(&mut self, operand: u16) {
        let hl = get_reg16!(self.regs, h, l);
        let result: u32 = (hl as u32).wrapping_add(operand as u32);

        // Z Flag Not Affected
        self.regs.clear_flag(Flag::N);
        self.regs.change_flag((hl ^ operand ^ result as u16) & 0x1000 != 0, Flag::H);
        self.regs.change_flag(result > 0xFFFF, Flag::C);

        set_reg16!(self.regs, h, l)(result as u16);
    }

    #[inline]
    fn add_sp(&mut self, io: &mut IO) -> u16 {
        let operand = self.read_next_byte(io) as i8 as u16;
        let result: u32 = (self.regs.sp as u32).wrapping_add(operand as u32);

        self.regs.clear_flags(Flag::Z as u8 | Flag::N as u8);
        self.regs.change_flag((self.regs.sp ^ operand ^ (result & 0xFFFF) as u16) & 0x10 != 0, Flag::H);
        self.regs.change_flag(((self.regs.sp ^ operand ^ result as u16) & 0x100) != 0, Flag::C);

        result as u16
    }

    // From https://forums.nesdev.com/viewtopic.php?t=15944
    #[inline]
    fn daa(&mut self) {
        let n = self.regs.get_flag(Flag::N);
        let c = self.regs.get_flag(Flag::C);
        let h = self.regs.get_flag(Flag::H);
        if !n {
            if c || self.regs.a > 0x99 { self.regs.a = self.regs.a.wrapping_add(0x60); self.regs.set_flag(Flag::C) }
            if h || self.regs.a & 0x0F > 0x09 { self.regs.a = self.regs.a.wrapping_add(0x06); }
        } else {
            if c { self.regs.a = self.regs.a.wrapping_sub(0x60); }
            if h { self.regs.a = self.regs.a.wrapping_sub(0x06); }
        }

        self.regs.change_flag(self.regs.a == 0, Flag::Z);
        self.regs.clear_flag(Flag::H);
    }

    #[inline]
    fn cpl(&mut self) {
        self.regs.a = !self.regs.a;
        self.regs.set_flags(Flag::N as u8 | Flag::H as u8);
    }

    #[inline]
    fn ccf(&mut self) {
        self.regs.change_flag(!self.regs.get_flag(Flag::C), Flag::C);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);
    }

    #[inline]
    fn scf(&mut self) {
        self.regs.set_flag(Flag::C);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);
    }

    #[inline]
    fn halt(&mut self, io: &mut IO) {
        if self.ime {
            self.is_halted = true;
        } else {
            if io.int_enable & io.int_flags & 0x1F == 0 {
                self.is_halted = true;
            } else {
                // HALT bug where PC is not incremented when fetching opcode
                let opcode = self.read_byte(io, self.regs.pc);
                self.decode_exec(io, opcode);
            }
        }
    }

    #[inline]
    fn swap(&mut self, value: u8) -> u8 {
        let return_val = (value << 4) | (value >> 4);

        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8 | Flag::C as u8);

        return_val
    }

    #[inline]
    fn rlc(&mut self, value: u8) -> u8 {
        self.regs.change_flag(value & 0x80 != 0, Flag::C);
        let return_val = value.rotate_left(1);

        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);

        return_val
    }

    #[inline]
    fn rl(&mut self, value: u8) -> u8 {
        let old_c = self.regs.get_flag(Flag::C) as u8;
        self.regs.change_flag(value & 0x80 != 0, Flag::C);
        let return_val = (value << 1) | old_c as u8;
        
        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);

        return_val
    }

    #[inline]
    fn rrc(&mut self, value: u8) -> u8 {
        self.regs.change_flag(value & 0x1 == 1, Flag::C);
        let return_val = value.rotate_right(1);

        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);

        return_val
    }

    #[inline]
    fn rr(&mut self, value: u8) -> u8 {
        let old_c = self.regs.get_flag(Flag::C) as u8;
        self.regs.change_flag(value & 0x1 == 1, Flag::C);
        let return_val = (value >> 1) | ((old_c as u8) << 7);
        
        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);

        return_val
    }

    #[inline]
    fn sla(&mut self, value: u8) -> u8 {
        self.regs.change_flag(value & 0x80 != 0, Flag::C);
        let return_val = value << 1;

        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);

        return_val
    }

    #[inline]
    fn sra(&mut self, value: u8) -> u8 {
        self.regs.change_flag(value & 0x1 == 1, Flag::C);
        let return_val = (value >> 1) | (value & 0x80);

        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);

        return_val
    }

    #[inline]
    fn srl(&mut self, value: u8) -> u8 {
        self.regs.change_flag(value & 0x1 == 1, Flag::C);
        let return_val = value >> 1;

        self.regs.change_flag(return_val == 0, Flag::Z);
        self.regs.clear_flags(Flag::N as u8 | Flag::H as u8);

        return_val
    }

    #[inline]
    fn bit(&mut self, value: u8, bit: u8) {
        self.regs.change_flag(value & (1 << bit) == 0, Flag::Z);

        self.regs.clear_flag(Flag::N);
        self.regs.set_flag(Flag::H);
    }

    #[inline]
    fn set(&self, value: u8, bit: u8) -> u8 {
        value | (1 << bit)
    }

    #[inline]
    fn res(&self, value: u8, bit: u8) -> u8 {
        value & !(1 << bit)
    }

    #[inline]
    fn relative(&mut self, io: &mut IO) -> u16 {
        let val = self.read_next_byte(io) as i8;
        self.regs.pc.wrapping_add(val as u16)
    }

    #[inline]
    fn call(&mut self, io: &mut IO) -> u16 {
        let addr = self.read_next_word(io);
        self.internal_cycle(io);
        self.stack_push16(io, self.regs.pc);
        addr
    }

    #[inline]
    fn rst(&mut self, io: &mut IO, addr: u16) {
        self.internal_cycle(io);
        self.stack_push16(io, self.regs.pc);
        self.regs.pc = addr;
    }

    #[inline]
    fn ret(&mut self, io: &mut IO) -> u16 {
        let addr = self.stack_pop16(io);
        self.internal_cycle(io);
        addr
    }
}
