use crate::debug::bmfont::BMFont;
use crate::render::resource::TrackedRenderPass;
use bytemuck_derive::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct TextVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

pub struct TextRenderer {
    font: BMFont,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,

    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,

    vb: wgpu::Buffer,
    vertex_count: u32,
    screen_w: f32,
    screen_h: f32,
}

impl TextRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        screen_w: u32,
        screen_h: u32,
    ) -> Self {
        // --- load font atlas ---
        let atlas_bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/font_atlas.png"
        ));
        let image = image::load_from_memory(atlas_bytes).unwrap().to_rgba8();
        let (w, h) = image.dimensions();
        let raw = image.as_raw();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("font_atlas"),
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
            raw,
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
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // --- load FNT ---
        let fnt_text = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/font_atlas.fnt"
        ));
        let font = BMFont::from_fnt(fnt_text);

        // --- shader ---
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

        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text_pl"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text_pipeline"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<TextVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
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

        let vb = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("text_vb"),
            size: 256 * 1024,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            font,
            texture,
            view,
            sampler,
            pipeline,
            bind_group,
            vb,
            vertex_count: 0,
            screen_w: screen_w as f32,
            screen_h: screen_h as f32,
        }
    }

    fn to_ndc(&self, x: f32, y: f32) -> [f32; 2] {
        [
            (x / self.screen_w) * 2.0 - 1.0,
            1.0 - (y / self.screen_h) * 2.0,
        ]
    }

    pub fn set_text(&mut self, queue: &wgpu::Queue, text: &str, px: f32, py: f32) {
        let mut vertices = Vec::<TextVertex>::new();
        let mut cx = px;

        for ch in text.chars() {
            if let Some(g) = self.font.chars.get(&(ch as u32)) {
                let gx = cx + g.xoffset;
                let gy = py + g.yoffset;

                let u1 = g.x / self.font.scale_w;
                let v1 = g.y / self.font.scale_h;
                let u2 = (g.x + g.w) / self.font.scale_w;
                let v2 = (g.y + g.h) / self.font.scale_h;

                let p1 = self.to_ndc(gx, gy);
                let p2 = self.to_ndc(gx + g.w, gy);
                let p3 = self.to_ndc(gx + g.w, gy + g.h);
                let p4 = self.to_ndc(gx, gy + g.h);

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

                cx += g.xadvance;
            }
        }

        if vertices.is_empty() {
            self.vertex_count = 0;
            return;
        }

        queue.write_buffer(&self.vb, 0, bytemuck::cast_slice(&vertices));
        self.vertex_count = vertices.len() as u32;
    }

    pub fn draw<'a>(&'a self, pass: &mut TrackedRenderPass<'a>) {
        if self.vertex_count == 0 {
            return;
        }

        pass.set_render_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vb.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }
}
