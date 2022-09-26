pub mod camera;
pub mod gpu;
mod grid_renderer;
pub mod gui;
pub mod image_util;
pub mod input;
pub mod math;
pub mod rule;
pub mod sim_renderer;
pub mod simulation;
pub mod time;
pub mod wgsl_preproc;

use winit::event::WindowEvent;

use crate::renderer::Renderer;

use self::{
    camera::Camera,
    gpu::Gpu,
    gui::Gui,
    input::Input,
    math::UVec2,
    simulation::{neural_parameters::NeuralCreationParameters, Simulation},
    time::Time,
};

pub enum RemakeError {
    RuleError,
    NeuralError,
}

pub struct App {
    pub camera: Camera,
    pub time: Time,
    pub simulation: Simulation,
    pub input: Input,
}

impl App {
    pub fn new(size: UVec2, time: Time, gpu: &Gpu) -> App {
        let camera = Camera::new();
        let s = Simulation::new(gpu, size);
        App {
            time,
            camera,
            input: Input::new(),
            simulation: s,
        }
    }

    pub fn handle_input(&mut self, event: &WindowEvent, gui: &Gui, renderer: &Renderer) {
        if gui.is_handling_input() {
            self.input.mouseup();
            return;
        }
        self.input.handle_input(event);

        self.camera.handle_input(&self.input, renderer);
    }
}
