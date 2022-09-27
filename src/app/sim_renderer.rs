use std::collections::HashMap;

use crate::app::{
    gpu::Gpu,
    math::{IVec2, Vertex},
    wgsl_preproc::WgslPreProcessor,
    App,
};

use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, Buffer};

#[derive(PartialEq, Eq, Hash)]
pub enum RendererType {
    Totalistic,
    Neural,
}

pub struct SimulationRenderer {
    pub render_pipelines: HashMap<RendererType, wgpu::RenderPipeline>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub sampler: wgpu::Sampler,
    pub size: IVec2,
    renderer_type: RendererType,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RenderParams {
    window_size: [i32; 2],
    sim_size: [i32; 2],
}
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    }, // A
    Vertex {
        position: [-1.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    }, // B
    Vertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    }, // C
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    }, // D
];

const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

impl SimulationRenderer {
    fn create_pipeline(shader: &wgpu::ShaderModule, gpu: &Gpu) -> wgpu::RenderPipeline {
        let texture_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("SimulationRenderer_texture_bind_group_layout"),
                });

        let render_params_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                    label: Some("render_params_bind_group_layout"),
                });

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),

                    bind_group_layouts: &[
                        &texture_bind_group_layout,
                        &render_params_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        // 4.
                        format: gpu.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // 2.
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None, // 1.
                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview: None, // 5.
            });
        render_pipeline
    }

    pub fn new(gpu: &Gpu, size: IVec2, r_type: RendererType) -> Self {
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let shader_types = vec![RendererType::Totalistic, RendererType::Neural];
        let mut pipeline_map: HashMap<RendererType, wgpu::RenderPipeline> = HashMap::new();
        for s in shader_types {
            let shader_str = match s {
                RendererType::Totalistic => "totalistic_render.wgsl",
                RendererType::Neural => "neural_render.wgsl",
            };

            let shader_src = WgslPreProcessor::load_and_process(shader_str, "./shaders").unwrap();
            let shader = gpu
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Shader"),
                    source: wgpu::ShaderSource::Wgsl(shader_src.into()),
                });
            let pipeline = SimulationRenderer::create_pipeline(&shader, gpu);
            pipeline_map.insert(s, pipeline);
        }

        let vertex_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer: Buffer =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index buffer"),
                    contents: bytemuck::cast_slice(&INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });
        let num_indices = INDICES.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
            sampler,
            size,
            render_pipelines: pipeline_map,
            renderer_type: r_type,
        }
    }

    pub fn set_renderer_type(&mut self, r_type: RendererType) {
        self.renderer_type = r_type;
    }

    pub fn resize(&mut self, new_size: IVec2) {
        self.size = new_size;
    }

    fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipelines[&self.renderer_type]
    }

    fn render_simulation(
        &mut self,
        gpu: &Gpu,
        app: &mut App,
        output: &wgpu::SurfaceTexture,
    ) -> Result<wgpu::CommandBuffer, wgpu::SurfaceError> {
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let render_tex = app.simulation.get_current_texture();
        let texture_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.get_pipeline().get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &render_tex.create_view(&wgpu::TextureViewDescriptor::default()),
                    ), // CHANGED!
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler), // CHANGED!
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let camera_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::bytes_of(&app.camera.to_buffer()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let isize: IVec2 = app.simulation.size.into();
        let r_params = RenderParams {
            window_size: self.size.as_slice(),
            sim_size: isize.as_slice(),
        };
        let render_params_buffer =
            gpu.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Render Params Buffer"),
                    contents: bytemuck::bytes_of(&r_params),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let render_params_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.get_pipeline().get_bind_group_layout(1),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: render_params_buffer.as_entire_binding(),
                },
            ],
            label: Some("Render_Params_bind_group"),
        });

        {
            // 1.
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what [[location(0)]] in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.get_pipeline());
            render_pass.set_bind_group(0, &texture_bind_group, &[]);
            render_pass.set_bind_group(1, &render_params_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
        Ok(encoder.finish())
    }

    pub fn render(
        &mut self,
        gpu: &Gpu,
        app: &mut App,
        output: &wgpu::SurfaceTexture,
    ) -> Result<wgpu::CommandBuffer, wgpu::SurfaceError> {
        // submit will accept anything that implements IntoIter
        let sim_render_result = self.render_simulation(gpu, app, output);
        match sim_render_result {
            Ok(sim_render_command_buffer) => {
                app.time.render_tick();
                match app.time.get_fps() {
                    Some(fps) => {
                        let sim_state = app.simulation.get_simulation_state_mut();
                        sim_state.fps = fps.render_fps as u32;
                        sim_state.ups = fps.update_fps as u32;
                    }
                    None => {}
                }

                //Sync gui sim state to real sim state

                while app.time.can_update() && !app.simulation.get_simulation_state_mut().paused {
                    app.simulation.step(gpu);
                    app.time.update_tick();
                }
                Ok(sim_render_command_buffer)
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
}
