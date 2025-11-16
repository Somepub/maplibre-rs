pub struct BitmapFont {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub char_width: u32,
    pub char_height: u32,
    pub columns: u32,
    pub rows: u32,
    pub start_char: u32,
}

pub struct TextLabel {
    pub text: String,
    pub lat: f64,
    pub lon: f64,
}

pub struct TextRenderer {
    pub font: BitmapFont,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
}

impl TextRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let img_bytes = include_bytes!("../../assets/font_atlas.png");
        let tex = image::load_from_memory(img_bytes).unwrap().to_rgba8();

        let dimensions = tex.dimensions();
        let tex_data = tex.as_raw();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("font_texture"),
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            tex_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            },
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Shader uses screen coordinates + UVs
        let shader = device.create_shader_module(wgpu::include_wgsl!("text_shader.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // texture
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
                // sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("text_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
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
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            font: BitmapFont {
                texture,
                texture_view,
                sampler,
                char_width: 12,
                char_height: 20,
                columns: 16,
                rows: 6,
                start_char: 32,
            },
            pipeline,
            bind_group,
        }
    }

    pub fn draw_text(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        text: &str,
        screen_x: f32,
        screen_y: f32,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);

        let mut x = screen_x;
        let y = screen_y;

        for ch in text.chars() {
            let idx = ch as u32 - self.font.start_char;

            let col = idx % self.font.columns;
            let row = idx / self.font.columns;

            let u1 = col as f32 / self.font.columns as f32;
            let v1 = row as f32 / self.font.rows as f32;
            let u2 = u1 + 1.0 / self.font.columns as f32;
            let v2 = v1 + 1.0 / self.font.rows as f32;

            // TODO: push vertices via vertex buffer
            // For now: draw quad via push constants (next step)

            x += self.font.char_width as f32;
        }
    }
}
