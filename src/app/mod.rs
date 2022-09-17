use std::time::Duration;

pub mod camera;
pub mod continuous;
pub mod gpu_interface;
pub mod gui;
pub mod image_util;
pub mod math;
pub mod renderer;
pub mod rule;
pub mod simulator;
pub mod time;
pub mod totalistic;
pub mod wgsl_preproc;

use wgpu::SurfaceTexture;
use winit::{
    event::{MouseButton, WindowEvent},
    window::Window,
};

use self::{
    camera::Camera,
    continuous::Continuous,
    gpu_interface::GPUInterface,
    math::{FVec3, IVec2},
    renderer::Renderer,
    rule::Rule,
    simulator::Simulator,
    time::Time,
    totalistic::Totalistic,
};

pub type ImageType = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

pub enum SimParams {
    Totalistic(Rule),
    Continuous,
}

pub struct App {
    pub gpu: GPUInterface,
    pub renderer: Renderer,
    camera: Camera,
    time: Time,
    sim: Box<dyn Simulator>,
    mouse_down: bool,
    mouse_drag_pos: FVec3,
    mouse_drag_start: bool,
    latest_mouse_pos: FVec3,
}

impl App {
    pub fn new(
        renderer_size: IVec2,
        sim_params: SimParams,
        window: &Window,
        input_image: ImageType,
        time: Time,
    ) -> App {
        let gpu: GPUInterface = GPUInterface::new(&window);
        /*let input_image = image::load_from_memory(include_bytes!("gol1.png"))
            .unwrap()
            .to_rgba8();
        //let input_image = Totalistic::random_image(width, height);*/

        let sim: Box<dyn Simulator> = match sim_params {
            SimParams::Totalistic(rule) => Box::new(Totalistic::new(&gpu, &input_image, rule)),
            SimParams::Continuous => Box::new(Continuous::new(&gpu, &input_image)),
        };

        let renderer = Renderer::new(&gpu, renderer_size, sim_params, window);

        let camera = Camera::new();
        App {
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
            latest_mouse_pos: FVec3::default(),
            mouse_drag_start: false,
        }
    }

    pub fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        let rend_result =
            self.renderer
                .render(&self.gpu, &self.sim, &self.camera, window, &self.time);
        if rend_result.is_err() {
            return rend_result;
        }

        self.time.render_tick();
        match self.time.get_fps() {
            Some(fps) => {
                println!("FPS: {}, Updates/Sec: {}", fps.render_fps, fps.update_fps);
                let sim_state = self.sim.get_simulation_state_mut();
                sim_state.fps = fps.render_fps as u32;
                sim_state.ups = fps.update_fps as u32;
            }
            None => {}
        }

        //Sync gui sim state to real sim state
        self.sim.sync_state_from_gui(&mut self.renderer);
        while self.time.can_update() && !self.sim.get_simulation_state_mut().paused {
            self.sim.step(&self.gpu);
            self.time.update_tick();
        }

        Ok(())
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        if (self.renderer.gui.is_handling_input()) {
            self.mouseup();
            return true;
        }
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
                //Update mouse position
                self.latest_mouse_pos = FVec3 {
                    x: position.x as f32,
                    y: position.y as f32,
                    z: 0.0,
                };

                if self.mouse_down {
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
                        self.mousedown();
                    }
                    _ => {}
                },
                winit::event::ElementState::Released => match button {
                    MouseButton::Left => {
                        self.mouseup();
                    }
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

    fn mouseup(&mut self) {
        self.mouse_down = false;
    }

    fn mousedown(&mut self) {
        self.mouse_down = true;
        self.mouse_drag_start = true;
        self.mouse_drag_pos = self.latest_mouse_pos;
    }
}
