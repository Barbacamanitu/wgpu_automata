use std::rc::Rc;

use crate::app::{
    gpu::{bindgroup::ToBindgroup, Gpu},
    image_util::InputImageType,
};

pub struct ComputeTextures {
    textures: [wgpu::Texture; 2],
    bind_group_layout: Rc<wgpu::BindGroupLayout>,
    current_frame: usize,
}

impl ComputeTextures {
    pub fn set_image(&mut self, img: InputImageType, gpu: &Gpu) {
        let (read, write) = self.get_read_write();
        let tex = &self.textures[write];
        let (width, height) = img.dimensions();
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        gpu.queue.write_texture(
            tex.as_image_copy(),
            bytemuck::cast_slice(img.as_raw()),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * width),
                rows_per_image: None, // Doesn't need to be specified as we are writing a single image.
            },
            texture_size,
        );
    }

    fn get_read_write(&self) -> (usize, usize) {
        let mut read = 0;
        let mut write = 1;
        if self.current_frame % 2 == 1 {
            read = 1;
            write = 0;
        }
        (read, write)
    }

    pub fn set_current_frame(&mut self, frame: usize) {
        self.current_frame = frame;
    }

    pub fn new(
        layout: Rc<wgpu::BindGroupLayout>,
        input_image: InputImageType,
        gpu: &Gpu,
    ) -> ComputeTextures {
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
        ComputeTextures {
            textures: [input_texture, output_texture],
            current_frame: 0,
            bind_group_layout: layout.clone(),
        }
    }

    pub fn get_read_texture(&self) -> &wgpu::Texture {
        let (read, write) = self.get_read_write();
        &self.textures[read]
    }
}

impl ToBindgroup for ComputeTextures {
    fn to_bind_group(&self, gpu: &crate::app::gpu::Gpu) -> wgpu::BindGroup {
        let (read, write) = self.get_read_write();
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compute bind group"),
            layout: self.bind_group_layout.as_ref(),
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
            ],
        })
    }
}
