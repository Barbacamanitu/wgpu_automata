use image::{Pixel, Rgba};

use super::{gpu::Gpu, math::IVec2};

pub struct ImageUtil {}

#[allow(dead_code)]
pub type InputImageType = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

#[allow(dead_code)]
pub struct ImageData {
    pub size: IVec2,
    pub data: Vec<u8>,
}

impl ImageUtil {
    pub fn random_image_monochrome(w: u32, h: u32) -> InputImageType {
        let mut image_buffer: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            image::ImageBuffer::new(w, h);
        for (_x, _y, p) in image_buffer.enumerate_pixels_mut() {
            let black = Rgba::from_slice(&[0, 0, 0, 255]);
            let white = Rgba::from_slice(&[255, 255, 255, 255]);
            if rand::random() {
                *p = *black;
            } else {
                *p = *white;
            }
        }
        image_buffer
    }

    pub fn random_image_color(w: u32, h: u32) -> InputImageType {
        let mut image_buffer: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            image::ImageBuffer::new(w, h);

        for (_x, _y, p) in image_buffer.enumerate_pixels_mut() {
            let r = rand::random::<u8>();
            let g = rand::random::<u8>();
            let b = rand::random::<u8>();
            let c: [u8; 4] = [r, g, b, 255];
            let color = Rgba::from_slice(&c);
            *p = *color;
        }
        image_buffer
    }

    #[allow(dead_code)]
    pub fn tex_to_buffer(tex: &wgpu::Texture, gpu: &Gpu, width: u32, height: u32) -> ImageData {
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let padded_bytes_per_row = ImageUtil::padded_bytes_per_row(width);
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
                texture: tex,
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
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        gpu.queue.submit(Some(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        // let mapping = buffer_slice.map_async(wgpu::MapMode::Read, |a| a.unwrap());

        gpu.device.poll(wgpu::Maintain::Wait);

        let padded_data = buffer_slice.get_mapped_range();

        let mut pixels: Vec<u8> = vec![0; unpadded_bytes_per_row * height as usize];
        for (padded, pixels) in padded_data
            .chunks_exact(padded_bytes_per_row)
            .zip(pixels.chunks_exact_mut(unpadded_bytes_per_row))
        {
            pixels.copy_from_slice(&padded[..unpadded_bytes_per_row]);
        }

        ImageData {
            size: IVec2 {
                x: width as i32,
                y: height as i32,
            },
            data: pixels,
        }
    }

    /// Compute the next multiple of 256 for texture retrieval padding.
    fn padded_bytes_per_row(width: u32) -> usize {
        let bytes_per_row = width as usize * 4;
        let padding = (256 - bytes_per_row % 256) % 256;
        bytes_per_row + padding
    }
}
