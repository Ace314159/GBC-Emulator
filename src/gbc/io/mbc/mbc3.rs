use super::MemoryBankController;
use super::MemoryHandler;
use super::Header;

pub struct MBC3 {
    rom_mask: usize,
    ram_mask: usize,

    rom: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize, // RAM Bank or RTC Register depending on value
    ram_enable: bool, // RAM and RTC Registers Enable
    external_ram: Vec<u8>,
    rtc_registers: [u8; 5],
    latched_rtc_registers: [u8; 5],
    latch_clock_data: bool,
    halt_timer: bool,

    clock_counter: u8,
    rtc_counter: u16,

    has_timer: bool,
    has_ram: bool,
    _has_battery: bool,
}

impl MBC3 {
    pub fn new(header: Header, rom: Vec<u8>, has_timer: bool, has_ram: bool, has_battery: bool) -> Self {
        let ram_size = header.get_ram_size();
        let rom_size = header.get_rom_size();
        assert_eq!(rom_size, rom.len());
        MBC3 {
            rom_mask: rom_size / 0x4000 - 1,
            ram_mask: if ram_size == 0x0800 || ram_size == 0 { 0 } else { ram_size / 0x2000 - 1 },

            rom,
            rom_bank: 1,
            ram_bank: 0,
            ram_enable: false,
            external_ram: vec![0; ram_size],
            rtc_registers: [0; 5],
            latched_rtc_registers: [0; 5],
            latch_clock_data: false,
            halt_timer: false,

            clock_counter: 0,
            rtc_counter: 0,

            has_timer,
            has_ram: has_ram && ram_size > 0,
            _has_battery: has_battery,
        }
    }
}

impl MemoryHandler for MBC3 {
    fn read(&self, addr: u16) -> u8 {
        match addr & 0xC000 {
            0x0000 => self.rom[addr as usize],
            0x4000 => self.rom[self.rom_bank * 0x4000 + (addr - 0x4000) as usize],
            0x8000 => if self.ram_enable {
                if self.ram_bank <= 0x3 && self.has_ram {
                    self.external_ram[self.ram_bank * 0x2000 + (addr as usize - 0xA000)]
                } else if self.ram_bank >= 0x8 && self.ram_bank <= 0xC && self.has_timer {
                    self.latched_rtc_registers[self.ram_bank - 0x8]
                } else { 0xFF }
            } else { 0xFF },
            _ => panic!("Shouldn't be here!"),
        }
    }
    
    fn write(&mut self, addr: u16, value: u8) {
        match addr & 0xE000 {
            0x0000 => self.ram_enable = value & 0x0F == 0x0A,
            0x2000 => {
                let mut bank = (value & 0x7F) as usize;
                if bank == 0 { bank = 1; };
                self.rom_bank = bank & self.rom_mask;
            },
            0x4000 => self.ram_bank = (value as usize) & self.ram_mask,
            0x6000 => if self.ram_enable && self.has_timer {
                let new_latch_clock_data = value & 0x1 != 0;
                if !self.latch_clock_data && new_latch_clock_data {
                    for (i, reg) in self.rtc_registers.iter().enumerate() {
                        self.latched_rtc_registers[i] = *reg;
                    }
                }
                self.latch_clock_data = new_latch_clock_data;
            },
            0xA000 => if self.ram_enable {
                if self.ram_bank <= 0x3 && self.has_ram {
                    self.external_ram[self.ram_bank * 0x2000 + (addr as usize - 0xA000)] = value;
                } else if self.ram_bank >= 0x8 && self.ram_bank <= 0xC && self.has_timer {
                    self.rtc_registers[self.ram_bank - 0x8] = value;

                    self.halt_timer = self.rtc_registers[4] & 0x40 != 0;
                }
            },
            _ => panic!("Shouldn't be here!"),
        }
    }
}

impl MemoryBankController for MBC3 {
    fn get_boot_rom_ptr(&mut self) -> *mut [u8; 0x100] {
        self.rom[..0x100].as_mut_ptr() as *mut [u8; 0x100]
    }

    fn emulate_clock(&mut self) {
        if !self.has_timer || self.halt_timer { return }

        if self.rtc_counter == 32768 {
            if self.rtc_registers[0] == 59 { // Seconds
                self.rtc_registers[0] = 0;
                if self.rtc_registers[1] == 59 { // Minutes
                    self.rtc_registers[1] = 0;
                    if self.rtc_registers[2] == 23 { // Hours
                        self.rtc_registers[2] = 0;
                        if self.rtc_registers[3] == 0xFF { // Days
                            self.rtc_registers[3] = 0;
                            if self.rtc_registers[4] & 0x1 != 0 {
                                self.rtc_registers[4] = self.rtc_registers[4] & !0x1 | 0x80;
                            } else {
                                self.rtc_registers[4] |= 0x1;
                            }
                        } else { self.rtc_registers[3] += 1 }
                    } else { self.rtc_registers[2] += 1 }
                } else { self.rtc_registers[1] += 1 }
            } else { self.rtc_registers[0] += 1 }
            self.rtc_counter = 0;
        }

        if self.clock_counter == 32 {
            self.rtc_counter += 1;
            self.clock_counter = 0;
        } else { self.clock_counter += 1; }
    }
}
