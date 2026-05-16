struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct BlurUniforms {
    radius: f32,
    direction_x: f32,
    direction_y: f32,
    _pad: f32,
};

@group(0) @binding(0) var _base_texture: texture_2d<f32>;
@group(0) @binding(1) var _base_sampler: sampler;
@group(1) @binding(0) var source_texture: texture_2d<f32>;
@group(1) @binding(1) var source_sampler: sampler;
@group(2) @binding(0) var<uniform> blur: BlurUniforms;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 2.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(2.0, 0.0),
    );

    var out: VertexOut;
    out.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    out.uv = uvs[vertex_index];
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(source_texture));
    let radius = max(blur.radius, 1.0);
    let direction = vec2<f32>(blur.direction_x, blur.direction_y) / dims;
    let offsets = array<f32, 5>(-1.0, -0.5, 0.0, 0.5, 1.0);
    let weights = array<f32, 5>(0.06136, 0.24477, 0.38774, 0.24477, 0.06136);

    var color = vec4<f32>(0.0);
    for (var i = 0u; i < 5u; i = i + 1u) {
        color = color + textureSample(source_texture, source_sampler, in.uv + direction * offsets[i] * radius) * weights[i];
    }
    return color;
}
