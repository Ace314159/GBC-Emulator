use sdl2::audio::{AudioQueue, AudioSpecDesired};

pub struct Audio {
    queue: AudioQueue<f32>,
}

impl Audio {
    pub const SAMPLE_RATE: i32 = 44100;

    pub fn new(sdl_ctx: &sdl2::Sdl) -> Self {
        let desired_spec = AudioSpecDesired {
            freq: Some(Audio::SAMPLE_RATE),
            channels: Some(1),
            samples: None,
        };
        let audio = Audio {
            queue: sdl_ctx.audio().unwrap().open_queue(None, &desired_spec).unwrap(),
        };
        audio.queue.resume();
        audio
    }

    pub fn queue(&self, sample: f32) {
        self.queue.queue(&[sample]);
    }
}
