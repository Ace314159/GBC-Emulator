use super::MemoryHandler;
use super::Channel;

use super::timer::Timer;
use super::length_counter::LengthCounter;


pub struct Tone {
    // Registers
    wave_duty: u8, // 2 bit
    length_reload: u8, // 6 bit
    initial_volume: u8, // 4 bit
    inc_envelope: bool,
    envelope_num: u8, // 3 bit
    pub freq: u16, // 11 bit
    use_length: bool,
    
    // Sample Generation
    timer: Timer,
    duty_pos: usize,
    length_counter: LengthCounter,
    volume: u8,
    envelope_counter: u32,
    envelope_sweep_counter: u8,
}

impl MemoryHandler for Tone {
    fn read(&self, addr: u16) -> u8 {
        macro_rules! shift { ($bit:ident, $num:expr) => { ((self.$bit as u8) << $num) } }

        match addr {
            0xFF16 => shift!(wave_duty, 6) | 0x1F,
            0xFF17 => shift!(initial_volume, 4) | shift!(inc_envelope, 3) | self.envelope_num,
            0xFF18 => 0xFF,
            0xFF19 => 0xBF | shift!(use_length, 6),
            _ => panic!("Unexpected Address for "),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF16 => {
                self.wave_duty = value >> 6;
                self.length_reload = value & 0x1F;
                self.length_counter.reload(64 - self.length_reload);
            },
            0xFF17 => {
                self.initial_volume = value >> 4;
                self.volume = self.initial_volume;
                self.inc_envelope = value & 0x8 != 0;
                self.envelope_num = value & 0x7;
                self.envelope_counter = 0x4000;
                self.envelope_sweep_counter = self.envelope_num;
            },
            0xFF18 => {
                self.freq = self.freq & !0xFF | value as u16;
            }
            0xFF19 => {
                if value & 0x80 != 0 {
                    self.length_counter.reload(self.length_reload);
                    self.length_counter.enable();
                    self.timer.reload(2048 - self.freq);
                    self.duty_pos = 0;
                    self.volume = self.initial_volume;
                }
                self.use_length = value & 0x40 != 0;
                self.freq = self.freq & !0x700 | (value as u16 & 0x7) << 8;
            }
            _ => {},
        }
    }
}

impl Channel for Tone {
    fn emulate_clock(&mut self) {
        if !self.length_counter.enabled() { return }

        if self.timer.clock(2048 - self.freq) {
            self.duty_pos = (self.duty_pos + 1) % 8;
        }
    }
    
    fn clock_sweep(&mut self) {}

    fn clock_length_counter(&mut self) {
        if self.use_length {
            self.length_counter.clock();
        }
    }

    fn clock_envelope(&mut self) {
        if self.envelope_sweep_counter > 0 {
            self.envelope_sweep_counter -= 1;
            if self.inc_envelope {
                if self.volume < 15 { self.volume += 1; }
                else { self.envelope_sweep_counter = 0 }
            } else {
                if self.volume > 0 { self.volume -= 1 }
                else { self.envelope_sweep_counter = 0 }
            }
        }
    }

    fn generate_sample(&self) -> f32 {
        if self.length_counter.enabled() {
            self.volume as f32 * Tone::DUTY_CYCLES[self.wave_duty as usize][self.duty_pos] as f32
        }
        else { 0.0 }
    }

    fn playing_sound(&self) -> bool {
        self.length_counter.enabled()
    }
}

impl Tone {
    const DUTY_CYCLES: [[u8; 8]; 4] = [
        [0, 0, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 1, 1, 1],
        [0, 1, 1, 1, 1, 1, 1, 0],
    ];

    pub fn new() -> Self {
        Tone {
            // Registers
            wave_duty: 0,
            length_reload: 0,
            initial_volume: 0,
            inc_envelope: false,
            envelope_num: 0,
            freq: 0,
            
            // Sample Generation
            use_length: false,
            timer: Timer::new(0),
            duty_pos: 0,
            length_counter: LengthCounter::new(),
            volume: 0,
            envelope_counter: 0,
            envelope_sweep_counter: 0,
        }
    }
}
