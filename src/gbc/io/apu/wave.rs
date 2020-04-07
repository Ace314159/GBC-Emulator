use super::MemoryHandler;
use super::Channel;

use super::timer::Timer;
use super::length_counter::LengthCounter;

pub struct Wave {
    // Registers
    enabled: bool,
    length_reload: u8,
    output_level: u8, // 2 bit
    freq: u16, // 11 bit
    use_length: bool,

    // Sample Generation
    timer: Timer,
    length_counter: LengthCounter,
    wave_table: [u8; 0x10], // 32 4-bit samples
    wave_pos: usize,
}

impl MemoryHandler for Wave {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF1A => (self.enabled as u8) << 7 | 0x7F,
            0xFF1B => 0xFF,
            0xFF1C => 0x9F | self.output_level << 5,
            0xFF1D => 0xFF,
            0xFF1E => 0xBF | (self.use_length as u8) << 6,
            _ => panic!("Unexpected Addres for Wave")
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF1A => self.enabled = value & 0x80 != 0,
            0xFF1B => {
                self.length_reload = value;
                self.length_counter.reload(256 - self.length_reload as u16);
            },
            0xFF1C => self.output_level = (value >> 5) & 0x3,
            0xFF1D => self.freq = self.freq & !0xFF | value as u16,
            0xFF1E => {
                if value & 0x80 != 0 {
                    self.length_counter.enable(256);
                    self.timer.reload(2048 - self.freq);
                    self.wave_pos = 0;
                }
                self.use_length = value & 0x40 != 0;
                self.freq = self.freq & !0x700 | (value as u16 & 0x7) << 8;
            },
            _ => panic!("Unexpected Addres for Wave"),
        };
    }
}

impl Channel for Wave {
    fn emulate_clock(&mut self) {
        if !self.length_counter.enabled() { return }

        if self.timer.clock((2048 - self.freq) * 2) {
            self.wave_pos = (self.wave_pos + 1) % (2 * self.wave_table.len());
        }
    }

    fn clock_sweep(&mut self) {}

    fn clock_length_counter(&mut self) {
        if self.use_length {
            self.length_counter.clock();
        }
    }

    fn clock_envelope(&mut self) {}

    fn generate_sample(&self) -> f32 {
        if self.length_counter.enabled() || self.output_level == 0 {
            let sample: u8 = self.wave_table[self.wave_pos / 2] >> self.output_level;
            if self.wave_pos % 2 == 1 {
                (sample & 0x0F) as f32
            } else {
                (sample >> 4) as f32
            }
        } else { 0.0 }
    }

    fn playing_sound(&self) -> bool {
        self.length_counter.enabled()
    }
}

impl Wave {
    pub fn new() -> Self {
        Wave {
            // Registers
            enabled: false,
            length_reload: 0,
            output_level: 0, // 2 bit
            freq: 0,
            use_length: false,

            // Sample Generation
            timer: Timer::new(0),
            length_counter: LengthCounter::new(),
            wave_table: [0; 0x10],
            wave_pos: 0,
        }
    }

    pub fn read_wave_table(&self, addr: u16) -> u8 {
        self.wave_table[addr as usize - 0xFF30]
    }

    pub fn write_wave_table(&mut self, addr: u16, value: u8) {
        self.wave_table[addr as usize - 0xFF30] = value;
    }
}
