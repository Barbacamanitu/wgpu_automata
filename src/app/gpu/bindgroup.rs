use std::rc::Rc;

use super::Gpu;

pub trait ToBindgroup {
    fn to_bind_group(&self, gpu: &Gpu) -> wgpu::BindGroup;
}
