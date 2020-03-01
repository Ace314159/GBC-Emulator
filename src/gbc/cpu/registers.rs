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
    pub A: u8,
    pub F: u8,
    pub B: u8,
    pub C: u8,
    pub D: u8,
    pub E: u8,
    pub H: u8,
    pub L: u8,
    pub SP: u16,
    pub PC: u16,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            A: 0,
            F: 0,
            B: 0,
            C: 0,
            D: 0,
            E: 0,
            H: 0,
            L: 0,
            SP: 0,
            PC: 0,
        }
    }

    pub fn change_flag(&mut self, condition: bool, flag: Flag) {
        if condition {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    #[inline]
    pub fn set_flag(&mut self, flag: Flag) {
        self.F |= flag as u8;
    }

    #[inline]
    pub fn set_flags(&mut self, flags: u8) {
        self.F |= flags;
    }

    #[inline]
    pub fn clear_flag(&mut self, flag: Flag) {
        self.F &= !(flag as u8);
    }

    #[inline]
    pub fn clear_flags(&mut self, flags: u8) {
        self.F &= !flags;
    }

    #[inline]
    pub fn getFlag(&self, flag: Flag) -> bool {
        return self.F & (flag as u8) != 0;
    }
}
