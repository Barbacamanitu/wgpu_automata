use std::{thread, time::Duration};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod camera;
mod computer;
mod continuous;
mod gpu_interface;
mod gui;
mod image_util;
mod math;
mod renderer;
mod rule;
mod simulator;
mod time;
mod totalistic;
mod wgsl_preproc;

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    math::IVec2,
    rule::Rule,
    simulator::{SimParams, Simulator},
    time::Time,
};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }
    let p = PhysicalPosition::new(0, 0);
    let renderer_size = IVec2::new(1920, 1080);
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(renderer_size.x, renderer_size.y))
        .with_title("GPU_Automata")
        .with_position(p)
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
    let mut sim: Simulator = Simulator::new(
        renderer_size,
        SimParams::Continuous,
        &window,
        input_image,
        time,
    );
    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

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
                let output = sim.gpu.surface.get_current_texture().unwrap();
                match sim.render(&window, output) {
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
