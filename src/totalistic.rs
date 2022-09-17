use bytemuck::{Pod, Zeroable};
use image::{Pixel, Rgba};
use pollster::FutureExt;
use regex::Regex;
use wgpu::util::DeviceExt;

use crate::{
    computer::Computer, gpu_interface::GPUInterface, math::IVec2, rule::Rule,
    wgsl_preproc::WgslPreProcessor,
};

pub struct Totalistic {
    compute_pipeline: wgpu::ComputePipeline,
    textures: [wgpu::Texture; 2],
    current_frame: usize,
    rules: Rule,
    pub size: IVec2,
}

impl Computer for Totalistic {
    fn get_current_frame(&self) -> usize {
        self.current_frame
    }

    fn get_textures(&self) -> &[wgpu::Texture; 2] {
        &self.textures
    }

    fn step(&mut self, gpu: &GPUInterface) {
        let (read, write) = self.get_read_write();
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let rules_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Rules"),
                contents: bytemuck::bytes_of(&self.rules),
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
                    resource: rules_buffer.as_entire_binding(),
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
    }

    fn get_size(&self) -> IVec2 {
        self.size
    }
}

impl Totalistic {
    pub fn new(
        gpu: &GPUInterface,
        input_image: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
        rules: Rule,
    ) -> Totalistic {
        let shader_root = "./shaders";
        let shader_src =
            WgslPreProcessor::load_and_process("totalistic.wgsl", shader_root).unwrap();
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("totalistic shader"),
                source: wgpu::ShaderSource::Wgsl(shader_src.into()),
            });

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Totalistic compute pipeline"),
                layout: None,
                module: &shader,
                entry_point: "totalistic_main",
            });

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

        Totalistic {
            compute_pipeline: pipeline,
            textures: [input_texture, output_texture],

            current_frame: 0,
            rules: rules,
            size: IVec2::new(img_dims.0 as i32, img_dims.1 as i32),
        }
    }
}
