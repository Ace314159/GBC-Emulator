mod cpu;
mod io;

use cpu::CPU;
use io::IO;

use std::fs;

pub struct GBC {
    cpu: CPU,
    io: IO,
}

impl GBC {
    pub fn new(rom_file: &String) -> Self {
        // Stores boot rom and swapped portions of game rom
        let mut boot_rom = fs::read("CGB_ROM.bin").unwrap();

        let mut gbc = GBC {
            cpu: CPU::new(),
            io: IO::new(fs::read(rom_file).unwrap()),
        };

        gbc.io.swap_boot_rom(&mut boot_rom);
        gbc.cpu.emulate_boot_rom(&mut gbc.io);
        gbc.io.swap_boot_rom(&mut boot_rom);

        gbc
    }

    pub fn emulate(&mut self) {
        self.cpu.emulate(&mut self.io);
    }
    
    pub fn is_running(&self) -> bool {
        !self.io.should_close
    }

    const CLOCK_SPEED: u32 = 4194304 / 4;
}
