use std::{thread, time::Duration};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod gpu_interface;
mod renderer;
mod time;
mod totalistic;
mod wgsl_preproc;

use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    gpu_interface::GPUInterface, renderer::Renderer, time::Time, totalistic::Totalistic,
    wgsl_preproc::WgslPreProcessor,
};

// main.rs
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2], // NEW!
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // NEW!
                },
            ],
        }
    }
}

// main.rs
// Changed

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

pub fn test() {
    let processor: WgslPreProcessor = WgslPreProcessor::new("./shaders");
    let shader = processor.load_and_process("Totalistic.wgsl");
    println!("Processed: {}", shader);
}

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
    let (width, height) = (1024, 1024);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(width, height))
        .build(&event_loop)
        .unwrap();
    let mut gpu: GPUInterface = GPUInterface::new(&window).await;
    /*let input_image = image::load_from_memory(include_bytes!("gol1.png"))
    .unwrap()
    .to_rgba8();*/
    let input_image = Totalistic::random_image(width, height);
    let mut totalistic: Totalistic = Totalistic::new(&gpu, &input_image);
    let mut time = Time::new(
        50,
        Duration::from_secs(1),
        Duration::from_millis(10),
        Duration::from_millis(00),
    );
    let mut state = Renderer::new(&gpu);
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

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size, &mut gpu);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        state.resize(**new_inner_size, &mut gpu);
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
            match state.render(&gpu, &totalistic) {
                Ok(_) => {
                    time.render_tick();
                }
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(gpu.size, &mut gpu),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }

            while time.can_update() {
                totalistic.step(&gpu);
                time.update_tick();
            }

            match time.get_fps() {
                Some(fps) => {
                    println!("FPS: {}, Updates/Sec: {}", fps.render_fps, fps.update_fps);
                }
                None => {}
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        _ => {}
    });
}
