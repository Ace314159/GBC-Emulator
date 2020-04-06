use super::MemoryHandler;
use super::Voice;

pub struct Tone {
    // Registers
    wave_duty: u8, // 2 bit
    length_data: u8, // 6 bit
    initial_volume: u8, // 4 bit
    inc_envelope: bool,
    envelope_num: u8, // 3 bit
    freq_data: u16, // 11 bit
    playing_sound: bool,
    use_length: bool,
    
    // Sample Generation
    duty_clock: u32,
    duty_pos: usize,
    length: u32,
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
                self.length_data = value & 0x1F;
                self.length = (64 - self.length_data as u32) * 4096;
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
                self.freq_data = self.freq_data & !0xFF | value as u16;
            }
            0xFF19 => {
                if value & 0x80 != 0 { self.playing_sound = true }
                self.use_length = value & 0x40 != 0;
                self.freq_data = self.freq_data & !0x700 | (value as u16 & 0x7) << 8;
            }
            _ => {},
        }
    }
}

impl Voice for Tone {
    fn emulate_clock(&mut self) {
        if !self.playing_sound { return }

        if self.use_length {
            if self.length == 0 { self.playing_sound = false; return }
            else { self.length -= 1 }
        }

        if self.envelope_counter > 0 {
            if self.envelope_counter == 0 {
                self.envelope_sweep_counter -= 1;
                if self.inc_envelope {
                    if self.volume < 15 { self.volume += 1; }
                    else { self.envelope_sweep_counter = 0 }
                } else {
                    if self.volume > 0 { self.volume -= 1 }
                    else { self.envelope_sweep_counter = 0 }
                }
                self.envelope_counter = 0x4000;
            } else { self.envelope_counter -= 1 }
        }

        if self.duty_clock == 0 {
            self.reset_duty_clock();
            self.duty_pos = (self.duty_pos + 1) % 8;
        } else { self.duty_clock -= 1 }
    }

    fn generate_sample(&self) -> f32 {
        if self.playing_sound {
            self.volume as f32 * Tone::DUTY_CYCLES[self.wave_duty as usize][self.duty_pos] as f32
        }
        else { 0.0 }
    }

    fn playing_sound(&self) -> bool {
        self.playing_sound
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
            length_data: 0,
            initial_volume: 0,
            inc_envelope: false,
            envelope_num: 0,
            freq_data: 0,
            playing_sound: false,
            use_length: false,
            
            // Sample Generation
            duty_clock: 0,
            duty_pos: 0,
            length: 0,
            volume: 0,
            envelope_counter: 0,
            envelope_sweep_counter: 0,
        }
    }

    fn reset_duty_clock(&mut self) {
        self.duty_clock = 2048 - self.freq_data as u32;
    }
}
