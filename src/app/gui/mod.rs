pub mod error_window;
pub mod gui_window;
pub mod neural_window;
use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::{event::Event, window::Window};

use super::{gpu::Gpu, sim_renderer::SimulationRenderer, simulation::SimulationState, App};

use self::gui_window::GuiWindow;

pub struct Gui {
    platform: Platform,
    render_pass: RenderPass,
    gui_window: GuiWindow,
}
impl Gui {
    pub fn is_handling_input(&self) -> bool {
        let ctx = self.platform.context();
        ctx.is_using_pointer() || ctx.wants_keyboard_input() || ctx.is_pointer_over_area()
    }

    pub fn get_simulation_state_mut(&mut self) -> &mut SimulationState {
        &mut self.gui_window.sim_state
    }

    pub fn new(gpu: &Gpu, window: &Window) -> Gui {
        let size = window.inner_size();
        let platform = Platform::new(PlatformDescriptor {
            physical_width: (size.width as u32) / 2,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        // We use the egui_wgpu_backend crate as the render backend.
        let egui_rpass = RenderPass::new(&gpu.device, gpu.config.format, 1);

        // Display the demo application that ships with egui.
        let gui_window = GuiWindow::new();
        Gui {
            platform,
            render_pass: egui_rpass,
            gui_window: gui_window,
        }
    }

    pub fn handle_events(&mut self, event: &Event<()>) {
        self.platform.handle_event(event);
    }

    pub fn render(
        &mut self,
        gpu: &Gpu,
        window: &Window,
        output: &wgpu::SurfaceTexture,
        app: &mut App,
        sim_renderer: &mut SimulationRenderer,
    ) -> wgpu::CommandBuffer {
        self.platform
            .update_time(app.time.get_elapsed().as_secs_f64());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Gui Render Encoder"),
            });
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Begin to draw the UI frame.
        self.platform.begin_frame();

        // Draw the demo application.
        self.gui_window
            .ui(&self.platform.context(), gpu, app, sim_renderer);
        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let full_output = self.platform.end_frame(Some(window));
        let paint_jobs = self.platform.context().tessellate(full_output.shapes);

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            physical_width: gpu.config.width,
            physical_height: gpu.config.height,
            scale_factor: window.scale_factor() as f32,
        };
        let tdelta: egui::TexturesDelta = full_output.textures_delta;
        self.render_pass
            .add_textures(&gpu.device, &gpu.queue, &tdelta)
            .expect("add texture ok");
        self.render_pass
            .update_buffers(&gpu.device, &gpu.queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        self.render_pass
            .execute(
                &mut encoder,
                &output_view,
                &paint_jobs,
                &screen_descriptor,
                None,
            )
            .unwrap();
        let command_buffer = encoder.finish();
        command_buffer
    }
}
