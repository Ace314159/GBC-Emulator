#[macro_export]
macro_rules! set_reg8 {
    ($regs:expr, $reg:ident) => {
        |value: u8| $regs.$reg = value;
    }
}

#[macro_export]
macro_rules! get_reg16 {
    ($regs:expr, $high:ident, $low:ident) => { 
        ($regs.$high as u16) << 8 | ($regs.$low as u16)
    }
}

#[macro_export]
macro_rules! set_reg16 {
    ($regs:expr, $high:ident, $low:ident) => { 
    |value: u16| {
        $regs.$high = (value >> 8) as u8;
        $regs.$low = (value & 0xFF) as u8;
    }
}
}

pub enum Flag {
    Z = 0x80,
    N = 0x40,
    H = 0x20,
    C = 0x10,
}

pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }

    #[inline]
    pub fn change_flag(&mut self, condition: bool, flag: Flag) {
        if condition {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    #[inline]
    pub fn set_flag(&mut self, flag: Flag) {
        self.f |= flag as u8;
    }

    #[inline]
    pub fn set_flags(&mut self, flags: u8) {
        self.f |= flags;
    }

    #[inline]
    pub fn clear_flag(&mut self, flag: Flag) {
        self.f &= !(flag as u8);
    }

    #[inline]
    pub fn clear_flags(&mut self, flags: u8) {
        self.f &= !flags;
    }

    #[inline]
    pub fn get_flag(&self, flag: Flag) -> bool {
        self.f & (flag as u8) != 0
    }
}
