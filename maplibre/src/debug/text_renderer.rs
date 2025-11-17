use crate::render::resource::TrackedRenderPass;
use bytemuck_derive::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct TextVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

pub struct BitmapFont {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub char_w: f32,
    pub char_h: f32,
    pub cols: u32,
    pub rows: u32,
    pub start: u32,
}

pub struct TextRenderer {
    font: BitmapFont,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}

impl TextRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let atlas = include_bytes!("../../assets/font_atlas.png");
        let rgba = image::load_from_memory(atlas).unwrap().to_rgba8();
        let (w, h) = rgba.dimensions();
        let bytes = rgba.as_raw();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("text_font_texture"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * w),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("text_shader.wgsl"));

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("text_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("text_bind_group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text_pipeline_layout"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<TextVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("text_vb"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            font: BitmapFont {
                texture,
                view,
                sampler,
                char_w: 12.0,
                char_h: 20.0,
                cols: 16,
                rows: 6,
                start: 32,
            },
            pipeline,
            bind_group,
            vertex_buffer,
            vertex_count: 0,
        }
    }

    pub fn set_text(&mut self, queue: &wgpu::Queue, text: &str, x: f32, y: f32) {
        let mut vertices = Vec::<TextVertex>::new();
        let mut cx = x;

        for ch in text.chars() {
            let idx = (ch as u32).saturating_sub(self.font.start);
            let col = idx % self.font.cols;
            let row = idx / self.font.cols;

            let u1 = col as f32 / self.font.cols as f32;
            let v1 = row as f32 / self.font.rows as f32;
            let u2 = u1 + 1.0 / self.font.cols as f32;
            let v2 = v1 + 1.0 / self.font.rows as f32;

            let w = self.font.char_w;
            let h = self.font.char_h;

            let p1 = [cx, y];
            let p2 = [cx + w, y];
            let p3 = [cx + w, y + h];
            let p4 = [cx, y + h];

            vertices.extend_from_slice(&[
                TextVertex {
                    pos: p1,
                    uv: [u1, v1],
                },
                TextVertex {
                    pos: p2,
                    uv: [u2, v1],
                },
                TextVertex {
                    pos: p3,
                    uv: [u2, v2],
                },
                TextVertex {
                    pos: p1,
                    uv: [u1, v1],
                },
                TextVertex {
                    pos: p3,
                    uv: [u2, v2],
                },
                TextVertex {
                    pos: p4,
                    uv: [u1, v2],
                },
            ]);

            cx += w;
        }

        if !vertices.is_empty() {
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
            self.vertex_count = vertices.len() as u32;
        } else {
            self.vertex_count = 0;
        }
    }

    pub fn draw<'w>(&'w self, pass: &mut TrackedRenderPass<'w>) {
        if self.vertex_count == 0 {
            return;
        }

        pass.set_render_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }
}
