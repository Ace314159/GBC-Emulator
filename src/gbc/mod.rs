mod mmu;
mod cpu;

use mmu::MMU;
use cpu::CPU;

use std::fs;

pub struct GBC {
    mmu: MMU,
    cpu: CPU,
}

impl GBC {
    pub fn new(rom_file: &String) -> Self {
        let mut rom = fs::read(rom_file).unwrap();
        let mut boot_rom = fs::read("DMG_ROM.bin").unwrap();
        
        let boot_rom_len = boot_rom.len();
        assert_eq!(boot_rom_len, 0x100);
        unsafe {
            let x= boot_rom[..boot_rom_len].as_mut_ptr() as *mut [u8; 0x100];
            let y = rom[..boot_rom_len].as_mut_ptr() as *mut[u8; 0x100];
            std::ptr::swap(x, y);
        }

        let mut gbc = GBC {
            mmu: MMU::new(rom),
            cpu: CPU::new(),
        };

        gbc.cpu.emulate_boot_rom(&mut gbc.mmu);
        gbc.mmu.swap_boot_rom(&mut boot_rom);

        gbc
    }

    pub fn emulate(&mut self) {
        self.cpu.emulate_instr(&mut self.mmu);
    }
}
