use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::app::{
    gpu::{self, bindgroup::ToBindgroup, Gpu},
    math::UVec2,
};

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct NeuralFilterBuffer {
    w0: f32,
    w1: f32,
    w2: f32,
    w3: f32,
    w4: f32,
    w5: f32,
    w6: f32,
    w7: f32,
    w8: f32,
}

#[derive(Clone, Copy)]
pub struct NeuralFilter {
    pub weights: [f32; 9],
}

impl Default for NeuralFilter {
    fn default() -> Self {
        Self {
            weights: [-0.72, 0.90, -0.68, 0.92, 0.68, 0.91, -0.68, 0.9, -0.72],
        }
    }
}

impl NeuralFilter {
    pub fn to_buffer(&self) -> NeuralFilterBuffer {
        NeuralFilterBuffer {
            w0: self.weights[0],
            w1: self.weights[1],
            w2: self.weights[2],
            w3: self.weights[3],
            w4: self.weights[4],
            w5: self.weights[5],
            w6: self.weights[6],
            w7: self.weights[7],
            w8: self.weights[8],
        }
    }

    pub fn from_slice(s: &[f32; 9]) -> NeuralFilter {
        NeuralFilter { weights: *s }
    }
}

#[derive(Clone)]
pub struct NeuralCreationParameters {
    pub size: UVec2,
    pub filter: NeuralFilter,
}

pub struct NeuralParameters {
    pub filter: NeuralFilter,
    bind_group_layout: Rc<wgpu::BindGroupLayout>,
}

impl NeuralParameters {
    pub fn new(layout: Rc<wgpu::BindGroupLayout>) -> NeuralParameters {
        NeuralParameters {
            filter: NeuralFilter::default(),
            bind_group_layout: layout.clone(),
        }
    }
}

impl ToBindgroup for NeuralParameters {
    fn to_bind_group(&self, gpu: &Gpu) -> wgpu::BindGroup {
        let filter_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Neural Filter Buffer"),
                contents: bytemuck::bytes_of(&self.filter.to_buffer()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Neural Parameters bind group"),
            layout: self.bind_group_layout.as_ref(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: filter_buffer.as_entire_binding(),
            }],
        })
    }
}
