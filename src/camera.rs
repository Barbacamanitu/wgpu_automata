use bytemuck::{Pod, Zeroable};
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl FVec3 {
    pub fn new(x: f32, y: f32, z: f32) -> FVec3 {
        FVec3 { x: x, y: y, z: z }
    }
}

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
