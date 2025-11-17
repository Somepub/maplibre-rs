// =========================================================
// Vertex input → TextVertex { pos: vec2, uv: vec2 }
// =========================================================
struct VSIn {
    @location(0) pos : vec2<f32>,
    @location(1) uv  : vec2<f32>,
};

struct VSOut {
    @builtin(position) position : vec4<f32>,
    @location(0) uv : vec2<f32>,
};

@vertex
fn vs_main(in: VSIn) -> VSOut {
    var out: VSOut;

    // Convert pixel coords → NDC
    let x_ndc = (in.pos.x / 1024.0) * 2.0 - 1.0;
    let y_ndc = 1.0 - (in.pos.y / 1024.0) * 2.0;

    out.position = vec4<f32>(x_ndc, y_ndc, 0.0, 1.0);
    out.uv = in.uv;

    return out;
}

// =========================================================
// Fragment Shader
// =========================================================

@group(0) @binding(0)
var myTexture : texture_2d<f32>;

@group(0) @binding(1)
var mySampler : sampler;

struct FSIn {
    @location(0) uv : vec2<f32>,
};

@fragment
fn fs_main(in: FSIn) -> @location(0) vec4<f32> {
    let color = textureSample(myTexture, mySampler, in.uv);

    // Non-transparent pixels only
    if (color.a < 0.1) {
        discard;
    }

    return color;
}
