pub mod camera;
pub mod gpu_interface;
pub mod gui;
pub mod image_util;
pub mod input;
pub mod math;
pub mod neural;
pub mod rule;
pub mod sim_renderer;
pub mod simulator;
pub mod time;
pub mod totalistic;
pub mod wgsl_preproc;

use winit::event::WindowEvent;

use crate::renderer::Renderer;

use self::{
    camera::Camera,
    gpu_interface::GPUInterface,
    gui::Gui,
    input::Input,
    neural::{Neural, NeuralParams},
    simulator::Simulator,
    time::Time,
    totalistic::{Totalistic, TotalisticParams},
};

pub enum RemakeError {
    RuleError,
    NeuralError,
}

#[derive(Clone)]
pub enum SimParams {
    Totalistic(TotalisticParams),
    Neural(NeuralParams),
}

pub struct App {
    pub camera: Camera,
    pub time: Time,
    pub sim: Box<dyn Simulator>,
    pub input: Input,
}

impl App {
    pub fn new(sim_params: SimParams, time: Time, gpu: &GPUInterface) -> App {
        let sim: Box<dyn Simulator> = match sim_params {
            SimParams::Totalistic(params) => Box::new(Totalistic::new(&gpu, params).unwrap()),
            SimParams::Neural(params) => Box::new(Neural::new(&gpu, params).unwrap()),
        };

        let camera = Camera::new();
        App {
            time,
            sim: sim,
            camera,
            input: Input::new(),
        }
    }

    pub fn remake(&mut self, sim_params: SimParams, gpu: &GPUInterface) -> Result<(), RemakeError> {
        let sim: Box<dyn Simulator> = match sim_params {
            SimParams::Totalistic(params) => {
                let totalistic = Totalistic::new(&gpu, params);
                match totalistic {
                    Ok(t) => Box::new(t),
                    Err(_err) => return Err(RemakeError::RuleError),
                }
            }
            SimParams::Neural(params) => {
                let neural = Neural::new(&gpu, params);
                match neural {
                    Ok(n) => Box::new(n),
                    Err(_err) => return Err(RemakeError::NeuralError),
                }
            }
        };
        self.sim = sim;
        Ok(())
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
