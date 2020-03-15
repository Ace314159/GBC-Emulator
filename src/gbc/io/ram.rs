use super::MemoryHandler;

pub struct RAM {
    offset: usize,
    mem: Vec<u8>,
}

impl RAM {
    pub fn new(start_addr: usize, end_addr: usize) -> Self {
        RAM {
            offset: start_addr,
            mem: vec![0; end_addr - start_addr + 1],
        }
    }
}

impl MemoryHandler for RAM {
    fn read(&self, addr: u16) -> u8 {
        self.mem[addr as usize - self.offset]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.mem[addr as usize - self.offset] = value;
    }
}
