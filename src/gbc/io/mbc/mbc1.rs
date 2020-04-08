use super::MemoryBankController;
use super::MemoryHandler;
use super::Header;

pub struct MBC1 {
    rom_mask: usize,
    ram_mask: usize,

    rom: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,
    is_ram_banking: bool,
    external_ram: Vec<u8>,

    has_ram: bool,
    _has_battery: bool,
}

impl MBC1 {
    pub fn new(header: Header, rom: Vec<u8>, has_ram: bool, has_battery: bool) -> Self {
        let ram_size = header.get_ram_size();
        let rom_size = header.get_rom_size();
        assert_eq!(rom_size, rom.len());
        MBC1 {
            rom_mask: rom_size / 0x4000 - 1,
            ram_mask: if ram_size == 0x0800 || ram_size == 0 { 0 } else { ram_size / 0x2000 - 1 },

            rom,
            rom_bank: 1,
            ram_bank: 0,
            ram_enable: false,
            is_ram_banking: false,
            external_ram: vec![0; ram_size],

            has_ram,
            _has_battery: has_battery,
        }
    }
}

impl MemoryHandler for MBC1 {
    fn read(&self, addr: u16) -> u8 {
        match addr & 0xC000 {
            0x0000 => if self.is_ram_banking { self.rom[(self.rom_bank & 0x60) * 0x4000 + addr as usize] }
                else { self.rom[addr as usize] },
            0x4000 => self.rom[self.rom_bank * 0x4000 + (addr - 0x4000) as usize],
            0x8000 => if self.ram_enable && self.has_ram {
                self.external_ram[self.ram_bank * 0x2000 + (addr as usize - 0xA000)]
            } else { 0xFF },
            _ => panic!("Shouldn't be here!"),
        }
    }
    
    fn write(&mut self, addr: u16, value: u8) {
        match addr & 0xE000 {
            0x0000 => self.ram_enable = value & 0x0F == 0x0A,
            0x2000 => {
                let mut bank = (value & 0x1F) as usize;
                if bank == 0 { bank = 1; };
                self.rom_bank = (self.rom_bank & !0x1F | bank) & self.rom_mask;
            },
            0x4000 => if self.is_ram_banking && self.has_ram { self.ram_bank = (value as usize & 0x3) & self.ram_mask }
                      else { self.rom_bank = (self.rom_bank & !0xE0 | (value as usize & 0x3) << 5) & self.rom_mask },
            0x6000 => {
                self.is_ram_banking = value & 0x1 != 0;
                if self.is_ram_banking && self.has_ram {
                    self.rom_bank = (self.rom_bank & 0x1F) & self.rom_mask;
                } else {
                    self.ram_bank = 0;
                }
            },
            0xA000 => if self.ram_enable && self.has_ram {
                self.external_ram[self.ram_bank * 0x2000 + (addr as usize - 0xA000)] = value
            },
            _ => panic!("Shouldn't be here!"),
        }
    }
}

impl MemoryBankController for MBC1 {
    fn get_boot_rom_ptr(&mut self) -> *mut [u8; 0x100] {
        self.rom[..0x100].as_mut_ptr() as *mut [u8; 0x100]
    }
}
