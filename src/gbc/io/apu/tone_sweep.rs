use super::MemoryHandler;
use super::Tone;
use super::Channel;

pub struct ToneSweep {
    tone: Tone,

    // Registers
    sweep_period: u8, // 3 bit
    sweep_negate: bool,
    sweep_shift: u8, // 3 bit

    // Sample Generation
    sweep_enabled: bool,
    sweep_counter: u8,
    freq_latch: u16,
    enabled: bool,
}

impl MemoryHandler for ToneSweep {
    fn read(&self, addr: u16) -> u8 {
        macro_rules! shift { ($bit:ident, $num:expr) => { ((self.$bit as u8) << $num) } }

        match addr {
            0xFF10 => {
                shift!(sweep_period, 4) | shift!(sweep_negate, 3) | self.sweep_shift
            },
            _ => self.tone.read(addr + 5),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF10 => {
                self.sweep_period = (value >> 4) & 0x7;
                self.sweep_negate = value & 0x8 != 0;
                self.sweep_shift = value & 0x7;
            },
            0xFF14 => {
                self.tone.write(addr + 5, value);
                if value & 0x80 != 0 {
                    self.enabled = true;
                    self.freq_latch = self.tone.freq;
                    self.sweep_counter = if self.sweep_period == 0 { 8 } else { self.sweep_period };
                    self.sweep_enabled = self.sweep_period != 0 || self.sweep_shift != 0;
                    if self.sweep_shift != 0 {
                        self.overflow_check(self.calc_new_freq());
                    }
                }
            }
            _ => self.tone.write(addr + 5, value),
        }
    }
}

impl Channel for ToneSweep {
    fn emulate_clock(&mut self) {
        self.tone.emulate_clock();
    }

    fn clock_sweep(&mut self) {
        self.sweep_counter = self.sweep_counter.wrapping_sub(1);
        if self.sweep_enabled && self.sweep_counter == 0 {
            if self.sweep_period != 0 {
                let new_freq = self.calc_new_freq();
                if !self.overflow_check(new_freq) {
                    self.tone.freq = new_freq;
                    self.freq_latch = new_freq;
                    self.overflow_check(self.calc_new_freq());
                }
            }
            self.sweep_counter = if self.sweep_period == 0 { 8 } else { self.sweep_period };
        }
    }

    fn clock_length_counter(&mut self) { self.tone.clock_length_counter() }
    fn clock_envelope(&mut self) { self.tone.clock_envelope() }
    fn generate_sample(&self) -> f32 { if self.enabled { self.tone.generate_sample() } else { 0.0 } }
    fn playing_sound(&self) -> bool { self.tone.playing_sound() }
}

impl ToneSweep {
    pub fn new() -> Self {
        ToneSweep {
            tone: Tone::new(),

            // Registers
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,

            // Sample Generation
            sweep_enabled: false,
            sweep_counter: 8,
            freq_latch: 0,
            enabled: true,
        }
    }

    fn calc_new_freq(&self) -> u16 {
        let operand = self.freq_latch >> self.sweep_shift;
        if self.sweep_negate {
            self.freq_latch.wrapping_add(!operand).wrapping_add(1)
        } else {
            self.freq_latch.wrapping_add(operand)
        }
    }

    fn overflow_check(&mut self, new_freq: u16) -> bool {
        if new_freq >= 0x800 {
            self.enabled = false;
            self.sweep_enabled = false;
            true
        } else { false }
    }
}
