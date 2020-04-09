mod screen;
mod cgb_ppu;
mod gb_ppu;

pub use cgb_ppu::CgbPPU;
pub use gb_ppu::GbPPU;
use super::MemoryHandler;
use super::IO;

pub trait PPU: MemoryHandler {
    fn emulate_clock(&mut self) -> u8;
    
    fn read_vram_bank(&self) -> u8;
    fn read_cgb_palettes(&self, addr: u16) -> u8;

    fn write_vram_bank(&mut self, value: u8);
    fn write_cgb_palettes(&mut self, addr: u16, value: u8);
    fn write_hdma(&mut self, addr: u16, value: u8);

    fn set_double_speed(&mut self, double_speed: bool);

    fn oam_dma(&mut self) -> (bool, u16, u16);
    fn oam_write(&mut self, addr: u16, value: u8);

    fn gdma(&mut self);
}
