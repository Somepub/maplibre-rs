// text_shader.wgsl
@group(0) @binding(0)
var font_tex: texture_2d<f32>;

@group(0) @binding(1)
var font_sampler: sampler;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vert_id: u32) -> VSOut {
    // Quad covering entire screen
    var pos = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0)
    );

    var uv = pos[vert_id];

    var out: VSOut;
    out.pos = vec4<f32>(pos[vert_id] * 2.0 - 1.0, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let color = textureSample(font_tex, font_sampler, in.uv);
    return color;
}
