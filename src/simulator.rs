use std::time::Duration;

use winit::{event::WindowEvent, window::Window};

use crate::{
    gpu_interface::GPUInterface, renderer::Renderer, rule::Rule, time::Time, totalistic::Totalistic,
};

pub struct IVec2 {
    pub x: u32,
    pub y: u32,
}
pub type ImageType = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

pub struct Simulator {
    pub gpu: GPUInterface,
    pub renderer: Renderer,
    time: Time,
    totalistic: Totalistic,
    rule: Rule,
    size: IVec2,
}

impl Simulator {
    pub fn new(size: IVec2, rule_str: &str, window: &Window, input_image: ImageType) -> Simulator {
        let gpu: GPUInterface = pollster::block_on(GPUInterface::new(&window));
        /*let input_image = image::load_from_memory(include_bytes!("gol1.png"))
            .unwrap()
            .to_rgba8();
        //let input_image = Totalistic::random_image(width, height);*/

        let rule = Rule::from_rule_str(rule_str).unwrap();
        let totalistic: Totalistic = Totalistic::new(&gpu, &input_image, rule);
        let time = Time::new(
            1,
            Duration::from_secs(1),
            Duration::from_millis(10),
            Duration::from_millis(10),
        );
        let renderer = Renderer::new(&gpu);
        let totalistic = Totalistic::new(&gpu, &input_image, rule);
        Simulator {
            gpu,
            renderer,
            time,
            totalistic,
            rule,
            size,
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let rend_result = self.renderer.render(&self.gpu, &self.totalistic);
        if rend_result.is_err() {
            return rend_result;
        }

        self.time.render_tick();

        while self.time.can_update() {
            self.totalistic.step(&self.gpu);
            self.time.update_tick();
        }

        match self.time.get_fps() {
            Some(fps) => {
                println!("FPS: {}, Updates/Sec: {}", fps.render_fps, fps.update_fps);
            }
            None => {}
        }
        Ok(())
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size, &mut self.gpu);
    }
}
