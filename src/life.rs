use pollster::FutureExt;

use crate::gpu_interface::GPUInterface;

pub struct Life {}

impl Life {
    pub async fn run(gpu: &GPUInterface) {
        let input_image = image::load_from_memory(include_bytes!("test1.png"))
            .unwrap()
            .to_rgba8();
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

        let shader = gpu
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("Grayscale shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("grayscale.wgsl").into()),
            });

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Grayscale pipeline"),
                layout: None,
                module: &shader,
                entry_point: "grayscale_main",
            });

        let texture_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &input_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &output_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
            ],
        });

        let texture_bind_group2 = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &output_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &input_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
            ],
        });

        // Dispatch

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let (dispatch_with, dispatch_height) =
                Life::compute_work_group_count((texture_size.width, texture_size.height), (16, 16));
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Grayscale pass"),
            });
            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &texture_bind_group, &[]);
            compute_pass.dispatch(dispatch_with, dispatch_height, 1);
            //compute_pass.set_bind_group(0, &texture_bind_group2, &[]);
            //compute_pass.dispatch(dispatch_with, dispatch_height, 1);
        }

        //Pass # 2

        // Get the result.

        let padded_bytes_per_row = Life::padded_bytes_per_row(width);
        let unpadded_bytes_per_row = width as usize * 4;

        let output_buffer_size =
            padded_bytes_per_row as u64 * height as u64 * std::mem::size_of::<u8>() as u64;
        let output_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(padded_bytes_per_row as u32),
                    rows_per_image: std::num::NonZeroU32::new(height),
                },
            },
            texture_size,
        );
        gpu.queue.submit(Some(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        let mapping = buffer_slice.map_async(wgpu::MapMode::Read);

        gpu.device.poll(wgpu::Maintain::Wait);
        mapping.block_on();

        let padded_data = buffer_slice.get_mapped_range();

        let mut pixels: Vec<u8> = vec![0; unpadded_bytes_per_row * height as usize];
        for (padded, pixels) in padded_data
            .chunks_exact(padded_bytes_per_row)
            .zip(pixels.chunks_exact_mut(unpadded_bytes_per_row))
        {
            pixels.copy_from_slice(&padded[..unpadded_bytes_per_row]);
        }

        if let Some(output_image) =
            image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, &pixels[..])
        {
            output_image.save("test2.png");
        }
    }

    fn compute_work_group_count(
        (width, height): (u32, u32),
        (workgroup_width, workgroup_height): (u32, u32),
    ) -> (u32, u32) {
        let x = (width + workgroup_width - 1) / workgroup_width;
        let y = (height + workgroup_height - 1) / workgroup_height;

        (x, y)
    }

    /// Compute the next multiple of 256 for texture retrieval padding.
    fn padded_bytes_per_row(width: u32) -> usize {
        let bytes_per_row = width as usize * 4;
        let padding = (256 - bytes_per_row % 256) % 256;
        bytes_per_row + padding
    }
}
