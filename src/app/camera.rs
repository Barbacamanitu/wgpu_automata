use bytemuck::{Pod, Zeroable};

use super::math::FVec3;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Camera {
    pub position: FVec3,
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: FVec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            zoom: 1.0,
        }
    }
}
