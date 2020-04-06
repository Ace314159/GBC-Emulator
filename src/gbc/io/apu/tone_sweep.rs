use super::MemoryHandler;
use super::Tone;
use super::Channel;

pub struct ToneSweep {
    tone: Tone,

    // Registers
    sweep_time: u8, // 3 bit
    dec_sweep: bool,
    sweep_num: u8, // 3 bit

    // Sample Generation
    sweep_counter: u32,
    sweep_num_left: u8,
}

impl MemoryHandler for ToneSweep {
    fn read(&self, addr: u16) -> u8 {
        macro_rules! shift { ($bit:ident, $num:expr) => { ((self.$bit as u8) << $num) } }

        match addr {
            0xFF10 => {
                shift!(sweep_time, 4) | shift!(dec_sweep, 3) | self.sweep_num
            },
            _ => self.tone.read(addr + 5),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF10 => {
                self.sweep_time = (value >> 4) & 0x7;
                self.dec_sweep = value & 0x4 != 0;
                self.sweep_num = value & 0x7;
                self.sweep_counter = 0x2000;
                self.sweep_num_left = self.sweep_time;
            },
            _ => self.tone.write(addr + 5, value),
        }
    }
}

impl Channel for ToneSweep {
    fn emulate_clock(&mut self) {
        self.tone.emulate_clock();
        
        if self.playing_sound() {
            if self.sweep_counter == 0 {
                self.sweep_counter = 0x2000;
            } else { self.sweep_counter -= 1 }
        }
    }

    fn clock_sweep(&mut self) {

    }

    fn clock_length_counter(&mut self) { self.tone.clock_length_counter() }
    fn clock_envelope(&mut self) { self.tone.clock_envelope() }
    fn generate_sample(&self) -> f32 { self.tone.generate_sample() }
    fn playing_sound(&self) -> bool { self.tone.playing_sound() }
}

impl ToneSweep {
    pub fn new() -> Self {
        ToneSweep {
            tone: Tone::new(),

            // Registers
            sweep_time: 0,
            dec_sweep: false,
            sweep_num: 0,

            // Sample Generation
            sweep_counter: 0,
            sweep_num_left: 0,
        }
    }
}
