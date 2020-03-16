extern crate sdl2;
extern crate gl;

use sdl2::event::Event;
use sdl2::video::{GLContext, Window, GLProfile, SwapInterval};

use std::time::{SystemTime};
pub struct Screen {
    gl_ctx: GLContext,
    sdl_ctx: sdl2::Sdl,
    window: Window,
    prev_time: SystemTime,
    frames_passed: u32,

    screen_tex: u32,
    fbo: u32,
    pub pixels: [u8; 3 * Screen::WIDTH as usize * Screen::HEIGHT as usize],

    should_close: bool,
}

impl Screen {
    pub const WIDTH: u32 = 160;
    pub const HEIGHT: u32 = 144;
    const SCALE: u32 = 3;

    pub fn new() -> Self {
        let sdl_ctx = sdl2::init().unwrap();
        let video_subsystem = sdl_ctx.video().unwrap();
        
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);

        let window = video_subsystem.window("GBC Emulator",
                    Screen::WIDTH * Screen::SCALE, Screen::HEIGHT * Screen::SCALE)
                    .opengl().build().unwrap();

        let gl_ctx = window.gl_create_context().unwrap();
        gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
        video_subsystem.gl_set_swap_interval(SwapInterval::Immediate).unwrap();

        debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
        debug_assert_eq!(gl_attr.context_version(), (3, 3));

        let mut screen_tex = 0u32;
        let mut fbo = 0u32; 
        let color_black = [0f32, 0f32, 0f32];

        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(gl_debug_callback), std::ptr::null_mut());

            gl::GenTextures(1, &mut screen_tex as *mut u32);
            gl::BindTexture(gl::TEXTURE_2D, screen_tex);
            gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, &color_black as *const f32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, Screen::WIDTH as i32,
                Screen::HEIGHT as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, std::ptr::null_mut());

            gl::GenFramebuffers(1, &mut fbo as *mut u32);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, screen_tex, 0);
        }

        Screen {
            gl_ctx,
            sdl_ctx,
            prev_time: SystemTime::now(),
            frames_passed: 0,

            screen_tex,
            fbo,
            window,
            pixels: [0; 3 * Screen::WIDTH as usize * Screen::HEIGHT as usize],

            should_close: false,
        }
    }

    pub fn render(&mut self) {
        let width = (Screen::SCALE * Screen::WIDTH) as i32;
        let height = (Screen::SCALE * Screen::HEIGHT) as i32;
        let mut tex_x = 0i32;
        let mut tex_y = 0i32;
        if width * Screen::HEIGHT as i32 > height * Screen::WIDTH as i32 {
            let scaled_width = (width as f32 / height as f32 * height as f32) as i32;
            tex_x = (width - scaled_width) / 2;
            tex_y = 0;
        } else if (width * Screen::HEIGHT as i32) < (height * Screen::HEIGHT as i32) {
            let scaled_height = (height as f32 / width as f32 * width as f32) as i32;
            tex_x = 0;
            tex_y = (height - scaled_height) / 2;
        }

        unsafe {
            gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, Screen::WIDTH as i32, Screen::HEIGHT as i32,
                gl::RGB, gl::UNSIGNED_BYTE, &self.pixels as *const _ as *const std::ffi::c_void);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.fbo);
            gl::BlitFramebuffer(0, 0, Screen::WIDTH as i32, Screen::HEIGHT as i32,
                tex_x, tex_y, width - tex_x, height - tex_y, gl::COLOR_BUFFER_BIT, gl::NEAREST);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
        }
        
        self.window.gl_swap_window();
        let mut event_pump = self.sdl_ctx.event_pump().unwrap();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    self.should_close = true;
                    break;
                },
                _ => {}
            }
        }

        self.frames_passed += 1;
        let cur_time = SystemTime::now();
        let time_passed = cur_time.duration_since(self.prev_time).unwrap().as_secs();
        if time_passed >= 1 {
            let fps = self.frames_passed as f64 / time_passed as f64;
            self.window.set_title(&format!("GBC Emulator - {} FPS", fps)).unwrap();
            self.frames_passed = 0;
            self.prev_time = cur_time;
        }
    }

    pub fn should_close(&self) -> bool {
        self.should_close
    }
}

extern "system"
fn gl_debug_callback(_source: u32, _type: u32, _id: u32, _sev: u32, _len: i32,
    message: *const i8, _param: *mut std::ffi::c_void) {
    
    unsafe {
        let message = std::ffi::CStr::from_ptr(message).to_str().unwrap();
        panic!("OpenGL Debug message: {}", message);
    }
}
