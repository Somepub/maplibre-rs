struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct VertexIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexIn) -> VSOut {
    var out: VSOut;
    out.pos = vec4<f32>(in.pos, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

@group(0) @binding(0) var font_tex: texture_2d<f32>;
@group(0) @binding(1) var font_sampler: sampler;

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    return textureSample(font_tex, font_sampler, in.uv);
}
