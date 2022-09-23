use std::any::Any;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use super::{
    gpu_interface::GPUInterface,
    image_util::ImageUtil,
    math::{IVec2, UVec2},
    simulator::{SimulationState, Simulator},
    wgsl_preproc::WgslPreProcessor,
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
pub struct NeuralParams {
    pub size: UVec2,
    pub filter: NeuralFilter,
}

#[derive(Debug)]
pub enum NeuralCreationError {
    ShaderError,
}

pub struct Neural {
    compute_pipeline: wgpu::ComputePipeline,
    textures: [wgpu::Texture; 2],
    current_frame: usize,
    pub size: IVec2,
    sim_state: SimulationState,
    filter: NeuralFilter,
}

impl Simulator for Neural {
    fn get_current_frame(&self) -> usize {
        self.current_frame
    }

    fn get_textures(&self) -> &[wgpu::Texture; 2] {
        &self.textures
    }

    fn do_step(&mut self, gpu: &GPUInterface) {
        let (read, write) = self.get_read_write();
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let filter_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("filter Buffer"),
                contents: bytemuck::bytes_of(&self.filter.to_buffer()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let compute_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compute bind group"),
            layout: &self.compute_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self.textures[read].create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &self.textures[write].create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: filter_buffer.as_entire_binding(),
                },
            ],
        });
        // Dispatch

        let (dispatch_with, dispatch_height) =
            self.compute_work_group_count((self.size.x as u32, self.size.y as u32), (16, 16));
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Totalistic step"),
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(dispatch_with, dispatch_height, 1);
        }

        gpu.queue.submit(Some(encoder.finish()));
        self.current_frame += 1;
        self.get_simulation_state_mut().generations = self.current_frame;
    }

    fn get_size(&self) -> IVec2 {
        self.size
    }

    fn get_read_write(&self) -> (usize, usize) {
        let mut read = 0;
        let mut write = 1;
        if self.get_current_frame() % 2 == 1 {
            read = 1;
            write = 0;
        }
        (read, write)
    }

    fn get_current_texture(&self) -> &wgpu::Texture {
        &self.get_textures()[self.get_read_write().1]
    }

    fn compute_work_group_count(
        &self,
        (width, height): (u32, u32),
        (workgroup_width, workgroup_height): (u32, u32),
    ) -> (u32, u32) {
        let x = (width + workgroup_width - 1) / workgroup_width;
        let y = (height + workgroup_height - 1) / workgroup_height;

        (x, y)
    }

    fn get_simulation_state_mut(&mut self) -> &mut super::simulator::SimulationState {
        &mut self.sim_state
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Neural {
    pub fn new(gpu: &GPUInterface, params: NeuralParams) -> Result<Neural, NeuralCreationError> {
        let shader_root = "./shaders";
        let shader_process = WgslPreProcessor::load_and_process("neural.wgsl", shader_root);
        match shader_process {
            Ok(shader_src) => {
                let shader = gpu
                    .device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("Neural shader"),
                        source: wgpu::ShaderSource::Wgsl(shader_src.into()),
                    });

                let pipeline =
                    gpu.device
                        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                            label: Some("Neural compute pipeline"),
                            layout: None,
                            module: &shader,
                            entry_point: "main",
                        });

                let input_image = ImageUtil::random_image_color(params.size.x, params.size.y);
                let (width, height) = input_image.dimensions();

                let texture_size = wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                };

                let input_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("input texture"),
                    size: texture_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::STORAGE_BINDING,
                });

                let output_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("output texture"),
                    size: texture_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::STORAGE_BINDING,
                });
                gpu.queue.write_texture(
                    input_texture.as_image_copy(),
                    bytemuck::cast_slice(input_image.as_raw()),
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: std::num::NonZeroU32::new(4 * width),
                        rows_per_image: None, // Doesn't need to be specified as we are writing a single image.
                    },
                    texture_size,
                );

                let img_dims = input_image.dimensions();
                Ok(Neural {
                    compute_pipeline: pipeline,
                    textures: [input_texture, output_texture],

                    current_frame: 0,
                    size: IVec2::new(img_dims.0 as i32, img_dims.1 as i32),
                    sim_state: SimulationState::default(),
                    filter: params.filter,
                })
            }
            Err(preproc_err) => Err(NeuralCreationError::ShaderError),
        }
    }

    pub fn set_filter(&mut self, filter: NeuralFilter) {
        self.filter = filter;
    }
}
