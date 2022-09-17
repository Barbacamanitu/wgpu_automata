use wgpu::util::DeviceExt;

use super::{
    gpu_interface::GPUInterface,
    math::IVec2,
    simulator::{SimulationState, Simulator},
    wgsl_preproc::WgslPreProcessor,
};

pub struct Continuous {
    compute_pipeline: wgpu::ComputePipeline,
    textures: [wgpu::Texture; 2],
    current_frame: usize,
    pub size: IVec2,
    sim_state: SimulationState,
}

impl Simulator for Continuous {
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
}

impl Continuous {
    pub fn new(
        gpu: &GPUInterface,
        input_image: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    ) -> Continuous {
        let shader_root = "./shaders";
        let shader_src =
            WgslPreProcessor::load_and_process("continuous.wgsl", shader_root).unwrap();
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("continuous shader"),
                source: wgpu::ShaderSource::Wgsl(shader_src.into()),
            });

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("continuous compute pipeline"),
                layout: None,
                module: &shader,
                entry_point: "main",
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

        Continuous {
            compute_pipeline: pipeline,
            textures: [input_texture, output_texture],

            current_frame: 0,
            size: IVec2::new(img_dims.0 as i32, img_dims.1 as i32),
            sim_state: SimulationState::default(),
        }
    }
}
