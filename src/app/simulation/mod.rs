/* Simulation controls the computation portion of the app. Contains the state of the sim and ability to step the simulation forward. Handles all compute pipelines and passes.
*/

use std::rc::Rc;

use self::{
    compute_textures::ComputeTextures, neural_parameters::NeuralParameters,
    totalistic_parameters::TotalisticParameters,
};

use super::{
    gpu::{bindgroup::ToBindgroup, Gpu},
    gui::Gui,
    image_util::{self, ImageUtil},
    math::UVec2,
    wgsl_preproc::WgslPreProcessor,
};

pub mod compute_textures;
pub mod neural_parameters;
pub mod totalistic_parameters;

#[derive(PartialEq, Clone, Copy)]
pub enum SimulationType {
    Totalistic,
    Neural,
}

pub struct NeuralState {
    pipeline: wgpu::ComputePipeline,
    pub params: NeuralParameters,
}

pub struct TotalisticState {
    pipeline: wgpu::ComputePipeline,
    pub params: TotalisticParameters,
}
pub struct SimulationState {
    pub paused: bool,
    //Frames per second
    pub fps: u32,
    //Updates per second
    pub ups: u32,
    pub generations: usize,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            paused: false,
            fps: 0,
            ups: 0,
            generations: 0,
        }
    }
}

pub struct Simulation {
    simulation_type: SimulationType,
    pub neural_state: NeuralState,
    pub totalistic_state: TotalisticState,
    compute_textures: ComputeTextures,
    current_frame: usize,
    sim_state: SimulationState,
    pub size: UVec2,
}

impl NeuralState {
    pub fn new(gpu: &Gpu) -> NeuralState {
        let layout = NeuralState::create_pipeline(gpu);
        let params_bind_group_layout = Rc::new(layout.get_bind_group_layout(1));
        NeuralState {
            pipeline: layout,
            params: NeuralParameters::new(params_bind_group_layout),
        }
    }
    pub fn create_pipeline(gpu: &Gpu) -> wgpu::ComputePipeline {
        let shader_root = "./shaders";
        let shader_src = WgslPreProcessor::load_and_process("neural.wgsl", shader_root).unwrap();
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Neural shader"),
                source: wgpu::ShaderSource::Wgsl(shader_src.into()),
            });

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Neural compute pipeline"),
                layout: None,
                module: &shader,
                entry_point: "main",
            });
        pipeline
    }
}

impl TotalisticState {
    pub fn new(gpu: &Gpu) -> TotalisticState {
        let layout = TotalisticState::create_pipeline(gpu);
        let params_bind_group_layout = Rc::new(layout.get_bind_group_layout(1));
        TotalisticState {
            pipeline: layout,
            params: TotalisticParameters::new(params_bind_group_layout),
        }
    }
    pub fn create_pipeline(gpu: &Gpu) -> wgpu::ComputePipeline {
        let shader_root = "./shaders";
        let shader_src =
            WgslPreProcessor::load_and_process("totalistic.wgsl", shader_root).unwrap();
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Totalistic shader"),
                source: wgpu::ShaderSource::Wgsl(shader_src.into()),
            });

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Totalistic compute pipeline"),
                layout: None,
                module: &shader,
                entry_point: "main",
            });
        pipeline
    }
}

impl Simulation {
    pub fn new(gpu: &Gpu, size: UVec2) -> Simulation {
        let n_state = NeuralState::new(gpu);
        let textures_layout = Rc::new(n_state.pipeline.get_bind_group_layout(0));
        let input_image = image_util::ImageUtil::random_image_monochrome(size.x, size.y);
        Simulation {
            simulation_type: SimulationType::Totalistic,
            neural_state: NeuralState::new(gpu),
            totalistic_state: TotalisticState::new(gpu),
            compute_textures: ComputeTextures::new(textures_layout, input_image, gpu),
            current_frame: 0,
            sim_state: SimulationState::default(),
            size,
        }
    }

    pub fn remake(&mut self, gpu: &Gpu, size: UVec2, s_type: SimulationType) {
        self.size = size;
        let input_image = match s_type {
            SimulationType::Totalistic => ImageUtil::random_image_monochrome(size.x, size.y),
            SimulationType::Neural => ImageUtil::random_image_color(size.x, size.y),
        };
        let layout = Rc::new(self.neural_state.pipeline.get_bind_group_layout(0));
        self.compute_textures = ComputeTextures::new(layout, input_image, gpu);
        self.current_frame = 0;
        self.simulation_type = s_type;
    }

    pub fn step(&mut self, gpu: &Gpu) {
        if !self.get_simulation_state_mut().paused {
            self.do_step(gpu);
        }
    }

    fn do_step(&mut self, gpu: &Gpu) {
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.compute_textures.set_current_frame(self.current_frame);
        let texture_bind_group = self.compute_textures.to_bind_group(gpu);
        let params_bind_group = match self.simulation_type {
            SimulationType::Totalistic => self.totalistic_state.params.to_bind_group(gpu),
            SimulationType::Neural => self.neural_state.params.to_bind_group(gpu),
        };
        let pipeline = match self.simulation_type {
            SimulationType::Totalistic => &self.totalistic_state.pipeline,
            SimulationType::Neural => &self.neural_state.pipeline,
        };
        // Dispatch

        let (dispatch_with, dispatch_height) =
            self.compute_work_group_count((self.size.x as u32, self.size.y as u32), (16, 16));
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Totalistic step"),
            });
            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, &texture_bind_group, &[]);
            compute_pass.set_bind_group(1, &params_bind_group, &[]);

            compute_pass.dispatch_workgroups(dispatch_with, dispatch_height, 1);
        }

        gpu.queue.submit(Some(encoder.finish()));
        self.current_frame += 1;
        self.get_simulation_state_mut().generations = self.current_frame;
    }

    fn compute_work_group_count(
        &self,
        (width, height): (u32, u32),
        (workgroup_width, workgroup_height): (u32, u32),
    ) -> (u32, u32) {
        let x = (width + workgroup_width - 1) / workgroup_width;
        let y = (height + workgroup_height - 1) / workgroup_height;

        (x, y)
    }

    pub fn get_simulation_state_mut(&mut self) -> &mut SimulationState {
        &mut self.sim_state
    }

    pub fn get_current_texture(&self) -> &wgpu::Texture {
        self.compute_textures.get_read_texture()
    }

    pub fn sync_state_from_gui(&mut self, gui: &mut Gui) {
        let sim_state = self.get_simulation_state_mut();
        let gui_sim_state = gui.get_simulation_state_mut();
        sim_state.paused = gui_sim_state.paused;
        gui_sim_state.fps = sim_state.fps;
        gui_sim_state.ups = sim_state.ups;
        gui_sim_state.generations = sim_state.generations;
        let _s = self.size;
    }
}
