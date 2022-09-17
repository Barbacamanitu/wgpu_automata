use std::time::{Duration, Instant};

pub struct Time {
    last_frame: Instant,
    last_update: Instant,
    last_fps_check: Instant,
    fps_update_duration: Duration,
    max_update_time: Duration,
    update_delay: Duration,
    target_updates_per_frame: u32,
    frame_since_last_fps_check: u32,
    updates_since_last_fps_check: u32,
    updates_this_frame: u32,
    start_time: Instant,
}

pub struct FPSData {
    pub render_fps: f32,
    pub update_fps: f32,
}

impl Time {
    pub fn new(
        updates_per_frame: u32,
        fps_update_duration: Duration,
        max_update_time: Duration,
        update_delay: Duration,
    ) -> Time {
        Time {
            last_frame: Instant::now(),
            last_fps_check: Instant::now(),
            fps_update_duration: fps_update_duration,
            max_update_time,
            target_updates_per_frame: updates_per_frame,
            frame_since_last_fps_check: 0,
            updates_since_last_fps_check: 0,
            updates_this_frame: 0,
            update_delay,
            last_update: Instant::now(),
            start_time: Instant::now(),
        }
    }

    pub fn get_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    //Returns None if not enough time has passed since the last FPS check. *
    pub fn get_fps(&mut self) -> Option<FPSData> {
        let elapsed = self.last_fps_check.elapsed();
        if elapsed > self.fps_update_duration {
            let fps =
                (self.frame_since_last_fps_check as f32) / self.fps_update_duration.as_secs_f32();
            let ups =
                (self.updates_since_last_fps_check as f32) / self.fps_update_duration.as_secs_f32();
            self.frame_since_last_fps_check = 0;
            self.updates_since_last_fps_check = 0;
            self.last_fps_check = Instant::now();

            return Some(FPSData {
                render_fps: fps,
                update_fps: ups,
            });
        }
        None
    }

    //Checks to see if the simulation can update again.Bases this on how long the current frame has lasted, and how many updates have happened this frame.
    pub fn can_update(&self) -> bool {
        let is_time_left = self.last_frame.elapsed() < self.max_update_time;
        let max_frames_reached: bool = self.updates_this_frame >= self.target_updates_per_frame;
        let delayed_enough = self.last_update.elapsed() > self.update_delay;
        (is_time_left && !max_frames_reached && delayed_enough)
    }

    //returns true if the simulation should run another update tick. This checks to see if there's enough time, and that the max updates per frame havent happened.
    pub fn update_tick(&mut self) {
        self.updates_this_frame += 1;
        self.updates_since_last_fps_check += 1;
        self.last_update = Instant::now();
    }

    pub fn render_tick(&mut self) {
        self.last_frame = Instant::now();
        self.frame_since_last_fps_check += 1;
        self.updates_this_frame = 0;
    }
}
