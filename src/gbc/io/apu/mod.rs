mod tone;
mod audio;

use super::MemoryHandler;
use tone::Tone;
use audio::Audio;

pub struct APU {
    // Registers
    tone: Tone,
    // Channel Control
    enable_left: bool,
    left_volume: u8,
    enable_right: bool,
    right_volume: u8,

    // Audio Output
    audio: Audio,
    sample_sum: f32,
    sample_count: u32,
}

impl MemoryHandler for APU {
    fn read(&self, addr: u16) -> u8 {
        macro_rules! shift { ($bit:ident, $num:expr) => { ((self.$bit as u8) << $num) } }

        match addr {
            0xFF16 ..= 0xFF19 => self.tone.read(addr),
            0xFF24 => {
                shift!(enable_left, 7) | shift!(left_volume, 4) | shift!(enable_right, 3) | self.right_volume
            }
            _ => { 0xFF }, //panic!("Unexpted Address for APU!"),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF16 ..= 0xFF19 => self.tone.write(addr, value),
            0xFF24 => {
                self.enable_left = value & 0x80 != 0;
                self.left_volume = (value >> 4) & 0x7;
                self.enable_right = value & 0x8 != 0;
                self.right_volume = value & 0x7;
            }
            _ => {}, //panic!("Unexpected Address for Serial!"),
        }
    }
}

impl APU {
    pub fn new(sdl_ctx: &sdl2::Sdl) -> Self {
        APU {
            // Registers
            tone: Tone::new(),
            enable_left: false,
            left_volume: 0,
            enable_right: false,
            right_volume: 0,

            // Audio Output
            audio: Audio::new(sdl_ctx),
            sample_sum: 0.0,
            sample_count: 0,
        }
    }

    pub fn emulate_clock(&mut self) {
        self.tone.emulate_clock();

        self.generate_sample();
    }

    fn generate_sample(&mut self) {
        self.sample_sum += self.tone.generate_sample();
        self.sample_count += 1;

        if self.sample_count as f32 >= APU::CLOCKS_PER_SAMPLE {
            self.audio.queue(self.sample_sum / self.sample_count as f32 * 0.1);
            self.sample_sum = 0.0;
            self.sample_count = 0;
        }
    }

    const CLOCK_SPEED: u32 = 4194304 / 4;
    const CLOCKS_PER_SAMPLE: f32 = APU::CLOCK_SPEED as f32 / Audio::SAMPLE_RATE as f32;
}
