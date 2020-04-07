pub struct LengthCounter {
    length: u16,
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
        if self.enabled && self.length != 0 {
            self.length -= 1;
            if self.length == 0 { self.enabled = false }
        }
    }

    pub fn reload(&mut self, value: u16) {
        self.length = value;
    }

    pub fn enable(&mut self, reload: u16) {
        self.enabled = true;
        if self.length == 0 { self.length = reload; }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}