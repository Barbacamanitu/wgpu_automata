use winit::event::{MouseButton, WindowEvent};

use super::math::FVec2;

pub struct Input {
    pub mouse_down: bool,
    pub mouse_drag_pos: FVec2,
    pub mouse_drag_start: bool,
    pub latest_mouse_pos: FVec2,
    pub scroll_delta: f32,
    pub movement: FVec2,
    pub drag_offset: FVec2,
}

impl Input {
    pub fn new() -> Input {
        Input {
            mouse_down: false,
            mouse_drag_pos: FVec2::default(),
            mouse_drag_start: false,
            latest_mouse_pos: FVec2::default(),
            scroll_delta: 0.0,
            movement: FVec2::default(),
            drag_offset: FVec2::default(),
        }
    }

    #[allow(deprecated)]
    pub fn handle_input(&mut self, event: &WindowEvent) {
        self.scroll_delta = 0.0;
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                input,
                is_synthetic: _,
            } => match input.state {
                winit::event::ElementState::Pressed => {
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Left) {
                        self.movement.x = -1.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Right) {
                        self.movement.x = 1.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Up) {
                        self.movement.y = 1.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Down) {
                        self.movement.y = -1.0;
                    }
                }
                winit::event::ElementState::Released => {
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Left) {
                        self.movement.x = 0.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Right) {
                        self.movement.x = 0.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Up) {
                        self.movement.y = 0.0;
                    }
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Down) {
                        self.movement.y = 0.0;
                    }
                }
            },
            WindowEvent::CursorMoved {
                device_id: _,
                position,
                modifiers: _,
            } => {
                //Update mouse position
                self.latest_mouse_pos = FVec2::new(position.x as f32, position.y as f32);

                if self.mouse_down {
                    //Drag camera
                    let difference = self.latest_mouse_pos - self.mouse_drag_pos;
                    self.drag_offset = difference;
                    if self.mouse_drag_start {
                        self.mouse_drag_start = false;
                    }
                } else {
                    self.mouse_drag_pos = self.latest_mouse_pos;
                }
            }

            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
                modifiers: _,
            } => match delta {
                winit::event::MouseScrollDelta::LineDelta(_x, y) => {
                    self.scroll_delta = *y;
                }
                winit::event::MouseScrollDelta::PixelDelta(_pos) => {}
            },
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
                modifiers: _,
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
    }
    pub fn mouseup(&mut self) {
        self.mouse_down = false;
    }

    fn mousedown(&mut self) {
        self.mouse_down = true;
        self.mouse_drag_start = true;
        self.mouse_drag_pos = self.latest_mouse_pos;
    }
}
