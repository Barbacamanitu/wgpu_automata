use std::time::Duration;

use winit::{
    event::{MouseButton, WindowEvent},
    window::Window,
};

use crate::{
    camera::{Camera, FVec3},
    computer::Computer,
    continuous::Continuous,
    gpu_interface::GPUInterface,
    math::IVec2,
    renderer::Renderer,
    rule::Rule,
    time::Time,
    totalistic::Totalistic,
};

pub type ImageType = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

pub enum SimParams {
    Totalistic(Rule),
    Continuous,
}

pub struct Simulator {
    pub gpu: GPUInterface,
    pub renderer: Renderer,
    camera: Camera,
    time: Time,
    sim: Box<dyn Computer>,
    mouse_down: bool,
    mouse_drag_pos: FVec3,
    mouse_drag_start: bool,
}

impl Simulator {
    pub fn new(
        renderer_size: IVec2,
        sim_params: SimParams,
        window: &Window,
        input_image: ImageType,
    ) -> Simulator {
        let gpu: GPUInterface = pollster::block_on(GPUInterface::new(&window));
        /*let input_image = image::load_from_memory(include_bytes!("gol1.png"))
            .unwrap()
            .to_rgba8();
        //let input_image = Totalistic::random_image(width, height);*/

        let sim: Box<dyn Computer> = match sim_params {
            SimParams::Totalistic(rule) => Box::new(Totalistic::new(&gpu, &input_image, rule)),
            SimParams::Continuous => Box::new(Continuous::new(&gpu, &input_image)),
        };
        let time = Time::new(
            2,
            Duration::from_secs(1),
            Duration::from_millis(10),
            Duration::from_millis(0),
        );
        let renderer = Renderer::new(&gpu, renderer_size, sim_params);

        let camera = Camera::new();
        Simulator {
            gpu,
            renderer,
            time,
            sim: sim,
            camera,
            mouse_down: false,
            mouse_drag_pos: FVec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            mouse_drag_start: false,
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let rend_result = self.renderer.render(&self.gpu, &self.sim, &self.camera);
        if rend_result.is_err() {
            return rend_result;
        }

        self.time.render_tick();

        while self.time.can_update() {
            self.sim.step(&self.gpu);
            self.time.update_tick();
        }

        match self.time.get_fps() {
            Some(fps) => {
                println!("FPS: {}, Updates/Sec: {}", fps.render_fps, fps.update_fps);
            }
            None => {}
        }
        Ok(())
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let movement: f32 = 1.0;
        match event {
            WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => match input.state {
                winit::event::ElementState::Pressed => {
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Left) {
                        self.camera.position.x -= movement;
                        println!("Camera: {:?}", self.camera);
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Right) {
                        self.camera.position.x += movement;
                        println!("Camera: {:?}", self.camera);
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Up) {
                        self.camera.position.y += movement;
                        println!("Camera: {:?}", self.camera);
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Down) {
                        self.camera.position.y -= movement;
                        println!("Camera: {:?}", self.camera);
                    }
                }
                winit::event::ElementState::Released => {}
            },
            WindowEvent::CursorMoved {
                device_id,
                position,
                modifiers,
            } => {
                if self.mouse_down {
                    if self.mouse_drag_start {
                        self.mouse_drag_start = false;
                        self.mouse_drag_pos = FVec3 {
                            x: position.x as f32,
                            y: position.y as f32,
                            z: 0.0,
                        }
                    } else {
                        //Drag camera
                        let difference = FVec3 {
                            x: self.mouse_drag_pos.x - position.x as f32,
                            y: self.mouse_drag_pos.y - position.y as f32,
                            z: 0.0,
                        };
                        self.camera.position.x = difference.x * self.camera.zoom;
                        self.camera.position.y = difference.y * self.camera.zoom;
                    }
                }
            }

            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
                modifiers,
            } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    let change: f32 = -0.1;
                    self.camera.zoom = (self.camera.zoom + y * change).clamp(0.25, 5.0);
                }
                winit::event::MouseScrollDelta::PixelDelta(pos) => {}
            },
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                modifiers,
            } => match state {
                winit::event::ElementState::Pressed => match button {
                    MouseButton::Left => {
                        self.mouse_down = true;
                        self.mouse_drag_start = true;
                    }
                    _ => {}
                },
                winit::event::ElementState::Released => match button {
                    MouseButton::Left => self.mouse_down = false,
                    _ => {}
                },
            },
            _ => {}
        }
        false
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size, &mut self.gpu);
    }
}
