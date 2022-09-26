use std::rc::Rc;

use bytemuck::bytes_of;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferUsages,
};

use crate::app::{gpu::bindgroup::ToBindgroup, rule::Rule};

pub struct TotalisticParameters {
    pub rule: Rule,
    bind_group_layout: Rc<wgpu::BindGroupLayout>,
}

impl TotalisticParameters {
    pub fn new(layout: Rc<wgpu::BindGroupLayout>) -> TotalisticParameters {
        TotalisticParameters {
            rule: Rule::from_rule_str("B3/S23").unwrap(),
            bind_group_layout: layout.clone(),
        }
    }
}

impl ToBindgroup for TotalisticParameters {
    fn to_bind_group(&self, gpu: &crate::app::gpu::Gpu) -> wgpu::BindGroup {
        let rule_buffer = gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Rule buffer"),
            contents: bytes_of(&self.rule),
            usage: BufferUsages::UNIFORM,
        });
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Totalistic Params BindGroup"),
            layout: self.bind_group_layout.as_ref(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: rule_buffer.as_entire_binding(),
            }],
        })
    }
}
