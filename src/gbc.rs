mod memory;

use memory::Memory;

use std::fs;

pub struct GBC {
    memory: Memory,
}

impl GBC {
    pub fn new(rom_file: &String) -> Self {
        GBC {
            memory: Memory::new(fs::read(rom_file).unwrap()),
        }
    }
}
