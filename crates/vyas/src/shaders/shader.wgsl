struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

fn unpack_color_rgb(color: u32) -> vec3<f32> {
    let r = f32((color >> 16u) & 0xffu) / 255.0;
    let g = f32((color >> 8u) & 0xffu) / 255.0;
    let b = f32(color & 0xffu) / 255.0;

    return vec3<f32>(r, g, b);
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = unpack_color_rgb(model.color);
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
