mod tone_sweep;
mod tone;
mod audio;

use super::MemoryHandler;
use tone_sweep::ToneSweep;
use tone::Tone;
use audio::Audio;
use super::super::GBC;

pub struct APU {
    // Registers
    tone_sweep: ToneSweep,
    tone: Tone,
    // Channel Control / Volume
    enable_left_analog: bool,
    left_volume: u8,
    enable_right_analog: bool,
    right_volume: u8,
    // Sound Output
    noise_left_enable: bool,
    wave_left_enable: bool,
    tone_left_enable: bool,
    tone_sweep_left_enable: bool,
    noise_right_enable: bool,
    wave_right_enable: bool,
    tone_right_enable: bool,
    tone_sweep_right_enable: bool,
    enable_sound: bool,

    // Audio Output
    audio: Audio,
    left_sample_sum: f32,
    right_sample_sum: f32,
    sample_count: u32,
}

impl MemoryHandler for APU {
    fn read(&self, addr: u16) -> u8 {
        macro_rules! shift { ($bit:ident, $num:expr) => { ((self.$bit as u8) << $num) } }
        macro_rules! shift2 { ($channel:ident, $num:expr) => { ((self.$channel.playing_sound() as u8) << $num) } }

        match addr {
            0xFF10 ..= 0xFF14 => self.tone_sweep.read(addr),
            0xFF16 ..= 0xFF19 => self.tone.read(addr),
            0xFF24 => {
                shift!(enable_left_analog, 7) | shift!(left_volume, 4) | shift!(enable_right_analog, 3) | self.right_volume
            }
            0xFF25 => {
                shift!(noise_left_enable, 7) | shift!(wave_left_enable, 6) | shift!(tone_left_enable, 5) |
                shift!(tone_sweep_left_enable, 4) | shift!(noise_right_enable, 3) | shift!(wave_right_enable, 2) |
                shift!(tone_right_enable, 1) | shift!(tone_sweep_right_enable, 0)
            },
            0xFF26 => {
                shift!(enable_sound, 7) | shift2!(tone, 1)
            },
            _ => { 0xFF }, // panic!("Unexpted Address for APU!"),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF10 ..= 0xFF14 => self.tone_sweep.write(addr, value),
            0xFF16 ..= 0xFF19 => self.tone.write(addr, value),
            0xFF24 => {
                self.enable_left_analog = value & 0x80 != 0;
                self.left_volume = (value >> 4) & 0x7;
                self.enable_right_analog = value & 0x8 != 0;
                self.right_volume = value & 0x7;
            },
            0xFF25 => {
                self.noise_left_enable = value & 0x80 != 0;
                self.wave_left_enable = value & 0x40 != 0;
                self.tone_left_enable = value & 0x20 != 0;
                self.tone_sweep_left_enable = value & 0x10 != 0;
                self.noise_right_enable = value & 0x08 != 0;
                self.wave_right_enable = value & 0x04 != 0;
                self.tone_right_enable = value & 0x02 != 0;
                self.tone_sweep_right_enable = value & 0x01 != 0;
            },
            0xFF26 => {
                self.enable_sound = value & 0x80 != 0;
                if !self.enable_sound {
                    self.tone_sweep = ToneSweep::new();
                    self.tone = Tone::new();
                }
            },
            _ => {}, // panic!("Unexpected Address for APU!"),
        }
    }
}

impl APU {
    pub fn new(sdl_ctx: &sdl2::Sdl) -> Self {
        APU {
            // Registers
            tone_sweep: ToneSweep::new(),
            tone: Tone::new(),
            // Channel Control / Volume
            enable_left_analog: false,
            left_volume: 0,
            enable_right_analog: false,
            right_volume: 0,
            // Sound Output
            noise_left_enable: false,
            wave_left_enable: false,
            tone_left_enable: false,
            tone_sweep_left_enable: false,
            noise_right_enable: false,
            wave_right_enable: false,
            tone_right_enable: false,
            tone_sweep_right_enable: false,
            enable_sound: false,

            // Audio Output
            audio: Audio::new(sdl_ctx),
            left_sample_sum: 0.0,
            right_sample_sum: 0.0,
            sample_count: 0,
        }
    }

    pub fn emulate_clock(&mut self) {
        self.tone_sweep.emulate_clock();
        self.tone.emulate_clock();

        self.generate_sample();
    }

    fn generate_sample(&mut self) {
        if !self.enable_sound { return }

        if self.tone_sweep_left_enable { self.left_sample_sum += self.tone_sweep.generate_sample(); }
        if self.tone_left_enable { self.left_sample_sum += self.tone.generate_sample(); }

        if self.tone_sweep_right_enable { self.right_sample_sum += self.tone_sweep.generate_sample(); }
        if self.tone_right_enable { self.right_sample_sum += self.tone.generate_sample(); }
        
        self.sample_count += 1;

        if self.sample_count as f32 >= APU::CLOCKS_PER_SAMPLE {
            let left_sample = self.left_sample_sum / self.sample_count as f32 * APU::VOLUME_FACTOR;
            let right_sample = self.right_sample_sum / self.sample_count as f32 * APU::VOLUME_FACTOR;
            self.audio.queue(self.left_volume as f32 * left_sample, self.right_volume as f32 * right_sample);
            self.left_sample_sum = 0.0;
            self.right_sample_sum = 0.0;
            self.sample_count = 0;
        }
    }

    const CLOCKS_PER_SAMPLE: f32 = GBC::CLOCK_SPEED as f32 / Audio::SAMPLE_RATE as f32;
    const VOLUME_FACTOR: f32 = 0.005;
}

trait Voice {
    fn emulate_clock(&mut self);
    fn generate_sample(&self) -> f32;
    fn playing_sound(&self) -> bool;
}
