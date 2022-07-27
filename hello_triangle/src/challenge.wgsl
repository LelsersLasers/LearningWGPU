

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) x: f32,
    @location(2) y: f32,
};


@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.x = f32(1 - i32(in_vertex_index)) * 0.5;
    out.y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4(out.x, out.y, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(in.x, in.y, 0.01, 1.0);
}