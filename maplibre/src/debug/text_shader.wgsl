@group(0) @binding(0)
var myTexture: texture_2d<f32>;
@group(0) @binding(1)
var mySampler: sampler;

struct VSIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VSOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VSIn) -> VSOut {
    var out: VSOut;

    // pos already in NDC from Rust
    out.position = vec4<f32>(in.pos, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let c = textureSample(myTexture, mySampler, in.uv);
    let luminance = max(max(c.r, c.g), c.b);

    // Smooth alpha for nice edges
    let alpha = smoothstep(0.3, 0.8, luminance);

    if (alpha < 0.01) {
        discard;
    }

    return vec4<f32>(0.0, 0.0, 0.0, alpha);
}
