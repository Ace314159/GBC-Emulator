use super::MemoryHandler;

pub struct Serial {
    // Registers
    data: u8,
    control: u8,
}

impl Serial {
    pub fn new() -> Self {
        Serial {
            data: 0,
            control: 0,
        }
    }
}

impl MemoryHandler for Serial {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF01 => self.data,
            0xFF02 => self.control | 0x7E,
            _ => panic!("Unexpected Address for Serial!"),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF01 => self.data = value,
            0xFF02 => {
                self.control = value;
                if self.control == 0x81 {
                    print!("{}", self.data as char);
                    self.control &= !0x80;
                }
            },
            _ => panic!("Unexpected Address for Serial!"),
        }
    }
}
