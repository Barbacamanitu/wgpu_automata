use std::iter;
pub mod gui_window;
use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use wgpu::{CommandEncoder, SurfaceTexture};
use winit::{
    event::{Event, WindowEvent},
    window::Window,
};

use crate::{gpu_interface::GPUInterface, time::Time};

use self::gui_window::GuiWindow;

pub struct Gui {
    platform: Platform,
    render_pass: RenderPass,
    demo: GuiWindow,
}

impl Gui {
    pub fn new(gpu: &GPUInterface, window: &Window) -> Gui {
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
        let demo_app = GuiWindow::new();
        Gui {
            platform,
            render_pass: egui_rpass,
            demo: demo_app,
        }
    }

    pub fn handle_events(&mut self, event: &Event<()>) {
        self.platform.handle_event(event);
    }

    pub fn render(
        &mut self,
        time: &Time,
        gpu: &GPUInterface,
        window: &Window,
        output: &SurfaceTexture,
        encoder: &mut CommandEncoder,
    ) {
        self.platform.update_time(time.get_elapsed().as_secs_f64());

        let output_frame = output;
        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Begin to draw the UI frame.
        self.platform.begin_frame();

        // Draw the demo application.
        self.demo.ui(&self.platform.context());

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
            .execute(encoder, &output_view, &paint_jobs, &screen_descriptor, None)
            .unwrap();
        // Submit the commands.

        // Redraw egui

        /*self.render_pass
        .remove_textures(tdelta)
        .expect("remove texture ok");*/

        // Suppport reactive on windows only, but not on linux.
        // if _output.needs_repaint {
        //     *control_flow = ControlFlow::Poll;
        // } else {
        //     *control_flow = ControlFlow::Wait;
        // }
    }
}
