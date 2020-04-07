use super::MemoryHandler;
use super::Channel;

use super::timer::Timer;
use super::length_counter::LengthCounter;
use super::envelope::Envelope;

pub struct Noise {
    // Registers
    length_reload: u8, // 6 bit
    envelope: Envelope,
    clock_shift: u8, // 4 bit
    width_mode: bool,
    divisor_code: u8, // 3 bit
    use_length: bool,

    // Sample Generation
    timer: Timer,
    lfsr: u16, // 15 bit
    length_counter: LengthCounter,
}

impl MemoryHandler for Noise {
    fn read(&self, addr: u16) -> u8 {
        macro_rules! shift { ($bit:ident, $num:expr) => { ((self.$bit as u8) << $num) } }

        match addr {
            0xFF20 => 0xC0 | shift!(length_reload, 6),
            0xFF21 => self.envelope.get_reg(),
            0xFF22 => shift!(clock_shift, 4) | shift!(width_mode, 3) | self.divisor_code,
            0xFF23 => 0xBF | shift!(use_length, 6),
            _ => panic!("Unexpected Address for Noise")
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF20 => self.length_reload = value & 0x3F,
            0xFF21 => self.envelope.set_reg(value),
            0xFF22 => {
                self.clock_shift = (value >> 4) & 0xF;
                self.width_mode = value & 0x08 != 0;
                self.divisor_code = value & 0x7;
            },
            0xFF23 => {
                if value & 0x80 != 0 {
                    self.length_counter.enable(64);
                    self.timer.reload(Noise::DIVISORS[self.divisor_code as usize] << self.clock_shift);
                    self.envelope.reset();
                    self.lfsr = 0x7FFF;
                }
                self.use_length = value & 0x40 != 0;
            },
            _ => panic!("Unexpected Address for Noise")
        }
    }
}

impl Channel for Noise {
    fn emulate_clock(&mut self) {
        if !self.length_counter.enabled() { return }

        if self.timer.clock(Noise::DIVISORS[self.divisor_code as usize] << self.clock_shift) {
            let new_high = (self.lfsr & 0x1) ^ (self.lfsr >> 1 & 0x1);
            self.lfsr = new_high << 14 | self.lfsr >> 1;
            if self.width_mode {
                self.lfsr = self.lfsr & !0x40 | new_high << 6;
            }
        }
    }
    
    fn clock_sweep(&mut self) {}

    fn clock_length_counter(&mut self) {
        if self.use_length {
            self.length_counter.clock();
        }
    }

    fn clock_envelope(&mut self) {
        self.envelope.emulate_clock();
    }

    fn generate_sample(&self) -> f32 {
        if self.length_counter.enabled() {
            (self.envelope.get_volume() * (!self.lfsr as u8 & 0x1)) as f32
        } else { 0.0 }
    }

    fn playing_sound(&self) -> bool {
        self.length_counter.enabled()
    }
}

impl Noise {
    const DIVISORS: [u16; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

    pub fn new() -> Self {
        Noise {
            // Registers
            length_reload: 0,
            envelope: Envelope::new(),
            clock_shift: 0,
            width_mode: false,
            divisor_code: 0,
            use_length: false,

            // Sample Generation
            timer: Timer::new(0),
            lfsr: 0,
            length_counter: LengthCounter::new(),
        }
    }
}
