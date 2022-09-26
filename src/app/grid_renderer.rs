use super::gpu::Gpu;

pub struct GridRenderer {
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl GridRenderer {
    pub fn new(gpu: &Gpu) {}
}
