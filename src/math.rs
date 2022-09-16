use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FVec2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

impl From<FVec2> for FVec3 {
    fn from(v: FVec2) -> Self {
        FVec3 {
            x: v.x,
            y: v.y,
            z: 0.0,
        }
    }
}

impl IVec2 {
    pub fn new(x: i32, y: i32) -> IVec2 {
        IVec2 { x: x, y: y }
    }

    pub fn as_slice(&self) -> [i32; 2] {
        [self.x, self.y]
    }
}
