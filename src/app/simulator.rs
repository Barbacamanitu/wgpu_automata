use super::{gpu_interface::GPUInterface, math::IVec2, renderer::Renderer};

#[derive(Clone)]
pub struct SimulationState {
    pub paused: bool,
    //Frames per second
    pub fps: u32,
    //Updates per second
    pub ups: u32,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            paused: false,
            fps: 0,
            ups: 0,
        }
    }
}

pub trait Simulator {
    fn get_read_write(&self) -> (usize, usize) {
        let mut read = 0;
        let mut write = 1;
        if self.get_current_frame() % 2 == 1 {
            read = 1;
            write = 0;
        }
        (read, write)
    }

    fn get_current_texture(&self) -> &wgpu::Texture {
        &self.get_textures()[self.get_read_write().1]
    }

    fn get_current_frame(&self) -> usize;

    fn get_textures(&self) -> &[wgpu::Texture; 2];

    fn get_size(&self) -> IVec2;

    fn do_step(&mut self, gpu: &GPUInterface);
    fn step(&mut self, gpu: &GPUInterface) {
        if !self.get_simulation_state_mut().paused {
            self.do_step(gpu);
        }
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

    fn get_simulation_state_mut(&mut self) -> &mut SimulationState;

    fn sync_state_from_gui(&mut self, r: &mut Renderer) {
        let sim_state = self.get_simulation_state_mut();
        let gui_sim_state = r.gui.get_simulation_state_mut();
        sim_state.paused = gui_sim_state.paused;
        gui_sim_state.fps = sim_state.fps;
        gui_sim_state.ups = sim_state.ups;
    }
}
