mod app;

use app::{image_util, math::IVec2, rule::Rule, time::Time};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::{thread, time::Duration};

use crate::app::{App, SimParams};

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let renderer_size = IVec2::new(1920, 1080);
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(renderer_size.x, renderer_size.y))
        .with_title("GPU_Automata")
        .with_position(PhysicalPosition::new(0, 0))
        .build(&event_loop)
        .unwrap();
    /*(let input_image = image::load_from_memory(include_bytes!("gol1.png"))
    .unwrap()
    .into_rgba8();*/

    let input_image = image_util::ImageUtil::random_image_color(1920 / 2, 1080 / 2);
    let time = Time::new(
        10,
        Duration::from_secs(1),
        Duration::from_millis(10),
        Duration::from_millis(0),
    );
    let rule: Rule = Rule::from_rule_str("B3/S23").unwrap();
    let mut sim: App = App::new(
        renderer_size,
        SimParams::Totalistic(rule),
        &window,
        input_image,
        time,
    );

    event_loop.run(move |event, _, control_flow| {
        sim.renderer.handle_events(&event);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                //.Handle gui events

                if !sim.input(event) {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            sim.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so we have to dereference it twice
                            sim.resize(**new_inner_size);
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
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                match sim.render(&window) {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => sim.resize(sim.gpu.size),
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
