use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use super::MemoryHandler;
use super::IO;

pub struct Joypad {
    select_buttons: bool,
    select_dirs: bool,

    dir_down: bool,
    dir_up: bool,
    dir_left: bool,
    dir_right: bool,
    button_start: bool,
    button_select: bool,
    button_a: bool,
    button_b: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            select_buttons: false,
            select_dirs: false,
            dir_down: false,
            dir_up: false,
            dir_left: false,
            dir_right: false,
            button_start: false,
            button_select: false,
            button_a: false,
            button_b: false,
        }
    }

    pub fn update_inputs(&mut self, keyboard_events: &Vec<Event>) -> u8 {
        let old_bits = self.get_bits();
        for event in keyboard_events {
            match event {
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => { self.dir_down = true },
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => { self.dir_up = true },
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => { self.dir_left = true },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => { self.dir_right = true },
                Event::KeyDown { keycode: Some(Keycode::T), .. } => { self.button_start = true },
                Event::KeyDown { keycode: Some(Keycode::E), .. } => { self.button_select = true },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => { self.button_a = true },
                Event::KeyDown { keycode: Some(Keycode::A), .. } => { self.button_b = true },
                Event::KeyUp { keycode: Some(Keycode::Down), .. } => { self.dir_down = false },
                Event::KeyUp { keycode: Some(Keycode::Up), .. } => { self.dir_up = false },
                Event::KeyUp { keycode: Some(Keycode::Left), .. } => { self.dir_left = false },
                Event::KeyUp { keycode: Some(Keycode::Right), .. } => { self.dir_right = false },
                Event::KeyUp { keycode: Some(Keycode::T), .. } => { self.button_start = false },
                Event::KeyUp { keycode: Some(Keycode::E), .. } => { self.button_select = false },
                Event::KeyUp { keycode: Some(Keycode::S), .. } => { self.button_a = false },
                Event::KeyUp { keycode: Some(Keycode::A), .. } => { self.button_b = false },
                _ => {},
            }
        }

        if old_bits & !self.get_bits() != 0 {
            IO::JOYPAD_INT
        } else { 0 }
    }

    fn get_bits(&self) -> u8 {
        let mut input = 0xF;
        if self.select_buttons {
            input &= !((self.button_start as u8) << 3 | (self.button_select as u8) << 2 |
                    (self.button_b as u8) << 1 | (self.button_a as u8) << 0);
        }
        if self.select_dirs {
            input &= !((self.dir_down as u8) << 3 | (self.dir_up as u8) << 2 |
                    (self.dir_left as u8) << 1 | (self.dir_right as u8) << 0);
        }
        input
    }
}

impl MemoryHandler for Joypad {
    fn read(&self, addr: u16) -> u8 {
        assert_eq!(addr, 0xFF00);
        return 0xC0 | (self.select_buttons as u8) << 5 | (self.select_dirs as u8) << 4 | self.get_bits();
    }

    fn write(&mut self, addr: u16, value: u8) {
        assert_eq!(addr, 0xFF00);
        self.select_buttons = value & 0x20 != 0;
        self.select_dirs = value & 0x10 != 0;
    }
}
