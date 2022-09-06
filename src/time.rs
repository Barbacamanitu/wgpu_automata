use std::time::{Duration, Instant};

pub struct Time {
    last_update: Instant,
    last_render: Instant,
    update_duration: Duration,
    render_duration: Duration,
    fps_update_duration: Duration,
    last_fps_update: Instant,
    frame_count: u32,
    update_count: u32,
}

pub struct FPSData {
    pub render_fps: f32,
    pub update_fps: f32,
}

impl Time {
    pub fn new(
        update_duration: Duration,
        render_duration: Duration,
        fps_update_duration: Duration,
    ) -> Time {
        Time {
            last_update: Instant::now(),
            last_render: Instant::now(),
            update_duration,
            render_duration,
            fps_update_duration,
            last_fps_update: Instant::now(),
            frame_count: 0,
            update_count: 0,
        }
    }

    pub fn get_avg_fps(&mut self) -> Option<FPSData> {
        let elapsed = self.last_fps_update.elapsed();
        if elapsed > self.fps_update_duration {
            let fps = (self.frame_count as f32) / self.fps_update_duration.as_secs_f32();
            let ups = (self.update_count as f32) / self.fps_update_duration.as_secs_f32();
            self.frame_count = 0;
            self.update_count = 0;
            self.last_fps_update = Instant::now();

            return Some(FPSData {
                render_fps: fps,
                update_fps: ups,
            });
        }
        None
    }

    pub fn update_tick(&mut self) -> bool {
        let elapsed = self.last_update.elapsed();
        if elapsed > self.update_duration {
            self.last_update = Instant::now();
            self.update_count += 1;
            return true;
        }
        false
    }

    pub fn render_tick(&mut self) -> bool {
        let elapsed = self.last_render.elapsed();
        if elapsed > self.render_duration {
            self.last_render = Instant::now();
            self.frame_count += 1;
            return true;
        }
        false
    }
}
