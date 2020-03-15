use super::MemoryHandler;

pub struct Timer {
    divider_counter: u16,
    counter: u8,
    modulo: u8,
    enabled: bool,
    clock_select: u8,

    prev_counter_bit: bool,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            divider_counter: 0,
            counter: 0,
            modulo: 0,
            enabled: false,
            clock_select: 0,
            prev_counter_bit: false,
        }
    }

    const CLOCK_SELECT: [usize; 4] = [9, 3, 5, 7];

    pub fn emulate(&mut self) -> bool {
        let mut interrupt_called = false;
        self.divider_counter = self.divider_counter.wrapping_add(4);

        let bit = Timer::CLOCK_SELECT[self.clock_select as usize];
        let counter_bit = self.enabled && self.divider_counter & (1 << bit) != 0;
        if self.prev_counter_bit && !counter_bit {
            self.counter = if self.counter == 0xFF {
                interrupt_called = true;
                self.modulo
            } else { self.counter + 1 };
        }
        self.prev_counter_bit = counter_bit;

        interrupt_called
    }
}

impl MemoryHandler for Timer {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.divider_counter >> 8) as u8,
            0xFF05 => self.counter,
            0xFF06 => self.modulo,
            0xFF07 => ((self.enabled as u8) << 2) | self.clock_select,
            _ => panic!("Unexpected Address for Timer!")
        }
    }
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF04 => self.divider_counter = 0,
            0xFF05 => self.counter = value,
            0xFF06 => self.modulo = value,
            0xFF07 => { self.enabled = value & 0x04 != 0; self.clock_select = value & 0x03; }
            _ => panic!("Unexpected Address for Timer!"),
        }
    }
}
