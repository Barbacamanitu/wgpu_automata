use crate::app::{
    gpu_interface::GPUInterface,
    gui::Gui,
    math::IVec2,
    sim_renderer::{RendererType, SimulationRenderer},
    App,
};

// main.rs

use winit::window::Window;

pub struct Renderer {
    gui: Gui,
    sim_renderer: SimulationRenderer,
}

impl Renderer {
    pub fn new(
        gpu: &GPUInterface,
        size: IVec2,
        app_renderer_type: RendererType,
        window: &Window,
    ) -> Renderer {
        let gui = Gui::new(gpu, window);
        let sim_renderer = SimulationRenderer::new(gpu, size, app_renderer_type);
        Renderer { gui, sim_renderer }
    }

    pub fn get_sim_renderer(&self) -> &SimulationRenderer {
        &self.sim_renderer
    }

    pub fn get_gui_mut(&mut self) -> &mut Gui {
        &mut self.gui
    }

    pub fn get_gui(&self) -> &Gui {
        &self.gui
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, gpu: &mut GPUInterface) {
        if new_size.width > 0 && new_size.height > 0 {
            gpu.size = new_size;
            gpu.config.width = new_size.width;
            gpu.config.height = new_size.height;
            gpu.surface.configure(&gpu.device, &gpu.config);
            let s = IVec2::new(gpu.size.width as i32, gpu.size.height as i32);
            self.sim_renderer.resize(s);
        }
    }

    pub fn render(
        &mut self,
        gpu: &GPUInterface,
        app: &mut App,
        window: &Window,
    ) -> Result<(), wgpu::SurfaceError> {
        // submit will accept anything that implements IntoIter
        let output = gpu.surface.get_current_texture().unwrap();
        let sim_render = self.sim_renderer.render(gpu, app, &output).unwrap();
        let gui_render = self
            .gui
            .render(gpu, window, &output, app, &mut self.sim_renderer);
        app.sim.sync_state_from_gui(&mut self.gui);
        // let gui_render_response = gui.render(gpu, window, &output, app);
        gpu.queue.submit([sim_render, gui_render]);
        output.present();
        Ok(())
    }
}
