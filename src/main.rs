mod app;
mod renderer;
use app::{
    gpu::Gpu,
    math::{IVec2, UVec2},
    sim_renderer::RendererType,
    time::Time,
};
use renderer::Renderer;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::time::Duration;

use crate::app::App;

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let renderer_size = IVec2::new(1024, 1024);
    let sim_size = UVec2::new(2048, 2048);
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(renderer_size.x, renderer_size.y))
        .with_title("GPU_Automata")
        .with_position(PhysicalPosition::new(0, 0))
        .build(&event_loop)
        .unwrap();
    let mut gpu = Gpu::new(&window);

    let mut renderer = Renderer::new(&gpu, renderer_size, RendererType::Totalistic, &window);

    let time = Time::new(
        1,
        Duration::from_secs(1),
        Duration::from_millis(10),
        Duration::from_millis(32),
    );

    let mut app: App = App::new(sim_size, time, &gpu);
    event_loop.run(move |event, _, control_flow| {
        renderer.get_gui_mut().handle_events(&event);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                //.Handle gui events

                app.handle_input(event, renderer.get_gui(), &renderer);
                match event {
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(*physical_size, &mut gpu);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        renderer.resize(**new_inner_size, &mut gpu);
                    }
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                match renderer.render(&gpu, &mut app, &window) {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => renderer.resize(gpu.size, &mut gpu),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}
