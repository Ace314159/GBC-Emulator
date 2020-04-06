pub struct Timer {
    counter: u16,
}

impl Timer {
    pub fn new(reload: u16) -> Self {
        Timer {
            counter: reload,
        }
    }

    pub fn clock(&mut self, reload: u16) -> bool {
        if self.counter == 0 {
            self.counter = reload;
            true
        } else {
            self.counter -= 1;
            false
        }
    }

    pub fn reload(&mut self, reload: u16) {
        self.counter = reload;
    }
}
