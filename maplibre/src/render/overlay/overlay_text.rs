use super::{
    overlay_item::OverlayText,
    overlay_phase::{OverlayItem, OverlayKind},
};
use crate::{render::RenderPhase, tcs::world::World};
use wgpu::util::DeviceExt;

pub struct OverlayTextRenderer {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub vb: wgpu::Buffer,
    pub font: crate::debug::bmfont::BMFont,
    pub screen_w: f32,
    pub screen_h: f32,
}

impl OverlayTextRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        size: (u32, u32),
    ) -> Self {
        let (screen_w, screen_h) = (size.0 as f32, size.1 as f32);

        let bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/font_atlas.png"
        ));
        let img = image::load_from_memory(bytes).unwrap().to_rgba8();
        let (w, h) = img.dimensions();
        let raw = img.as_raw();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("overlay_text_atlas"),
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

        let font_text = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/font_atlas.fnt"
        ));
        let font = crate::debug::bmfont::BMFont::from_fnt(font_text);

        let shader = device.create_shader_module(wgpu::include_wgsl!("text_overlay.wgsl"));

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("overlay_text_bgl"),
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
            label: Some("overlay_text_bg"),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("overlay_text_pl"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("overlay_text_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 20,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0=>Float32x2, 1=>Float32x2, 2=>Float32x4],
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
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        let vb = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("overlay_text_vb"),
            size: 512 * 1024,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group,
            vb,
            font,
            screen_w,
            screen_h,
        }
    }

    fn ndc(&self, x: f32, y: f32) -> [f32; 2] {
        [
            (x / self.screen_w) * 2.0 - 1.0,
            1.0 - (y / self.screen_h) * 2.0,
        ]
    }

    pub fn build_text_vertices(&self, t: &OverlayText) -> Vec<f32> {
        let mut out = Vec::new();
        let mut x = t.position.x;

        for ch in t.text.chars() {
            if let Some(g) = self.font.chars.get(&(ch as u32)) {
                let px = x + g.xoffset;
                let py = t.position.y + g.yoffset;

                let ndc1 = self.ndc(px, py);
                let ndc2 = self.ndc(px + g.w, py);
                let ndc3 = self.ndc(px + g.w, py + g.h);
                let ndc4 = self.ndc(px, py + g.h);

                let u1 = g.x / self.font.scale_w;
                let v1 = g.y / self.font.scale_h;
                let u2 = (g.x + g.w) / self.font.scale_w;
                let v2 = (g.y + g.h) / self.font.scale_h;

                let r = t.color.x;
                let g_ = t.color.y;
                let b = t.color.z;
                let a = t.color.w;

                // 2 triangles
                out.extend_from_slice(&[
                    ndc1[0], ndc1[1], u1, v1, r, g_, b, a, ndc2[0], ndc2[1], u2, v1, r, g_, b, a,
                    ndc3[0], ndc3[1], u2, v2, r, g_, b, a, ndc1[0], ndc1[1], u1, v1, r, g_, b, a,
                    ndc3[0], ndc3[1], u2, v2, r, g_, b, a, ndc4[0], ndc4[1], u1, v2, r, g_, b, a,
                ]);

                x += g.xadvance;
            }
        }

        out
    }
}

pub fn extract_text(world: &mut World, phase: &mut RenderPhase<OverlayItem>) {
    if let Some(arr) = world.resources.get::<Vec<OverlayText>>() {
        for _ in arr.iter() {
            phase.items.push(OverlayItem {
                kind: OverlayKind::Text,
                z: 10000,
            });
        }
    }
}

pub fn draw(ctx: &mut RenderContext, _item: &OverlayItem) {
    let renderer = ctx.world.resources.get::<OverlayTextRenderer>().unwrap();
    let texts = ctx.world.resources.get::<Vec<OverlayText>>().unwrap();

    let mut pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("overlay_text_pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: ctx.get_current_view(),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            },
        })],
        depth_stencil_attachment: None,
    });

    pass.set_pipeline(&renderer.pipeline);
    pass.set_bind_group(0, &renderer.bind_group, &[]);

    for t in texts.iter() {
        let verts = renderer.build_text_vertices(t);
        ctx.queue
            .write_buffer(&renderer.vb, 0, bytemuck::cast_slice(&verts));
        pass.set_vertex_buffer(0, renderer.vb.slice(..));
        let vc = (verts.len() / 8) as u32;
        pass.draw(0..vc, 0..1);
    }
}
