// Blend mode compositing shader for egui_expressive
// Phase 3 GPU implementation — used via egui::PaintCallback with the wgpu backend.
//
// Bind group 0: base layer texture + sampler
// Bind group 1: blend layer texture + sampler
// Bind group 2: uniforms (blend_mode, opacity)

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var base_texture: texture_2d<f32>;
@group(0) @binding(1) var base_sampler: sampler;
@group(1) @binding(0) var blend_texture: texture_2d<f32>;
@group(1) @binding(1) var blend_sampler: sampler;

struct BlendUniforms {
    blend_mode: u32,
    opacity: f32,
    _pad0: f32,
    _pad1: f32,
};
@group(2) @binding(0) var<uniform> uniforms: BlendUniforms;

const BLEND_NORMAL: u32 = 0u;
const BLEND_MULTIPLY: u32 = 1u;
const BLEND_SCREEN: u32 = 2u;
const BLEND_OVERLAY: u32 = 3u;
const BLEND_DARKEN: u32 = 4u;
const BLEND_LIGHTEN: u32 = 5u;
const BLEND_COLOR_DODGE: u32 = 6u;
const BLEND_COLOR_BURN: u32 = 7u;
const BLEND_HARD_LIGHT: u32 = 8u;
const BLEND_SOFT_LIGHT: u32 = 9u;
const BLEND_DIFFERENCE: u32 = 10u;
const BLEND_EXCLUSION: u32 = 11u;
const BLEND_HUE: u32 = 12u;
const BLEND_SATURATION: u32 = 13u;
const BLEND_COLOR_MODE: u32 = 14u;
const BLEND_LUMINOSITY: u32 = 15u;

fn blend_channel(mode: u32, base: f32, blend: f32) -> f32 {
    switch mode {
        case BLEND_MULTIPLY: { return base * blend; }
        case BLEND_SCREEN: { return 1.0 - (1.0 - base) * (1.0 - blend); }
        case BLEND_OVERLAY: {
            if base < 0.5 { return 2.0 * base * blend; }
            return 1.0 - 2.0 * (1.0 - base) * (1.0 - blend);
        }
        case BLEND_DARKEN: { return min(base, blend); }
        case BLEND_LIGHTEN: { return max(base, blend); }
        case BLEND_COLOR_DODGE: {
            if blend >= 1.0 { return 1.0; }
            return min(1.0, base / (1.0 - blend));
        }
        case BLEND_COLOR_BURN: {
            if blend <= 0.0 { return 0.0; }
            return 1.0 - min(1.0, (1.0 - base) / blend);
        }
        case BLEND_HARD_LIGHT: {
            if blend < 0.5 { return 2.0 * base * blend; }
            return 1.0 - 2.0 * (1.0 - base) * (1.0 - blend);
        }
        case BLEND_SOFT_LIGHT: {
            if blend < 0.5 {
                return base - (1.0 - 2.0 * blend) * base * (1.0 - base);
            }
            let d = select(sqrt(base), ((16.0 * base - 12.0) * base + 4.0) * base, base <= 0.25);
            return base + (2.0 * blend - 1.0) * (d - base);
        }
        case BLEND_DIFFERENCE: { return abs(base - blend); }
        case BLEND_EXCLUSION: { return base + blend - 2.0 * base * blend; }
        default: { return blend; }
    }
}

fn rgb_to_hsl(rgb: vec3<f32>) -> vec3<f32> {
    let cmax = max(max(rgb.r, rgb.g), rgb.b);
    let cmin = min(min(rgb.r, rgb.g), rgb.b);
    let l = (cmax + cmin) * 0.5;
    if cmax == cmin { return vec3<f32>(0.0, 0.0, l); }
    let d = cmax - cmin;
    let s = select(d / (2.0 - cmax - cmin), d / (cmax + cmin), l < 0.5);
    var h: f32;
    if cmax == rgb.r { h = (rgb.g - rgb.b) / d + select(6.0, 0.0, rgb.g >= rgb.b); }
    else if cmax == rgb.g { h = (rgb.b - rgb.r) / d + 2.0; }
    else { h = (rgb.r - rgb.g) / d + 4.0; }
    return vec3<f32>(h / 6.0, s, l);
}

fn hue_to_rgb(p: f32, q: f32, t_in: f32) -> f32 {
    var t = t_in;
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0/6.0 { return p + (q - p) * 6.0 * t; }
    if t < 0.5 { return q; }
    if t < 2.0/3.0 { return p + (q - p) * (2.0/3.0 - t) * 6.0; }
    return p;
}

fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
    if hsl.y == 0.0 { return vec3<f32>(hsl.z, hsl.z, hsl.z); }
    let q = select(hsl.z + hsl.y - hsl.z * hsl.y, hsl.z * (1.0 + hsl.y), hsl.z < 0.5);
    let p = 2.0 * hsl.z - q;
    return vec3<f32>(
        hue_to_rgb(p, q, hsl.x + 1.0/3.0),
        hue_to_rgb(p, q, hsl.x),
        hue_to_rgb(p, q, hsl.x - 1.0/3.0),
    );
}

fn blend_hsl_modes(mode: u32, base: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    let bh = rgb_to_hsl(base);
    let blh = rgb_to_hsl(blend);
    switch mode {
        case BLEND_HUE: { return hsl_to_rgb(vec3<f32>(blh.x, bh.y, bh.z)); }
        case BLEND_SATURATION: { return hsl_to_rgb(vec3<f32>(bh.x, blh.y, bh.z)); }
        case BLEND_COLOR_MODE: { return hsl_to_rgb(vec3<f32>(blh.x, blh.y, bh.z)); }
        case BLEND_LUMINOSITY: { return hsl_to_rgb(vec3<f32>(bh.x, bh.y, blh.z)); }
        default: { return blend; }
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base = textureSample(base_texture, base_sampler, in.uv);
    let blend_raw = textureSample(blend_texture, blend_sampler, in.uv);
    let blend_a = blend_raw.a * uniforms.opacity;
    let mode = uniforms.blend_mode;

    var result_rgb: vec3<f32>;
    if mode >= BLEND_HUE {
        result_rgb = blend_hsl_modes(mode, base.rgb, blend_raw.rgb);
    } else {
        result_rgb = vec3<f32>(
            blend_channel(mode, base.r, blend_raw.r),
            blend_channel(mode, base.g, blend_raw.g),
            blend_channel(mode, base.b, blend_raw.b),
        );
    }

    // Porter-Duff source-over
    let out_a = blend_a + base.a * (1.0 - blend_a);
    if out_a < 0.0001 { return vec4<f32>(0.0, 0.0, 0.0, 0.0); }
    let out_rgb = (result_rgb * blend_a + base.rgb * base.a * (1.0 - blend_a)) / out_a;
    return vec4<f32>(out_rgb, out_a);
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(2.0, 1.0),
        vec2<f32>(0.0, -1.0),
    );
    var out: VertexOutput;
    out.position = vec4<f32>(pos[vi], 0.0, 1.0);
    out.uv = uvs[vi];
    return out;
}