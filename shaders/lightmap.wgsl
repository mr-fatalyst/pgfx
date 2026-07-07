// Fullscreen pass multiplying the scene by the lightmap.
// The pipeline uses Dst * Src blending; this shader just samples the lightmap.

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var t_lightmap: texture_2d<f32>;

@group(0) @binding(1)
var s_lightmap: sampler;

// Fullscreen triangle from the vertex index alone (no vertex buffer)
@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    var out: VertexOutput;
    let uv = vec2<f32>(f32((idx << 1u) & 2u), f32(idx & 2u));
    out.clip_position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_lightmap, s_lightmap, in.uv);
}
