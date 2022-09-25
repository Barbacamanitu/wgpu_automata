use bytemuck::{Pod, Zeroable};

const MOVE_SPEED: f32 = 0.04;
const ZOOM_SPEED: f32 = 0.4;
use crate::renderer::Renderer;

use super::{
    input::Input,
    math::{FVec2, FVec3},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraBuffer {
    pub position: FVec3,
    pub zoom: f32,
}

#[derive(Debug)]
pub struct Camera {
    position: FVec2,
    zoom: f32,
    drag_start_pos: FVec2,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: FVec2::default(),
            zoom: 1.0,
            drag_start_pos: FVec2::default(),
        }
    }

    pub fn to_buffer(&self) -> CameraBuffer {
        CameraBuffer {
            position: self.position.into(),
            zoom: self.zoom,
        }
    }

    pub fn handle_input(&mut self, input: &Input, renderer: &Renderer) {
        self.zoom = (self.zoom + input.scroll_delta * ZOOM_SPEED * self.zoom).clamp(1.0, 50.0);

        let drag_offset = input.drag_offset
            * FVec2::new(
                1.0 / renderer.get_sim_renderer().size.x as f32,
                -1.0 / renderer.get_sim_renderer().size.y as f32,
            );

        if input.mouse_drag_start {
            self.drag_start_pos = self.position.clone();
        }

        if input.mouse_down {
            self.position = self.drag_start_pos - (drag_offset * (1.0 / self.zoom));
        } else {
            self.position = self.position + input.movement * MOVE_SPEED * (1.0 / self.zoom);
        }
        self.position.x = self.position.x.clamp(-1.0, 1.0);
        self.position.y = self.position.y.clamp(-1.0, 1.0);
    }
}
