pub struct LengthCounter {
    length: u8,
    enabled: bool,
}

impl LengthCounter {
    pub fn new() -> Self {
        LengthCounter {
            length: 0,
            enabled: false,
        }
    }

    pub fn clock(&mut self) {
        if self.length > 0 {
            self.length -= 1;
        }
    }

    pub fn reload(&mut self, value: u8) {
        self.length = value;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}