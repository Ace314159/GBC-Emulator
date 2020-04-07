pub struct Envelope {
    // Registers
    reload: u8, // 4 bit
    inc: bool,
    period: u8, // 3 bit

    // Sample Generation
    volume: u8,
    counter: u8,
    done: bool,
}

impl Envelope {
    pub fn new() -> Self {
        Envelope {
            // Registers
            reload: 0,
            inc: false,
            period: 0,

            // Sample Generation
            volume: 0,
            counter: 0,
            done: true,
        }
    }

    pub fn emulate_clock(&mut self) {
        if !self.done && self.period != 0 {
            self.counter -= 1;
            if self.counter == 0 {
                if self.inc {
                    if self.volume < 15 { self.volume += 1; }
                    else { self.done = true }
                } else {
                    if self.volume > 0 { self.volume -= 1 }
                    else { self.done = true }
                }
                self.counter = self.period;
            }
        }
    }

    pub fn get_volume(&self) -> u8 {
        self.volume
    }

    pub fn get_reg(&self) -> u8 {
        self.reload << 4 | (self.inc as u8) << 3 | self.period
    }

    pub fn set_reg(&mut self, value: u8) {
        self.reload = value >> 4;
        self.inc = value & 0x8 != 0;
        self.period = value & 0x7;
        self.volume = self.reload;
        self.counter = self.period;
    }

    pub fn reset(&mut self) {
        self.counter = self.period;
        self.done = false;
        self.volume = self.reload;
    }
}
