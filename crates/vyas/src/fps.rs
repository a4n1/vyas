use std::time::{Duration, Instant};

pub(crate) struct FpsCounter {
    last_update: Instant,
    frame_count: u32,
    fps: f32,
}

impl FpsCounter {
    pub(crate) fn new() -> Self {
        Self {
            last_update: Instant::now(),
            frame_count: 0,
            fps: 0.0,
        }
    }

    pub(crate) fn tick(&mut self) {
        self.frame_count += 1;

        let elapsed = self.last_update.elapsed();

        if elapsed >= Duration::from_secs(1) {
            self.fps = self.frame_count as f32 / elapsed.as_secs_f32();

            println!("FPS: {:.1}", self.fps);

            self.frame_count = 0;
            self.last_update = Instant::now();
        }
    }
}
