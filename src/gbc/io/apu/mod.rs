mod tone_sweep;
mod tone;
mod wave;
mod noise;
mod audio;

mod timer;
mod length_counter;
mod envelope;

use super::MemoryHandler;
use tone_sweep::ToneSweep;
use tone::Tone;
use wave::Wave;
use noise::Noise;
use audio::Audio;
use super::IO;

pub struct APU {
    // Registers
    tone_sweep: ToneSweep,
    tone: Tone,
    wave: Wave,
    noise: Noise,
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

    // Frame Sequencer
    frame_sequencer_counter: u16,
    frame_sequencer_step: u8,

    // Audio Output
    audio: Audio,
    left_sample_sum: f32,
    right_sample_sum: f32,
    sample_count: u32,
    clock_count: f32,
    clocks_per_sample: f32,
}

impl MemoryHandler for APU {
    fn read(&self, addr: u16) -> u8 {
        macro_rules! shift { ($bit:ident, $num:expr) => { ((self.$bit as u8) << $num) } }
        macro_rules! shift2 { ($channel:ident, $num:expr) => { ((self.$channel.playing_sound() as u8) << $num) } }

        match addr {
            0xFF10 ..= 0xFF14 => self.tone_sweep.read(addr),
            0xFF16 ..= 0xFF19 => self.tone.read(addr),
            0xFF1A ..= 0xFF1E => self.wave.read(addr),
            0xFF20 ..= 0xFF23 => self.noise.read(addr),
            0xFF24 => {
                shift!(enable_left_analog, 7) | shift!(left_volume, 4) | shift!(enable_right_analog, 3) | self.right_volume
            }
            0xFF25 => {
                shift!(noise_left_enable, 7) | shift!(wave_left_enable, 6) | shift!(tone_left_enable, 5) |
                shift!(tone_sweep_left_enable, 4) | shift!(noise_right_enable, 3) | shift!(wave_right_enable, 2) |
                shift!(tone_right_enable, 1) | shift!(tone_sweep_right_enable, 0)
            },
            0xFF26 => {
                shift!(enable_sound, 7) | shift2!(noise, 3) | shift2!(wave, 2) | shift2!(tone, 1) | shift2!(tone_sweep, 0)
            },
            0xFF30 ..= 0xFF3F => self.wave.read_wave_table(addr),
            _ => 0xFF, // panic!("Unexpted Address for APU!"),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF10 ..= 0xFF14 => self.tone_sweep.write(addr, value),
            0xFF16 ..= 0xFF19 => self.tone.write(addr, value),
            0xFF1A ..= 0xFF1E => self.wave.write(addr, value),
            0xFF20 ..= 0xFF23 => self.noise.write(addr, value),
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
                    self.wave = Wave::new();
                    self.noise = Noise::new();
                }
            },
            0xFF30 ..= 0xFF3F => self.wave.write_wave_table(addr, value),
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
            wave: Wave::new(),
            noise: Noise:: new(),
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

            // Frame Sequencer
            frame_sequencer_counter: 0x800,
            frame_sequencer_step: 0,

            // Audio Output
            audio: Audio::new(sdl_ctx),
            left_sample_sum: 0.0,
            right_sample_sum: 0.0,
            sample_count: 0,
            clock_count: 0.0,
            clocks_per_sample: APU::GB_CLOCKS_PER_SAMPLE,
        }
    }

    pub fn emulate_clock(&mut self) {
        self.tone_sweep.emulate_clock();
        self.tone.emulate_clock();
        self.wave.emulate_clock();
        self.wave.emulate_clock();
        self.noise.emulate_clock();

        self.emulate_frame_counter();
        
        self.generate_sample();
    }

    pub fn set_double_speed(&mut self, double_speed: bool) {
        self.clocks_per_sample = if double_speed {
            APU::GBC_CLOCKS_PER_SAMPLE
        } else {
            APU::GB_CLOCKS_PER_SAMPLE
        };
    }

    fn emulate_frame_counter(&mut self) {
        if self.frame_sequencer_counter == 0 {
            self.frame_sequencer_counter = 0x800;
            match self.frame_sequencer_step {
                0 => self.clock_length_counters(),
                2 => { self.clock_length_counters(); self.clock_sweeps(); },
                4 => self.clock_length_counters(),
                6 => { self.clock_length_counters(); self.clock_sweeps(); }
                7 => self.clock_envelopes(),
                _ => {}
            }
            self.frame_sequencer_step = (self.frame_sequencer_step + 1) % 8;
        } else { self.frame_sequencer_counter -= 1 }
    }

    fn clock_length_counters(&mut self) {
        self.tone_sweep.clock_length_counter();
        self.tone.clock_length_counter();
        self.wave.clock_length_counter();
        self.noise.clock_length_counter();
    }

    fn clock_sweeps(&mut self) {
        self.tone_sweep.clock_sweep();
    }

    fn clock_envelopes(&mut self) {
        self.tone_sweep.clock_envelope();
        self.tone.clock_envelope();
        self.noise.clock_envelope();
    }

    fn generate_sample(&mut self) {
        if !self.enable_sound { return }

        if self.tone_sweep_left_enable { self.left_sample_sum += self.tone_sweep.generate_sample(); }
        if self.tone_left_enable { self.left_sample_sum += self.tone.generate_sample(); }
        if self.wave_left_enable { self.left_sample_sum += self.wave.generate_sample(); }
        if self.noise_left_enable { self.left_sample_sum += self.noise.generate_sample(); }

        if self.tone_sweep_right_enable { self.right_sample_sum += self.tone_sweep.generate_sample(); }
        if self.tone_right_enable { self.right_sample_sum += self.tone.generate_sample(); }
        if self.wave_right_enable { self.right_sample_sum += self.wave.generate_sample(); }
        if self.noise_right_enable { self.right_sample_sum += self.noise.generate_sample(); }
        
        self.sample_count += 1;
        self.clock_count += 1.0;

        if self.clock_count as f32 >= self.clocks_per_sample {
            let mut left_sample = self.left_sample_sum / self.sample_count as f32 / 7.5 - 1.0;
            left_sample *= (self.left_volume + 1) as f32;
            let mut right_sample = self.right_sample_sum / self.sample_count as f32 / 7.5 - 1.0;
            right_sample *= (self.right_volume + 1) as f32;
            self.audio.queue(APU::VOLUME_FACTOR * left_sample, APU::VOLUME_FACTOR * right_sample);
            self.left_sample_sum = 0.0;
            self.right_sample_sum = 0.0;
            self.sample_count = 0;
            self.clock_count -= self.clocks_per_sample;
        }
    }

    const GBC_CLOCKS_PER_SAMPLE: f32 = IO::GBC_CLOCK_SPEED as f32 / Audio::SAMPLE_RATE as f32;
    const GB_CLOCKS_PER_SAMPLE: f32 = IO::GB_CLOCK_SPEED as f32 / Audio::SAMPLE_RATE as f32;
    const VOLUME_FACTOR: f32 = 5e-3;
}

trait Channel {
    fn emulate_clock(&mut self);
    fn clock_sweep(&mut self);
    fn clock_length_counter(&mut self);
    fn clock_envelope(&mut self);
    fn generate_sample(&self) -> f32;
    fn playing_sound(&self) -> bool;
}
