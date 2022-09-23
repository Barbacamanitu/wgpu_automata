use std::ops::{Add, AddAssign, Mul, Sub};

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Default)]
pub struct FVec2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Default)]
pub struct UVec2 {
    pub x: u32,
    pub y: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Default)]
pub struct FVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Default)]
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

impl FVec2 {
    pub fn new(x: f32, y: f32) -> FVec2 {
        FVec2 { x: x, y: y }
    }
}

impl UVec2 {
    pub fn new(x: u32, y: u32) -> UVec2 {
        UVec2 { x: x, y: y }
    }
}

impl Add<FVec2> for FVec2 {
    type Output = FVec2;

    fn add(self, rhs: Self) -> Self::Output {
        FVec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign<FVec2> for FVec2 {
    fn add_assign(&mut self, rhs: FVec2) {
        *self = *self + rhs;
    }
}

impl Mul<f32> for FVec2 {
    type Output = FVec2;

    fn mul(self, rhs: f32) -> Self::Output {
        FVec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<FVec2> for FVec2 {
    type Output = FVec2;

    fn mul(self, rhs: FVec2) -> Self::Output {
        FVec2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Sub<FVec2> for FVec2 {
    type Output = FVec2;

    fn sub(self, rhs: FVec2) -> Self::Output {
        FVec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}
